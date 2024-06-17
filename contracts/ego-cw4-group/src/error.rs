use cosmwasm_std::{StdError, OverflowError};
use thiserror::Error;

use cw_controllers::{AdminError, HookError};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Message contained duplicate member: {member}")]
    DuplicateMember { member: String },
}
