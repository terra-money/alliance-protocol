use cosmwasm_std::Addr;

use crate::{error::ContractError, state::Config};

pub fn authorize_execution(config: Config, addr: Addr) -> Result<(), ContractError> {
    if addr != config.controller_addr || addr != config.governance_addr {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}
