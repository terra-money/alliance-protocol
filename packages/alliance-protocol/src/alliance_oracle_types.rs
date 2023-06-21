use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, StdError, Timestamp, Uint128};
use std::collections::HashMap;
use crate::signed_decimal::SignedDecimal;

#[cw_serde]
pub struct Config {
    pub data_expiry_seconds: u64,
    pub governance_addr: Addr,
    pub controller_addr: Addr,
}

#[cw_serde]
pub struct LunaInfo {
    pub luna_price: Decimal,
    pub update_timestamp: Timestamp,
}

impl Expire for LunaInfo {
    fn get_update_timestamp(&self) -> Timestamp {
        self.update_timestamp
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub controller_addr: String,
    pub governance_addr: String,
    pub data_expiry_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateChainsInfo { chains_info: ChainsInfo },
}

#[cw_serde]
pub struct ChainsInfo {
    pub luna_price: Decimal,
    pub protocols_info: Vec<ChainInfoMsg>,
}

impl ChainsInfo {
    pub fn to_luna_info(&self, update_timestamp: Timestamp) -> LunaInfo {
        LunaInfo {
            luna_price: self.luna_price,
            update_timestamp,
        }
    }
}

pub type ChainId = String;
#[cw_serde]
pub struct ChainInfoMsg {
    pub chain_id: ChainId,
    pub native_token: NativeToken,
    pub luna_alliances: Vec<LunaAlliance>,
    pub chain_alliances_on_phoenix: Vec<BaseAlliance>,
}

impl ChainInfoMsg {
    pub fn to_chain_info(&self, update_timestamp: Timestamp) -> ChainInfo {
        ChainInfo {
            chain_id: self.chain_id.clone(),
            native_token: self.native_token.clone(),
            luna_alliances: self.luna_alliances.clone(),
            chain_alliances_on_phoenix: self.chain_alliances_on_phoenix.clone(),
            update_timestamp,
        }
    }
}

#[cw_serde]
pub struct ChainInfo {
    pub chain_id: ChainId,
    pub update_timestamp: Timestamp,
    pub native_token: NativeToken,
    pub luna_alliances: Vec<LunaAlliance>,
    pub chain_alliances_on_phoenix: Vec<BaseAlliance>,
}

impl Expire for ChainInfo {
    fn get_update_timestamp(&self) -> Timestamp {
        self.update_timestamp
    }
}

pub trait Expire {
    fn get_update_timestamp(&self) -> Timestamp;

    fn is_expired(
        &self,
        data_expiry_seconds: u64,
        current_blocktime: Timestamp,
    ) -> Result<(), StdError> {
        let data_expiry_time = self
            .get_update_timestamp()
            .plus_seconds(data_expiry_seconds);

        if data_expiry_time < current_blocktime {
            let string_error = format!(
                "Data expired, current_blocktime: {}, data_expiry_time: {}",
                current_blocktime, data_expiry_time,
            );

            return Err(StdError::generic_err(string_error));
        }

        Ok(())
    }
}

#[cw_serde]
pub struct NativeToken {
    pub denom: String,
    pub token_price: Decimal,
    pub annual_provisions: Decimal,
}

#[cw_serde]
pub struct BaseAlliance {
    pub ibc_denom: String,
    pub rebase_factor: Decimal,
}

#[cw_serde]
pub struct LunaAlliance {
    pub ibc_denom: String,
    pub normalized_reward_weight: Decimal,
    pub annual_take_rate: Decimal,
    pub total_lsd_staked: Decimal,
    pub rebase_factor: Decimal,
}

#[cw_serde]
pub struct AssetStaked {
    pub denom: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct EmissionsDistribution {
    pub denom: String,
    pub distribution: SignedDecimal,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    QueryConfig {},
    #[returns(LunaInfo)]
    QueryLunaInfo {},
    #[returns(ChainInfo)]
    QueryChainInfo { chain_id: ChainId },
    #[returns(Vec<ChainInfo>)]
    QueryChainsInfo {},
    #[returns(Vec<ChainInfo>)]
    QueryChainsInfoUnsafe {},
    #[returns(Vec<EmissionsDistribution>)]
    QueryEmissionsDistributions(HashMap<ChainId, Vec<AssetStaked>>),
}

#[cw_serde]
pub struct MigrateMsg {}
