use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_asset::AssetInfoKey;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub governance_address: Addr,
    pub controller_address: Addr,
    pub last_reward_update_timestamp: Timestamp,
    pub alliance_token_denom: String,
    pub alliance_token_supply: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELIST: Map<AssetInfoKey, bool> = Map::new("whitelist");
pub const BALANCES: Map<(Addr, AssetInfoKey), Uint128> = Map::new("balances");
