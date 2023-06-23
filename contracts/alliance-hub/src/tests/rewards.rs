use crate::contract::execute;
use crate::query::query;
use crate::state::{
    ASSET_REWARD_DISTRIBUTION, ASSET_REWARD_RATE, TEMP_BALANCE, TOTAL_BALANCES,
    USER_ASSET_REWARD_RATE, VALIDATORS,
};
use crate::tests::helpers::{
    claim_rewards, query_all_rewards, query_rewards, set_alliance_asset, setup_contract, stake,
    whitelist_assets, DENOM,
};
use alliance_protocol::alliance_protocol::{
    AllPendingRewardsQuery, AssetDistribution, AssetQuery, ExecuteMsg, PendingRewardsRes, QueryMsg,
};
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{
    coin, coins, from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal, Response,
    SubMsg, Uint128, WasmMsg,
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
            &HashSet::from([Addr::unchecked("validator1")]),
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
            SubMsg::new(CosmosMsg::Stargate {
                type_url: "/alliance.alliance.MsgWithdrawDelegatorReward".to_string(),
                value: Binary::from(
                    MsgClaimDelegationRewards {
                        delegator_address: "cosmos2contract".to_string(),
                        validator_address: "validator1".to_string(),
                        denom: DENOM.to_string(),
                    }
                    .encode_to_vec()
                )
            }),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                funds: vec![],
                contract_addr: "cosmos2contract".to_string(),
                msg: to_binary(&ExecuteMsg::UpdateRewardsCallback).unwrap()
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
            &HashSet::from([Addr::unchecked("validator1"), Addr::unchecked("validator2")]),
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
    ASSET_REWARD_DISTRIBUTION
        .save(
            deps.as_mut().storage,
            &vec![
                AssetDistribution {
                    asset: AssetInfo::Native("aWHALE".to_string()),
                    distribution: Decimal::percent(10),
                },
                AssetDistribution {
                    asset: AssetInfo::Native("bWHALE".to_string()),
                    distribution: Decimal::percent(60),
                },
                AssetDistribution {
                    asset: AssetInfo::Native("aMONKEY".to_string()),
                    distribution: Decimal::percent(30),
                },
            ],
        )
        .unwrap();

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
fn claim_user_rewards() {
    let mut deps = mock_dependencies_with_balance(&[coin(2000000, "uluna")]);
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    whitelist_assets(deps.as_mut(), vec![AssetInfo::Native("aWHALE".to_string())]);
    stake(deps.as_mut(), "user1", 1000000, "aWHALE");
    stake(deps.as_mut(), "user2", 4000000, "aWHALE");

    ASSET_REWARD_DISTRIBUTION
        .save(
            deps.as_mut().storage,
            &vec![
                AssetDistribution {
                    asset: AssetInfo::Native("aWHALE".to_string()),
                    distribution: Decimal::percent(50),
                },
                AssetDistribution {
                    asset: AssetInfo::Native("bWHALE".to_string()),
                    distribution: Decimal::percent(50),
                },
            ],
        )
        .unwrap();
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
    whitelist_assets(deps.as_mut(), vec![AssetInfo::Native("aWHALE".to_string())]);
    stake(deps.as_mut(), "user1", 1000000, "aWHALE");
    stake(deps.as_mut(), "user2", 4000000, "aWHALE");

    ASSET_REWARD_DISTRIBUTION
        .save(
            deps.as_mut().storage,
            &vec![
                AssetDistribution {
                    asset: AssetInfo::Native("aWHALE".to_string()),
                    distribution: Decimal::percent(50),
                },
                AssetDistribution {
                    asset: AssetInfo::Native("bWHALE".to_string()),
                    distribution: Decimal::percent(50),
                },
            ],
        )
        .unwrap();
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
}

#[test]
fn claim_rewards_after_staking_and_unstaking() {

}