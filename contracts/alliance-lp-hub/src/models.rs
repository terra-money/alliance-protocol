use alliance_protocol::{alliance_protocol::{
    AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, AssetDistribution,
}, error::ContractError};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_asset::{Asset, AssetInfo};
use std::collections::{HashMap, HashSet};

pub type AssetDenom = String;

#[cw_serde]
pub struct Config {
    pub governance: Addr,
    pub controller: Addr,
    pub alliance_token_denom: String,
    pub alliance_token_supply: Uint128,
    pub reward_denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub governance: String,
    pub controller: String,
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
    ModifyAssets(Vec<ModifyAsset>),

    UpdateRewardsCallback {},
    AllianceDelegate(AllianceDelegateMsg),
    AllianceUndelegate(AllianceUndelegateMsg),
    AllianceRedelegate(AllianceRedelegateMsg),
    RebalanceEmissions {},
    RebalanceEmissionsCallback {},
}

#[cw_serde]
pub struct ModifyAsset {
    pub asset_info: AssetInfo,
    pub rewards_rate: Option<Decimal>,
    pub delete: bool,
}

impl ModifyAsset {
    pub fn new(asset_info: AssetInfo, rewards_rate: Option<Decimal>, delete: bool) -> Self {
        ModifyAsset {
            asset_info,
            rewards_rate,
            delete,
        }
    }

    pub fn is_valid_reward_rate(&self) -> Result<Decimal, ContractError> {
        match self.rewards_rate {
            Some(rate) => {
                if rate < Decimal::zero() || rate > Decimal::one() {
                    return Err(ContractError::InvalidRewardRate(rate, self.asset_info.to_string()));
                }
                Ok(rate)
            },
            None => {
                return Err(ContractError::InvalidRewardRate(Decimal::zero(), self.asset_info.to_string()));
            }
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
