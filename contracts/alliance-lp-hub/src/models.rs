use alliance_protocol::alliance_protocol::{
        AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, AssetDistribution,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};
use std::collections::{HashMap, HashSet};

pub type AssetDenom = String;

#[cw_serde]
pub struct Config {
    pub governance: Addr,
    pub controller: Addr,
    pub fee_collector: Addr,
    pub astro_incentives: Addr,
    pub alliance_token_denom: String,
    pub alliance_token_supply: Uint128,
    pub reward_denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub governance: String,
    pub controller: String,
    pub fee_collector_address: String,
    pub astro_incentives_address: String,
    pub reward_denom: String,
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
    ClaimRewards(AssetInfo),

    // Alliance interactions used to delegate, undelegate and redelegate 
    AllianceDelegate(AllianceDelegateMsg),
    AllianceUndelegate(AllianceUndelegateMsg),
    AllianceRedelegate(AllianceRedelegateMsg),

    // Rewards related messages
    UpdateRewards {},
    UpdateAllianceRewardsCallback {},
    UpdateAstroRewardsCallback {},
}

#[cw_serde]
pub struct ModifyAssetPair {
    pub asset_info: AssetInfo,
    pub reward_asset_info: Option<AssetInfo>,
    pub delete: bool,
}

impl ModifyAssetPair {
    pub fn new(asset_info: AssetInfo, reward_asset_info: Option<AssetInfo>, delete: bool) -> Self {
        ModifyAssetPair {
            asset_info,
            reward_asset_info,
            delete,
        }
    }
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
    RewardDistribution {},

    #[returns(StakedBalanceRes)]
    StakedBalance(AssetQuery),

    #[returns(PendingRewardsRes)]
    PendingRewards(AssetQuery),

    #[returns(Vec<StakedBalanceRes>)]
    AllStakedBalances(AllStakedBalancesQuery),

    #[returns(Vec<PendingRewardsRes>)]
    AllPendingRewards(AllPendingRewardsQuery),

    #[returns(Vec<StakedBalanceRes>)]
    TotalStakedBalances {},
}
pub type WhitelistedAssetsResponse = HashMap<AssetDenom, Vec<AssetInfo>>;

#[cw_serde]
pub struct AllPendingRewardsQuery {
    pub address: String,
}

#[cw_serde]
pub struct AllStakedBalancesQuery {
    pub address: String,
}

#[cw_serde]
pub struct PendingRewardsRes {
    pub deposit_asset: AssetInfo,
    pub reward_asset: AssetInfo,
    pub rewards: Uint128,
}

#[cw_serde]
pub struct AssetQuery {
    pub address: String,
    pub deposit_asset: AssetInfo,
    pub reward_asset: AssetInfo,
}

#[cw_serde]
pub struct StakedBalanceRes {
    pub deposit_asset: AssetInfo,
    pub balance: Uint128,
}