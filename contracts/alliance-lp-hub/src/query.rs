use crate::models::{
    AllPendingRewardsQuery, AllStakedBalancesQuery, AssetQuery, PendingRewardsRes, QueryMsg,
    StakedBalanceRes, WhitelistedAssetsResponse,
};
use alliance_protocol::alliance_oracle_types::EmissionsDistribution;
use alliance_protocol::signed_decimal::{Sign, SignedDecimal};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_asset::{AssetInfo, AssetInfoKey, AssetInfoUnchecked};
use std::collections::HashMap;

use crate::state::{
    ASSET_REWARD_RATE, BALANCES, CONFIG, TOTAL_BALANCES, UNCLAIMED_REWARDS, USER_ASSET_REWARD_RATE,
    VALIDATORS, WHITELIST,
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
        QueryMsg::AllStakedBalances(query) => get_all_staked_balances(deps, query)?,
        QueryMsg::AllPendingRewards(query) => get_all_pending_rewards(deps, query)?,
        QueryMsg::TotalStakedBalances {} => get_total_staked_balances(deps)?,
    })
}

fn get_config(deps: Deps) -> StdResult<Binary> {
    let cfg = CONFIG.load(deps.storage)?;

    to_json_binary(&cfg)
}

fn get_validators(deps: Deps) -> StdResult<Binary> {
    let validators = VALIDATORS.load(deps.storage)?;

    to_json_binary(&validators)
}

fn get_whitelisted_assets(deps: Deps) -> StdResult<Binary> {
    let whitelist = WHITELIST.range(deps.storage, None, None, Order::Ascending);
    let mut res: WhitelistedAssetsResponse = HashMap::new();

    for item in whitelist {
        let (key, _) = item?;
        let asset = key.check(deps.api, None)?;

        res.entry(asset.to_string()).or_default().push(asset)
    }

    to_json_binary(&res)
}

fn get_rewards_distribution(deps: Deps) -> StdResult<Binary> {
    let whitelist: StdResult<Vec<(AssetInfoUnchecked, Decimal)>> = WHITELIST
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let whitelist = whitelist?;

    let reward_distribution: Vec<EmissionsDistribution> = whitelist
        .iter()
        .map(|(asset_info, distribution)| EmissionsDistribution {
            denom: asset_info.check(deps.api, None).unwrap().to_string(),
            distribution: SignedDecimal::from_decimal(*distribution, Sign::Positive),
        })
        .collect();
    to_json_binary(&reward_distribution)
}

fn get_staked_balance(deps: Deps, asset_query: AssetQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let key = (addr, asset_query.deposit_asset.clone().into());
    let balance = BALANCES.load(deps.storage, key)?;

    to_json_binary(&StakedBalanceRes {
        deposit_asset: asset_query.deposit_asset,
        balance,
    })
}

fn get_pending_rewards(deps: Deps, asset_query: AssetQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let reward_asset = AssetInfoKey::from(asset_query.reward_asset.clone());
    let deposit_asset = AssetInfoKey::from(asset_query.deposit_asset.clone());

    let key = (addr.clone(), deposit_asset.clone(), reward_asset.clone());
    let user_reward_rate = USER_ASSET_REWARD_RATE.load(deps.storage, key)?;

    let asset_reward_rate =
        ASSET_REWARD_RATE.load(deps.storage, (deposit_asset.clone(), reward_asset.clone()))?;
    let user_balance = BALANCES.load(deps.storage, (addr.clone(), deposit_asset).clone())?;
    let unclaimed_rewards = UNCLAIMED_REWARDS
        .load(deps.storage, (addr, reward_asset))
        .unwrap_or_default();

    let pending_rewards = (asset_reward_rate - user_reward_rate) * user_balance;

    to_json_binary(&PendingRewardsRes {
        deposit_asset: asset_query.deposit_asset,
        reward_asset: asset_query.reward_asset,
        rewards: pending_rewards + unclaimed_rewards,
    })
}

fn get_all_staked_balances(deps: Deps, asset_query: AllStakedBalancesQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let whitelist = WHITELIST.range(deps.storage, None, None, Order::Ascending);
    let mut res: Vec<StakedBalanceRes> = Vec::new();

    for asset_res in whitelist {
        // Build the required key to recover the BALANCES
        let (asset_key, _) = asset_res?;
        let checked_asset_info = asset_key.check(deps.api, None)?;
        let asset_info_key = AssetInfoKey::from(checked_asset_info.clone());
        let stake_key = (addr.clone(), asset_info_key);
        let balance = BALANCES
            .load(deps.storage, stake_key)
            .unwrap_or(Uint128::zero());

        // Append the request
        res.push(StakedBalanceRes {
            deposit_asset: checked_asset_info,
            balance,
        })
    }

    to_json_binary(&res)
}

fn get_all_pending_rewards(deps: Deps, query: AllPendingRewardsQuery) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&query.address)?;
    let all_pending_rewards: StdResult<Vec<PendingRewardsRes>> = USER_ASSET_REWARD_RATE
        .sub_prefix(addr.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (assets, user_reward_rate) = item?;

            let deposit_asset = AssetInfoKey::from(assets.0.check(deps.api, None)?);
            let reward_asset = AssetInfoKey::from(assets.1.check(deps.api, None)?);

            let asset_reward_rate = ASSET_REWARD_RATE
                .load(deps.storage, (deposit_asset.clone(), reward_asset.clone()))?;
            let user_balance =
                BALANCES.load(deps.storage, (addr.clone(), deposit_asset.clone()))?;
            let unclaimed_rewards = UNCLAIMED_REWARDS
                .load(deps.storage, (addr.clone(), deposit_asset))
                .unwrap_or_default();
            let alliance_pending_rewards = (asset_reward_rate - user_reward_rate) * user_balance;

            Ok(PendingRewardsRes {
                rewards: alliance_pending_rewards + unclaimed_rewards,
                deposit_asset: assets.0.check(deps.api, None)?,
                reward_asset: AssetInfo::Native(config.alliance_reward_denom.to_string()),
            })
        })
        .collect::<StdResult<Vec<PendingRewardsRes>>>();

    to_json_binary(&all_pending_rewards?)
}

fn get_total_staked_balances(deps: Deps) -> StdResult<Binary> {
    let total_staked_balances: StdResult<Vec<StakedBalanceRes>> = TOTAL_BALANCES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|total_balance| -> StdResult<StakedBalanceRes> {
            let (asset, balance) = total_balance?;
            Ok(StakedBalanceRes {
                deposit_asset: asset.check(deps.api, None)?,
                balance,
            })
        })
        .collect();
    to_json_binary(&total_staked_balances?)
}
