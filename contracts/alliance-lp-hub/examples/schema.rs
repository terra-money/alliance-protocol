use alliance_lp_hub::models::{InstantiateMsg, ExecuteMsg, QueryMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
