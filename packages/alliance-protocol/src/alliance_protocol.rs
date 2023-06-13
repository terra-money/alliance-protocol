use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_asset::{Asset, AssetInfo};

#[cw_serde]
pub struct InstantiateMsg {
}

#[cw_serde]
pub enum ExecuteMsg {
    // Public functions
    Stake,
    Unstake(Asset),
    ClaimRewards(AssetInfo),

    // Privileged functions
    WhitelistAsset(Vec<AssetInfo>),
    RemoveAsset(Vec<AssetInfo>),
    UpdateRewards,
    AllianceDelegate(AllianceDelegateMsg),
    AllianceUndelegate(AllianceUndelegateMsg),
    AllianceRedelegate(AllianceRedelegateMsg),
    RebalanceEmissions,
}

#[cw_serde]
pub struct AllianceDelegation {
    validator: String,
    amount: Uint128,
}

#[cw_serde]
pub struct AllianceDelegateMsg {
    delegations: Vec<AllianceDelegation>
}

#[cw_serde]
pub struct AllianceUndelegateMsg {
    undelegations: Vec<AllianceDelegation>
}

#[cw_serde]
pub struct AllianceRedelegation {
    src_validator: String,
    dst_validator: String,
    amount: Uint128,
}

#[cw_serde]
pub struct AllianceRedelegateMsg {
    redelegations: Vec<AllianceRedelegation>
}

#[cw_serde]
pub enum QueryMsg {
    Config,
    WhitelistedCoins,
    RewardDistribution,
    StakedBalance(AssetQuery),
    PendingRewards(AssetQuery),
}

#[cw_serde]
pub struct AssetQuery {
    address: String,
    asset: AssetInfo,
}

#[cw_serde]
pub struct MigrateMsg {
}