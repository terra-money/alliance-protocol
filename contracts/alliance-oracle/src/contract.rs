use std::collections::HashMap;
use std::env;

use alliance_protocol::alliance_oracle_types::{
    AssetStaked, ChainId, ChainInfo, ChainsInfo, Config, EmissionsDistribution, ExecuteMsg, Expire,
    InstantiateMsg, MigrateMsg, QueryMsg,
};
use alliance_protocol::signed_decimal::{Sign, SignedDecimal};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{CHAINS_INFO, CONFIG, LUNA_INFO};
use crate::utils;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-alliance-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let controller_addr = deps.api.addr_validate(&msg.controller_addr)?;

    CONFIG.save(
        deps.storage,
        &Config {
            data_expiry_seconds: msg.data_expiry_seconds,
            controller_addr,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("data_expiry_seconds", msg.data_expiry_seconds.to_string())
        .add_attribute("controller_addr", msg.controller_addr))
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
        QueryMsg::QueryChainsInfoUnsafe {} => get_chains_info_unsafe(deps)?,
        QueryMsg::QueryEmissionsDistributions(query) => {
            get_emissions_distribution_info(deps, env, query)?
        }
    })
}

pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let cfg = CONFIG.load(deps.storage)?;

    to_json_binary(&cfg)
}

pub fn get_luna_info(deps: Deps, env: Env) -> StdResult<Binary> {
    let luna_info = LUNA_INFO.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    luna_info.is_expired(cfg.data_expiry_seconds, env.block.time)?;

    to_json_binary(&luna_info)
}

pub fn get_chain_info(deps: Deps, env: Env, chain_id: ChainId) -> StdResult<Binary> {
    let chains_info = CHAINS_INFO.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    for chain_info in &chains_info {
        if chain_info.chain_id == chain_id {
            chain_info.is_expired(cfg.data_expiry_seconds, env.block.time)?;
            return to_json_binary(&chain_info);
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

    to_json_binary(&chains_info)
}

pub fn get_chains_info_unsafe(deps: Deps) -> StdResult<Binary> {
    let chains_info = CHAINS_INFO.load(deps.storage)?;
    to_json_binary(&chains_info)
}

pub fn get_emissions_distribution_info(
    deps: Deps,
    _env: Env,
    chains: HashMap<ChainId, Vec<AssetStaked>>,
) -> StdResult<Binary> {
    // Information posted on chain periodically from oracle-feeder-go
    // https://github.com/terra-money/oracle-feeder-go.
    let chains_info = CHAINS_INFO.load(deps.storage)?;
    let luna = LUNA_INFO.load(deps.storage)?;

    // Incognitas to discover in the first for loop:
    let mut chains_value: Vec<(ChainInfo, SignedDecimal)> = vec![];
    let mut denom_rebase: HashMap<String, Decimal> = HashMap::new();

    // First go through all chains and calculate the average yield for all alliances that accepts LUNA as a staking asset
    for chain_info in chains_info {
        if chains.contains_key(&chain_info.chain_id) {
            // Accumulated amount of USD distributed to the Terra minus
            // the value of LUNA taken by take_rate
            let mut chain_accumulated_value = SignedDecimal::zero();

            for alliance in chain_info.luna_alliances.clone() {
                // Calculate the amount of chain native tokens distributed
                // to the alliance in a year denominated in USD.
                let tokens_distributed_to_alliance = chain_info.native_token.annual_provisions
                    * alliance.normalized_reward_weight
                    * chain_info.native_token.token_price;

                // Calculate the total amount of LUNA staked with this alliance
                // on this chain based on the amount of LSD's staked and their rebase factor.
                let total_luna_staked = alliance.total_lsd_staked * alliance.rebase_factor;

                // Calculate the amount of USD distributed to the Terra minus
                // the value of LUNA taken by take_rate
                let value =
                    SignedDecimal::from_decimal(tokens_distributed_to_alliance, Sign::Positive)
                        - (alliance.annual_take_rate * total_luna_staked * luna.luna_price);

                chain_accumulated_value += value;
            }
            chains_value.push((chain_info.clone(), chain_accumulated_value));

            for alliance in chain_info.chain_alliances_on_phoenix.clone() {
                denom_rebase.insert(alliance.ibc_denom.clone(), alliance.rebase_factor);
            }
        }
    }

    let mut emission_distribution = vec![];
    for (chain_info, chain_value) in chains_value {
        // Get the whitelisted asset base on the function parameter chains.ChainId
        let whitelisted_assets = chains
            .get(&chain_info.chain_id)
            .ok_or(StdError::generic_err(format!(
                "Error getting whitelisted assets for chain {:?}",
                &chain_info.chain_id
            )))?;

        let mut total_staked = Decimal::zero();
        for asset in whitelisted_assets {
            let staked = Decimal::from_atomics(asset.amount, 0).map_err(|_| {
                StdError::generic_err(format!(
                    "Error converting staked amount to decimal for asset {:?}",
                    asset.amount
                ))
            })?;
            total_staked += staked * denom_rebase.get(&asset.denom).unwrap_or(&Decimal::one());
        }
        for asset in whitelisted_assets {
            // If rebase is not set, use 1 as the rebase factor
            let denom_rebase = *denom_rebase.get(&asset.denom).unwrap_or(&Decimal::one());
            let staked_before_rebase = Decimal::from_atomics(asset.amount, 0).map_err(|_| {
                StdError::generic_err(format!(
                    "Error converting staked amount to decimal for asset {:?}",
                    asset.amount
                ))
            })?;
            let staked = staked_before_rebase * denom_rebase;
            let distribution = if staked.is_zero() {
                SignedDecimal::zero()
            } else {
                chain_value * staked / total_staked
            };
            emission_distribution.push(EmissionsDistribution {
                denom: asset.denom.to_string(),
                distribution,
            });
        }
    }

    to_json_binary(&emission_distribution)
}
