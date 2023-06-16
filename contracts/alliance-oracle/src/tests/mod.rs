use crate::contract::{execute, query};
use alliance_protocol::alliance_oracle_types::{
    ChainId, ChainInfo, ChainInfoMsg, ChainsInfo, ExecuteMsg, LunaAlliance, LunaInfo, NativeToken,
    QueryMsg,
};
use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    Decimal,
};
use std::str::FromStr;

pub mod test_utils;

#[test]
fn test_update_oracle_data() {
    // Create the environment where the contract will be executed
    let mut deps = test_utils::setup_contract();

    // Msg representing off-chain oracle data that is send to the oracle contract
    let msg = ExecuteMsg::UpdateChainsInfo {
        chains_info: ChainsInfo {
            luna_price: Decimal::from_str("0.589013565473308100").unwrap(),
            protocols_info: vec![ChainInfoMsg {
                chain_id: "chain-1".to_string(),
                native_token: NativeToken {
                    denom: "udenom".to_string(),
                    token_price: Decimal::from_str("0.028076098081623823").unwrap(),
                    annual_inflation: Decimal::from_str("0.04").unwrap(),
                },
                luna_alliances: vec![LunaAlliance {
                    ibc_denom: String::from(
                        "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
                    ),
                    normalized_reward_weight: Decimal::from_str("0.023809523809523810").unwrap(),
                    annual_take_rate: Decimal::from_str("0.009999998624824108").unwrap(),
                    total_lsd_staked: Decimal::from_str("126195.966393").unwrap(),
                    rebase_factor: Decimal::from_str("1.178655688356438636").unwrap(),
                }],
            }],
        },
    };

    // Create the controller_addr sender to successfully send the transaction to the contract
    let info = mock_info("controller_addr", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(1, res.attributes.len());
    assert_eq!("action", res.attributes[0].key);
    assert_eq!("update_chains_info", res.attributes[0].value);

    // Query the chains info to validate the data was stored correctly in the contract
    let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryChainsInfo).unwrap();
    let res: Vec<(ChainId, ChainInfo)> = from_binary(&res).unwrap();
    assert_eq!(1, res.len());

    let key = res[0].0.clone();
    assert_eq!("chain-1", key);

    let chain_info = res[0].1.clone();
    assert_eq!(
        NativeToken {
            denom: "udenom".to_string(),
            token_price: Decimal::from_str("0.028076098081623823").unwrap(),
            annual_inflation: Decimal::from_str("0.04").unwrap(),
        },
        chain_info.native_token
    );

    let luna_alliances = chain_info.luna_alliances.clone();
    assert_eq!(1, luna_alliances.len());
    assert_eq!(
        LunaAlliance {
            ibc_denom: String::from(
                "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
            ),
            normalized_reward_weight: Decimal::from_str("0.023809523809523810").unwrap(),
            annual_take_rate: Decimal::from_str("0.009999998624824108").unwrap(),
            total_lsd_staked: Decimal::from_str("126195.966393").unwrap(),
            rebase_factor: Decimal::from_str("1.178655688356438636").unwrap(),
        },
        luna_alliances[0]
    );

    // Query the Luna info to validate the data was stored correctly in the contract
    let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryLunaInfo).unwrap();
    let luna_info: LunaInfo = from_binary(&res).unwrap();
    assert_eq!(
        Decimal::from_str("0.589013565473308100").unwrap(),
        luna_info.luna_price
    );

    // Query the chain info by id to validate the data was stored correctly in the contract
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::QueryChainInfo {
            chain_id: String::from("chain-1"),
        },
    )
    .unwrap();
    let chain_info: ChainInfo = from_binary(&res).unwrap();
    assert_eq!("chain-1", chain_info.chain_id);
    assert_eq!(
        NativeToken {
            denom: "udenom".to_string(),
            token_price: Decimal::from_str("0.028076098081623823").unwrap(),
            annual_inflation: Decimal::from_str("0.04").unwrap(),
        },
        chain_info.native_token
    );
    assert_eq!(
        LunaAlliance {
            ibc_denom: String::from(
                "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
            ),
            normalized_reward_weight: Decimal::from_str("0.023809523809523810").unwrap(),
            annual_take_rate: Decimal::from_str("0.009999998624824108").unwrap(),
            total_lsd_staked: Decimal::from_str("126195.966393").unwrap(),
            rebase_factor: Decimal::from_str("1.178655688356438636").unwrap(),
        },
        chain_info.luna_alliances[0]
    );
}
