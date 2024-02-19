use alliance_protocol::error::ContractError;
use cosmwasm_std::{Addr, MessageInfo};
use cw_asset::{AssetInfo, AssetInfoKey};

use crate::models::Config;

// Controller is used to perform administrative operations that deals with delegating the virtual
// tokens to the expected validators
pub fn is_controller(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.controller_addr {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

// Only governance (through a on-chain prop) can change the whitelisted assets
pub fn is_governance(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.governance_addr {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn from_string_to_asset_info(denom: String) -> Result<AssetInfoKey, ContractError> {
    if denom.starts_with("ibc/") || denom.starts_with("factory/") {
        let asset_info = AssetInfoKey::from(AssetInfo::Native(denom));
        return Ok(asset_info);
    } else if denom.starts_with("terra") {
        let from = Addr::unchecked(denom);
        let asset_info = AssetInfoKey::from(AssetInfo::Cw20(from));
        return Ok(asset_info);
    }

    Err(ContractError::InvalidDenom(denom))
}

pub fn get_string_without_prefix(asset_info: AssetInfo) -> String {
    return asset_info.to_string().split(':').collect::<Vec<&str>>()[1].to_string();
}
