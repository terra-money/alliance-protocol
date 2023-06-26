use alliance_protocol::alliance_protocol::{
    AllPendingRewardsQuery, AssetQuery, PendingRewardsRes, QueryMsg, StakedBalanceRes,
    WhitelistedAssetsResponse,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult, Uint128};
use cw_asset::{AssetInfo, AssetInfoKey};
use std::collections::HashMap;

use crate::state::{
    ASSET_REWARD_DISTRIBUTION, ASSET_REWARD_RATE, BALANCES, CONFIG, UNCLAIMED_REWARDS,
    USER_ASSET_REWARD_RATE, VALIDATORS, WHITELIST,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(match msg {
        QueryMsg::Config {} => get_config(deps)?,
        QueryMsg::Validators {} => get_validators(deps)?,
        QueryMsg::WhitelistedAssets {} => get_whitelisted_assets(deps)?,
        QueryMsg::RewardDistribution {} => get_rewards_distribution(deps)?,
        QueryMsg::StakedBalance(asset_query) => get_staked_balance(deps, asset_query)?,
        QueryMsg::PendingRewards(asset_query) => get_pending_rewards(deps, asset_query)?,
        QueryMsg::AllPendingRewards(query) => get_all_pending_rewards(deps, query)?,
    })
}

fn get_config(deps: Deps) -> StdResult<Binary> {
    let cfg = CONFIG.load(deps.storage)?;

    to_binary(&cfg)
}

fn get_validators(deps: Deps) -> StdResult<Binary> {
    let validators = VALIDATORS.load(deps.storage)?;

    to_binary(&validators)
}

fn get_whitelisted_assets(deps: Deps) -> StdResult<Binary> {
    let whitelist = WHITELIST.range(deps.storage, None, None, Order::Ascending);
    let mut res: WhitelistedAssetsResponse = HashMap::new();

    for item in whitelist {
        let (key, chain_id) = item?;
        let asset = key.check(deps.api, None)?;
        res.entry(chain_id).or_insert_with(Vec::new).push(asset)
    }

    to_binary(&res)
}

fn get_rewards_distribution(deps: Deps) -> StdResult<Binary> {
    let asset_rewards_distr = ASSET_REWARD_DISTRIBUTION.load(deps.storage)?;

    to_binary(&asset_rewards_distr)
}

fn get_staked_balance(deps: Deps, asset_query: AssetQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let key = (addr, asset_query.asset.clone().into());
    let balance = BALANCES.load(deps.storage, key)?;

    to_binary(&StakedBalanceRes {
        asset: asset_query.asset,
        balance,
    })
}

fn get_pending_rewards(deps: Deps, asset_query: AssetQuery) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let key = (addr, AssetInfoKey::from(asset_query.asset.clone()));
    let user_reward_rate = USER_ASSET_REWARD_RATE.load(deps.storage, key.clone())?;
    let asset_reward_rate =
        ASSET_REWARD_RATE.load(deps.storage, AssetInfoKey::from(asset_query.asset.clone()))?;
    let user_balance = BALANCES.load(deps.storage, key.clone())?;
    let unclaimed_rewards = UNCLAIMED_REWARDS
        .load(deps.storage, key)
        .unwrap_or(Uint128::zero());
    let pending_rewards = (asset_reward_rate - user_reward_rate) * user_balance;

    to_binary(&PendingRewardsRes {
        rewards: unclaimed_rewards + pending_rewards,
        staked_asset: asset_query.asset,
        reward_asset: AssetInfo::Native(config.reward_denom),
    })
}

fn get_all_pending_rewards(deps: Deps, query: AllPendingRewardsQuery) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&query.address)?;
    let all_pending_rewards: StdResult<Vec<PendingRewardsRes>> = USER_ASSET_REWARD_RATE
        .prefix(addr.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (asset, user_reward_rate) = item?;
            let asset = asset.check(deps.api, None)?;
            let asset_reward_rate =
                ASSET_REWARD_RATE.load(deps.storage, AssetInfoKey::from(asset.clone()))?;
            let user_balance = BALANCES.load(
                deps.storage,
                (addr.clone(), AssetInfoKey::from(asset.clone())),
            )?;
            let unclaimed_rewards = UNCLAIMED_REWARDS
                .load(
                    deps.storage,
                    (addr.clone(), AssetInfoKey::from(asset.clone())),
                )
                .unwrap_or(Uint128::zero());
            let pending_rewards = (asset_reward_rate - user_reward_rate) * user_balance;
            Ok(PendingRewardsRes {
                rewards: pending_rewards + unclaimed_rewards,
                staked_asset: asset,
                reward_asset: AssetInfo::Native(config.reward_denom.to_string()),
            })
        })
        .collect::<StdResult<Vec<PendingRewardsRes>>>();

    to_binary(&all_pending_rewards?)
}
