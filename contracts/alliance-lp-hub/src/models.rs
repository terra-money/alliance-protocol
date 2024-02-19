use alliance_protocol::alliance_protocol::{
    AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, AssetDistribution,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};
use std::collections::HashSet;

pub type AssetDenom = String;

#[cw_serde]
pub struct Config {
    pub governance_addr: Addr,
    pub controller_addr: Addr,

    pub astro_incentives_addr: Addr,
    pub alliance_reward_denom: AssetInfo,

    pub alliance_token_denom: String,
    pub alliance_token_supply: Uint128,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub governance_addr: String,
    pub controller_addr: String,

    pub astro_incentives_addr: String,
    pub alliance_reward_denom: AssetInfo,

    pub alliance_token_subdenom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Privileged function used to whitelist,
    // modify or delete assets from the allowed list
    ModifyAssetPairs(Vec<ModifyAssetPair>),

    // Both functions are used to stake,
    // - Stake is used for CosmosSDK::Coin
    // - Receive is used for CW20 tokens
    Receive(Cw20ReceiveMsg),
    Stake {},

    // Used to do the other operations for staked assets
    Unstake(Asset),
    UnstakeCallback(Asset, Addr),
    ClaimRewards(AssetInfo),

    // Alliance interactions used to delegate, undelegate and redelegate
    AllianceDelegate(AllianceDelegateMsg),
    AllianceUndelegate(AllianceUndelegateMsg),
    AllianceRedelegate(AllianceRedelegateMsg),

    // Rewards related messages
    UpdateRewards {},
    UpdateAllianceRewardsCallback {},
}

#[cw_serde]
pub struct ModifyAssetPair {
    pub asset_distribution: Uint128,
    pub asset_info: AssetInfo,
    pub reward_asset_info: Option<AssetInfo>,
    pub delete: bool,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},

    #[returns(HashSet<Addr>)]
    Validators {},

    #[returns(WhitelistedAssetsResponse)]
    WhitelistedAssets {},

    #[returns(Vec<AssetDistribution>)]
    AllianceRewardsDistribution {},

    #[returns(Vec<StakedBalanceRes>)]
    ContractBalances {},

    #[returns(StakedBalanceRes)]
    StakedBalance(StakedAssetQuery),

    #[returns(PendingRewards)]
    PendingRewards(AssetQuery),

    #[returns(Vec<StakedBalanceRes>)]
    AddressStakedBalances(AddressStakedBalancesQuery),

    #[returns(Vec<PendingRewardsRes>)]
    AddressPendingRewards(AddressPendingRewardsQuery),
}
pub type WhitelistedAssetsResponse = Vec<AssetInfo>;

#[cw_serde]
pub struct AddressPendingRewardsQuery {
    pub address: String,
}

#[cw_serde]
pub struct AddressStakedBalancesQuery {
    pub address: String,
}

#[cw_serde]
pub struct PendingRewardsRes {
    pub deposit_asset: AssetInfo,
    pub reward_asset: AssetInfo,
    pub rewards: Uint128,
}

#[cw_serde]
pub struct PendingRewards {
    pub deposit_asset: Option<AssetInfo>,
    pub reward_asset: Option<AssetInfo>,
    pub rewards: Uint128,
}

impl PendingRewards {
    pub fn new(deposit_asset: AssetInfo, reward_asset: AssetInfo, rewards: Uint128) -> Self {
        PendingRewards {
            deposit_asset: Some(deposit_asset),
            reward_asset: Some(reward_asset),
            rewards,
        }
    }
}

#[cw_serde]
pub struct AssetQuery {
    pub address: String,
    pub deposit_asset: AssetInfo,
    pub reward_asset: AssetInfo,
}

#[cw_serde]
pub struct StakedAssetQuery {
    pub address: String,
    pub deposit_asset: AssetInfo,
}

#[cw_serde]
pub struct StakedBalanceRes {
    pub deposit_asset: AssetInfo,
    pub balance: Uint128,
}
