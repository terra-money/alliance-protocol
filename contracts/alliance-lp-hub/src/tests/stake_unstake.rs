use crate::contract::execute;
use crate::models::{ExecuteMsg, StakedBalanceRes, ModifyAsset};
use crate::state::{BALANCES, TOTAL_BALANCES};
use crate::tests::helpers::{
    query_all_staked_balances, setup_contract, stake, unstake, modify_asset, stake_cw20
};
use alliance_protocol::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Addr, BankMsg, CosmosMsg, Response, Uint128, Decimal};
use cw_asset::{Asset, AssetInfo, AssetInfoKey};
use std::collections::HashMap;

#[test]
fn test_stake() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![
            ModifyAsset {
                asset_info: AssetInfo::native(Addr::unchecked("native_asset")),
                rewards_rate: Some(Decimal::new(Uint128::new(500_000_000_000_000_000u128))),
                delete: false,
            }
        ]
    );

    let res = stake(deps.as_mut(), "user1", 100, "native_asset");
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "native:native_asset"),
            ("amount", "100"),
        ])
    );

    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("native_asset".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(100));

    // Stake more
    let res = stake(deps.as_mut(), "user1", 100, "native_asset");
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "native:native_asset"),
            ("amount", "100"),
        ])
    );
    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("native_asset".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(200));

    let total_balance = TOTAL_BALANCES
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("native_asset".to_string())),
        )
        .unwrap();
    assert_eq!(total_balance, Uint128::new(200));

    let total_balances_res = query_all_staked_balances(deps.as_ref());
    assert_eq!(
        total_balances_res,
        vec![StakedBalanceRes {
            asset: AssetInfo::Native("native_asset".to_string()),
            balance: Uint128::new(200),
        }]
    );
}

#[test]
fn test_stake_cw20() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![
            ModifyAsset {
                asset_info: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
                rewards_rate: Some(Decimal::new(Uint128::new(500_000_000_000_000_000u128))),
                delete: false,
            }
        ]
    );

    let res = stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset");
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "cw20:cw20_asset"),
            ("amount", "100"),
        ])
    );

    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Cw20(Addr::unchecked("cw20_asset"))),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(100));

    // Stake more
    let res = stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset");
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "cw20:cw20_asset"),
            ("amount", "100"),
        ])
    );
    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Cw20(Addr::unchecked("cw20_asset"))),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(200));

    let total_balance = TOTAL_BALANCES
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Cw20(Addr::unchecked("cw20_asset"))),
        )
        .unwrap();
    assert_eq!(total_balance, Uint128::new(200));

    let total_balances_res = query_all_staked_balances(deps.as_ref());
    assert_eq!(
        total_balances_res,
        vec![StakedBalanceRes {
            asset: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
            balance: Uint128::new(200),
        }]
    );
}

#[test]
fn test_unstake() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![
            ModifyAsset {
                asset_info: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
                rewards_rate: Some(Decimal::new(Uint128::new(500_000_000_000_000_000u128))),
                delete: false,
            }
        ]
    );
    stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset");

    let res = unstake(deps.as_mut(), "user1", 50, "cw20_asset");
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

    modify_asset(
        deps.as_mut(),
        vec![
            ModifyAsset {
                asset_info: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
                rewards_rate: Some(Decimal::new(Uint128::new(500_000_000_000_000_000u128))),
                delete: false,
            }
        ]
    );
    stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset");

    // User does not have any staked asset
    let info = mock_info("user2", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("cw20_asset", 100u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes more than they have
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("cw20_asset", 101u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes zero amount
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("cw20_asset", 0u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::AmountCannotBeZero {});
}
