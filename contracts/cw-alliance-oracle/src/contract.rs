use alliance_protocol::alliance_oracle_types::{
    ChainId, ChainInfo, ChainInfoMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{Config, CHAINS_INFO, CONFIG};
use crate::utils;

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
    let controller_addr = deps.api.addr_validate(&msg.controller_addr)?;
    let governance_addr = deps.api.addr_validate(&msg.governance_addr)?;

    CONFIG.save(
        deps.storage,
        &Config {
            governance_addr,
            controller_addr,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("controller_addr", msg.controller_addr)
        .add_attribute("governance_addr", msg.governance_addr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateChainsInfo { chains_info } => {
            update_chains_info(deps, env, info, chains_info)
        }
    }
}

fn update_chains_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    chains_info: Vec<ChainInfoMsg>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    utils::authorize_execution(config, info.sender)?;

    for chain_info in chains_info {
        let (chain_id, chain_info) = chain_info.to_chain_info(env.block.time);
        CHAINS_INFO.save(deps.storage, chain_id, &chain_info)?;
    }

    Ok(Response::new().add_attribute("action", "update_chains_info"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::ChainInfo { chain_id } => to_binary(&CHAINS_INFO.load(deps.storage, chain_id)?),
        QueryMsg::ChainsInfo => {
            let items = CHAINS_INFO
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<(ChainId, ChainInfo)>>>()?;

            to_binary(&items)
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            controller_addr: "controller_addr".to_string(),
            governance_addr: "governance_addr".to_string(),
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
