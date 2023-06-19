use alliance_protocol::alliance_protocol::QueryMsg;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, Env, StdResult};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => {}
        QueryMsg::WhitelistedAssets => {}
        QueryMsg::RewardDistribution => {}
        QueryMsg::StakedBalance(_) => {}
        QueryMsg::PendingRewards(_) => {}
    }
    Ok(Binary::default())
}
