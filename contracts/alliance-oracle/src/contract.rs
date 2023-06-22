use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::env;
use std::hash::Hash;

use alliance_protocol::alliance_oracle_types::{
    AssetStaked, ChainId, ChainInfo, ChainsInfo, Config, EmissionsDistribution, ExecuteMsg, Expire,
    InstantiateMsg, QueryMsg,
};
use alliance_protocol::signed_decimal::{Sign, SignedDecimal};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Decimal256, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{CHAINS_INFO, CONFIG, LUNA_INFO};
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
            data_expiry_seconds: msg.data_expiry_seconds,
            governance_addr,
            controller_addr,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("data_expiry_seconds", msg.data_expiry_seconds.to_string())
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
    chains_info: ChainsInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    utils::authorize_execution(config, info.sender)?;
    let mut parsed_chains_info: Vec<ChainInfo> = vec![];

    for chain_info in &chains_info.protocols_info {
        let chain_info = chain_info.to_chain_info(env.block.time);

        parsed_chains_info.push(chain_info);
    }

    let luna_info = chains_info.to_luna_info(env.block.time);
    LUNA_INFO.save(deps.storage, &luna_info)?;
    CHAINS_INFO.save(deps.storage, &parsed_chains_info)?;

    Ok(Response::new().add_attribute("action", "update_chains_info"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(match msg {
        QueryMsg::QueryConfig {} => get_config(deps)?,
        QueryMsg::QueryLunaInfo {} => get_luna_info(deps, env)?,
        QueryMsg::QueryChainInfo { chain_id } => get_chain_info(deps, env, chain_id)?,
        QueryMsg::QueryChainsInfo {} => get_chains_info(deps, env)?,
        QueryMsg::QueryChainsInfoUnsafe {} => get_chains_info_unsafe(deps, env)?,
        QueryMsg::QueryEmissionsDistributions(query) => {
            get_emissions_distribution_info(deps, env, query)?
        }
    })
}

pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let cfg = CONFIG.load(deps.storage)?;

    to_binary(&cfg)
}

pub fn get_luna_info(deps: Deps, env: Env) -> StdResult<Binary> {
    let luna_info = LUNA_INFO.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    luna_info.is_expired(cfg.data_expiry_seconds, env.block.time)?;

    to_binary(&luna_info)
}

pub fn get_chain_info(deps: Deps, env: Env, chain_id: ChainId) -> StdResult<Binary> {
    let chains_info = CHAINS_INFO.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    for chain_info in &chains_info {
        if chain_info.chain_id == chain_id {
            chain_info.is_expired(cfg.data_expiry_seconds, env.block.time)?;
            return to_binary(&chain_info);
        }
    }

    let string_error = format!("Chain not available by id: {:?}", chain_id);
    Err(StdError::generic_err(string_error))
}

pub fn get_chains_info(deps: Deps, env: Env) -> StdResult<Binary> {
    let chains_info = CHAINS_INFO.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    for chain_info in &chains_info {
        chain_info.is_expired(cfg.data_expiry_seconds, env.block.time)?;
    }

    to_binary(&chains_info)
}

pub fn get_chains_info_unsafe(deps: Deps, env: Env) -> StdResult<Binary> {
    let chains_info = CHAINS_INFO.load(deps.storage)?;
    to_binary(&chains_info)
}

pub fn get_emissions_distribution_info(
    deps: Deps,
    _env: Env,
    chains: HashMap<ChainId, Vec<AssetStaked>>,
) -> StdResult<Binary> {
    let chains_info = CHAINS_INFO.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;
    let luna = LUNA_INFO.load(deps.storage)?;

    let mut chain_aprs: Vec<(ChainInfo, SignedDecimal)> = vec![];
    let mut denom_rebase: HashMap<String, Decimal> = HashMap::new();

    // First go through all chains and calculate the average yield for all alliances that accepts LUNA as a staking asset
    for chain_info in chains_info.clone() {
        if chains.contains_key(&chain_info.chain_id) {
            let mut apr_sum = SignedDecimal::zero();
            let mut total_staked = SignedDecimal::zero();
            for alliance in chain_info.luna_alliances.clone() {
                let numerator = chain_info.native_token.annual_provisions
                    * alliance.normalized_reward_weight
                    * chain_info.native_token.token_price;
                let staked = alliance.total_lsd_staked * alliance.rebase_factor;
                let apr = SignedDecimal::from_decimal(numerator / luna.luna_price, Sign::Positive)
                    - (alliance.annual_take_rate * staked);
                apr_sum += apr;
                total_staked += staked;
            }
            let average_apr = if total_staked.is_zero() {
                SignedDecimal::zero()
            } else {
                apr_sum / total_staked
            };
            chain_aprs.push((chain_info.clone(), average_apr));

            for alliance in chain_info.chain_alliances_on_phoenix.clone() {
                denom_rebase.insert(alliance.ibc_denom.clone(), alliance.rebase_factor);
            }
        }
    }

    let mut emission_distribution = vec![];
    for (chain_info, apr) in chain_aprs {
        let whitelisted_assets = chains
            .get(&chain_info.chain_id)
            .ok_or(StdError::generic_err(format!(
                "Error getting whitelisted assets for chain {:?}",
                &chain_info.chain_id
            )))?;
        let total_staked = whitelisted_assets
            .iter()
            .fold(Decimal::zero(), |acc, asset| {
                acc + Decimal::from_atomics(asset.amount, 0).unwrap_or(
                    Decimal::zero() * denom_rebase.get(&asset.denom).unwrap_or(&Decimal::one()),
                )
            });
        for asset in whitelisted_assets {
            let staked = Decimal::from_atomics(asset.amount, 0).unwrap_or(Decimal::zero())
                * denom_rebase
                    .get(&asset.denom)
                    .unwrap_or(&Decimal::one())
                    .clone();
            let distribution = if staked.is_zero() {
                SignedDecimal::zero()
            } else {
                apr * staked / total_staked
            };
            emission_distribution.push(EmissionsDistribution {
                denom: asset.denom.to_string(),
                distribution,
            });
        }
    }

    to_binary(&emission_distribution)
}
