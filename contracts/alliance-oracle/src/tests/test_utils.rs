use alliance_protocol::alliance_oracle_types::{Config, InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    from_json,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Empty, OwnedDeps,
};

use crate::contract::{instantiate, query};

pub fn setup_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        data_expiry_seconds: 60,
        controller_addr: "controller_addr".to_string(),
    };
    let info = mock_info("creator", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let cfg = query(deps.as_ref(), mock_env(), QueryMsg::QueryConfig {}).unwrap();

    let cfg: Config = from_json(&cfg).unwrap();
    assert_eq!("controller_addr", cfg.controller_addr);
    assert_eq!(60, cfg.data_expiry_seconds);

    deps
}
