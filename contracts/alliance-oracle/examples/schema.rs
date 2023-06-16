use alliance_protocol::alliance_oracle_types::{InstantiateMsg, ExecuteMsg, QueryMsg};
use cosmwasm_schema::write_api;


fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
