use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomMsg, Uint128};

#[cw_serde]
pub enum CustomExecuteMsg {
    Token(TokenExecuteMsg),
}

impl CustomMsg for CustomExecuteMsg {}

#[cw_serde]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}

#[cw_serde]
pub struct Metadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
}

#[cw_serde]
pub enum TokenExecuteMsg {
    CreateDenom {
        subdenom: String,
        metadata: Metadata,
    },
    MintTokens {
        denom: String,
        amount: Uint128,
        mint_to_address: String,
    },
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: String,
    },
}
