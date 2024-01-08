use crate::contract::instantiate;
use crate::models::InstantiateMsg;
use alliance_protocol::token_factory::CustomExecuteMsg;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{DepsMut, Response};

pub fn setup_contract(deps: DepsMut) -> Response<CustomExecuteMsg> {
    let info = mock_info("admin", &[]);
    let env = mock_env();

    let init_msg = InstantiateMsg {
        governance: "gov".to_string(),
        controller: "controller".to_string(),
        reward_denom: "uluna".to_string(),
    };
    instantiate(deps, env, info, init_msg).unwrap()
}
