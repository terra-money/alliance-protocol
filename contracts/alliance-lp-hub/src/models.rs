use alliance_protocol::{
    alliance_protocol::{
        AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, AssetDistribution,
    },
    error::ContractError,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coins, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoKey};
use std::collections::{HashMap, HashSet};

pub type AssetDenom = String;

#[cw_serde]
pub struct Config {
    pub governance: Addr,
    pub controller: Addr,
    pub fee_collector_addr: Addr,

    pub astro_reward_denom: String,
    pub astro_incentives_addr: Addr,

    pub alliance_token_denom: String,
    pub alliance_token_supply: Uint128,
    pub alliance_reward_denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub governance: String,
    pub controller: String,
    pub fee_collector_addr: String,

    pub astro_reward_denom: String,
    pub astro_incentives_addr: String,

    pub alliance_reward_denom: String,
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

#[derive(Clone, Default)]
pub struct AstroClaimRewardsPosition {
    pub deposited_asset: String,
    pub rewards: Coins,
}

pub fn from_string_to_asset_info(denom: String) -> Result<AssetInfoKey, ContractError> {
    if denom.starts_with("ibc/") || denom.starts_with("factory/") {
        let asset_info = AssetInfoKey::from(AssetInfo::Native(denom));
        return Ok(asset_info);
    } else if denom.starts_with("terra") {
        let from = Addr::unchecked(denom);
        let asset_info = AssetInfoKey::from(AssetInfo::Cw20(from));
        return Ok(asset_info);
    }

    Err(ContractError::InvalidDenom(denom))
}
