use crate::contract::{execute, instantiate};
use crate::models::{
    AllPendingRewardsQuery, AssetQuery, Config, ExecuteMsg, InstantiateMsg, ModifyAssetPair,
    PendingRewardsRes, QueryMsg, StakedBalanceRes,
};
use crate::query::query;
use crate::state::CONFIG;
use alliance_protocol::alliance_protocol::{
    AllianceDelegateMsg, AllianceDelegation, AllianceRedelegateMsg, AllianceRedelegation,
    AllianceUndelegateMsg,
};
use alliance_protocol::error::ContractError;
use alliance_protocol::token_factory::CustomExecuteMsg;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coin, from_json, Addr, Binary, Deps, DepsMut, Response, StdResult, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};

pub const DENOM: &str = "token_factory/token";

pub fn setup_contract(deps: DepsMut) -> Response<CustomExecuteMsg> {
    let info = mock_info("admin", &[]);
    let env = mock_env();

    let init_msg = InstantiateMsg {
        governance: "gov".to_string(),
        fee_collector_address: "collector_address".to_string(),
        astro_incentives_address: "astro_incentives".to_string(),
        controller: "controller".to_string(),
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

pub fn modify_asset(deps: DepsMut, assets: Vec<ModifyAssetPair>) -> Result<Response, ContractError> {
    let info = mock_info("gov", &[]);
    let env = mock_env();

    let msg = ExecuteMsg::ModifyAssetPairs(assets);
    execute(deps, env, info, msg)
}

pub fn stake(deps: DepsMut, user: &str, amount: u128, denom: &str) -> Result<Response, ContractError> {
    let info = mock_info(user, &[coin(amount, denom)]);
    let env = mock_env();
    let msg = ExecuteMsg::Stake {};
    execute(deps, env, info, msg)
}

pub fn stake_cw20(deps: DepsMut, user: &str, amount: u128, denom: &str) -> Result<Response, ContractError> {
    let mut info = mock_info(user, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: String::from(user),
        amount: Uint128::new(amount),
        msg: Binary::default(),
    });
    info.sender = Addr::unchecked(denom.to_owned());
    execute(deps, env, info, msg)
}

pub fn unstake(deps: DepsMut, user: &str, asset: Asset) -> Result<Response, ContractError> {
    let info = mock_info(user, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Unstake(asset);
    execute(deps, env, info, msg)
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

pub fn query_rewards(
    deps: Deps,
    user: &str,
    deposit_asset: &str,
    reward_asset: &str,
) -> PendingRewardsRes {
    from_json(
        query(
            deps,
            mock_env(),
            QueryMsg::PendingRewards(AssetQuery {
                address: user.to_string(),
                deposit_asset: AssetInfo::Native(deposit_asset.to_string()),
                reward_asset: AssetInfo::Native(reward_asset.to_string()),
            }),
        )
        .unwrap(),
    )
    .unwrap()
}

pub fn query_all_rewards(deps: Deps, user: &str) -> Vec<PendingRewardsRes> {
    from_json(
        query(
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

pub fn query_all_staked_balances(deps: Deps) -> Vec<StakedBalanceRes> {
    from_json(query(deps, mock_env(), QueryMsg::TotalStakedBalances {}).unwrap()).unwrap()
}
