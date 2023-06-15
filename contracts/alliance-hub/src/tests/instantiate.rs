use crate::contract::reply;
use crate::state::CONFIG;
use crate::tests::helpers::setup_contract;
use crate::token_factory::{CustomExecuteMsg, DenomUnit, Metadata, TokenExecuteMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{
    Binary, CosmosMsg, Reply, Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128,
};

#[test]
fn test_setup_contract() {
    let mut deps = mock_dependencies();
    let res = setup_contract(deps.as_mut());
    let denom = "ualliance";
    let symbol = "ALLIANCE";
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![("action", "instantiate"),])
            .add_submessage(SubMsg::reply_on_success(
                CosmosMsg::Custom(CustomExecuteMsg::Token(TokenExecuteMsg::CreateDenom {
                    subdenom: denom.to_string(),
                    metadata: Metadata {
                        description: "Staking token for the alliance protocol".to_string(),
                        denom_units: vec![DenomUnit {
                            denom: "ualliance".to_string(),
                            exponent: 0,
                            aliases: vec![],
                        }],
                        base: denom.to_string(),
                        display: symbol.to_string(),
                        name: "Alliance Token".to_string(),
                        symbol: symbol.to_string(),
                    },
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
            data: Some(Binary::from(&[
                10, 17, 116, 111, 107, 101, 110, 95, 102, 97, 99, 116, 111, 114, 121, 47, 119, 116,
                102,
            ])),
        }),
    };
    let res = reply(deps.as_mut(), mock_env(), msg).unwrap();
    let sub_msg = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
        TokenExecuteMsg::MintTokens {
            amount: Uint128::from(1000000000000u128),
            denom: "token_factory/wtf".to_string(),
            mint_to_address: "cosmos2contract".to_string(),
        },
    )));
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![
                ("alliance_token_denom", "token_factory/wtf"),
                ("alliance_token_total_supply", "1000000000000"),
            ])
            .add_submessage(sub_msg)
    );

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.alliance_token_denom, "token_factory/wtf");
    assert_eq!(
        config.alliance_token_supply,
        Uint128::from(1000000000000u128)
    );
}
