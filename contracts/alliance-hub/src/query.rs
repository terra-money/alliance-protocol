use alliance_protocol::alliance_protocol::{
    AssetQuery, PendingRewardsRes, QueryMsg, StakedBalanceRes,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
use cw_asset::AssetInfoBase;

use crate::state::{
    ASSET_REWARD_DISTRIBUTION, BALANCES, CONFIG, USER_ASSET_REWARD_RATE, WHITELIST,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(match msg {
        QueryMsg::Config {} => get_config(deps)?,
        QueryMsg::WhitelistedAssets {} => get_whitelisted_assets(deps)?,
        QueryMsg::RewardDistribution {} => get_rewards_distribution(deps)?,
        QueryMsg::StakedBalance(asset_query) => get_staked_balance(deps, asset_query)?,
        QueryMsg::PendingRewards(asset_query) => get_pending_rewards(deps, asset_query)?,
    })
}

pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let cfg = CONFIG.load(deps.storage)?;

    to_binary(&cfg)
}

pub fn get_whitelisted_assets(deps: Deps) -> StdResult<Binary> {
    let whitelist = WHITELIST.range(deps.storage, None, None, Order::Ascending);
    let mut res: Vec<AssetInfoBase<String>> = Vec::new();

    for item in whitelist {
        let (key, _) = item?;
        res.push(key);
    }

    to_binary(&res)
}

pub fn get_rewards_distribution(deps: Deps) -> StdResult<Binary> {
    let asset_rewards_distr = ASSET_REWARD_DISTRIBUTION.load(deps.storage)?;

    to_binary(&asset_rewards_distr)
}

pub fn get_staked_balance(deps: Deps, asset_query: AssetQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let key = (addr, asset_query.asset.clone().into());
    let balance = BALANCES.load(deps.storage, key)?;

    to_binary(&StakedBalanceRes::new(asset_query.asset, balance))
}

pub fn get_pending_rewards(deps: Deps, asset_query: AssetQuery) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&asset_query.address)?;
    let key = (addr, asset_query.asset.clone().into());
    let pending_rewards = USER_ASSET_REWARD_RATE.load(deps.storage, key)?;

    to_binary(&PendingRewardsRes::new(asset_query.asset, pending_rewards))
}
