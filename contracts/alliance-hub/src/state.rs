use alliance_protocol::alliance_protocol::{AssetDistribution, Config};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_asset::AssetInfoKey;
use cw_storage_plus::{Item, Map};
use std::collections::HashSet;

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELIST: Map<AssetInfoKey, bool> = Map::new("whitelist");

pub const BALANCES: Map<(Addr, AssetInfoKey), Uint128> = Map::new("balances");
pub const TOTAL_BALANCES: Map<AssetInfoKey, Uint128> = Map::new("total_balances");

pub const VALIDATORS: Item<HashSet<Addr>> = Item::new("validators");

// claim with the old distribution
// Update this from rebalance
// if the dist is negative set the distribution to 0
pub const ASSET_REWARD_DISTRIBUTION: Item<Vec<AssetDistribution>> =
    Item::new("asset_reward_distribution");

pub const ASSET_REWARD_RATE: Map<AssetInfoKey, Decimal> = Map::new("asset_reward_rate");
pub const USER_ASSET_REWARD_RATE: Map<(Addr, AssetInfoKey), Decimal> =
    Map::new("user_asset_reward_rate");
pub const UNCLAIMED_REWARDS: Map<Addr, Uint128> = Map::new("unclaimed_rewards");

pub const TEMP_BALANCE: Item<Uint128> = Item::new("temp_balance");
