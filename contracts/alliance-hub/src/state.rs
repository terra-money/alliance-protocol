use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_asset::{AssetInfo, AssetInfoKey};
use cw_storage_plus::{Item, Map};
use std::collections::HashSet;

#[cw_serde]
pub struct Config {
    pub governance: Addr,
    pub controller: Addr,
    pub oracle: Addr,
    pub last_reward_update_timestamp: Timestamp,
    pub alliance_token_denom: String,
    pub alliance_token_supply: Uint128,
    pub reward_denom: String,
}

#[cw_serde]
pub struct AssetDistribution {
    pub asset: AssetInfo,
    pub distribution: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELIST: Map<AssetInfoKey, bool> = Map::new("whitelist");

pub const BALANCES: Map<(Addr, AssetInfoKey), Uint128> = Map::new("balances");
pub const TOTAL_BALANCES: Map<AssetInfoKey, Uint128> = Map::new("total_balances");

pub const VALIDATORS: Item<HashSet<Addr>> = Item::new("validators");

pub const ASSET_REWARD_DISTRIBUTION: Item<Vec<AssetDistribution>> =
    Item::new("asset_reward_distribution");
pub const ASSET_REWARD_RATE: Map<AssetInfoKey, Decimal> = Map::new("asset_reward_rate");
pub const USER_ASSET_REWARD_RATE: Map<(Addr, AssetInfoKey), Decimal> =
    Map::new("user_asset_reward_rate");
pub const UNCLAIMED_REWARDS: Map<Addr, Uint128> = Map::new("unclaimed_rewards");

pub const TEMP_BALANCE: Item<Uint128> = Item::new("temp_balance");
