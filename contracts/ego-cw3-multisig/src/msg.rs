use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Empty};
use cw3::UncheckedDepositInfo;

use crate::state::Executor;

#[cw_serde]
pub struct InstantiateMsg {
    // this is the group contract that contains the member list
    pub group_addr: String,
    pub executor: Option<Executor>
}

#[cw_serde]
pub enum ExecuteMsg {
    ProposeLoanOffer {
        loan_offer: LoanOffer,
        title: String,
        description: String,
        loan_deposit: Option<UncheckedDepositInfo>
    },
    ExecuteLoanOffer {
        loan_action: LoanAction,
        loan_id: u64,
        msgs: Vec<CosmosMsg<Empty>>,
        loan_deposit: Option<UncheckedDepositInfo>
    },
    Close {
        loan_id: u64,
    },
    //// Handles update hook messages from the group contract
    // MemberChangedHook(MemberChangedHookMsg),
}

#[cw_serde]
pub enum LoanOffer {
    LoanRequest,
    LoanDisbursmentRequest
}

#[cw_serde]
pub enum LoanAction {
    AcceptLoan,
    DisburseLoan
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw_utils::ThresholdResponse)]
    Threshold {},
    #[returns(cw3::ProposalResponse)]
    Proposal { proposal_id: u64 },
    #[returns(cw3::ProposalListResponse)]
    ListProposals {
        loan_offer: Option<LoanOffer>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::ProposalListResponse)]
    ReverseProposals {
        start_before: Option<u64>,
        limit: Option<u32>,
    },
    /// Gets the current configuration.
    #[returns(crate::state::Config)]
    Config {},
}
