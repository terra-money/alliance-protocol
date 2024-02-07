use crate::models::{AddressStakedBalancesQuery, QueryMsg, StakedAssetQuery, StakedBalanceRes};

use crate::query::query;
use crate::state::{BALANCES, WHITELIST};
use crate::tests::helpers::{set_alliance_asset, setup_contract};
use crate::tests::mock_querier::mock_dependencies;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, from_json, Addr, Decimal, Uint128};
use cw_asset::{AssetInfo, AssetInfoKey};
use std::borrow::BorrowMut;

#[test]
fn test_query_address_staked_balance() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    let addr_key = Addr::unchecked("usr_addr");
    let deposit_key = AssetInfoKey::from(AssetInfo::Native("deposit_native".to_string()));
    let deposit_key2 = AssetInfoKey::from(AssetInfo::Native("deposit_native2".to_string()));

    WHITELIST
        .save(
            deps.storage.borrow_mut(),
            deposit_key.clone(),
            &Decimal::one(),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.storage.borrow_mut(),
            deposit_key2.clone(),
            &Decimal::one(),
        )
        .unwrap();

    let key = (addr_key.clone(), deposit_key.clone());
    BALANCES
        .save(deps.storage.borrow_mut(), key, &Uint128::new(1))
        .unwrap();

    let res: Vec<StakedBalanceRes> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AddressStakedBalances(AddressStakedBalancesQuery {
                address: "usr_addr".to_string(),
            }),
        )
        .unwrap(),
    )
    .unwrap();

    let expected: Vec<StakedBalanceRes> = vec![
        StakedBalanceRes {
            deposit_asset: AssetInfo::Native("deposit_native".to_string()),
            balance: Uint128::one(),
        },
        StakedBalanceRes {
            deposit_asset: AssetInfo::Native("deposit_native2".to_string()),
            balance: Uint128::zero(),
        },
    ];

    assert_eq!(res, expected);
}

#[test]
fn test_query_address_staked_balance_by_token() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    let addr_key = Addr::unchecked("usr_addr");
    let deposit_key = AssetInfoKey::from(AssetInfo::Native("deposit_native".to_string()));
    let deposit_key2 = AssetInfoKey::from(AssetInfo::Native("deposit_native2".to_string()));

    WHITELIST
        .save(
            deps.storage.borrow_mut(),
            deposit_key.clone(),
            &Decimal::one(),
        )
        .unwrap();
    WHITELIST
        .save(
            deps.storage.borrow_mut(),
            deposit_key2.clone(),
            &Decimal::one(),
        )
        .unwrap();

    let key = (addr_key.clone(), deposit_key.clone());
    BALANCES
        .save(deps.storage.borrow_mut(), key, &Uint128::new(1))
        .unwrap();

    let res: StakedBalanceRes = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::StakedBalance(StakedAssetQuery {
                address: "usr_addr".to_string(),
                deposit_asset: AssetInfo::Native("deposit_native".to_string()),
            }),
        )
        .unwrap(),
    )
    .unwrap();

    let expected = StakedBalanceRes {
        deposit_asset: AssetInfo::Native("deposit_native".to_string()),
        balance: Uint128::one(),
    };

    assert_eq!(res, expected);
}

#[test]
fn test_query_address_staked_balance_by_token_to_zero() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());

    let res: StakedBalanceRes = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::StakedBalance(StakedAssetQuery {
                address: "usr_addr".to_string(),
                deposit_asset: AssetInfo::Native("deposit_native".to_string()),
            }),
        )
        .unwrap(),
    )
    .unwrap();

    let expected = StakedBalanceRes {
        deposit_asset: AssetInfo::Native("deposit_native".to_string()),
        balance: Uint128::zero(),
    };

    assert_eq!(res, expected);
}
