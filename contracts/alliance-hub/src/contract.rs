use alliance_protocol::alliance_protocol::{
    AllianceDelegateMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::CosmosMsg::Custom;
use cosmwasm_std::{
    coin, to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsg, Timestamp, Uint128,
};
use cw2::set_contract_version;
use cw_asset::{Asset, AssetInfo, AssetInfoKey};
use cw_utils::parse_instantiate_response_data;

use terra_proto_rs::alliance::alliance::{MsgDelegate, MsgUndelegate};
use terra_proto_rs::cosmos::base::v1beta1::Coin;
use terra_proto_rs::traits::Message;

use crate::error::ContractError;
use crate::state::{Config, BALANCES, CONFIG, WHITELIST};
use crate::token_factory::{CustomExecuteMsg, DenomUnit, Metadata, TokenExecuteMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-alliance-protocol";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CREATE_REPLY_ID: u64 = 1;
const CLAIM_REWARD_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let governance_address = deps.api.addr_validate(&msg.governance_address)?;
    let controller_address = deps.api.addr_validate(&msg.controller_address)?;
    let denom = "ualliance";
    let symbol = "ALLIANCE";
    let create_msg = TokenExecuteMsg::CreateDenom {
        subdenom: denom.to_string(),
        metadata: Metadata {
            description: "Staking token for the alliance protocol".to_string(),
            denom_units: vec![DenomUnit {
                denom: "ualliance".to_string(),
                exponent: 0,
                aliases: vec![],
            }],
            base: denom.to_string(),
            display: symbol.to_string(),
            name: "Alliance Token".to_string(),
            symbol: symbol.to_string(),
        },
    };
    let sub_msg = SubMsg::reply_on_success(
        CosmosMsg::Custom(CustomExecuteMsg::Token(create_msg)),
        CREATE_REPLY_ID,
    );
    let config = Config {
        governance_address,
        controller_address,
        alliance_token_denom: "".to_string(),
        alliance_token_supply: Uint128::zero(),
        last_reward_update_timestamp: Timestamp::default(),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attributes(vec![("action", "instantiate")])
        .add_submessage(sub_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::WhitelistAssets(assets) => whitelist_assets(deps, env, info, assets),
        ExecuteMsg::RemoveAssets(assets) => remove_assets(deps, env, info, assets),

        ExecuteMsg::Stake => stake(deps, env, info),
        ExecuteMsg::Unstake(asset) => unstake(deps, env, info, asset),
        ExecuteMsg::ClaimRewards(_) => Ok(Response::new()),

        ExecuteMsg::UpdateRewards => Ok(Response::new()),
        ExecuteMsg::AllianceDelegate(msg) => alliance_delegate(deps, env, info, msg),
        ExecuteMsg::AllianceUndelegate(_) => Ok(Response::new()),
        ExecuteMsg::AllianceRedelegate(_) => Ok(Response::new()),
        ExecuteMsg::RebalanceEmissions => Ok(Response::new()),
    }
}

fn whitelist_assets(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<AssetInfo>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.governance_address {
        return Err(ContractError::Unauthorized {});
    }
    for asset in &assets {
        let asset_key = AssetInfoKey::from(asset.clone());
        WHITELIST.save(deps.storage, asset_key, &true)?;
    }
    let assets_str = assets
        .iter()
        .map(|asset| asset.to_string())
        .collect::<Vec<String>>()
        .join(",");
    Ok(Response::new().add_attributes(vec![
        ("action", "whitelist_assets"),
        ("assets", &assets_str),
    ]))
}

fn remove_assets(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<AssetInfo>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.governance_address {
        return Err(ContractError::Unauthorized {});
    }
    for asset in &assets {
        let asset_key = AssetInfoKey::from(asset.clone());
        WHITELIST.remove(deps.storage, asset_key);
    }
    let assets_str = assets
        .iter()
        .map(|asset| asset.to_string())
        .collect::<Vec<String>>()
        .join(",");
    Ok(Response::new().add_attributes(vec![("action", "remove_assets"), ("assets", &assets_str)]))
}

fn stake(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::OnlySingleAssetAllowed {});
    }
    if info.funds[0].amount.is_zero() {
        return Err(ContractError::AmountCannotBeZero {});
    }
    let asset = AssetInfo::native(&info.funds[0].denom);
    let asset_key = AssetInfoKey::from(&asset);
    let whitelisted = WHITELIST
        .load(deps.storage, asset_key.clone())
        .unwrap_or(false);
    if !whitelisted {
        return Err(ContractError::AssetNotWhitelisted {});
    }
    let sender = info.sender.clone();

    // TODO: Before updating the balance, we need to calculate of amount of rewards accured for this user
    BALANCES.update(
        deps.storage,
        (sender, asset_key.clone()),
        |balance| -> Result<_, ContractError> {
            match balance {
                Some(balance) => Ok(balance + info.funds[0].amount),
                None => Ok(info.funds[0].amount),
            }
        },
    )?;
    Ok(Response::new().add_attributes(vec![
        ("action", "stake"),
        ("user", &info.sender.to_string()),
        ("asset", &asset.to_string()),
        ("amount", &info.funds[0].amount.to_string()),
    ]))
}

fn unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Asset,
) -> Result<Response, ContractError> {
    let asset_key = AssetInfoKey::from(asset.info.clone());
    let sender = info.sender.clone();
    if asset.amount.is_zero() {
        return Err(ContractError::AmountCannotBeZero {});
    }

    // TODO: Calculate rewards accured and claim it

    BALANCES.update(
        deps.storage,
        (sender, asset_key.clone()),
        |balance| -> Result<_, ContractError> {
            match balance {
                Some(balance) => {
                    if balance < asset.amount {
                        return Err(ContractError::InsufficientBalance {});
                    }
                    Ok(balance - asset.amount)
                }
                None => Err(ContractError::InsufficientBalance {}),
            }
        },
    )?;
    let msg = asset.transfer_msg(&info.sender)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "unstake"),
            ("user", &info.sender.to_string()),
            ("asset", &asset.info.to_string()),
            ("amount", &asset.amount.to_string()),
        ])
        .add_message(msg))
}

fn alliance_delegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AllianceDelegateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.controller_address {
        return Err(ContractError::Unauthorized {});
    }
    if msg.delegations.is_empty() {
        return Err(ContractError::EmptyDelegation {});
    }
    let mut msgs: Vec<CosmosMsg<Empty>> = vec![];
    for delegation in msg.delegations {
        let delegate_msg = MsgDelegate {
            amount: Some(Coin {
                denom: config.alliance_token_denom.clone(),
                amount: delegation.amount.to_string(),
            }),
            delegator_address: env.contract.address.to_string(),
            validator_address: delegation.validator.to_string(),
        };
        msgs.push(CosmosMsg::Stargate {
            type_url: "/alliance.alliance.MsgDelegate".to_string(),
            value: Binary::from(delegate_msg.encode_to_vec()),
        });
    }
    Ok(Response::new()
        .add_attributes(vec![("action", "alliance_delegate")])
        .add_messages(msgs))
}

fn alliance_undelegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AllianceDelegateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.controller_address {
        return Err(ContractError::Unauthorized {});
    }
    let mut msgs = vec![];
    for delegation in msg.delegations {
        let undelegate_msg = MsgUndelegate {
            amount: Some(Coin {
                denom: config.alliance_token_denom.clone(),
                amount: delegation.amount.to_string(),
            }),
            delegator_address: env.contract.address.to_string(),
            validator_address: delegation.validator.to_string(),
        };
        let msg = CosmosMsg::Stargate {
            type_url: "/alliance.alliance.MsgUndelegate".to_string(),
            value: Binary::from(undelegate_msg.encode_to_vec()),
        };
        msgs.push(msg);
    }
    Ok(Response::new()
        .add_attributes(vec![("action", "alliance_undelegate")])
        .add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => {}
        QueryMsg::WhitelistedAssets => {}
        QueryMsg::RewardDistribution => {}
        QueryMsg::StakedBalance(_) => {}
        QueryMsg::PendingRewards(_) => {}
    }
    Ok(Binary::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
    match reply.id {
        CREATE_REPLY_ID => {
            let response = reply.result.unwrap();
            // It works because the response data is a protobuf encoded string that contains the denom in the first slot (similar to the contract instantiation response)
            let denom = parse_instantiate_response_data(response.data.unwrap().as_slice())
                .map_err(|_| ContractError::Std(StdError::generic_err("parse error".to_string())))?
                .contract_address;
            let total_supply = Uint128::from(1000_000_000_000u128);
            let sub_msg = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
                TokenExecuteMsg::MintTokens {
                    denom: denom.clone(),
                    amount: total_supply.clone(),
                    mint_to_address: env.contract.address.to_string(),
                },
            )));
            CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
                config.alliance_token_denom = denom.clone();
                config.alliance_token_supply = total_supply.clone();
                Ok(config)
            })?;
            Ok(Response::new()
                .add_attributes(vec![
                    ("alliance_token_denom", denom.clone()),
                    ("alliance_token_total_supply", total_supply.to_string()),
                ])
                .add_submessage(sub_msg))
        }
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {}
}
