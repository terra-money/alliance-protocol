use alliance_protocol::alliance_oracle_types::{ChainId, ChainInfo, Config, LunaInfo};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHAINS_INFO: Map<ChainId, ChainInfo> = Map::new("chains_info");
pub const LUNA_INFO: Item<LunaInfo> = Item::new("luna_info");
