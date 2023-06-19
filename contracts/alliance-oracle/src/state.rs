use alliance_protocol::alliance_oracle_types::{ChainInfo, Config, LunaInfo};
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHAINS_INFO: Item<Vec<ChainInfo>> = Item::new("chains_info");
pub const LUNA_INFO: Item<LunaInfo> = Item::new("luna_info");
