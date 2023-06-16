use crate::contract::execute;
use crate::error::ContractError;
use crate::state::{Config, CONFIG};
use crate::tests::helpers::{alliance_delegate, setup_contract};
use alliance_protocol::alliance_protocol::{AllianceDelegateMsg, AllianceDelegation, ExecuteMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Binary, CosmosMsg, StdResult, SubMsg, Uint128};
use terra_proto_rs::alliance::alliance::MsgDelegate;
use terra_proto_rs::cosmos::base::v1beta1::Coin;
use terra_proto_rs::traits::Message;

#[test]
fn test_alliance_delegate() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let denom = "token_factory/token";
    // set alliance token denom
    CONFIG
        .update(deps.as_mut().storage, |c| -> StdResult<_> {
            Ok(Config {
                alliance_token_denom: denom.to_string(),
                alliance_token_supply: Uint128::new(1000000000000),
                ..c
            })
        })
        .unwrap();
    let res = alliance_delegate(
        deps.as_mut(),
        vec![("validator1", 100), ("validator2", 400)],
    );

    let delegate_msg_1 = MsgDelegate {
        amount: Some(Coin {
            denom: denom.to_string(),
            amount: "200000000000".to_string(),
        }),
        delegator_address: "cosmos2contract".to_string(),
        validator_address: "validator1".to_string(),
    };
    let delegate_msg_2 = MsgDelegate {
        amount: Some(Coin {
            denom: denom.to_string(),
            amount: "800000000000".to_string(),
        }),
        delegator_address: "cosmos2contract".to_string(),
        validator_address: "validator2".to_string(),
    };

    assert_eq!(res.messages.len(), 2);
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Stargate {
                type_url: "/alliance.alliance.MsgDelegate".to_string(),
                value: Binary::from(delegate_msg_1.encode_to_vec()),
            }),
            SubMsg::new(CosmosMsg::Stargate {
                type_url: "/alliance.alliance.MsgDelegate".to_string(),
                value: Binary::from(delegate_msg_2.encode_to_vec()),
            }),
        ]
    );
}

#[test]
fn test_alliance_delegation_invalid() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    let denom = "token_factory/token";
    // set alliance token denom
    CONFIG
        .update(deps.as_mut().storage, |c| -> StdResult<_> {
            Ok(Config {
                alliance_token_denom: denom.to_string(),
                alliance_token_supply: Uint128::new(1000000000000),
                ..c
            })
        })
        .unwrap();
    let info = mock_info("user", &vec![]);
    let msg = AllianceDelegateMsg {
        delegations: vec![AllianceDelegation {
            validator: "validator1".to_string(),
            amount: Uint128::new(100),
        }],
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::AllianceDelegate(msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let info = mock_info("controller", &vec![]);
    let msg = AllianceDelegateMsg {
        delegations: vec![],
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::AllianceDelegate(msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::EmptyDelegation {});
}
