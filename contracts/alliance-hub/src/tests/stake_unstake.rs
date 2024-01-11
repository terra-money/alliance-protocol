use crate::contract::execute;
use crate::models::{ExecuteMsg, StakedBalanceRes};
use crate::state::{BALANCES, TOTAL_BALANCES};
use crate::tests::helpers::{
    query_all_staked_balances, setup_contract, stake, unstake, whitelist_assets,
};
use alliance_protocol::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Addr, BankMsg, CosmosMsg, Response, Uint128};
use cw_asset::{Asset, AssetInfo, AssetInfoKey};
use std::collections::HashMap;

#[test]
fn test_stake() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    whitelist_assets(
        deps.as_mut(),
        HashMap::from([(
            "chain-1".to_string(),
            vec![AssetInfo::Native("asset1".to_string())],
        )]),
    );

    let res = stake(deps.as_mut(), "user1", 100, "asset1");
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "native:asset1"),
            ("amount", "100"),
        ])
    );

    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("asset1".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(100));

    // Stake more
    let res = stake(deps.as_mut(), "user1", 100, "asset1");
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "native:asset1"),
            ("amount", "100"),
        ])
    );
    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("asset1".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(200));

    let total_balance = TOTAL_BALANCES
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("asset1".to_string())),
        )
        .unwrap();
    assert_eq!(total_balance, Uint128::new(200));

    let total_balances_res = query_all_staked_balances(deps.as_ref());
    assert_eq!(
        total_balances_res,
        vec![StakedBalanceRes {
            asset: AssetInfo::Native("asset1".to_string()),
            balance: Uint128::new(200),
        }]
    );
}

#[test]
fn test_stake_invalid() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    whitelist_assets(
        deps.as_mut(),
        HashMap::from([(
            "chain-1".to_string(),
            vec![AssetInfo::Native("asset1".to_string())],
        )]),
    );
    // Stake an unwhitelisted asset
    let msg = ExecuteMsg::Stake {};
    let info = mock_info("user1", &[coin(100, "asset2")]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::AssetNotWhitelisted("native:asset1".to_string()));

    // Stake multiple assets in a single call
    let msg = ExecuteMsg::Stake {};
    let info = mock_info("user1", &[coin(100, "asset1"), coin(100, "asset2")]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::OnlySingleAssetAllowed {});

    // Stake nothing in a single call
    let msg = ExecuteMsg::Stake {};
    let info = mock_info("user1", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::OnlySingleAssetAllowed {});

    // Stake zero amount
    let msg = ExecuteMsg::Stake {};
    let info = mock_info("user1", &[coin(0, "asset1")]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::AmountCannotBeZero {});
}

#[test]
fn test_unstake() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    whitelist_assets(
        deps.as_mut(),
        HashMap::from([(
            "chain-1".to_string(),
            vec![AssetInfo::Native("asset1".to_string())],
        )]),
    );
    stake(deps.as_mut(), "user1", 100, "asset1");

    let res = unstake(deps.as_mut(), "user1", 50, "asset1");
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "unstake"),
                ("user", "user1"),
                ("asset", "native:asset1"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".into(),
                amount: vec![coin(50, "asset1")],
            }))
    );

    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("asset1".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(50));

    let res = unstake(deps.as_mut(), "user1", 50, "asset1");
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "unstake"),
                ("user", "user1"),
                ("asset", "native:asset1"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".into(),
                amount: vec![coin(50, "asset1")],
            }))
    );

    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("asset1".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(0));

    let total_balance = TOTAL_BALANCES
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("asset1".to_string())),
        )
        .unwrap();
    assert_eq!(total_balance, Uint128::new(0));
}

#[test]
fn test_unstake_invalid() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    whitelist_assets(
        deps.as_mut(),
        HashMap::from([(
            "chain-1".to_string(),
            vec![AssetInfo::Native("asset1".to_string())],
        )]),
    );
    stake(deps.as_mut(), "user1", 100, "asset1");

    // User does not have any staked asset
    let info = mock_info("user2", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("asset1", 100u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes more than they have
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("asset1", 101u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes zero amount
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("asset1", 0u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::AmountCannotBeZero {});
}
