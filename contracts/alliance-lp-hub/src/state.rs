use crate::models::Config;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_asset::AssetInfoKey;
use cw_storage_plus::{Item, Map};
use std::collections::HashSet;

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELIST: Map<AssetInfoKey, Decimal> = Map::new("whitelist");

// Used to keep track of all staked tokens
// by total balances and by user balances
// when staking and unstaking this two maps
// must be addedup or subtracted
pub const TOTAL_BALANCES: Map<AssetInfoKey, Uint128> = Map::new("total_balances");
pub const BALANCES: Map<(Addr, AssetInfoKey), Uint128> = Map::new("balances");

pub const VALIDATORS: Item<HashSet<String>> = Item::new("validators");

// The following map is used to store the rewards
// with the following structure:
// - AssetInfoKey: is the asset that is being deposited,
// - AssetInfoKey: is the asset that is being rewarded,
// - Decimal: is the reward rate,
pub const ASSET_REWARD_RATE: Map<(AssetInfoKey, AssetInfoKey), Decimal> =
    Map::new("asset_reward_rate");

// The following map is used to store the user rewards
// with the following structure:
// - Addr: is the address of the user,
// - AssetInfoKey: is the asset that is being deposited,
// - AssetInfoKey: is the asset that is being rewarded,
// - Decimal: is the reward rate,
pub const USER_ASSET_REWARD_RATE: Map<(Addr, AssetInfoKey, AssetInfoKey), Decimal> =
    Map::new("user_asset_reward_rate");

// The following map is used to keep track of the unclaimed rewards
// - Addr: is the address of the user,
// - AssetInfoKey: is the asset that is being deposited,
// - AssetInfoKey: is the asset that is being rewarded,
// - Decimal: is the reward rate,
pub const UNCLAIMED_REWARDS: Map<(Addr, AssetInfoKey, AssetInfoKey), Uint128> = Map::new("unclaimed_rewards");

pub const TEMP_BALANCE: Map<AssetInfoKey, Uint128> = Map::new("temp_balance");

// Temporary variable used to store the user address
// so we can access it on reply_claim_astro_rewards
// callback function and account for the rewards
pub const TEMP_USR_ADDR: Item<Addr> = Item::new("temp_addr_stake");
