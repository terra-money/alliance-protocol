use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub controller_addr: String,
    pub governance_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateChainsInfo { chains_info: Vec<ChainInfoMsg> },
}

pub type ChainId = String;

#[cw_serde]
pub struct ChainInfoMsg {
    pub chain_id: ChainId,
    pub native_token: NativeToken,
    pub luna_alliances: Vec<LunaAlliance>,
}

// create a method for ChainInfoMsg to parse it into ChainInfo
// where update_timestamp from ChainInfo is set to the current block time
impl ChainInfoMsg {
    pub fn to_chain_info(&self, update_timestamp: Timestamp) -> (ChainId, ChainInfo) {
        (
            self.chain_id.clone(),
            ChainInfo {
                chain_id: self.chain_id.clone(),
                native_token: self.native_token.clone(),
                luna_alliances: self.luna_alliances.clone(),
                update_timestamp,
            },
        )
    }
}

#[cw_serde]
pub struct ChainInfo {
    pub chain_id: ChainId,
    pub update_timestamp: Timestamp,
    pub native_token: NativeToken,
    pub luna_alliances: Vec<LunaAlliance>,
}

#[cw_serde]
pub struct NativeToken {
    pub denom: String,
    pub token_price: Uint128,
    pub annual_inflation: Uint128,
}

#[cw_serde]
pub struct LunaAlliance {
    pub ibc_denom: String,
    pub normalized_reward_weight: Uint128,
    pub annual_take_rate: Uint128,
    pub total_staked: Uint128,
    pub rebase_factor: Uint128,
}

#[cw_serde]
pub enum QueryMsg {
    Config,
    ChainsInfo,
    ChainInfo { chain_id: ChainId },
}

#[cw_serde]
pub struct MigrateMsg {}
