use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;

#[cw_serde]
pub enum ExecuteAstroMsg {
    /// Receives a message of type [`Cw20ReceiveMsg`]. Handles cw20 LP token deposits.
    Receive(Cw20ReceiveMsg),
    /// Stake LP tokens in the Generator. LP tokens staked on behalf of recipient if recipient is set.
    /// Otherwise LP tokens are staked on behalf of message sender.
    Deposit { recipient: Option<String> },
    /// Update rewards and return it to user.
    ClaimRewards {
        /// The LP token cw20 address or token factory denom
        lp_tokens: Vec<String>,
    },
    Withdraw {
        /// The LP token cw20 address or token factory denom
        lp_token: String,
        /// The amount to withdraw. Must not exceed total staked amount.
        amount: Uint128,
    },
}

#[cw_serde]
/// Cw20 hook message template
pub enum Cw20Msg {
    Deposit {
        recipient: Option<String>,
    },
    /// Besides this enum variant is redundant we keep this for backward compatibility with old pair contracts
    DepositFor(String),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryAstroMsg {
    /// RewardInfo returns reward information for a specified LP token
    #[returns(Vec<RewardInfo>)]
    RewardInfo { lp_token: String },
    /// PendingToken returns the amount of rewards that can be claimed by an account that deposited a specific LP token in a generator
    #[returns(Vec<Asset>)]
    PendingRewards { lp_token: String, user: String },
    /// Deposit returns the LP token amount deposited in a specific generator
    #[returns(Uint128)]
    Deposit { lp_token: String, user: String },
}

#[cw_serde]
pub struct Asset {
    /// Information about an asset stored in a [`AssetInfo`] struct
    pub info: AssetInfo,
    /// A token amount
    pub amount: Uint128,
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
#[cw_serde]
#[derive(Hash, Eq)]
pub enum AstroAssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },
    /// Native token
    NativeToken { denom: String },
}
