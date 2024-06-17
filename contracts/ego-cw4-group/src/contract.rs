#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, SubMsg, Uint64
};
use cw2::set_contract_version;
use cw4::{Member, MemberChangedHookMsg, MemberDiff, MemberListResponse, MemberResponse, TotalWeightResponse};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::error::ContractError;
use crate::helpers::validate_unique_members;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{MemberDeposit, ADMIN, HOOKS, MEMBERS, MEMBER_DEPOSIT, TOTAL};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ego-cw4-group";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut deps = deps;

    let admin = &msg.admin;
    let admin_addr = deps.api.addr_validate(admin).ok();
    ADMIN.set(deps.branch(), admin_addr)?;

    let member_deposit = msg
        .member_deposit
        .map(|deposit| deposit.into_checked(deps.as_ref()))
        .transpose()
        .map(|deposit| MemberDeposit { deposit })?;

    MEMBER_DEPOSIT.save(deps.storage, &member_deposit)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => Ok(ADMIN.execute_update_admin(
            deps,
            info,
            admin.map(|admin| api.addr_validate(&admin)).transpose()?,
        )?),
        ExecuteMsg::UpdateMembers { add, remove } => {
            execute_update_members(deps, env, info, add, remove)
        }
        ExecuteMsg::AddHook { addr } => {
            Ok(HOOKS.execute_add_hook(&ADMIN, deps, info, api.addr_validate(&addr)?)?)
        }
        ExecuteMsg::RemoveHook { addr } => {
            Ok(HOOKS.execute_remove_hook(&ADMIN, deps, info, api.addr_validate(&addr)?)?)
        }
    }
}

pub fn execute_update_members(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    add: Vec<Member>,
    remove: Vec<String>,
) -> Result<Response, ContractError> {
    let attributes = vec![
        attr("action", "update_members"),
        attr("added", add.len().to_string()),
        attr("removed", remove.len().to_string()),
        attr("sender", &info.sender),
    ];

    // make the local update
    let diff = update_members(deps.branch(), env.block.height, info.sender, add, remove)?;
    // call all registered hooks
    let messages = HOOKS.prepare_hooks(deps.storage, |h| {
        diff.clone().into_cosmos_msg(h).map(SubMsg::new)
    })?;
    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(attributes))
} 

pub fn update_members(
    deps: DepsMut,
    height: u64,
    sender: Addr,
    mut to_add: Vec<Member>,
    to_remove: Vec<String>,
) -> Result<MemberChangedHookMsg, ContractError> {
    validate_unique_members(&mut to_add)?;
    let to_add = to_add; // let go of mutability

    ADMIN.assert_admin(deps.as_ref(), &sender)?;

    let mut total = Uint64::from(TOTAL.load(deps.storage)?);
    let mut diffs: Vec<MemberDiff> = vec![];

    // add all new members and update total
    for add in to_add.into_iter() {
        let add_addr = deps.api.addr_validate(&add.addr)?;
        MEMBERS.update(deps.storage, &add_addr, height, |old| -> StdResult<_> {
            total = total.checked_sub(Uint64::from(old.unwrap_or_default()))?;
            total = total.checked_add(Uint64::from(add.weight))?;
            diffs.push(MemberDiff::new(add.addr, old, Some(add.weight)));
            Ok(add.weight)
        })?;
    }

    for remove in to_remove.into_iter() {
        let remove_addr = deps.api.addr_validate(&remove)?;
        let old = MEMBERS.may_load(deps.storage, &remove_addr)?;
        // Only process this if they were actually in the list before
        if let Some(weight) = old {
            diffs.push(MemberDiff::new(remove, Some(weight), None));
            total = total.checked_sub(Uint64::from(weight))?;
            MEMBERS.remove(deps.storage, &remove_addr, height)?;
        }
    }

    TOTAL.save(deps.storage, &total.u64(), height)?;
    Ok(MemberChangedHookMsg { diffs })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Member {
            addr,
            at_height: height,
        } => to_json_binary(&query_member(deps, addr, height)?),
        QueryMsg::ListMembers { start_after, limit } => {
            to_json_binary(&query_list_members(deps, start_after, limit)?)
        }
        QueryMsg::TotalWeight { at_height: height } => {
            to_json_binary(&query_total_weight(deps, height)?)
        }
        QueryMsg::Admin {} => to_json_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Hooks {} => to_json_binary(&HOOKS.query_hooks(deps)?),
    }
}

pub fn query_total_weight(deps: Deps, height: Option<u64>) -> StdResult<TotalWeightResponse> {
    let weight = match height {
        Some(h) => TOTAL.may_load_at_height(deps.storage, h),
        None => TOTAL.may_load(deps.storage),
    }?
    .unwrap_or_default();
    Ok(TotalWeightResponse { weight })
}

pub fn query_member(deps: Deps, addr: String, height: Option<u64>) -> StdResult<MemberResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let weight = match height {
        Some(h) => MEMBERS.may_load_at_height(deps.storage, &addr, h),
        None => MEMBERS.may_load(deps.storage, &addr),
    }?;
    Ok(MemberResponse { weight })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_list_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MemberListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.as_ref().map(Bound::exclusive);

    let members = MEMBERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, weight)| Member {
                addr: addr.into(),
                weight,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(MemberListResponse { members })
}