use crate::contract::execute;
use crate::models::{ExecuteMsg, ModifyAsset, PendingRewardsRes};
use crate::state::{ASSET_REWARD_RATE, TEMP_BALANCE, TOTAL_BALANCES, UNALLOCATED_REWARDS, USER_ASSET_REWARD_RATE, VALIDATORS, WHITELIST};
use crate::tests::helpers::{
    claim_rewards, query_all_rewards, query_rewards, set_alliance_asset, setup_contract, stake,
    unstake, modify_asset, DENOM,
};
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal, Response, SubMsg,
    Uint128, WasmMsg,
};
use cw_asset::{AssetInfo, AssetInfoKey};
use std::collections::HashSet;
use terra_proto_rs::alliance::alliance::MsgClaimDelegationRewards;
use terra_proto_rs::traits::Message;

#[test]
fn test_update_rewards() {
    let mut deps = mock_dependencies_with_balance(&[coin(1000000, "uluna")]);
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
                msg: to_json_binary(&ExecuteMsg::UpdateRewardsCallback {}).unwrap()
            }))
        ]
    );
    let prev_balance = TEMP_BALANCE.load(deps.as_ref().storage).unwrap();
    assert_eq!(prev_balance, Uint128::new(1000000));
}

#[test]
fn test_update_rewards_with_funds_sent() {
    let mut deps = mock_dependencies_with_balance(&[coin(1000000, "uluna")]);
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    VALIDATORS
        .save(
            deps.as_mut().storage,
            &HashSet::from(["validator1".to_string(), "validator2".to_string()]),
        )
        .unwrap();

    deps.querier
        .update_balance("cosmos2contract", vec![coin(2000000, "uluna")]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("user", &[coin(1000000, "uluna")]),
        ExecuteMsg::UpdateRewards {},
    )
    .unwrap();
    let prev_balance = TEMP_BALANCE.load(deps.as_ref().storage).unwrap();
    assert_eq!(res.messages.len(), 3);
    assert_eq!(prev_balance, Uint128::new(1000000));
}

#[test]
fn update_reward_callback() {
    let mut deps = mock_dependencies_with_balance(&[coin(2000000, "uluna")]);
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
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())), &Decimal::percent(10)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())), &Decimal::percent(60)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aMONKEY".to_string())), &Decimal::percent(30)).unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();

    let a_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
        )
        .unwrap();
    assert_eq!(
        a_whale_rate,
        Decimal::from_atomics(Uint128::one(), 1).unwrap()
    );
    let b_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
        )
        .unwrap();
    assert_eq!(
        b_whale_rate,
        Decimal::from_atomics(Uint128::new(6), 0).unwrap()
    );
    ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("cMONKEY".to_string())),
        )
        .unwrap_err();

    assert_eq!(
        res,
        Response::new().add_attributes(vec![("action", "update_rewards_callback"),])
    );
}

#[test]
fn update_reward_callback_with_unallocated() {
    let mut deps = mock_dependencies_with_balance(&[coin(2000000, "uluna")]);
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
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())), &Decimal::percent(10)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())), &Decimal::percent(60)).unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
        .unwrap();

    let a_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
        )
        .unwrap();
    assert_eq!(
        a_whale_rate,
        Decimal::from_atomics(Uint128::one(), 1).unwrap()
    );
    let b_whale_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())),
        )
        .unwrap();
    assert_eq!(
        b_whale_rate,
        Decimal::from_atomics(Uint128::new(6), 0).unwrap()
    );
    let unallocated_rewards =  UNALLOCATED_REWARDS.load(deps.as_ref().storage).unwrap();
    assert_eq!(unallocated_rewards, Uint128::new(300000));

    assert_eq!(
        res,
        Response::new().add_attributes(vec![("action", "update_rewards_callback"),])
    );
}

#[test]
fn claim_user_rewards() {
    let mut deps = mock_dependencies_with_balance(&[coin(2000000, "uluna")]);
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([
            ModifyAsset{
                asset_info: AssetInfo::Native("aWHALE".to_string()),
                delete: false,
            }
        ]),
    );
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())), &Decimal::percent(50)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())), &Decimal::percent(50)).unwrap();

    stake(deps.as_mut(), "user1", 1000000, "aWHALE");
    stake(deps.as_mut(), "user2", 4000000, "aWHALE");

    TEMP_BALANCE
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();

    let rewards = query_rewards(deps.as_ref(), "user1", "aWHALE");
    assert_eq!(
        rewards,
        PendingRewardsRes {
            rewards: Uint128::new(100000),
            reward_asset: AssetInfo::Native("uluna".to_string()),
            staked_asset: AssetInfo::Native("aWHALE".to_string()),
        }
    );

    let all_rewards = query_all_rewards(deps.as_ref(), "user1");
    assert_eq!(
        all_rewards,
        vec![PendingRewardsRes {
            rewards: Uint128::new(100000),
            reward_asset: AssetInfo::Native("uluna".to_string()),
            staked_asset: AssetInfo::Native("aWHALE".to_string()),
        }]
    );

    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![
                ("action", "claim_rewards"),
                ("user", "user1"),
                ("asset", "native:aWHALE"),
                ("reward_amount", "100000"),
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
            ),
        )
        .unwrap();
    let asset_reward_rate = ASSET_REWARD_RATE
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
        )
        .unwrap();
    assert_eq!(user_reward_rate, asset_reward_rate);

    let rewards = query_rewards(deps.as_ref(), "user1", "aWHALE");
    assert_eq!(
        rewards,
        PendingRewardsRes {
            rewards: Uint128::new(0),
            reward_asset: AssetInfo::Native("uluna".to_string()),
            staked_asset: AssetInfo::Native("aWHALE".to_string()),
        }
    );

    let all_rewards = query_all_rewards(deps.as_ref(), "user1");
    assert_eq!(
        all_rewards,
        vec![PendingRewardsRes {
            rewards: Uint128::new(0),
            reward_asset: AssetInfo::Native("uluna".to_string()),
            staked_asset: AssetInfo::Native("aWHALE".to_string()),
        }]
    );

    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new().add_attributes(vec![
            ("action", "claim_rewards"),
            ("user", "user1"),
            ("asset", "native:aWHALE"),
            ("reward_amount", "0"),
        ])
    );

    // Update more rewards
    deps.querier
        .update_balance("cosmos2contract", vec![coin(1900000 + 100000, "uluna")]);
    TEMP_BALANCE
        .save(deps.as_mut().storage, &Uint128::new(1900000))
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();
    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![
                ("action", "claim_rewards"),
                ("user", "user1"),
                ("asset", "native:aWHALE"),
                ("reward_amount", "10000"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".to_string(),
                amount: coins(10000, "uluna"),
            }))
    );
}

#[test]
fn claim_user_rewards_after_staking() {
    let mut deps = mock_dependencies_with_balance(&[coin(2000000, "uluna")]);
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([
            ModifyAsset{
                asset_info: AssetInfo::Native("aWHALE".to_string()),
                delete: false,
            }
        ]),
    );
    stake(deps.as_mut(), "user1", 1000000, "aWHALE");
    stake(deps.as_mut(), "user2", 4000000, "aWHALE");

    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())), &Decimal::percent(50)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())), &Decimal::percent(50)).unwrap();

    TEMP_BALANCE
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();

    stake(deps.as_mut(), "user1", 1000000, "aWHALE");

    let res = claim_rewards(deps.as_mut(), "user1", "aWHALE");
    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![
                ("action", "claim_rewards"),
                ("user", "user1"),
                ("asset", "native:aWHALE"),
                ("reward_amount", "100000"),
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
            ("action", "claim_rewards"),
            ("user", "user1"),
            ("asset", "native:aWHALE"),
            ("reward_amount", "0"),
        ])
    );
}

#[test]
fn claim_rewards_after_staking_and_unstaking() {
    let mut deps = mock_dependencies_with_balance(&[coin(2000000, "uluna")]);
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([
            ModifyAsset{
                asset_info: AssetInfo::Native("aWHALE".to_string()),
                delete: false,
            },
            ModifyAsset{
                asset_info: AssetInfo::Native("bWHALE".to_string()),
                delete: false,
            }
        ]),
    );
    stake(deps.as_mut(), "user1", 1000000, "aWHALE");
    stake(deps.as_mut(), "user2", 4000000, "aWHALE");
    stake(deps.as_mut(), "user2", 1000000, "bWHALE");
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())), &Decimal::percent(50)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())), &Decimal::percent(50)).unwrap();

    TEMP_BALANCE
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();
    claim_rewards(deps.as_mut(), "user1", "aWHALE");

    // Get asset reward rate
    let prev_rate = ASSET_REWARD_RATE
        .load(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
        )
        .unwrap();

    // Unstake
    unstake(deps.as_mut(), "user1", 1000000, "aWHALE");

    // Accrue rewards again
    TEMP_BALANCE
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();

    let curr_rate = ASSET_REWARD_RATE
        .load(
            deps.as_mut().storage,
            AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())),
        )
        .unwrap();
    assert!(curr_rate > prev_rate);

    // User 1 stakes back
    stake(deps.as_mut(), "user1", 1000000, "aWHALE");

    // User 1 should not have any rewards
    let rewards = query_rewards(deps.as_ref(), "user1", "aWHALE");
    assert_eq!(rewards.rewards, Uint128::zero());

    // User 2 should receive all the rewards in the contract
    let rewards = query_rewards(deps.as_ref(), "user2", "aWHALE");
    assert_eq!(rewards.rewards, Uint128::new(900000));
    let rewards = query_rewards(deps.as_ref(), "user2", "bWHALE");
    assert_eq!(rewards.rewards, Uint128::new(1000000));
}

#[test]
fn claim_rewards_after_rebalancing_emissions() {
    let mut deps = mock_dependencies_with_balance(&[coin(2000000, "uluna")]);
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        Vec::from([
            ModifyAsset{
                asset_info: AssetInfo::Native("aWHALE".to_string()),
                delete: false,
            },
            ModifyAsset{
                asset_info: AssetInfo::Native("bWHALE".to_string()),
                delete: false,
            }
        ]),
    );
    stake(deps.as_mut(), "user1", 1000000, "aWHALE");
    stake(deps.as_mut(), "user2", 1000000, "bWHALE");

    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())), &Decimal::percent(50)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())), &Decimal::percent(50)).unwrap();

    TEMP_BALANCE
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();

    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("aWHALE".to_string())), &Decimal::percent(100)).unwrap();
    WHITELIST.save(deps.as_mut().storage, AssetInfoKey::from(AssetInfo::Native("bWHALE".to_string())), &Decimal::percent(0)).unwrap();

    TEMP_BALANCE
        .save(deps.as_mut().storage, &Uint128::new(1000000))
        .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("cosmos2contract", &[]),
        ExecuteMsg::UpdateRewardsCallback {},
    )
    .unwrap();

    let rewards = query_rewards(deps.as_ref(), "user1", "aWHALE");
    assert_eq!(rewards.rewards, Uint128::new(1500000));
    // User 2 should receive all the rewards in the contract
    let rewards = query_rewards(deps.as_ref(), "user2", "bWHALE");
    assert_eq!(rewards.rewards, Uint128::new(500000));
}
