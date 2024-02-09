use crate::astro_models::{Cw20Msg, ExecuteAstroMsg};
use crate::contract::execute;
use crate::models::{ExecuteMsg, ModifyAssetPair, StakedBalanceRes};
use crate::state::{USER_BALANCES, TOTAL_BALANCES};
use crate::tests::helpers::{
    modify_asset, query_contract_balances, setup_contract, stake, stake_cw20, unstake,
    unstake_callback,
};
use crate::tests::mock_querier::mock_dependencies;
use alliance_protocol::error::ContractError;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    coin, coins, to_json_binary, Addr, Coin, CosmosMsg, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoKey};

#[test]
fn test_stake_multiple_tokens() {
    let mut deps = mock_dependencies(None);
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::native(Addr::unchecked("native_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();

    let info = mock_info(
        "user1",
        &[coin(100, "native_asset"), coin(100, "native_asset2")],
    );
    let env = mock_env();
    let msg = ExecuteMsg::Stake {};
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::OnlySingleAssetAllowed {});

    let info = mock_info("user1", &[coin(0, "native_asset")]);
    let env = mock_env();
    let msg = ExecuteMsg::Stake {};
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::AmountCannotBeZero {});
}

#[test]
fn test_stake() {
    let mut deps = mock_dependencies(None);
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::native(Addr::unchecked("native_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();

    let res = stake(deps.as_mut(), "user1", 100, "native_asset").unwrap();
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "native:native_asset"),
            ("amount", "100"),
        ])
    );

    let balance = USER_BALANCES
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
    let res = stake(deps.as_mut(), "user1", 100, "native_asset").unwrap();
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "native:native_asset"),
            ("amount", "100"),
        ])
    );
    let balance = USER_BALANCES
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

    let total_balances_res = query_contract_balances(deps.as_ref());
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
    let mut deps = mock_dependencies(Some(&[Coin::new(1000, "token")]));
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::native(Addr::unchecked("factory/astro_native")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();

    let res = stake(deps.as_mut(), "user1", 100, "factory/astro_native").unwrap();

    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "stake"),
                ("user", "user1"),
                ("asset", "native:factory/astro_native"),
                ("amount", "100"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "astro_incentives".to_string(),
                msg: to_json_binary(&ExecuteAstroMsg::Deposit { recipient: None }).unwrap(),
                funds: vec![Coin {
                    denom: "factory/astro_native".to_string(),
                    amount: Uint128::new(100),
                }],
            }))
    );

    let balance = USER_BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::Native("factory/astro_native".to_string())),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(100));
}

#[test]
fn test_stake_cw20() {
    let mut deps = mock_dependencies(None);
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();

    let res = stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset").unwrap();
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "cw20:cw20_asset"),
            ("amount", "100"),
        ])
    );

    let balance = USER_BALANCES
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
    let res = stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset").unwrap();
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "stake"),
            ("user", "user1"),
            ("asset", "cw20:cw20_asset"),
            ("amount", "100"),
        ])
    );
    let balance = USER_BALANCES
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

    let total_balances_res = query_contract_balances(deps.as_ref());
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
    let mut deps = mock_dependencies(Some(&[Coin::new(1000, "token")]));
    setup_contract(deps.as_mut());
    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::Cw20(Addr::unchecked("terra_astro_cw20")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();

    let res = stake_cw20(deps.as_mut(), "user1", 100, "terra_astro_cw20").unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "stake"),
                ("user", "user1"),
                ("asset", "cw20:terra_astro_cw20"),
                ("amount", "100"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "terra_astro_cw20".to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Send {
                    contract: "astro_incentives".to_string(),
                    amount: Uint128::new(100),
                    msg: to_json_binary(&Cw20Msg::Deposit { recipient: None }).unwrap(),
                })
                .unwrap(),
                funds: vec![],
            }))
    );
}

#[test]
fn test_unstake() {
    let mut deps = mock_dependencies(None);
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::native("native_asset"),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();
    stake(deps.as_mut(), "user1", 100, "native_asset").unwrap();

    let asset_info = Asset::native(Addr::unchecked("native_asset"), 50u128);
    let res = unstake(deps.as_mut(), "user1", asset_info).unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "unstake_alliance_lp"),
                ("user", "user1"),
                ("asset", "native:native_asset"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "cosmos2contract".to_string(),
                msg: to_json_binary(&ExecuteMsg::UnstakeCallback(
                    Asset::native("native_asset", Uint128::new(50)),
                    Addr::unchecked("user1"),
                ))
                .unwrap(),
                funds: vec![],
            }))
    );

    let balance = USER_BALANCES
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
    let res = unstake(deps.as_mut(), "user1", asset_info).unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("action", "unstake_alliance_lp"),
                ("user", "user1"),
                ("asset", "native:native_asset"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "cosmos2contract".to_string(),
                msg: to_json_binary(&ExecuteMsg::UnstakeCallback(
                    Asset::native("native_asset", Uint128::new(50)),
                    Addr::unchecked("user1"),
                ))
                .unwrap(),
                funds: vec![],
            }))
    );

    let balance = USER_BALANCES
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
fn test_unstake_cw20_from_astro() {
    let mut deps = mock_dependencies(Some(&coins(100, "terra_astro_cw20")));
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::cw20(Addr::unchecked("terra_astro_cw20")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();
    stake_cw20(deps.as_mut(), "user1", 100, "terra_astro_cw20").unwrap();

    let asset_info = Asset::cw20(Addr::unchecked("terra_astro_cw20"), 50u128);
    let res = unstake(deps.as_mut(), "user1", asset_info).unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "astro_incentives".to_string(),
                msg: to_json_binary(&ExecuteAstroMsg::Withdraw {
                    lp_token: "terra_astro_cw20".to_string(),
                    amount: Uint128::new(50),
                })
                .unwrap(),
                funds: vec![],
            }))
            .add_attributes(vec![
                ("action", "unstake_alliance_lp"),
                ("user", "user1"),
                ("asset", "cw20:terra_astro_cw20"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "cosmos2contract".to_string(),
                msg: to_json_binary(&ExecuteMsg::UnstakeCallback(
                    Asset::cw20(Addr::unchecked("terra_astro_cw20"), Uint128::new(50)),
                    Addr::unchecked("user1"),
                ))
                .unwrap(),
                funds: vec![],
            }))
    );

    let balance = USER_BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::cw20(Addr::unchecked("terra_astro_cw20"))),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(50));

    let asset_info = Asset::cw20(Addr::unchecked("terra_astro_cw20"), 50u128);
    let res = unstake(deps.as_mut(), "user1", asset_info.clone()).unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "astro_incentives".to_string(),
                msg: to_json_binary(&ExecuteAstroMsg::Withdraw {
                    lp_token: "terra_astro_cw20".to_string(),
                    amount: Uint128::new(50),
                })
                .unwrap(),
                funds: vec![],
            }))
            .add_attributes(vec![
                ("action", "unstake_alliance_lp"),
                ("user", "user1"),
                ("asset", "cw20:terra_astro_cw20"),
                ("amount", "50"),
            ])
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "cosmos2contract".to_string(),
                msg: to_json_binary(&ExecuteMsg::UnstakeCallback(
                    Asset::cw20(Addr::unchecked("terra_astro_cw20"), Uint128::new(50)),
                    Addr::unchecked("user1"),
                ))
                .unwrap(),
                funds: vec![],
            }))
    );

    let res = unstake_callback(
        deps.as_mut(),
        "cosmos2contract",
        "user1",
        asset_info.clone(),
    );
    assert_eq!(
        res.unwrap(),
        Response::new()
            .add_message(asset_info.transfer_msg(Addr::unchecked("user1")).unwrap())
            .add_attributes(vec![("action", "unstake_alliance_lp_callback"),])
    );

    let balance = USER_BALANCES
        .load(
            deps.as_ref().storage,
            (
                Addr::unchecked("user1"),
                AssetInfoKey::from(AssetInfo::cw20(Addr::unchecked("terra_astro_cw20"))),
            ),
        )
        .unwrap();
    assert_eq!(balance, Uint128::new(0));

    let total_balance = TOTAL_BALANCES
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::cw20(Addr::unchecked("terra_astro_cw20"))),
        )
        .unwrap();
    assert_eq!(total_balance, Uint128::new(0));
}

#[test]
fn test_unstake_cw20_invalid() {
    let mut deps = mock_dependencies(None);
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::Cw20(Addr::unchecked("cw20_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();
    stake_cw20(deps.as_mut(), "user1", 100, "cw20_asset").unwrap();

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
    let mut deps = mock_dependencies(None);
    setup_contract(deps.as_mut());

    modify_asset(
        deps.as_mut(),
        vec![ModifyAssetPair {
            asset_distribution: Uint128::new(1),
            asset_info: AssetInfo::native(Addr::unchecked("native_asset")),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }],
    )
    .unwrap();
    stake(deps.as_mut(), "user1", 100, "native_asset").unwrap();

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
