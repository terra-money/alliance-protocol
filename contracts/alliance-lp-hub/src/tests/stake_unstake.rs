use crate::astro_models::{Cw20Msg, ExecuteAstroMsg};
use crate::contract::execute;
use crate::models::{ExecuteMsg, ModifyAssetPair, StakedBalanceRes};
use crate::state::{BALANCES, TOTAL_BALANCES};
use crate::tests::helpers::{
    modify_asset, query_all_staked_balances, setup_contract, stake, stake_cw20, unstake,
};
use crate::tests::mock_querier::mock_dependencies as astro_mock_dependencies;
use alliance_protocol::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Response, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_asset::{Asset, AssetInfo, AssetInfoKey};

#[test]
fn test_stake() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_info: AssetInfo::native(Addr::unchecked("native_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
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
            deposit_asset: AssetInfo::Native("native_asset".to_string()),
            balance: Uint128::new(200),
        }]
    );
}

#[test]
fn test_stake_astro_token() {
    let mut deps = astro_mock_dependencies(&[Coin::new(1000, "token")]);
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_info: AssetInfo::native(Addr::unchecked("astro_existent_native_coin")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    );

    let res = stake(deps.as_mut(), "user1", 100, "astro_existent_native_coin");

    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "stake"),
                ("user", "user1"),
                ("asset", "native:astro_existent_native_coin"),
                ("amount", "100"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "astro_incentives".to_string(),
                msg: to_json_binary(&ExecuteAstroMsg::Deposit { recipient: None }).unwrap(),
                funds: vec![Coin {
                    denom: "astro_existent_native_coin".to_string(),
                    amount: Uint128::new(100),
                }],
            }))
    );

    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("astro_existent_native_coin".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(100));
}

#[test]
fn test_stake_cw20() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_info: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
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
            deposit_asset: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
            balance: Uint128::new(200),
        }]
    );
}

#[test]
fn test_stake_astro_token_cw20() {
    let mut deps = astro_mock_dependencies(&[Coin::new(1000, "token")]);
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_info: AssetInfo::Cw20(Addr::unchecked("astro_existent_cw20")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    );

    let res = stake_cw20(deps.as_mut(), "user1", 100, "astro_existent_cw20");
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "stake"),
                ("user", "user1"),
                ("asset", "cw20:astro_existent_cw20"),
                ("amount", "100"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "astro_existent_cw20".to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Send {
                    contract: "astro_incentives".to_string(),
                    amount: Uint128::new(100),
                    msg: to_json_binary(&Cw20ReceiveMsg {
                        sender: "cosmos2contract".to_string(),
                        amount: Uint128::new(100),
                        msg: to_json_binary(&Cw20Msg::Deposit { recipient: None }).unwrap(),
                    })
                    .unwrap(),
                })
                .unwrap(),
                funds: vec![],
            }))
    );
}

#[test]
fn test_unstake() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_info: AssetInfo::native("native_asset"),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    );
    stake(deps.as_mut(), "user1", 100, "native_asset");

    let asset_info = Asset::native(Addr::unchecked("native_asset"), 50u128);
    let res = unstake(deps.as_mut(), "user1", asset_info);
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "unstake"),
                ("user", "user1"),
                ("asset", "native:native_asset"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".into(),
                amount: vec![coin(50, "native_asset")],
            }))
    );

    let balance = BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::native(Addr::unchecked("native_asset"))),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(50));

    let asset_info = Asset::native(Addr::unchecked("native_asset"), 50u128);
    let res = unstake(deps.as_mut(), "user1", asset_info);
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "unstake"),
                ("user", "user1"),
                ("asset", "native:native_asset"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".into(),
                amount: vec![coin(50, "native_asset")],
            }))
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
    assert_eq!(balance, Uint128::new(0));

    let total_balance = TOTAL_BALANCES
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("native_asset".to_string())),
        )
        .unwrap();
    assert_eq!(total_balance, Uint128::new(0));
}

#[test]
fn test_unstake_cw20_invalid() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_info: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    );
    stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset");

    // User does not have any staked asset
    let info = mock_info("user2", &[]);
    let msg = ExecuteMsg::Unstake(Asset::cw20(Addr::unchecked("cw20_asset"), 100u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes more than they have
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::cw20(Addr::unchecked("cw20_asset"), 101u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes zero amount
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::cw20(Addr::unchecked("cw20_asset"), 0u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::AmountCannotBeZero {});
}

#[test]
fn test_unstake_native_invalid() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_info: AssetInfo::native(Addr::unchecked("native_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    );
    stake(deps.as_mut(), "user1", 100, "native_asset");

    // User does not have any staked asset
    let info = mock_info("user2", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("native_asset", 100u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes more than they have
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("native_asset", 101u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientBalance {});

    // User unstakes zero amount
    let info = mock_info("user1", &[]);
    let msg = ExecuteMsg::Unstake(Asset::native("native_asset", 0u128));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::AmountCannotBeZero {});
}
