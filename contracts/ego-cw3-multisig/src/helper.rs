use cosmwasm_std::{Addr, DepsMut, StdResult, Storage};

use crate::state::{next_id, Loan, Status, LOANS};

pub fn setup_loan_request_offer(
    deps: DepsMut,title: String, 
    actor: &Addr,
    description: String
) -> (Loan, u64) {
    let loan = Loan {
        title,
        description,
        collacteral: None,
        amount_to_disburse: None,
        status: Status::Open,
        expires: None,
        reciever: Some(actor.clone()),
        loaner: None
    };

    let loan_id = store_loan(deps.storage, &loan).unwrap();
    (loan, loan_id)
}

pub fn setup_loan_disbursement_offer(
    deps: DepsMut,title: String, 
    actor: &Addr,
    description: String
) -> (Loan, u64) {
    let loan = Loan {
        title,
        description,
        collacteral: None,
        amount_to_disburse: None,
        status: Status::Open,
        expires: None,
        reciever: None,
        loaner: Some(actor.clone()),
    };

    let loan_id = store_loan(deps.storage, &loan).unwrap();
    (loan, loan_id)
}

pub fn accept_loan(
    deps: DepsMut, loan: &Loan,
    actor: &Addr,
) {
}

pub fn disburse_loan(
    deps: DepsMut, loan: &Loan, 
    actor: &Addr
) {
}

fn store_loan(store: &mut dyn Storage, loan: &Loan) -> StdResult<u64> {
    let id = next_id(store)?;
    LOANS.save(store, id, &loan)?;
    Ok(id)
}