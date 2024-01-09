use alliance_protocol::alliance_oracle_types::Config;
use alliance_protocol::error::ContractError;
use cosmwasm_std::Addr;

pub fn authorize_execution(config: Config, addr: Addr) -> Result<(), ContractError> {
    if addr != config.controller_addr {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}
