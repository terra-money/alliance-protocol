use crate::contract::execute;
use crate::state::{
    AssetDistribution, Config, ASSET_REWARD_DISTRIBUTION, ASSET_REWARD_RATE, CONFIG, TEMP_BALANCE,
    TOTAL_BALANCES, VALIDATORS,
};
use crate::tests::helpers::{set_alliance_asset, setup_contract, DENOM};
use alliance_protocol::alliance_protocol::ExecuteMsg;
use cosmwasm_std::testing::{
    mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
};
use cosmwasm_std::{
    coin, to_binary, Addr, Binary, CosmosMsg, Decimal, Response, StdError, StdResult, SubMsg,
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
