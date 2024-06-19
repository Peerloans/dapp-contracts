
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Storage};
use cw3::DepositInfo;
use cw4::Cw4Contract;
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

use crate::error::ContractError;

/// Defines who is able to execute proposals once passed
#[cw_serde]
pub enum Executor {
    /// Any member of the voting group, even with 0 points
    Member,
    /// Only the given address
    Only(Addr),
}

#[cw_serde]
pub struct Config {
    pub group_addr: Cw4Contract,
    pub executor: Option<Executor>,
}

impl Config {
    // Executor can be set in 3 ways:
    // - Member: any member of the voting group is authorized
    // - Only: only passed address is authorized
    // - None: Everyone are authorized
    pub fn authorize(&self, querier: &QuerierWrapper, sender: &Addr) -> Result<(), ContractError> {
        if let Some(executor) = &self.executor {
            match executor {
                Executor::Member => {
                    self.group_addr
                        .is_member(querier, sender, None)?
                        .ok_or(ContractError::Unauthorized {})?;
                }
                Executor::Only(addr) => {
                    if addr != sender {
                        return Err(ContractError::Unauthorized {});
                    }
                }
            }
        }
        Ok(())
    }
}

#[cw_serde]
pub struct Loan {
    pub title: String,
    pub description: String,

    pub collacteral: Option<DepositInfo>,
    pub amount_to_disburse: Option<DepositInfo>,
    pub status: Status,
    pub expires: Option<Expiration>,
    
    pub reciever: Option<Addr>,
    pub loaner: Option<Addr>
}

#[cw_serde]
pub enum Status {
    Open,
    Close,
    Active
}

// unique items
pub const CONFIG: Item<Config> = Item::new("config");
pub const LOAN_COUNT: Item<u64> = Item::new("loan_count");

// multiple-item map
pub const LOANS: Map<u64, Loan> = Map::new("loans");

pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = LOAN_COUNT.may_load(store)?.unwrap_or_default() + 1;
    LOAN_COUNT.save(store, &id)?;
    Ok(id)
}