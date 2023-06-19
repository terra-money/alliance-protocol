use crate::contract::reply;
use crate::state::CONFIG;
use crate::tests::helpers::setup_contract;
use crate::token_factory::{CustomExecuteMsg, DenomUnit, Metadata, TokenExecuteMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{
    Binary, CosmosMsg, Reply, Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128,
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
            data: Some(Binary::from(String::from("ualliance").to_bytes().unwrap())),
        }),
    };
    let res = reply(deps.as_mut(), mock_env(), msg).unwrap();
    let sub_msg = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
        TokenExecuteMsg::MintTokens {
            amount: Uint128::from(1000000000000u128),
            denom: "ualliance".to_string(),
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

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.alliance_token_denom, "ualliance");
    assert_eq!(
        config.alliance_token_supply,
        Uint128::from(1000000000000u128)
    );
}
