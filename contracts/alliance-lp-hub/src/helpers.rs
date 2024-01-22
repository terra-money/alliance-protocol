use alliance_protocol::error::ContractError;
use cosmwasm_std::MessageInfo;

use crate::models::Config;


// Controller is used to perform administrative operations that deals with delegating the virtual
// tokens to the expected validators
pub fn is_controller(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.controller {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

// Only governance (through a on-chain prop) can change the whitelisted assets
pub fn is_governance(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.governance {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}
