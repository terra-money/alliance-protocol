use crate::astro_models::ExecuteAstroMsg;
use crate::contract::{execute, reply};
use crate::helpers::from_string_to_asset_info;
use crate::models::{AssetQuery, ExecuteMsg, ModifyAssetPair, PendingRewardsRes, QueryMsg};
use crate::query::query;
use crate::state::{
    ASSET_REWARD_RATE, TEMP_BALANCE, TOTAL_BALANCES, USER_ASSET_REWARD_RATE, VALIDATORS, WHITELIST,
};
use crate::tests::helpers::{
    claim_rewards, modify_asset, query_all_rewards, query_rewards, set_alliance_asset,
    setup_contract, stake, stake_cw20, unstake, DENOM,
};
use crate::tests::mock_querier::mock_dependencies;
use alliance_protocol::error::ContractError;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    coin, coins, from_json, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal, Event,
    Reply, Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use cw_asset::{Asset, AssetInfo, AssetInfoKey};
use std::borrow::BorrowMut;
use std::collections::HashSet;
use terra_proto_rs::alliance::alliance::MsgClaimDelegationRewards;
use terra_proto_rs::traits::Message;

#[test]
fn test_update_rewards() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    VALIDATORS
        .save(
            deps.as_mut().storage,
            &HashSet::from(["validator1".to_string()]),
        )
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[]),
        ExecuteMsg::UpdateRewards {},
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::reply_on_error(
                CosmosMsg::Stargate {
                    type_url: "/alliance.alliance.MsgClaimDelegationRewards".to_string(),
                    value: Binary::from(
                        MsgClaimDelegationRewards {
                            delegator_address: "cosmos2contract".to_string(),
                            validator_address: "validator1".to_string(),
                            denom: DENOM.to_string(),
                        }
                        .encode_to_vec()
                    )
                },
                2
            ),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                funds: vec![],
                contract_addr: "cosmos2contract".to_string(),
                msg: to_json_binary(&ExecuteMsg::UpdateAllianceRewardsCallback {}).unwrap()
            }))
        ]
    );
    let prev_balance = TEMP_BALANCE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
        )
        .unwrap();
    assert_eq!(prev_balance, Uint128::new(1000000));
}

#[test]
fn test_update_rewards_with_funds_sent() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    VALIDATORS
        .save(
            deps.as_mut().storage,
            &HashSet::from(["validator1".to_string(), "validator2".to_string()]),
        )
        .unwrap();

    deps.querier
        .base
        .update_balance("cosmos2contract", vec![coin(2000000, "uluna")]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[coin(1000000, "uluna")]),
        ExecuteMsg::UpdateRewards {},
    )
    .unwrap();
    let prev_balance = TEMP_BALANCE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
        )
        .unwrap();
    assert_eq!(res.messages.len(), 3);
    assert_eq!(prev_balance, Uint128::new(1000000));
}

#[test]
fn update_alliance_reward_callback() {
    let mut deps = mock_dependencies(Some(&[coin(2000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    TOTAL_BALANCES
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    TOTAL_BALANCES
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Uint128::new(100000),
        )
        .unwrap();

    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Decimal::percent(10),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Decimal::percent(60),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aMONKEY".to_string())),
            &Decimal::percent(30),
        )
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();

    let a_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(
        a_whale_rate,
        Decimal::from_atomics(Uint128::one(), 1).unwrap()
    );
    let b_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(
        b_whale_rate,
        Decimal::from_atomics(Uint128::new(6), 0).unwrap()
    );
    ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("cMONKEY".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap_err();

    assert_eq!(
        res,
        Response::new().add_attributes(vec![("action", "update_alliance_rewards_callback"),])
    );
}

#[test]
fn update_alliance_reward_unauthorized_callback() {
    let mut deps = mock_dependencies(Some(&[coin(2000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("unauthorized_sender", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn update_alliance_rewards_callback_with_unallocated() {
    let mut deps = mock_dependencies(Some(&[coin(2000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    TOTAL_BALANCES
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    TOTAL_BALANCES
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Uint128::new(100000),
        )
        .unwrap();

    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Decimal::percent(10),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Decimal::percent(60),
        )
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();

    let a_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(
        a_whale_rate,
        Decimal::from_atomics(Uint128::one(), 1).unwrap()
    );
    let b_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(
        b_whale_rate,
        Decimal::from_atomics(Uint128::new(6), 0).unwrap()
    );

    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![("action", "update_alliance_rewards_callback")])
            .add_message(BankMsg::Send {
                to_address: "controller".to_string(),
                amount: vec![coin(300000, "uluna")]
            })
    );
}

#[test]
fn claim_user_rewards() {
    let mut deps = mock_dependencies(Some(&[coin(2000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([ModifyAssetPair {
            asset_info: AssetInfo::Native("aWHALE".to_string()),
            asset_distribution: Uint128::new(1),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }]),
    )
    .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();

    stake(deps.as_mut(), "user1", 1000000, "aWHALE").unwrap();
    stake(deps.as_mut(), "user2", 4000000, "aWHALE").unwrap();

    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();

    let rewards = query_rewards(deps.as_ref(), "user1", "aWHALE", "uluna");
    assert_eq!(
        rewards,
        PendingRewardsRes {
            rewards: Uint128::new(100000),
            deposit_asset: AssetInfo::Native("aWHALE".to_string()),
            reward_asset: AssetInfo::Native("uluna".to_string()),
        }
    );

    let all_rewards = query_all_rewards(deps.as_ref(), "user1");
    assert_eq!(
        all_rewards,
        vec![PendingRewardsRes {
            rewards: Uint128::new(100000),
            deposit_asset: AssetInfo::Native("aWHALE".to_string()),
            reward_asset: AssetInfo::Native("uluna".to_string()),
        }]
    );

    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![
                ("action", "claim_alliance_lp_rewards"),
                ("user", "user1"),
                ("asset", "native:aWHALE"),
                ("alliance_reward_amount", "100000"),
                ("astro_reward_amount", "0"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".to_string(),
                amount: coins(100000, "uluna"),
            }))
    );

    let user_reward_rate = USER_ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();
    let asset_reward_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(user_reward_rate, asset_reward_rate);

    let rewards = query_rewards(deps.as_ref(), "user1", "aWHALE", "uluna");
    assert_eq!(
        rewards,
        PendingRewardsRes {
            rewards: Uint128::zero(),
            deposit_asset: AssetInfo::Native("aWHALE".to_string()),
            reward_asset: AssetInfo::Native("uluna".to_string()),
        }
    );

    let all_rewards = query_all_rewards(deps.as_ref(), "user1");
    assert_eq!(
        all_rewards,
        vec![
            PendingRewardsRes {
                rewards: Uint128::zero(),
                deposit_asset: AssetInfo::Native("aWHALE".to_string()),
                reward_asset: AssetInfo::Cw20(Addr::unchecked("astro_reward_denom".to_string())),
            },
            PendingRewardsRes {
                rewards: Uint128::zero(),
                deposit_asset: AssetInfo::Native("aWHALE".to_string()),
                reward_asset: AssetInfo::Native("uluna".to_string()),
            }
        ]
    );

    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new().add_attributes(vec![
            ("action", "claim_alliance_lp_rewards"),
            ("user", "user1"),
            ("asset", "native:aWHALE"),
            ("alliance_reward_amount", "0"),
            ("astro_reward_amount", "0"),
        ])
    );

    // Update more rewards
    deps.querier
        .base
        .update_balance("cosmos2contract", vec![coin(1900000 + 100000, "uluna")]);
    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1900000),
        )
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();
    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![
                ("action", "claim_alliance_lp_rewards"),
                ("user", "user1"),
                ("asset", "native:aWHALE"),
                ("alliance_reward_amount", "10000"),
                ("astro_reward_amount", "0"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".to_string(),
                amount: coins(10000, "uluna"),
            }))
    );
}

#[test]
fn claim_user_rewards_after_staking() {
    let mut deps = mock_dependencies(Some(&[coin(2000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([ModifyAssetPair {
            asset_info: AssetInfo::Native("aWHALE".to_string()),
            asset_distribution: Uint128::new(1),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }]),
    )
    .unwrap();
    stake(deps.as_mut(), "user1", 1000000, "aWHALE").unwrap();
    stake(deps.as_mut(), "user2", 4000000, "aWHALE").unwrap();

    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();

    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();

    stake(deps.as_mut(), "user1", 1000000, "aWHALE").unwrap();

    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![
                ("action", "claim_alliance_lp_rewards"),
                ("user", "user1"),
                ("asset", "native:aWHALE"),
                ("alliance_reward_amount", "100000"),
                ("astro_reward_amount", "0"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".to_string(),
                amount: coins(100000, "uluna"),
            }))
    );

    // Claiming again should get 0 rewards
    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new().add_attributes(vec![
            ("action", "claim_alliance_lp_rewards"),
            ("user", "user1"),
            ("asset", "native:aWHALE"),
            ("alliance_reward_amount", "0"),
            ("astro_reward_amount", "0"),
        ])
    );
}

#[test]
fn claim_rewards_after_staking_and_unstaking() {
    let mut deps = mock_dependencies(Some(&[coin(2000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([
            ModifyAssetPair {
                asset_info: AssetInfo::Native("aWHALE".to_string()),
                asset_distribution: Uint128::new(1),
                reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
                delete: false,
            },
            ModifyAssetPair {
                asset_info: AssetInfo::Native("bWHALE".to_string()),
                asset_distribution: Uint128::new(1),
                reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
                delete: false,
            },
        ]),
    )
    .unwrap();
    stake(deps.as_mut(), "user1", 1000000, "aWHALE").unwrap();
    stake(deps.as_mut(), "user2", 4000000, "aWHALE").unwrap();
    stake(deps.as_mut(), "user2", 1000000, "bWHALE").unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();

    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();
    claim_rewards(deps.as_mut(), "user1", "aWHALE");

    // Get asset reward rate
    let prev_rate = ASSET_REWARD_RATE
        .load(
            deps.as_mut().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();

    // Unstake
    let asset_info = Asset::native(Addr::unchecked("aWHALE"), 1000000u128);
    unstake(deps.as_mut(), "user1", asset_info).unwrap();

    // Accrue rewards again
    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();

    let curr_rate = ASSET_REWARD_RATE
        .load(
            deps.as_mut().storage,
            (
                AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
                AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            ),
        )
        .unwrap();
    assert!(curr_rate > prev_rate);

    // User 1 stakes back
    stake(deps.as_mut(), "user1", 1000000, "aWHALE").unwrap();

    // User 1 should not have any rewards
    let res = query_rewards(deps.as_ref(), "user1", "aWHALE", "uluna");
    assert_eq!(
        res,
        PendingRewardsRes {
            rewards: Uint128::zero(),
            deposit_asset: AssetInfo::Native("aWHALE".to_string()),
            reward_asset: AssetInfo::Native("uluna".to_string()),
        }
    );

    // User 2 should receive all the rewards in the contract
    let res = query_rewards(deps.as_ref(), "user2", "aWHALE", "uluna");
    assert_eq!(
        res,
        PendingRewardsRes {
            rewards: Uint128::new(900000),
            deposit_asset: AssetInfo::Native("aWHALE".to_string()),
            reward_asset: AssetInfo::Native("uluna".to_string()),
        }
    );
    let res = query_rewards(deps.as_ref(), "user2", "bWHALE", "uluna");
    assert_eq!(
        res,
        PendingRewardsRes {
            rewards: Uint128::new(1000000),
            deposit_asset: AssetInfo::Native("bWHALE".to_string()),
            reward_asset: AssetInfo::Native("uluna".to_string()),
        }
    );
}

#[test]
fn claim_rewards_after_rebalancing_emissions() {
    let mut deps = mock_dependencies(Some(&[coin(2000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([
            ModifyAssetPair {
                asset_info: AssetInfo::Native("aWHALE".to_string()),
                asset_distribution: Uint128::new(1),
                reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
                delete: false,
            },
            ModifyAssetPair {
                asset_info: AssetInfo::Native("bWHALE".to_string()),
                asset_distribution: Uint128::new(1),
                reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
                delete: false,
            },
        ]),
    )
    .unwrap();
    stake(deps.as_mut(), "user1", 1000000, "aWHALE").unwrap();
    stake(deps.as_mut(), "user2", 1000000, "bWHALE").unwrap();

    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Decimal::percent(50),
        )
        .unwrap();

    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();

    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
            &Decimal::percent(100),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
            &Decimal::percent(0),
        )
        .unwrap();

    TEMP_BALANCE
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
            &Uint128::new(1000000),
        )
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateAllianceRewardsCallback {},
    )
    .unwrap();

    let rewards = query_rewards(deps.as_ref(), "user1", "aWHALE", "uluna");
    assert_eq!(rewards.rewards, Uint128::new(1500000));
    // User 2 should receive all the rewards in the contract
    let rewards = query_rewards(deps.as_ref(), "user2", "bWHALE", "uluna");
    assert_eq!(rewards.rewards, Uint128::new(500000));
}

#[test]
fn test_update_rewards_with_astro_rewards() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    WHITELIST
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::cw20(Addr::unchecked(
                "terra_astro_cw20".to_string(),
            ))),
            &Decimal::percent(10),
        )
        .unwrap();
    VALIDATORS
        .save(
            deps.as_mut().storage,
            &HashSet::from(["validator1".to_string()]),
        )
        .unwrap();
    TOTAL_BALANCES
        .save(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::cw20(Addr::unchecked(
                "terra_astro_cw20".to_string(),
            ))),
            &Uint128::new(1000000),
        )
        .unwrap();
    stake_cw20(deps.as_mut(), "user", 1000000, "terra_astro_cw20").unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[]),
        ExecuteMsg::UpdateRewards {},
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::reply_always(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "astro_incentives".to_string(),
                    msg: to_json_binary(&ExecuteAstroMsg::ClaimRewards {
                        lp_tokens: vec!["terra_astro_cw20".to_string()],
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                3
            ),
            SubMsg::reply_on_error(
                CosmosMsg::Stargate {
                    type_url: "/alliance.alliance.MsgClaimDelegationRewards".to_string(),
                    value: Binary::from(
                        MsgClaimDelegationRewards {
                            delegator_address: "cosmos2contract".to_string(),
                            validator_address: "validator1".to_string(),
                            denom: DENOM.to_string(),
                        }
                        .encode_to_vec()
                    )
                },
                2
            ),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                funds: vec![],
                contract_addr: "cosmos2contract".to_string(),
                msg: to_json_binary(&ExecuteMsg::UpdateAllianceRewardsCallback {}).unwrap()
            }))
        ]
    );
    let prev_balance = TEMP_BALANCE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("uluna".to_string())),
        )
        .unwrap();
    assert_eq!(prev_balance, Uint128::new(1000000));

    let reply_msg = Reply {
        id: 3,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![Event::new("wasm")
                .add_attribute("_contract_address", "cosmos2contract")
                .add_attribute("action", "claim_alliance_lp_rewards")
                .add_attribute("claimed_position", "terra_astro_cw20")
                .add_attribute("claimed_reward", "1factory/astro")
                .add_attribute("claimed_reward", "1factory/astro")],
            data: None,
        }),
    };
    let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_attributes(vec![("action", "claim_alliance_lp_astro_rewards_success")])
    );

    let rewatd_rate_key = (
        from_string_to_asset_info("terra_astro_cw20".to_string()).unwrap(),
        from_string_to_asset_info("factory/astro".to_string()).unwrap(),
    );
    let balances = ASSET_REWARD_RATE
        .load(deps.storage.borrow_mut(), rewatd_rate_key)
        .unwrap();
    assert_eq!(balances, Decimal::new(Uint128::new(1000000000000)));

    let res: PendingRewardsRes = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::PendingRewards(AssetQuery {
                address: "user".to_string(),
                deposit_asset: AssetInfo::Cw20(Addr::unchecked("terra_astro_cw20")),
                reward_asset: AssetInfo::Native("factory/astro".to_string()),
            }),
        )
        .unwrap(),
    )
    .unwrap();

    let expected = PendingRewardsRes {
        rewards: Uint128::one(),
        deposit_asset: AssetInfo::Cw20(Addr::unchecked("terra_astro_cw20")),
        reward_asset: AssetInfo::Native("factory/astro".to_string()),
    };

    assert_eq!(res, expected);
}
