use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, Decimal};
use cw_asset::AssetInfo;

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
pub struct MigrateMsg {}


#[cw_serde]
pub struct AssetDistribution {
    pub asset: AssetInfo,
    pub distribution: Decimal,
}
