use crate::contract::reply;
use crate::query::query;
use crate::tests::helpers::setup_contract;
use crate::token_factory::{CustomExecuteMsg, DenomUnit, Metadata, TokenExecuteMsg};
use alliance_protocol::alliance_protocol::{Config, QueryMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{
    from_json, Addr, Binary, CosmosMsg, Reply, Response, SubMsg, SubMsgResponse, SubMsgResult,
    Timestamp, Uint128, 
};
use terra_proto_rs::traits::MessageExt;

#[test]
fn test_setup_contract() {
    let mut deps = mock_dependencies();
    let res = setup_contract(deps.as_mut());
    let denom = "ualliance";
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![("action", "instantiate"),])
            .add_submessage(SubMsg::reply_on_success(
                CosmosMsg::Custom(CustomExecuteMsg::Token(TokenExecuteMsg::CreateDenom {
                    subdenom: denom.to_string(),
                })),
                1
            ))
    );

    // Instantiate is a two steps process that's why
    // alliance_token_denom and alliance_token_supply
    // will be populated on reply.
    let query_config = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_json(&query_config).unwrap();
    assert_eq!(
        config,
        Config {
            governance: Addr::unchecked("gov"),
            controller: Addr::unchecked("controller"),
            oracle: Addr::unchecked("oracle"),
            reward_denom: "uluna".to_string(),
            alliance_token_denom: "".to_string(),
            alliance_token_supply: Uint128::new(0),
            last_reward_update_timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_reply_create_token() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    // Build reply message
    let msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(Binary::from(
                String::from("factory/cosmos2contract/ualliance")
                    .to_bytes()
                    .unwrap(),
            )),
        }),
    };
    let res = reply(deps.as_mut(), mock_env(), msg).unwrap();
    let sub_msg = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
        TokenExecuteMsg::MintTokens {
            amount: Uint128::from(1000000000000u128),
            denom: "factory/cosmos2contract/ualliance".to_string(),
            mint_to_address: "cosmos2contract".to_string(),
        },
    )));
    let sub_msg_metadata = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
        TokenExecuteMsg::SetMetadata {
            denom: "factory/cosmos2contract/ualliance".to_string(),
            metadata: Metadata {
                description: "Staking token for the alliance protocol".to_string(),
                denom_units: vec![DenomUnit {
                    denom: "factory/cosmos2contract/ualliance".to_string(),
                    exponent: 0,
                    aliases: vec![],
                }],
                base: "factory/cosmos2contract/ualliance".to_string(),
                display: "factory/cosmos2contract/ualliance".to_string(),
                name: "Alliance Token".to_string(),
                symbol: "ALLIANCE".to_string(),
            },
        },
    )));
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("alliance_token_denom", "factory/cosmos2contract/ualliance"),
                ("alliance_token_total_supply", "1000000000000"),
            ])
            .add_submessage(sub_msg)
            .add_submessage(sub_msg_metadata)
    );

    let query_config = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_json(&query_config).unwrap();
    assert_eq!(
        config,
        Config {
            governance: Addr::unchecked("gov"),
            controller: Addr::unchecked("controller"),
            oracle: Addr::unchecked("oracle"),
            reward_denom: "uluna".to_string(),
            alliance_token_denom: "factory/cosmos2contract/ualliance".to_string(),
            alliance_token_supply: Uint128::new(1000000000000),
            last_reward_update_timestamp: Timestamp::default(),
        }
    );
}
