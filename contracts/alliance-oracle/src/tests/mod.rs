use crate::contract::{execute, query};
use crate::state::{CHAINS_INFO, LUNA_INFO};
use alliance_protocol::alliance_oracle_types::{
    AssetStaked, BaseAlliance, ChainInfo, ChainInfoMsg, ChainsInfo, EmissionsDistribution,
    ExecuteMsg, LunaAlliance, LunaInfo, NativeToken, QueryMsg,
};
use alliance_protocol::signed_decimal::SignedDecimal;
use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info},
    Decimal, Uint128,
};
use std::collections::HashMap;
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
                    annual_provisions: Decimal::from_str("0.04").unwrap(),
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
                chain_alliances_on_phoenix: vec![BaseAlliance {
                    ibc_denom: String::from("ibc/randomd_denom"),
                    rebase_factor: Decimal::from_str("1.22").unwrap(),
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
    let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryChainsInfo {}).unwrap();
    let res: Vec<ChainInfo> = from_json(&res).unwrap();
    assert_eq!(1, res.len());

    let chain_info = res[0].clone();
    assert_eq!(
        NativeToken {
            denom: "udenom".to_string(),
            token_price: Decimal::from_str("0.028076098081623823").unwrap(),
            annual_provisions: Decimal::from_str("0.04").unwrap(),
        },
        chain_info.native_token
    );

    assert_eq!(1, chain_info.luna_alliances.len());
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
    assert_eq!(1, chain_info.chain_alliances_on_phoenix.len());
    assert_eq!(
        BaseAlliance {
            ibc_denom: String::from("ibc/randomd_denom",),
            rebase_factor: Decimal::from_str("1.22").unwrap(),
        },
        chain_info.chain_alliances_on_phoenix[0]
    );

    // Query the Luna info to validate the data was stored correctly in the contract
    let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryLunaInfo {}).unwrap();
    let luna_info: LunaInfo = from_json(&res).unwrap();
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
    let chain_info: ChainInfo = from_json(&res).unwrap();
    assert_eq!("chain-1", chain_info.chain_id);
    assert_eq!(
        NativeToken {
            denom: "udenom".to_string(),
            token_price: Decimal::from_str("0.028076098081623823").unwrap(),
            annual_provisions: Decimal::from_str("0.04").unwrap(),
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

#[test]
fn test_emissions_distribution() {
    let mut deps = test_utils::setup_contract();
    let env = mock_env();
    CHAINS_INFO
        .save(
            deps.as_mut().storage,
            &vec![ChainInfo {
                chain_id: "chain-1".to_string(),
                update_timestamp: env.block.time,
                native_token: NativeToken {
                    denom: "udenom".to_string(),
                    token_price: Decimal::from_str("0.006").unwrap(),
                    annual_provisions: Decimal::from_str("40000000").unwrap(),
                },
                luna_alliances: vec![LunaAlliance {
                    ibc_denom: String::from(
                        "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
                    ),
                    normalized_reward_weight: Decimal::from_str("0.01").unwrap(),
                    annual_take_rate: Decimal::from_str("0.003").unwrap(),
                    total_lsd_staked: Decimal::from_str("21979").unwrap(),
                    rebase_factor: Decimal::from_str("1").unwrap(),
                }],
                chain_alliances_on_phoenix: vec![BaseAlliance {
                    ibc_denom: String::from("ibc/randomd_denom"),
                    rebase_factor: Decimal::from_str("1").unwrap(),
                }],
            }],
        )
        .unwrap();

    LUNA_INFO
        .save(
            deps.as_mut().storage,
            &LunaInfo {
                luna_price: Decimal::from_str("0.61").unwrap(),
                update_timestamp: env.block.time,
            },
        )
        .unwrap();

    let msg = QueryMsg::QueryEmissionsDistributions(HashMap::from([(
        "chain-1".to_string(),
        vec![AssetStaked {
            denom: "ibc/randomd_denom".to_string(),
            amount: Uint128::new(1_000_000),
        }],
    )]));

    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let res_parsed: Vec<EmissionsDistribution> = from_json(&res).unwrap();
    assert_eq!(
        res_parsed,
        vec![EmissionsDistribution {
            denom: "ibc/randomd_denom".to_string(),
            distribution: SignedDecimal::from_str("2359.77843").unwrap(),
        }]
    )
}

#[test]
fn test_emissions_distribution_2() {
    let mut deps = test_utils::setup_contract();
    let env = mock_env();
    CHAINS_INFO
        .save(
            deps.as_mut().storage,
            &vec![
                ChainInfo {
                    chain_id: "chain-1".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom".to_string(),
                        token_price: Decimal::from_str("0.006").unwrap(),
                        annual_provisions: Decimal::from_str("40000000").unwrap(),
                    },
                    luna_alliances: vec![LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.01").unwrap(),
                        annual_take_rate: Decimal::from_str("0.003").unwrap(),
                        total_lsd_staked: Decimal::from_str("21979").unwrap(),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
                ChainInfo {
                    chain_id: "chain-2".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom2".to_string(),
                        token_price: Decimal::from_str("0.02337").unwrap(),
                        annual_provisions: Decimal::from_str("24304822.32").unwrap(),
                    },
                    luna_alliances: vec![LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889555",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.0238").unwrap(),
                        annual_take_rate: Decimal::from_str("0.01").unwrap(),
                        total_lsd_staked: Decimal::from_str("116527.585").unwrap(),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom2"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
            ],
        )
        .unwrap();

    LUNA_INFO
        .save(
            deps.as_mut().storage,
            &LunaInfo {
                luna_price: Decimal::from_str("0.61").unwrap(),
                update_timestamp: env.block.time,
            },
        )
        .unwrap();

    let msg = QueryMsg::QueryEmissionsDistributions(HashMap::from([
        (
            "chain-1".to_string(),
            vec![AssetStaked {
                denom: "ibc/randomd_denom".to_string(),
                amount: Uint128::new(1_000_000),
            }],
        ),
        (
            "chain-2".to_string(),
            vec![AssetStaked {
                denom: "ibc/randomd_denom2".to_string(),
                amount: Uint128::new(1_000_000),
            }],
        ),
    ]));

    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let res_parsed: Vec<EmissionsDistribution> = from_json(&res).unwrap();
    assert_eq!(
        res_parsed,
        vec![
            EmissionsDistribution {
                denom: "ibc/randomd_denom".to_string(),
                distribution: SignedDecimal::from_str("2359.77843").unwrap(),
            },
            EmissionsDistribution {
                denom: "ibc/randomd_denom2".to_string(),
                distribution: SignedDecimal::from_str("12807.66973481792").unwrap(),
            }
        ]
    )
}

#[test]
fn test_emissions_distribution_3() {
    let mut deps = test_utils::setup_contract();
    let env = mock_env();
    CHAINS_INFO
        .save(
            deps.as_mut().storage,
            &vec![
                ChainInfo {
                    chain_id: "chain-1".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom".to_string(),
                        token_price: Decimal::from_str("0.006").unwrap(),
                        annual_provisions: Decimal::from_str("40000000").unwrap(),
                    },
                    luna_alliances: vec![LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.01").unwrap(),
                        annual_take_rate: Decimal::from_str("0.003").unwrap(),
                        total_lsd_staked: Decimal::from_str("21979").unwrap(),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
                ChainInfo {
                    chain_id: "chain-2".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom2".to_string(),
                        token_price: Decimal::from_str("0.02337").unwrap(),
                        annual_provisions: Decimal::from_str("24304822.32").unwrap(),
                    },
                    luna_alliances: vec![LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889555",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.0238").unwrap(),
                        annual_take_rate: Decimal::from_str("0.01").unwrap(),
                        total_lsd_staked: Decimal::from_str("58263.7925").unwrap(),
                        rebase_factor: Decimal::from_str("2").unwrap(),
                    }],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom2"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
            ],
        )
        .unwrap();

    LUNA_INFO
        .save(
            deps.as_mut().storage,
            &LunaInfo {
                luna_price: Decimal::from_str("0.61").unwrap(),
                update_timestamp: env.block.time,
            },
        )
        .unwrap();

    let msg = QueryMsg::QueryEmissionsDistributions(HashMap::from([
        (
            "chain-1".to_string(),
            vec![AssetStaked {
                denom: "ibc/randomd_denom".to_string(),
                amount: Uint128::new(1_000_000),
            }],
        ),
        (
            "chain-2".to_string(),
            vec![AssetStaked {
                denom: "ibc/randomd_denom2".to_string(),
                amount: Uint128::new(1_000_000),
            }],
        ),
    ]));

    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let res_parsed: Vec<EmissionsDistribution> = from_json(&res).unwrap();
    assert_eq!(
        res_parsed,
        vec![
            EmissionsDistribution {
                denom: "ibc/randomd_denom".to_string(),
                distribution: SignedDecimal::from_str("2359.77843").unwrap(),
            },
            EmissionsDistribution {
                denom: "ibc/randomd_denom2".to_string(),
                distribution: SignedDecimal::from_str("12807.66973481792").unwrap(),
            }
        ]
    )
}

#[test]
fn test_emissions_distribution_4() {
    let mut deps = test_utils::setup_contract();
    let env = mock_env();
    CHAINS_INFO
        .save(
            deps.as_mut().storage,
            &vec![
                ChainInfo {
                    chain_id: "chain-1".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom".to_string(),
                        token_price: Decimal::from_str("0.006").unwrap(),
                        annual_provisions: Decimal::from_str("40000000").unwrap(),
                    },
                    luna_alliances: vec![LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.01").unwrap(),
                        annual_take_rate: Decimal::from_str("0.003").unwrap(),
                        total_lsd_staked: Decimal::from_str("21979").unwrap(),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
                ChainInfo {
                    chain_id: "chain-2".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom2".to_string(),
                        token_price: Decimal::from_str("0.02337").unwrap(),
                        annual_provisions: Decimal::from_str("24304822.32").unwrap(),
                    },
                    luna_alliances: vec![
                            LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889555",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.0238").unwrap(),
                        annual_take_rate: Decimal::from_str("0.01").unwrap(),
                        total_lsd_staked: Decimal::from_str("58263.7925").unwrap(),
                        rebase_factor: Decimal::from_str("2").unwrap(),
                    },
                         LunaAlliance {
                             ibc_denom: String::from(
                                 "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889554",
                             ),
                             normalized_reward_weight: Decimal::from_str("0.0238").unwrap(),
                             annual_take_rate: Decimal::from_str("0.01").unwrap(),
                             total_lsd_staked: Decimal::from_str("116527.585").unwrap(),
                             rebase_factor: Decimal::from_str("1").unwrap(),
                         }
                    ],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom2"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
            ],
        )
        .unwrap();

    LUNA_INFO
        .save(
            deps.as_mut().storage,
            &LunaInfo {
                luna_price: Decimal::from_str("0.61").unwrap(),
                update_timestamp: env.block.time,
            },
        )
        .unwrap();

    let msg = QueryMsg::QueryEmissionsDistributions(HashMap::from([
        (
            "chain-1".to_string(),
            vec![AssetStaked {
                denom: "ibc/randomd_denom".to_string(),
                amount: Uint128::new(1_000_000),
            }],
        ),
        (
            "chain-2".to_string(),
            vec![
                AssetStaked {
                    denom: "ibc/randomd_denom2".to_string(),
                    amount: Uint128::new(2_000_000),
                },
                AssetStaked {
                    denom: "ibc/randomd_denom3".to_string(),
                    amount: Uint128::new(1_000_000),
                },
            ],
        ),
    ]));

    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let res_parsed: Vec<EmissionsDistribution> = from_json(&res).unwrap();
    assert_eq!(
        res_parsed,
        vec![
            EmissionsDistribution {
                denom: "ibc/randomd_denom".to_string(),
                distribution: SignedDecimal::from_str("2359.77843").unwrap(),
            },
            EmissionsDistribution {
                denom: "ibc/randomd_denom2".to_string(),
                distribution: SignedDecimal::from_str("17076.892979757226666666").unwrap(),
            },
            EmissionsDistribution {
                denom: "ibc/randomd_denom3".to_string(),
                distribution: SignedDecimal::from_str("8538.446489878613333333").unwrap(),
            }
        ]
    )
}

#[test]
fn test_emissions_distribution_5() {
    let mut deps = test_utils::setup_contract();
    let env = mock_env();
    CHAINS_INFO
        .save(
            deps.as_mut().storage,
            &vec![
                ChainInfo {
                    chain_id: "chain-1".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom".to_string(),
                        token_price: Decimal::from_str("0.006").unwrap(),
                        annual_provisions: Decimal::from_str("40000000").unwrap(),
                    },
                    luna_alliances: vec![LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.01").unwrap(),
                        annual_take_rate: Decimal::from_str("0.003").unwrap(),
                        total_lsd_staked: Decimal::from_str("21979").unwrap(),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
                ChainInfo {
                    chain_id: "chain-2".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "udenom2".to_string(),
                        token_price: Decimal::from_str("0.02337").unwrap(),
                        annual_provisions: Decimal::from_str("24304822.32").unwrap(),
                    },
                    luna_alliances: vec![
                        LunaAlliance {
                            ibc_denom: String::from(
                                "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889555",
                            ),
                            normalized_reward_weight: Decimal::from_str("0.0238").unwrap(),
                            annual_take_rate: Decimal::from_str("0.01").unwrap(),
                            total_lsd_staked: Decimal::from_str("116527.585").unwrap(),
                            rebase_factor: Decimal::from_str("1").unwrap(),
                        },
                        LunaAlliance {
                            ibc_denom: String::from(
                                "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889554",
                            ),
                            normalized_reward_weight: Decimal::from_str("0.0238").unwrap(),
                            annual_take_rate: Decimal::from_str("0.01").unwrap(),
                            total_lsd_staked: Decimal::from_str("116527.585").unwrap(),
                            rebase_factor: Decimal::from_str("0.5").unwrap(),
                        }
                    ],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/randomd_denom2"),
                        rebase_factor: Decimal::from_str("1").unwrap(),
                    }],
                },
            ],
        )
        .unwrap();

    LUNA_INFO
        .save(
            deps.as_mut().storage,
            &LunaInfo {
                luna_price: Decimal::from_str("0.61").unwrap(),
                update_timestamp: env.block.time,
            },
        )
        .unwrap();

    let msg = QueryMsg::QueryEmissionsDistributions(HashMap::from([
        (
            "chain-1".to_string(),
            vec![AssetStaked {
                denom: "ibc/randomd_denom".to_string(),
                amount: Uint128::new(1_000_000),
            }],
        ),
        (
            "chain-2".to_string(),
            vec![AssetStaked {
                denom: "ibc/randomd_denom2".to_string(),
                amount: Uint128::new(1_000_000),
            }],
        ),
    ]));

    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let res_parsed: Vec<EmissionsDistribution> = from_json(&res).unwrap();
    assert_eq!(
        res_parsed,
        vec![
            EmissionsDistribution {
                denom: "ibc/randomd_denom".to_string(),
                distribution: SignedDecimal::from_str("2359.77843").unwrap(),
            },
            EmissionsDistribution {
                denom: "ibc/randomd_denom2".to_string(),
                distribution: SignedDecimal::from_str("25970.74860388584").unwrap(),
            }
        ]
    )
}

#[test]
fn test_emissions_distribution_6() {
    let mut deps = test_utils::setup_contract();
    let env = mock_env();
    CHAINS_INFO
        .save(
            deps.as_mut().storage,
            &vec![
                ChainInfo {
                    chain_id: "migaloo-1".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "uwhale".to_string(),
                        token_price: Decimal::from_str("0.013524549476922628").unwrap(),
                        annual_provisions: Decimal::from_str("23736452.03675").unwrap(),
                    },
                    luna_alliances: vec![
                        LunaAlliance {
                        ibc_denom: String::from(
                            "ibc/05238E98A143496C8AF2B6067BABC84503909ECE9E45FBCBAC2CBA5C889FD82A",
                        ),
                        normalized_reward_weight: Decimal::from_str("0.02380952380952381").unwrap(),
                        annual_take_rate: Decimal::from_str("0.009999998624824108").unwrap(),
                        total_lsd_staked: Decimal::from_str("88823.181957").unwrap(),
                        rebase_factor: Decimal::from_str("1.218914581562127104").unwrap(),
                    },
                        LunaAlliance {
                            ibc_denom: String::from(
                                "ibc/40C29143BF4153B365089E40E437B7AA819672646C45BB0A5F1E10915A0B6708",
                            ),
                            normalized_reward_weight: Decimal::from_str("0.02380952380952381").unwrap(),
                            annual_take_rate: Decimal::from_str("0.009999998624824108").unwrap(),
                            total_lsd_staked: Decimal::from_str("87428.265672").unwrap(),
                            rebase_factor: Decimal::from_str("1.090788276739635233").unwrap(),
                        },
                    ],
                    chain_alliances_on_phoenix: vec![
                        BaseAlliance {
                        ibc_denom: String::from("ibc/B3F639855EE7478750CC8F82072307ED6E131A8EFF20345E1D136B50C4E5EC36"),
                        rebase_factor: Decimal::from_str("1.047512402009788029").unwrap(),
                    },
                        BaseAlliance {
                            ibc_denom: String::from("ibc/517E13F14A1245D4DE8CF467ADD4DA0058974CDCC880FA6AE536DBCA1D16D84E"),
                            rebase_factor: Decimal::from_str("1.037943014367716314").unwrap(),
                        },
                    ],
                },
                ChainInfo {
                    chain_id: "carbon-1".to_string(),
                    update_timestamp: env.block.time,
                    native_token: NativeToken {
                        denom: "swth".to_string(),
                        token_price: Decimal::from_str("0.00412912495495493").unwrap(),
                        annual_provisions: Decimal::from_str("135254591.043743106597827288").unwrap(),
                    },
                    luna_alliances: vec![
                        LunaAlliance {
                            ibc_denom: String::from(
                                "ibc/62A3870B9804FC3A92EAAA1F0F3F07E089DBF76CC521466CA33F5AAA8AD42290",
                            ),
                            normalized_reward_weight: Decimal::from_str("0.009803921568627451").unwrap(),
                            annual_take_rate: Decimal::from_str("0.003000000004214211").unwrap(),
                            total_lsd_staked: Decimal::from_str("30172.229513").unwrap(),
                            rebase_factor: Decimal::from_str("1.218914581562127104").unwrap(),
                        },
                        LunaAlliance {
                            ibc_denom: String::from(
                                "ibc/FBEE20115530F474F8BBE1460DA85437C3FBBFAF4A5DEBD71CA6B9C40559A161",
                            ),
                            normalized_reward_weight: Decimal::from_str("0.009803921568627451").unwrap(),
                            annual_take_rate: Decimal::from_str("0.003000000004214211").unwrap(),
                            total_lsd_staked: Decimal::from_str("28119.651782").unwrap(),
                            rebase_factor: Decimal::from_str("1.091394368297982073").unwrap(),
                        }
                    ],
                    chain_alliances_on_phoenix: vec![BaseAlliance {
                        ibc_denom: String::from("ibc/0E90026619DD296AD4EF9546396F292B465BAB6B5BE00ABD6162AA1CE8E68098"),
                        rebase_factor: Decimal::from_str("1.011507").unwrap(),
                    }],
                },
            ],
        )
        .unwrap();

    LUNA_INFO
        .save(
            deps.as_mut().storage,
            &LunaInfo {
                luna_price: Decimal::from_str("0.4208152366587659").unwrap(),
                update_timestamp: env.block.time,
            },
        )
        .unwrap();

    let msg = QueryMsg::QueryEmissionsDistributions(HashMap::from([
        (
            "migaloo-1".to_string(),
            vec![
                AssetStaked {
                    denom: "ibc/B3F639855EE7478750CC8F82072307ED6E131A8EFF20345E1D136B50C4E5EC36"
                        .to_string(),
                    amount: Uint128::new(9486165779396),
                },
                AssetStaked {
                    denom: "ibc/517E13F14A1245D4DE8CF467ADD4DA0058974CDCC880FA6AE536DBCA1D16D84E"
                        .to_string(),
                    amount: Uint128::new(2371896858539),
                },
            ],
        ),
        (
            "carbon-1".to_string(),
            vec![AssetStaked {
                denom: "ibc/0E90026619DD296AD4EF9546396F292B465BAB6B5BE00ABD6162AA1CE8E68098"
                    .to_string(),
                amount: Uint128::new(1082956075803),
            }],
        ),
    ]));

    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let res_parsed: Vec<EmissionsDistribution> = from_json(&res).unwrap();
    assert_eq!(
        res_parsed,
        vec![
            EmissionsDistribution {
                denom: "ibc/B3F639855EE7478750CC8F82072307ED6E131A8EFF20345E1D136B50C4E5EC36"
                    .to_string(),
                distribution: SignedDecimal::from_str("11564.766237041663021737").unwrap(),
            },
            EmissionsDistribution {
                denom: "ibc/517E13F14A1245D4DE8CF467ADD4DA0058974CDCC880FA6AE536DBCA1D16D84E"
                    .to_string(),
                distribution: SignedDecimal::from_str("2865.208859003817351703").unwrap(),
            },
            EmissionsDistribution {
                denom: "ibc/0E90026619DD296AD4EF9546396F292B465BAB6B5BE00ABD6162AA1CE8E68098"
                    .to_string(),
                distribution: SignedDecimal::from_str("10865.475734855229839771").unwrap(),
            }
        ]
    )
}
