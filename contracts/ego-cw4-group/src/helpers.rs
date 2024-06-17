use cw4::Member;

use crate::ContractError;


pub fn validate_unique_members(members: &mut [Member]) -> Result<(), ContractError> {
    members.sort_by(|a, b| a.addr.cmp(&b.addr));
    for (a, b) in members.iter().zip(members.iter().skip(1)) {
        if a.addr == b.addr {
            return Err(ContractError::DuplicateMember {
                member: a.addr.clone(),
            });
        }
    }

    Ok(())
}