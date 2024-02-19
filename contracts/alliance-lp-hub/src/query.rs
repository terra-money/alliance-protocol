use std::collections::HashMap;
use std::str::FromStr;

use crate::models::{
    AddressPendingRewardsQuery, AddressStakedBalancesQuery, AssetQuery, PendingRewardsRes,
    QueryMsg, StakedAssetQuery, StakedBalanceRes,
};
use alliance_protocol::alliance_oracle_types::EmissionsDistribution;
use alliance_protocol::signed_decimal::{Sign, SignedDecimal};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_asset::{AssetInfo, AssetInfoKey, AssetInfoUnchecked};

use crate::state::{
    CONFIG, TOTAL_ASSET_REWARD_RATE, TOTAL_BALANCES, UNCLAIMED_REWARDS, USER_ASSET_REWARD_RATE,
    USER_BALANCES, VALIDATORS, WHITELIST,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(match msg {
        QueryMsg::Config {} => get_config(deps)?,
        QueryMsg::Validators {} => get_validators(deps)?,
        QueryMsg::WhitelistedAssets {} => get_whitelisted_assets(deps)?,
        QueryMsg::AllianceRewardsDistribution {} => get_alliance_rewards_distribution(deps)?,
        QueryMsg::ContractBalances {} => get_total_contract_staked_balances(deps)?,

        QueryMsg::AddressStakedBalances(query) => get_address_staked_balances(deps, query)?,
        QueryMsg::AddressPendingRewards(query) => get_address_pending_rewards(deps, query)?,

        QueryMsg::StakedBalance(asset_query) => get_staked_balances(deps, asset_query)?,
        QueryMsg::PendingRewards(asset_query) => get_pending_rewards(deps, asset_query)?,
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
    let mut res: Vec<AssetInfo> = vec![];

    for item in whitelist {
        let (key, _) = item?;
        let asset = key.check(deps.api, None)?;

        res.push(asset)
    }

    to_json_binary(&res)
}

fn get_alliance_rewards_distribution(deps: Deps) -> StdResult<Binary> {
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

fn get_staked_balances(deps: Deps, asset_query: StakedAssetQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let key = (addr, AssetInfoKey::from(asset_query.deposit_asset.clone()));
    let balance = USER_BALANCES.load(deps.storage, key).unwrap_or_default();

    to_json_binary(&StakedBalanceRes {
        deposit_asset: asset_query.deposit_asset,
        balance,
    })
}

fn get_pending_rewards(deps: Deps, asset_query: AssetQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let deposit_asset = AssetInfoKey::from(asset_query.deposit_asset.clone());
    let reward_asset = AssetInfoKey::from(asset_query.reward_asset.clone());

    let user_balance = USER_BALANCES
        .load(deps.storage, (addr.clone(), deposit_asset.clone()))
        .unwrap_or_default();
    let asset_reward_rate = TOTAL_ASSET_REWARD_RATE
        .load(deps.storage, (deposit_asset.clone(), reward_asset.clone()))
        .unwrap_or_default();
    let key = (addr.clone(), deposit_asset.clone(), reward_asset.clone());
    let user_reward_rate = USER_ASSET_REWARD_RATE
        .load(deps.storage, key)
        .unwrap_or_default();
    let unclaimed_rewards = UNCLAIMED_REWARDS
        .load(deps.storage, (addr, deposit_asset, reward_asset))
        .unwrap_or_default();

    let pending_rewards = (asset_reward_rate - user_reward_rate) * user_balance;

    to_json_binary(&PendingRewardsRes {
        deposit_asset: asset_query.deposit_asset,
        reward_asset: asset_query.reward_asset,
        rewards: pending_rewards + unclaimed_rewards,
    })
}

fn get_address_staked_balances(
    deps: Deps,
    asset_query: AddressStakedBalancesQuery,
) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let whitelist = WHITELIST.range(deps.storage, None, None, Order::Ascending);
    let mut res: Vec<StakedBalanceRes> = Vec::new();

    for asset_res in whitelist {
        // Build the required key to recover the BALANCES
        let (asset_key, _) = asset_res?;
        let checked_asset_info = asset_key.check(deps.api, None)?;
        let asset_info_key = AssetInfoKey::from(checked_asset_info.clone());
        let stake_key = (addr.clone(), asset_info_key);

        let balance = USER_BALANCES
            .load(deps.storage, stake_key)
            .unwrap_or_default();

        // Append the request
        res.push(StakedBalanceRes {
            deposit_asset: checked_asset_info,
            balance,
        })
    }

    to_json_binary(&res)
}

fn get_address_pending_rewards(deps: Deps, query: AddressPendingRewardsQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&query.address)?;

    // Iterate over user balances, map and filter the 0 stake balances
    // because if the user has no deposit it means that will not have
    // any rewards so it we can save some gas costs. Returned structure:
    // <DepositedAssetAddress, AmountDeposited> = <native:uluna, 200>
    let user_balances: HashMap<String, Uint128> = USER_BALANCES
        .prefix(addr.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|f| match f {
            Ok((k, v)) => k
                .check(deps.api, None)
                .ok()
                .map(|asset_info| (asset_info.to_string(), v)),
            Err(_) => None,
        })
        .collect();

    // Iterate over the user balances and get the ASSET_REWARD_RATE for each
    // deposited asset even if the asset has 0 reward rate, it is useful for further
    // operations to recover the pending rewards and calculate the full rewards.
    // Returned structure: <DepositedAssetAddress, RewardAssetAddress ,AmountDeposited>
    let mut assets_reward_rate: HashMap<(String, String), Decimal> = HashMap::new();
    for (deposit_asset, _) in user_balances.iter() {
        let deposit_asset_info = AssetInfoUnchecked::from_str(deposit_asset).unwrap();

        let arr: HashMap<(String, String), Decimal> = TOTAL_ASSET_REWARD_RATE
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|f| {
                let ((deposit_asset_info_base, reward_asset_info_base), asset_reward_rate) =
                    f.unwrap();
                if deposit_asset_info_base == deposit_asset_info {
                    let reward_asset = reward_asset_info_base
                        .check(deps.api, None)
                        .unwrap()
                        .to_string();

                    Some(((deposit_asset.clone(), reward_asset), asset_reward_rate))
                } else {
                    None
                }
            })
            .collect();

        assets_reward_rate.extend(arr);
    }

    // Iterate over all the collected data and do similar operations to the function get_pending_rewards
    // but in this case we determine the inputs based on the two previous datasets user_balances and assets_reward_rate
    let all_pending_rewards: Vec<PendingRewardsRes> = assets_reward_rate
        .into_iter()
        .map(|f| {
            let ((deposit_asset, reward_asset), asset_reward_rate) = f;

            let deposit_asset_info = AssetInfoUnchecked::from_str(&deposit_asset)
                .unwrap()
                .check(deps.api, None)
                .unwrap();
            let deposit_asset_key = AssetInfoKey::from(deposit_asset_info.clone());

            let reward_asset_info = AssetInfoUnchecked::from_str(&reward_asset)
                .unwrap()
                .check(deps.api, None)
                .unwrap();
            let reward_asset_key = AssetInfoKey::from(reward_asset_info.clone());

            let user_balance = *user_balances.get(&deposit_asset).unwrap();
            let key = (addr.clone(), deposit_asset_key, reward_asset_key);
            let user_reward_rate = USER_ASSET_REWARD_RATE
                .load(deps.storage, key.clone())
                .unwrap_or_default();

            let unclaimed_rewards = UNCLAIMED_REWARDS
                .load(deps.storage, key)
                .unwrap_or_default();
            let pending_rewards = (asset_reward_rate - user_reward_rate) * user_balance;

            PendingRewardsRes {
                deposit_asset: deposit_asset_info,
                reward_asset: reward_asset_info,
                rewards: pending_rewards + unclaimed_rewards,
            }
        })
        .collect();

    to_json_binary(&all_pending_rewards)
}

fn get_total_contract_staked_balances(deps: Deps) -> StdResult<Binary> {
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
