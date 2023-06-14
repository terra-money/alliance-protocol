use crate::contract::{execute, instantiate};
use alliance_protocol::alliance_protocol::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coin, DepsMut, Response};
use cw_asset::{Asset, AssetInfo};

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

pub fn stake(deps: DepsMut, user: &str, amount: u128, denom: &str) -> Response {
    let info = mock_info(user, &vec![coin(amount, denom)]);
    let env = mock_env();
    let msg = ExecuteMsg::Stake;
    execute(deps, env, info, msg).unwrap()
}

pub fn unstake(deps: DepsMut, user: &str, amount: u128, denom: &str) -> Response {
    let info = mock_info(user, &vec![]);
    let env = mock_env();
    let msg = ExecuteMsg::Unstake(Asset::native(denom, amount));
    execute(deps, env, info, msg).unwrap()
}
