use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_asset::AssetInfoKey;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub controller_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
