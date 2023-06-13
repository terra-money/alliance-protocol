#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use alliance_protocol::alliance_protocol::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-alliance-protocol";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Stake => {}
        ExecuteMsg::Unstake(_) => {}
        ExecuteMsg::ClaimRewards(_) => {}
        ExecuteMsg::WhitelistAsset(_) => {}
        ExecuteMsg::RemoveAsset(_) => {}
        ExecuteMsg::UpdateRewards => {}
        ExecuteMsg::AllianceDelegate(_) => {}
        ExecuteMsg::AllianceUndelegate(_) => {}
        ExecuteMsg::AllianceRedelegate(_) => {}
        ExecuteMsg::RebalanceEmissions => {}
    }
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => {}
        QueryMsg::WhitelistedCoins => {}
        QueryMsg::RewardDistribution => {}
        QueryMsg::StakedBalance(_) => {}
        QueryMsg::PendingRewards(_) => {}
    }
    Ok(Binary::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
    }
}
