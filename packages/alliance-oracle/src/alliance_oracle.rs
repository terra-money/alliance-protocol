use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub controller_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateChainsInfo { chains_info: Vec<ChainInfo> },
}

#[cw_serde]
pub struct ChainInfo {
    pub chain_id: String,
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
}

#[cw_serde]
pub struct MigrateMsg {}
