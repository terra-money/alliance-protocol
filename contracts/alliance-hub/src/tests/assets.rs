use crate::contract::execute;
use crate::error::ContractError;
use crate::query::query;
use crate::state::WHITELIST;
use crate::tests::helpers::{remove_assets, setup_contract, whitelist_assets};
use alliance_protocol::alliance_protocol::{ExecuteMsg, QueryMsg, WhitelistedAssetsResponse};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_json, Response};
use cw_asset::{AssetInfo, AssetInfoKey};
use std::collections::HashMap;

#[test]
fn test_whitelist_assets() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    let res = whitelist_assets(
        deps.as_mut(),
        HashMap::from([(
            "chain-1".to_string(),
            vec![AssetInfo::Native("asset1".to_string())],
        )]),
    );
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "whitelist_assets"),
            ("chain_id", "chain-1"),
            ("assets", "native:asset1")
        ])
    );

    let res = whitelist_assets(
        deps.as_mut(),
        HashMap::from([(
            "chain-1".to_string(),
            vec![
                AssetInfo::Native("asset2".to_string()),
                AssetInfo::Native("asset3".to_string()),
            ],
        )]),
    );
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "whitelist_assets"),
            ("chain_id", "chain-1"),
            ("assets", "native:asset2,native:asset3")
        ])
    );

    let chain_id = WHITELIST
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("asset2".to_string())),
        )
        .unwrap();
    assert_eq!(chain_id, "chain-1".to_string());

    let res: WhitelistedAssetsResponse =
        from_json(&query(deps.as_ref(), mock_env(), QueryMsg::WhitelistedAssets {}).unwrap())
            .unwrap();
    assert_eq!(
        res,
        HashMap::from([(
            "chain-1".to_string(),
            vec![
                AssetInfo::Native("asset1".to_string()),
                AssetInfo::Native("asset2".to_string()),
                AssetInfo::Native("asset3".to_string())
            ]
        )])
    )
}

#[test]
fn test_whitelist_asset_unauthorized() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        ExecuteMsg::WhitelistAssets(HashMap::from([(
            "chain-1".to_string(),
            vec![AssetInfo::Native("asset1".to_string())],
        )])),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_remove_assets() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    whitelist_assets(
        deps.as_mut(),
        HashMap::from([(
            "chain-1".to_string(),
            vec![
                AssetInfo::Native("asset1".to_string()),
                AssetInfo::Native("asset2".to_string()),
            ],
        )]),
    );

    let response = remove_assets(deps.as_mut(), vec![AssetInfo::Native("asset1".to_string())]);
    assert_eq!(
        response,
        Response::default().add_attributes(vec![
            ("action", "remove_assets"),
            ("assets", "native:asset1")
        ])
    );

    WHITELIST
        .load(
            deps.as_ref().storage,
            AssetInfoKey::from(AssetInfo::Native("asset1".to_string())),
        )
        .unwrap_err();
}

#[test]
fn test_remove_assets_unauthorized() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        ExecuteMsg::RemoveAssets(vec![AssetInfo::Native("".to_string())]),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}
