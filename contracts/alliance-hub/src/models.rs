use alliance_protocol::{
    alliance_oracle_types::ChainId,
    alliance_protocol::{
        AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, AssetDistribution,
    },
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_asset::{Asset, AssetInfo};
use std::collections::{HashMap, HashSet};

#[cw_serde]
pub struct Config {
    pub governance: Addr,
    pub controller: Addr,
    pub oracle: Addr,
    pub last_reward_update_timestamp: Timestamp,
    pub alliance_token_denom: String,
    pub alliance_token_supply: Uint128,
    pub reward_denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub governance: String,
    pub controller: String,
    pub oracle: String,
    pub reward_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Public functions
    Stake {},
    Unstake(Asset),
    ClaimRewards(AssetInfo),
    UpdateRewards {},

    // Privileged functions
    WhitelistAssets(HashMap<ChainId, Vec<AssetInfo>>),
    RemoveAssets(Vec<AssetInfo>),
    UpdateRewardsCallback {},
    AllianceDelegate(AllianceDelegateMsg),
    AllianceUndelegate(AllianceUndelegateMsg),
    AllianceRedelegate(AllianceRedelegateMsg),
    RebalanceEmissions {},
    RebalanceEmissionsCallback {},
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
pub type WhitelistedAssetsResponse = HashMap<ChainId, Vec<AssetInfo>>;

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
    pub staked_asset: AssetInfo,
    pub reward_asset: AssetInfo,
    pub rewards: Uint128,
}

#[cw_serde]
pub struct AssetQuery {
    pub address: String,
    pub asset: AssetInfo,
}

#[cw_serde]
pub struct StakedBalanceRes {
    pub asset: AssetInfo,
    pub balance: Uint128,
}
