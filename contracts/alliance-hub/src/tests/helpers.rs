use crate::contract::{execute, instantiate};
use crate::query::query;
use crate::state::CONFIG;
use crate::token_factory::CustomExecuteMsg;
use alliance_protocol::alliance_oracle_types::ChainId;
use alliance_protocol::alliance_protocol::{
    AllPendingRewardsQuery, AllianceDelegateMsg, AllianceDelegation, AllianceRedelegateMsg,
    AllianceRedelegation, AllianceUndelegateMsg, AssetQuery, Config, ExecuteMsg, InstantiateMsg,
    PendingRewardsRes, QueryMsg,
};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coin, from_binary, Deps, DepsMut, Response, StdResult, Uint128};
use cw_asset::{Asset, AssetInfo};
use std::collections::HashMap;

pub const DENOM: &str = "token_factory/token";

pub fn setup_contract(deps: DepsMut) -> Response<CustomExecuteMsg> {
    let info = mock_info("admin", &[]);
    let env = mock_env();

    let init_msg = InstantiateMsg {
        governance: "gov".to_string(),
        controller: "controller".to_string(),
        oracle: "oracle".to_string(),
        reward_denom: "uluna".to_string(),
    };
    instantiate(deps, env, info, init_msg).unwrap()
}

pub fn set_alliance_asset(deps: DepsMut) {
    CONFIG
        .update(deps.storage, |c| -> StdResult<_> {
            Ok(Config {
                alliance_token_denom: DENOM.to_string(),
                alliance_token_supply: Uint128::new(1000000000000),
                ..c
            })
        })
        .unwrap();
}

pub fn whitelist_assets(deps: DepsMut, assets: HashMap<ChainId, Vec<AssetInfo>>) -> Response {
    let info = mock_info("gov", &[]);
    let env = mock_env();

    let msg = ExecuteMsg::WhitelistAssets(assets);
    execute(deps, env, info, msg).unwrap()
}

pub fn remove_assets(deps: DepsMut, assets: Vec<AssetInfo>) -> Response {
    let info = mock_info("gov", &[]);
    let env = mock_env();

    let msg = ExecuteMsg::RemoveAssets(assets);
    execute(deps, env, info, msg).unwrap()
}

pub fn stake(deps: DepsMut, user: &str, amount: u128, denom: &str) -> Response {
    let info = mock_info(user, &[coin(amount, denom)]);
    let env = mock_env();
    let msg = ExecuteMsg::Stake {};
    execute(deps, env, info, msg).unwrap()
}

pub fn unstake(deps: DepsMut, user: &str, amount: u128, denom: &str) -> Response {
    let info = mock_info(user, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Unstake(Asset::native(denom, amount));
    execute(deps, env, info, msg).unwrap()
}

pub fn alliance_delegate(deps: DepsMut, delegations: Vec<(&str, u128)>) -> Response {
    let info = mock_info("controller", &[]);
    let env = mock_env();
    let delegations: Vec<AllianceDelegation> = delegations
        .iter()
        .map(|(addr, amount)| AllianceDelegation {
            validator: addr.to_string(),
            amount: Uint128::new(*amount),
        })
        .collect();
    let msg = ExecuteMsg::AllianceDelegate(AllianceDelegateMsg { delegations });
    execute(deps, env, info, msg).unwrap()
}

pub fn alliance_undelegate(deps: DepsMut, delegations: Vec<(&str, u128)>) -> Response {
    let info = mock_info("controller", &[]);
    let env = mock_env();
    let delegations: Vec<AllianceDelegation> = delegations
        .iter()
        .map(|(addr, amount)| AllianceDelegation {
            validator: addr.to_string(),
            amount: Uint128::new(*amount),
        })
        .collect();
    let msg = ExecuteMsg::AllianceUndelegate(AllianceUndelegateMsg {
        undelegations: delegations,
    });
    execute(deps, env, info, msg).unwrap()
}

pub fn alliance_redelegate(deps: DepsMut, redelegations: Vec<(&str, &str, u128)>) -> Response {
    let info = mock_info("controller", &[]);
    let env = mock_env();
    let redelegations: Vec<AllianceRedelegation> = redelegations
        .iter()
        .map(|(src, dst, amount)| AllianceRedelegation {
            src_validator: src.to_string(),
            dst_validator: dst.to_string(),
            amount: Uint128::new(*amount),
        })
        .collect();
    let msg = ExecuteMsg::AllianceRedelegate(AllianceRedelegateMsg { redelegations });
    execute(deps, env, info, msg).unwrap()
}

pub fn claim_rewards(deps: DepsMut, user: &str, denom: &str) -> Response {
    let info = mock_info(user, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::ClaimRewards(AssetInfo::Native(denom.to_string()));
    execute(deps, env, info, msg).unwrap()
}

pub fn query_rewards(deps: Deps, user: &str, denom: &str) -> PendingRewardsRes {
    from_binary(
        &query(
            deps,
            mock_env(),
            QueryMsg::PendingRewards(AssetQuery {
                address: user.to_string(),
                asset: AssetInfo::Native(denom.to_string()),
            }),
        )
        .unwrap(),
    )
    .unwrap()
}

pub fn query_all_rewards(deps: Deps, user: &str) -> Vec<PendingRewardsRes> {
    from_binary(
        &query(
            deps,
            mock_env(),
            QueryMsg::AllPendingRewards(AllPendingRewardsQuery {
                address: user.to_string(),
            }),
        )
        .unwrap(),
    )
    .unwrap()
}
