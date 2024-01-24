use crate::models::ModifyAssetPair;
use crate::tests::helpers::{modify_asset, setup_contract, stake};
use alliance_protocol::error::ContractError;
use cosmwasm_std::{Response, testing::mock_dependencies};
use cw_asset::AssetInfo;

#[test]
fn test_stake_non_whitelisted_asset() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let err = stake(deps.as_mut(), "user1", 100, "native_asset").unwrap_err();

    assert_eq!(
        err,
        ContractError::AssetNotWhitelisted("native:native_asset".to_string())
    );
}

#[test]
fn test_remove_asset() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    // Whitelist the pair aWHALE-uluna successfully
    let res = modify_asset(
        deps.as_mut(),
        Vec::from([ModifyAssetPair {
            asset_info: AssetInfo::Native("aWHALE".to_string()),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: false,
        }]),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_attributes(vec![("action", "modify_asset"), ("asset", "native:aWHALE"),])
    );

    // Try to remove the asset aWHALE, it should error because
    // the reward_asset_info is not defined
    let err = modify_asset(
        deps.as_mut(),
        Vec::from([ModifyAssetPair {
            asset_info: AssetInfo::Native("aWHALE".to_string()),
            reward_asset_info: None,
            delete: true,
        }]),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::MissingRewardAsset("native:aWHALE".to_string())
    );

    // Remove the asset pair aWHALE-uluna successfully because both
    // assets are defined
    let res = modify_asset(
        deps.as_mut(),
        Vec::from([ModifyAssetPair {
            asset_info: AssetInfo::Native("aWHALE".to_string()),
            reward_asset_info: Some(AssetInfo::Native("uluna".to_string())),
            delete: true,
        }]),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            ("action", "modify_asset"),
            ("asset", "native:aWHALE"),
            ("to_remove", "true"),
        ])
    );
}
