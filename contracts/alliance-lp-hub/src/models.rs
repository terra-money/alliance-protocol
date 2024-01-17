use alliance_protocol::alliance_protocol::{
        AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, AssetDistribution,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Decimal};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};
use std::collections::{HashMap, HashSet};
use alliance_protocol::alliance_oracle_types::EmissionsDistribution;

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
    pub astro_reward_denom: String
}

#[cw_serde]
pub struct InstantiateMsg {
    pub governance: String,
    pub controller: String,
    pub fee_collector_address: String,
    pub astro_incentives_address: String,
    pub reward_denom: String,
    pub astro_reward_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Both functions are used to stake,
    // - Stake is used for CosmosSDK::Coin
    // - Receive is used for CW20 tokens
    Stake {},
    Receive(Cw20ReceiveMsg),

    // Used to do the other operations
    // for staked assets
    Unstake(Asset),
    ClaimRewards(AssetInfo),
    UpdateRewards {},
    UpdateRewardsCallback {},

    // Privileged functions
    ModifyAssets(Vec<ModifyAsset>),

    AllianceDelegate(AllianceDelegateMsg),
    AllianceUndelegate(AllianceUndelegateMsg),
    AllianceRedelegate(AllianceRedelegateMsg),
    RebalanceEmissions(Vec<EmissionsDistribution>),
    RebalanceEmissionsCallback(Vec<EmissionsDistribution>),
}

#[cw_serde]
pub struct ModifyAsset {
    pub asset_info: AssetInfo,
    pub delete: bool,
}

impl ModifyAsset {
    pub fn new(asset_info: AssetInfo, delete: bool) -> Self {
        ModifyAsset {
            asset_info,
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
    pub staked_asset: AssetInfo,
    pub reward_asset: AssetInfo,
    pub alliance_rewards: Uint128,
    pub astro_rewards: Uint128,
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

#[cw_serde]
pub struct AssetRewardRate {
    pub alliance_reward_rate: Decimal,
    pub astro_reward_rate: Decimal,
}

impl AssetRewardRate {
    pub fn new(alliance_reward_rate: Decimal, astro_reward_rate: Decimal) -> Self {
        AssetRewardRate {
            alliance_reward_rate,
            astro_reward_rate,
        }
    }

    pub fn zero() -> Self {
        AssetRewardRate {
            alliance_reward_rate: Decimal::zero(),
            astro_reward_rate: Decimal::zero(),
        }
    }
}

#[cw_serde]
pub struct AssetUnclaimedRewards {
    pub alliance_reward_rate: Uint128,
    pub astro_reward_rate: Uint128,
}

impl AssetUnclaimedRewards {
    pub fn new(alliance_reward_rate: Uint128, astro_reward_rate: Uint128) -> Self {
        AssetUnclaimedRewards {
            alliance_reward_rate,
            astro_reward_rate,
        }
    }

    pub fn zero() -> Self {
        AssetUnclaimedRewards {
            alliance_reward_rate: Uint128::zero(),
            astro_reward_rate: Uint128::zero(),
        }
    }
}