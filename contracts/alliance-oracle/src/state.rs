use alliance_protocol::alliance_oracle_types::{ChainId, ChainInfo};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub governance_addr: Addr,
    pub controller_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHAINS_INFO: Map<ChainId, ChainInfo> = Map::new("chains_info");
