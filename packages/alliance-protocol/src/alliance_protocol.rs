use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_asset::{Asset, AssetInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub governance_address: String,
    pub controller_address: String,
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
pub enum QueryMsg {
    Config,
    WhitelistedAssets,
    RewardDistribution,
    StakedBalance(AssetQuery),
    PendingRewards(AssetQuery),
}

#[cw_serde]
pub struct AssetQuery {
    pub address: String,
    pub asset: AssetInfo,
}

#[cw_serde]
pub struct MigrateMsg {}
