use cosmwasm_std::Addr;

use crate::{state::Config, error::ContractError};


pub fn authorize_execution(config: Config ,addr: Addr) ->  Result<(), ContractError> {
    if addr != config.controller_addr || 
        addr != config.governance_addr {
        return Err(ContractError::Unauthorized {});
    }
    
    return Ok(())
}