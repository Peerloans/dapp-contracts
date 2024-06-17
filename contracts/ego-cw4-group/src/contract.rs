#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use cw4::Member;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, HelloResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ego-cw4-group";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    create(deps, msg.admin, msg.members, env.block.height)?;
    Ok(Response::default())
}

pub fn create(
    mut deps: DepsMut,
    admin: String,
    mut members: Vec<Member>,
    height: u64,
) -> Result<(), ContractError> {
    // validate_unique_members(&mut members)?;
    // let members = members; // let go of mutability

    // let admin_addr = admin
    //     .map(|admin| deps.api.addr_validate(&admin))
    //     .transpose()?;
    // ADMIN.set(deps.branch(), admin_addr)?;

    // let mut total = Uint64::zero();
    // for member in members.into_iter() {
    //     let member_weight = Uint64::from(member.weight);
    //     total = total.checked_add(member_weight)?;
    //     let member_addr = deps.api.addr_validate(&member.addr)?;
    //     MEMBERS.save(deps.storage, &member_addr, &member_weight.u64(), height)?;
    // }
    // TOTAL.save(deps.storage, &total.u64(), height)?;

    Ok(())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Dummy {} => try_execute(deps),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Hello {} => to_json_binary(&hello_world()?),
    }
}

pub fn try_execute(_deps: DepsMut) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("Not implemented")))
    // TODO: Ok(Response::new().add_attribute("method", "try_execute"))
}

fn hello_world() -> StdResult<HelloResponse> {
    Ok(HelloResponse {
        msg: String::from("Hello, Archway!"),
    })
}