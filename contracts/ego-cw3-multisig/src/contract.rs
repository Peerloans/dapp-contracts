#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult
};
use cw2::set_contract_version;
use cw3::UncheckedDepositInfo;
use cw4::{Cw4Contract, MemberChangedHookMsg};

use crate::error::ContractError;
use crate::helper::{accept_loan, disburse_loan, setup_loan_disbursement_offer, setup_loan_request_offer};
use crate::msg::{self, ExecuteMsg, InstantiateMsg, LoanAction, LoanOffer, QueryMsg};
use crate::state::{next_id, Config, Loan, Status, CONFIG, LOANS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ego-cw3-multisig";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let group_addr = Cw4Contract(deps.api.addr_validate(&msg.group_addr).map_err(|_| {
        ContractError::InvalidGroup {
            addr: msg.group_addr.clone(),
        }
    })?);

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cfg = Config {
        group_addr,
        executor: msg.executor
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProposeLoanOffer {
            loan_offer,
            title,
            description,
            loan_deposit
        } => execute_propose(deps, env, info, loan_offer, title, description, loan_deposit),
        ExecuteMsg::ExecuteLoanOffer { 
            loan_action,
            loan_id ,
            msgs,
            loan_deposit
        } => execute_execute(deps, env, info, loan_id, loan_action, msgs),
        ExecuteMsg::Close { loan_id } => execute_close(deps, env, info, loan_id),
        // ExecuteMsg::MemberChangedHook(MemberChangedHookMsg { diffs }) => {
        //     execute_membership_hook(deps, env, info, diffs)
        // }
    }
}

pub fn execute_propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    loan_offer: LoanOffer,
    title: String,
    description: String,
    loan_deposit: Option<UncheckedDepositInfo>
) -> Result<Response<Empty>, ContractError> {
    // only members of the multisig can create a proposal
    let actor = &info.sender;
    let cfg = CONFIG.load(deps.storage)?;
    let deposit_info = loan_deposit
        .map(|deposit| deposit.into_checked(deps.as_ref()))
        .transpose()?;

    // Check that the native deposit was paid (as needed).
    if let Some(deposit) = deposit_info.as_ref() {
        deposit.check_native_deposit_paid(&info)?;
    }

    // Take the cw20 token deposit, if required. We do this before
    // creating the proposal struct below so that we can avoid a clone
    // and move the loaded deposit info it.
    let take_deposit_msg = if let Some(deposit) = deposit_info.as_ref() {
        deposit.get_take_deposit_messages(&info.sender, &env.contract.address)?
    } else {
        vec![]
    };

    let (loan, loan_id) = match loan_offer {
        LoanOffer::LoanRequest => setup_loan_request_offer(
            deps, title, actor, description
        ),
        LoanOffer::LoanDisbursmentRequest => setup_loan_disbursement_offer(
            deps, title, actor, description
        )
    };

    Ok(Response::new()
        .add_messages(take_deposit_msg)
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("loan_id", loan_id.to_string())
        .add_attribute("status", format!("{:?}", loan.status)))
}

pub fn execute_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    loan_id: u64,
    loan_action: LoanAction,
    msgs: Vec<CosmosMsg>,
) -> Result<Response, ContractError> {
    let actor = &info.sender;
    let mut loan = LOANS.load(deps.storage, loan_id)?;

    if loan.status != Status::Open {
        return Err(ContractError::WrongExecuteStatus {});
    }

    // let cfg = CONFIG.load(deps.storage)?;
    // cfg.authorize(&deps.querier, &info.sender)?;


    // set it to executed
    loan.status = Status::Active;
    LOANS.save(deps.storage, loan_id, &loan)?;

    let _ = match loan_action {
        LoanAction::AcceptLoan => accept_loan(deps, &loan, actor),
        LoanAction::DisburseLoan => disburse_loan(deps, &loan, actor)
    };
    // Unconditionally refund here.
    // let response = match prop.deposit {
    //     Some(deposit) => {
    //         Response::new().add_message(deposit.get_return_deposit_message(&prop.proposer)?)
    //     }
    //     None => Response::new(),
    // };

    // dispatch all proposed messages
    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", loan_id.to_string()))
}

pub fn execute_close(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    loan_id: u64,
) -> Result<Response<Empty>, ContractError> {
    // anyone can trigger this if the vote passed

    let mut loan = LOANS.load(deps.storage, loan_id)?;
    if ![Status::Open, Status::Active].contains(&loan.status) {
        return Err(ContractError::WrongCloseStatus {});
    }
    // Avoid closing of Passed due to expiration proposals
    // if prop.current_status(&env.block) == Status::Passed {
    //     return Err(ContractError::WrongCloseStatus {});
    // }
    // if !prop.expires.is_expired(&env.block) {
    //     return Err(ContractError::NotExpired {});
    // }

    // set it to failed
    loan.status = Status::Close;
    LOANS.save(deps.storage, loan_id, &loan)?;

    // Refund the deposit if we have been configured to do so.
    // let mut response = Response::new();
    // if let Some(deposit) = prop.deposit {
    //     if deposit.refund_failed_proposals {
    //         response = response.add_message(deposit.get_return_deposit_message(&prop.proposer)?)
    //     }
    // }
    Ok(Response::default())
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::Hello {} => to_json_binary(&hello_world()?),
//     }
// }

