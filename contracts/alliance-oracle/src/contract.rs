use alliance_oracle::alliance_oracle::{ExecuteMsg, InstantiateMsg, QueryMsg, ChainInfo};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-alliance-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let controller_address = deps.api.addr_validate(&msg.controller_address)?;

    CONFIG.save(deps.storage, &Config {
        controller_address,
    })?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateChainsInfo{chains_info} => update_chains_info(deps, env, info, chains_info),
    }
}

fn update_chains_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<ChainInfo>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.controller_address {
        return Err(ContractError::Unauthorized {});
    }
    
    Ok(Response::new()
        .add_attribute("action", "update_chains_info"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => {
            return to_binary(&CONFIG.load(deps.storage)?);
        },
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info, mock_env};

    use super::*;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            controller_address: "controller_address".to_string(),
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
