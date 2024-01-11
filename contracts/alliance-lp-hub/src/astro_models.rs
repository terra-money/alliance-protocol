use cosmwasm_schema::{QueryResponses, cw_serde};
use cosmwasm_std::{Decimal, Addr};
use cw20::Cw20ReceiveMsg;



#[cw_serde]
pub enum ExecuteAstroMsg {
    /// Receives a message of type [`Cw20ReceiveMsg`]. Handles cw20 LP token deposits.
    Receive(Cw20ReceiveMsg),
    /// Stake LP tokens in the Generator. LP tokens staked on behalf of recipient if recipient is set.
    /// Otherwise LP tokens are staked on behalf of message sender.
    Deposit { recipient: Option<String> },
}


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryAstroMsg {
    /// RewardInfo returns reward information for a specified LP token
    #[returns(Vec<RewardInfo>)]
    RewardInfo { lp_token: String },
}

#[cw_serde]
pub struct RewardInfo {
    /// Defines [`AssetInfo`] of reward token as well as its type: protocol or external.
    pub reward: AstroRewardType,
    /// Reward tokens per second for the whole pool
    pub rps: Decimal,
    /// Last checkpointed reward per LP token
    pub index: Decimal,
    /// Orphaned rewards might appear between the time when pool
    /// gets incentivized and the time when first user stakes
    pub orphaned: Decimal,
}

#[cw_serde]
#[derive(Eq)]
/// This enum is a tiny wrapper over [`AssetInfo`] to differentiate between internal and external rewards.
/// External rewards always have a next_update_ts field which is used to update reward per second (or disable them).
pub enum AstroRewardType {
    /// Internal rewards aka ASTRO emissions don't have next_update_ts field and they are paid out from Vesting contract.
    Int(AstroAssetInfo),
    /// External rewards always have corresponding schedules. Reward is paid out from Generator contract balance.
    Ext {
        info: AstroAssetInfo,
        /// Time when next schedule should start
        next_update_ts: u64,
    },
}

/// This enum describes available Token types.
/// ## Examples
/// ```
/// # use cosmwasm_std::Addr;
/// # use astroport::asset::AssetInfo::{NativeToken, Token};
/// Token { contract_addr: Addr::unchecked("stake...") };
/// NativeToken { denom: String::from("uluna") };
/// ```
#[cw_serde]
#[derive(Hash, Eq)]
pub enum AstroAssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },
    /// Native token
    NativeToken { denom: String },
}