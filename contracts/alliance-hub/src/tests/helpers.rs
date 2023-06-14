use crate::contract::{execute, instantiate};
use alliance_protocol::alliance_protocol::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{DepsMut, Response};
use cw_asset::AssetInfo;

pub fn setup_contract(deps: DepsMut) {
    let info = mock_info("admin", &vec![]);
    let env = mock_env();

    let init_msg = InstantiateMsg {
        governance_address: "gov".to_string(),
        controller_address: "controller".to_string(),
    };
    instantiate(deps, env, info, init_msg).unwrap();
}

pub fn whitelist_assets(deps: DepsMut, assets: Vec<AssetInfo>) -> Response {
    let info = mock_info("gov", &vec![]);
    let env = mock_env();

    let msg = ExecuteMsg::WhitelistAssets(assets);
    execute(deps, env, info, msg).unwrap()
}

pub fn remove_assets(deps: DepsMut, assets: Vec<AssetInfo>) -> Response {
    let info = mock_info("gov", &vec![]);
    let env = mock_env();

    let msg = ExecuteMsg::RemoveAssets(assets);
    execute(deps, env, info, msg).unwrap()
}
