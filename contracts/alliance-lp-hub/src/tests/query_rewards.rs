use crate::models::{AddressPendingRewardsQuery, AssetQuery, PendingRewardsRes, QueryMsg};

use crate::query::query;
use crate::state::{
    TOTAL_ASSET_REWARD_RATE, UNCLAIMED_REWARDS, USER_ASSET_REWARD_RATE, USER_BALANCES,
};
use crate::tests::helpers::{set_alliance_asset, setup_contract};
use crate::tests::mock_querier::mock_dependencies;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, from_json, Addr, Decimal, Uint128};
use cw_asset::{AssetInfo, AssetInfoKey};
use std::borrow::BorrowMut;

#[test]
fn test_query_pending_rewards() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    let addr_key = Addr::unchecked("usr_addr");
    let deposit_key = AssetInfoKey::from(AssetInfo::Native("deposit_native".to_string()));
    let reward_key = AssetInfoKey::from(AssetInfo::Cw20(Addr::unchecked(
        "astro_reward_denom".to_string(),
    )));

    let key = (addr_key.clone(), deposit_key.clone(), reward_key.clone());
    USER_ASSET_REWARD_RATE
        .save(
            deps.storage.borrow_mut(),
            key,
            &Decimal::new(Uint128::new(0)),
        )
        .unwrap();

    let key = (deposit_key.clone(), reward_key.clone());
    TOTAL_ASSET_REWARD_RATE
        .save(
            deps.storage.borrow_mut(),
            key,
            &Decimal::new(Uint128::new(1)),
        )
        .unwrap();

    let key = (addr_key.clone(), reward_key.clone());
    USER_BALANCES
        .save(deps.storage.borrow_mut(), key, &Uint128::new(1))
        .unwrap();

    let key = (addr_key, deposit_key, reward_key);
    UNCLAIMED_REWARDS
        .save(deps.storage.borrow_mut(), key, &Uint128::new(1))
        .unwrap();

    let res: PendingRewardsRes = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::PendingRewards(AssetQuery {
                address: "usr_addr".to_string(),
                deposit_asset: AssetInfo::Native("deposit_native".to_string()),
                reward_asset: AssetInfo::Cw20(Addr::unchecked("astro_reward_denom")),
            }),
        )
        .unwrap(),
    )
    .unwrap();

    let expected = PendingRewardsRes {
        rewards: Uint128::one(),
        deposit_asset: AssetInfo::Native("deposit_native".to_string()),
        reward_asset: AssetInfo::Cw20(Addr::unchecked("astro_reward_denom")),
    };

    assert_eq!(res, expected);
}

#[test]
fn test_query_all_pending_rewards() {
    let mut deps = mock_dependencies(Some(&[coin(1000000, "uluna")]));
    setup_contract(deps.as_mut());
    set_alliance_asset(deps.as_mut());
    let addr_key = Addr::unchecked("usr_addr");
    let deposit_key = AssetInfoKey::from(AssetInfo::Native("deposit_native".to_string()));
    let reward_key = AssetInfoKey::from(AssetInfo::Cw20(Addr::unchecked(
        "astro_reward_denom".to_string(),
    )));

    let key = (addr_key.clone(), deposit_key.clone(), reward_key.clone());
    USER_ASSET_REWARD_RATE
        .save(
            deps.storage.borrow_mut(),
            key,
            &Decimal::new(Uint128::new(1)),
        )
        .unwrap();

    let key = (deposit_key.clone(), reward_key.clone());
    TOTAL_ASSET_REWARD_RATE
        .save(
            deps.storage.borrow_mut(),
            key,
            &Decimal::new(Uint128::new(1)),
        )
        .unwrap();

    let key = (addr_key.clone(), deposit_key.clone());
    USER_BALANCES
        .save(deps.storage.borrow_mut(), key, &Uint128::new(1))
        .unwrap();

    let key = (addr_key, deposit_key, reward_key);
    UNCLAIMED_REWARDS
        .save(deps.storage.borrow_mut(), key, &Uint128::new(1))
        .unwrap();

    let res: Vec<PendingRewardsRes> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AddressPendingRewards(AddressPendingRewardsQuery {
                address: "usr_addr".to_string(),
            }),
        )
        .unwrap(),
    )
    .unwrap();

    let expected = vec![PendingRewardsRes {
        rewards: Uint128::one(),
        deposit_asset: AssetInfo::Native("deposit_native".to_string()),
        reward_asset: AssetInfo::Cw20(Addr::unchecked("astro_reward_denom")),
    }];

    assert_eq!(res, expected);
}
