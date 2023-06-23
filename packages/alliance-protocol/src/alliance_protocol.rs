use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

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
pub struct AssetDistribution {
    pub asset: AssetInfo,
    pub distribution: Decimal,
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
    Stake,
    Unstake(Asset),
    ClaimRewards(AssetInfo),

    // Privileged functions
    WhitelistAssets(Vec<AssetInfo>),
    RemoveAssets(Vec<AssetInfo>),
    UpdateRewards,
    UpdateRewardsCallback,
    AllianceDelegate(AllianceDelegateMsg),
    AllianceUndelegate(AllianceUndelegateMsg),
    AllianceRedelegate(AllianceRedelegateMsg),
    RebalanceEmissions,
}

#[cw_serde]
pub struct AllianceDelegation {
    pub validator: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct AllianceDelegateMsg {
    pub delegations: Vec<AllianceDelegation>,
}

#[cw_serde]
pub struct AllianceUndelegateMsg {
    pub undelegations: Vec<AllianceDelegation>,
}

#[cw_serde]
pub struct AllianceRedelegation {
    pub src_validator: String,
    pub dst_validator: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct AllianceRedelegateMsg {
    pub redelegations: Vec<AllianceRedelegation>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},

    #[returns(Vec<AssetInfoBase<String>>)]
    WhitelistedAssets {},

    #[returns(Vec<AssetDistribution>)]
    RewardDistribution {},

    #[returns(StakedBalanceRes)]
    StakedBalance(AssetQuery),

    #[returns(PendingRewardsRes)]
    PendingRewards(AssetQuery),

    #[returns(Vec<PendingRewardsRes>)]
    AllPendingRewards(AllPendingRewardsQuery),
}

#[cw_serde]
pub struct AssetQuery {
    pub address: String,
    pub asset: AssetInfo,
}

#[cw_serde]
pub struct AllPendingRewardsQuery {
    pub address: String,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct StakedBalanceRes {
    pub asset: AssetInfo,
    pub balance: Uint128,
}

#[cw_serde]
pub struct PendingRewardsRes {
    pub staked_asset: AssetInfo,
    pub reward_asset: AssetInfo,
    pub rewards: Uint128,
}
