#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use alliance_protocol::alliance_oracle_types::QueryMsg as OracleQueryMsg;
use alliance_protocol::alliance_protocol::{
    AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, AssetDistribution, Config,
    ExecuteMsg, InstantiateMsg, MigrateMsg,
};
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin as CwCoin, CosmosMsg, Decimal, DepsMut, Empty, Env, MessageInfo,
    Order, Reply, Response, StdError, StdResult, Storage, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetInfoKey, AssetInfoUnchecked};
use cw_utils::parse_instantiate_response_data;
use std::collections::{HashMap, HashSet};

use alliance_protocol::alliance_oracle_types::{AssetStaked, ChainId, EmissionsDistribution};
use terra_proto_rs::alliance::alliance::{
    MsgClaimDelegationRewards, MsgDelegate, MsgRedelegate, MsgUndelegate,
};
use terra_proto_rs::cosmos::base::v1beta1::Coin;
use terra_proto_rs::traits::Message;

use crate::error::ContractError;
use crate::state::{
    ASSET_REWARD_DISTRIBUTION, ASSET_REWARD_RATE, BALANCES, CONFIG, TEMP_BALANCE, TOTAL_BALANCES,
    UNCLAIMED_REWARDS, USER_ASSET_REWARD_RATE, VALIDATORS, WHITELIST,
};
use crate::token_factory::{CustomExecuteMsg, DenomUnit, Metadata, TokenExecuteMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-alliance-protocol";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CREATE_REPLY_ID: u64 = 1;
const CLAIM_REWARD_ERROR_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let governance_address = deps.api.addr_validate(msg.governance.as_str())?;
    let controller_address = deps.api.addr_validate(msg.controller.as_str())?;
    let oracle_address = deps.api.addr_validate(msg.oracle.as_str())?;
    let create_msg = TokenExecuteMsg::CreateDenom {
        subdenom: "ualliance".to_string(),
    };
    let sub_msg = SubMsg::reply_on_success(
        CosmosMsg::Custom(CustomExecuteMsg::Token(create_msg)),
        CREATE_REPLY_ID,
    );
    let config = Config {
        governance: governance_address,
        controller: controller_address,
        oracle: oracle_address,
        alliance_token_denom: "".to_string(),
        alliance_token_supply: Uint128::zero(),
        last_reward_update_timestamp: Timestamp::default(),
        reward_denom: msg.reward_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    VALIDATORS.save(deps.storage, &HashSet::new())?;
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
        ExecuteMsg::WhitelistAssets(assets) => whitelist_assets(deps, info, assets),
        ExecuteMsg::RemoveAssets(assets) => remove_assets(deps, info, assets),

        ExecuteMsg::Stake {} => stake(deps, env, info),
        ExecuteMsg::Unstake(asset) => unstake(deps, info, asset),
        ExecuteMsg::ClaimRewards(asset) => claim_rewards(deps, info, asset),

        ExecuteMsg::AllianceDelegate(msg) => alliance_delegate(deps, env, info, msg),
        ExecuteMsg::AllianceUndelegate(msg) => alliance_undelegate(deps, env, info, msg),
        ExecuteMsg::AllianceRedelegate(msg) => alliance_redelegate(deps, env, info, msg),
        ExecuteMsg::UpdateRewards {} => update_rewards(deps, env, info),
        ExecuteMsg::RebalanceEmissions {} => rebalance_emissions(deps, env, info),

        ExecuteMsg::UpdateRewardsCallback {} => update_reward_callback(deps, env, info),
        ExecuteMsg::RebalanceEmissionsCallback {} => rebalance_emissions_callback(deps, env, info),
    }
}

fn whitelist_assets(
    deps: DepsMut,
    info: MessageInfo,
    assets_request: HashMap<ChainId, Vec<AssetInfo>>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_governance(&info, &config)?;
    let mut attrs = vec![("action".to_string(), "whitelist_assets".to_string())];
    for (chain_id, assets) in &assets_request {
        for asset in assets {
            let asset_key = AssetInfoKey::from(asset.clone());
            WHITELIST.save(deps.storage, asset_key.clone(), chain_id)?;
            ASSET_REWARD_RATE.update(deps.storage, asset_key, |rate| -> StdResult<_> {
                Ok(rate.unwrap_or(Decimal::zero()))
            })?;
        }
        attrs.push(("chain_id".to_string(), chain_id.to_string()));
        let assets_str = assets
            .iter()
            .map(|asset| asset.to_string())
            .collect::<Vec<String>>()
            .join(",");

        attrs.push(("assets".to_string(), assets_str.to_string()));
    }
    Ok(Response::new().add_attributes(attrs))
}

fn remove_assets(
    deps: DepsMut,
    info: MessageInfo,
    assets: Vec<AssetInfo>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // Only allow the governance address to update whitelisted assets
    is_governance(&info, &config)?;
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

fn stake(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::OnlySingleAssetAllowed {});
    }
    if info.funds[0].amount.is_zero() {
        return Err(ContractError::AmountCannotBeZero {});
    }
    let asset = AssetInfo::native(&info.funds[0].denom);
    let asset_key = AssetInfoKey::from(&asset);
    WHITELIST
        .load(deps.storage, asset_key.clone())
        .map_err(|_| ContractError::AssetNotWhitelisted {})?;
    let sender = info.sender.clone();

    let rewards = _claim_reward(deps.storage, sender.clone(), asset.clone())?;
    if !rewards.is_zero() {
        UNCLAIMED_REWARDS.update(
            deps.storage,
            (sender.clone(), asset_key.clone()),
            |balance| -> Result<_, ContractError> {
                Ok(balance.unwrap_or(Uint128::zero()) + rewards)
            },
        )?;
    }

    BALANCES.update(
        deps.storage,
        (sender.clone(), asset_key.clone()),
        |balance| -> Result<_, ContractError> {
            match balance {
                Some(balance) => Ok(balance + info.funds[0].amount),
                None => Ok(info.funds[0].amount),
            }
        },
    )?;
    TOTAL_BALANCES.update(
        deps.storage,
        asset_key.clone(),
        |balance| -> Result<_, ContractError> {
            Ok(balance.unwrap_or(Uint128::zero()) + info.funds[0].amount)
        },
    )?;

    let asset_reward_rate = ASSET_REWARD_RATE
        .load(deps.storage, asset_key.clone())
        .unwrap_or(Decimal::zero());
    USER_ASSET_REWARD_RATE.save(deps.storage, (sender, asset_key), &asset_reward_rate)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "stake"),
        ("user", info.sender.as_ref()),
        ("asset", &asset.to_string()),
        ("amount", &info.funds[0].amount.to_string()),
    ]))
}

fn unstake(deps: DepsMut, info: MessageInfo, asset: Asset) -> Result<Response, ContractError> {
    let asset_key = AssetInfoKey::from(asset.info.clone());
    let sender = info.sender.clone();
    if asset.amount.is_zero() {
        return Err(ContractError::AmountCannotBeZero {});
    }

    let rewards = _claim_reward(deps.storage, sender.clone(), asset.info.clone())?;
    if !rewards.is_zero() {
        UNCLAIMED_REWARDS.update(
            deps.storage,
            (sender.clone(), asset_key.clone()),
            |balance| -> Result<_, ContractError> {
                Ok(balance.unwrap_or(Uint128::zero()) + rewards)
            },
        )?;
    }

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
    TOTAL_BALANCES.update(
        deps.storage,
        asset_key,
        |balance| -> Result<_, ContractError> {
            let balance = balance.unwrap_or(Uint128::zero());
            if balance < asset.amount {
                return Err(ContractError::InsufficientBalance {});
            }
            Ok(balance - asset.amount)
        },
    )?;

    let msg = asset.transfer_msg(&info.sender)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "unstake"),
            ("user", info.sender.as_ref()),
            ("asset", &asset.info.to_string()),
            ("amount", &asset.amount.to_string()),
        ])
        .add_message(msg))
}

fn claim_rewards(
    deps: DepsMut,
    info: MessageInfo,
    asset: AssetInfo,
) -> Result<Response, ContractError> {
    let user = info.sender;
    let config = CONFIG.load(deps.storage)?;
    let rewards = _claim_reward(deps.storage, user.clone(), asset.clone())?;
    let unclaimed_rewards = UNCLAIMED_REWARDS
        .load(
            deps.storage,
            (user.clone(), AssetInfoKey::from(asset.clone())),
        )
        .unwrap_or(Uint128::zero());
    let final_rewards = rewards + unclaimed_rewards;
    UNCLAIMED_REWARDS.remove(
        deps.storage,
        (user.clone(), AssetInfoKey::from(asset.clone())),
    );
    let response = Response::new().add_attributes(vec![
        ("action", "claim_rewards"),
        ("user", user.as_ref()),
        ("asset", &asset.to_string()),
        ("reward_amount", &final_rewards.to_string()),
    ]);
    if !final_rewards.is_zero() {
        let rewards_asset = Asset {
            info: AssetInfo::Native(config.reward_denom),
            amount: final_rewards,
        };
        Ok(response.add_message(rewards_asset.transfer_msg(&user)?))
    } else {
        Ok(response)
    }
}

fn _claim_reward(
    storage: &mut dyn Storage,
    user: Addr,
    asset: AssetInfo,
) -> Result<Uint128, ContractError> {
    let asset_key = AssetInfoKey::from(&asset);
    let user_reward_rate = USER_ASSET_REWARD_RATE.load(storage, (user.clone(), asset_key.clone()));
    let asset_reward_rate = ASSET_REWARD_RATE.load(storage, asset_key.clone())?;

    if let Ok(user_reward_rate) = user_reward_rate {
        let user_staked = BALANCES.load(storage, (user.clone(), asset_key.clone()))?;
        let rewards = ((asset_reward_rate - user_reward_rate)
            * Decimal::from_atomics(user_staked, 0)?)
        .to_uint_floor();
        if rewards.is_zero() {
            Ok(Uint128::zero())
        } else {
            USER_ASSET_REWARD_RATE.save(storage, (user, asset_key), &asset_reward_rate)?;
            Ok(rewards)
        }
    } else {
        // If cannot find user_reward_rate, assume this is the first time they are staking and set it to the current asset_reward_rate
        USER_ASSET_REWARD_RATE.save(storage, (user, asset_key), &asset_reward_rate)?;

        Ok(Uint128::zero())
    }
}

fn alliance_delegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AllianceDelegateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_controller(&info, &config)?;
    if msg.delegations.is_empty() {
        return Err(ContractError::EmptyDelegation {});
    }
    let mut validators = VALIDATORS.load(deps.storage)?;
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
        validators.insert(delegation.validator);
    }
    VALIDATORS.save(deps.storage, &validators)?;
    Ok(Response::new()
        .add_attributes(vec![("action", "alliance_delegate")])
        .add_messages(msgs))
}

fn alliance_undelegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AllianceUndelegateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_controller(&info, &config)?;
    if msg.undelegations.is_empty() {
        return Err(ContractError::EmptyDelegation {});
    }
    let mut msgs = vec![];
    for delegation in msg.undelegations {
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

fn alliance_redelegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AllianceRedelegateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_controller(&info, &config)?;
    if msg.redelegations.is_empty() {
        return Err(ContractError::EmptyDelegation {});
    }
    let mut msgs = vec![];
    let mut validators = VALIDATORS.load(deps.storage)?;
    for redelegation in msg.redelegations {
        let src_validator = redelegation.src_validator;
        let dst_validator = redelegation.dst_validator;
        let redelegate_msg = MsgRedelegate {
            amount: Some(Coin {
                denom: config.alliance_token_denom.clone(),
                amount: redelegation.amount.to_string(),
            }),
            delegator_address: env.contract.address.to_string(),
            validator_src_address: src_validator.to_string(),
            validator_dst_address: dst_validator.to_string(),
        };
        let msg = CosmosMsg::Stargate {
            type_url: "/alliance.alliance.MsgRedelegate".to_string(),
            value: Binary::from(redelegate_msg.encode_to_vec()),
        };
        msgs.push(msg);
        validators.insert(dst_validator);
    }
    VALIDATORS.save(deps.storage, &validators)?;
    Ok(Response::new()
        .add_attributes(vec![("action", "alliance_redelegate")])
        .add_messages(msgs))
}

fn update_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let reward_sent_in_tx: Option<&CwCoin> =
        info.funds.iter().find(|c| c.denom == config.reward_denom);
    let sent_balance = if let Some(coin) = reward_sent_in_tx {
        coin.amount
    } else {
        Uint128::zero()
    };
    let reward_asset = AssetInfo::native(config.reward_denom.clone());
    let contract_balance =
        reward_asset.query_balance(&deps.querier, env.contract.address.clone())?;

    // Contract balance is guaranteed to be greater than sent balance
    // since contract balance = previous contract balance + sent balance > sent balance
    TEMP_BALANCE.save(deps.storage, &(contract_balance - sent_balance))?;
    let validators = VALIDATORS.load(deps.storage)?;
    let sub_msgs: Vec<SubMsg> = validators
        .iter()
        .map(|v| {
            let msg = MsgClaimDelegationRewards {
                delegator_address: env.contract.address.to_string(),
                validator_address: v.to_string(),
                denom: config.alliance_token_denom.clone(),
            };
            let msg = CosmosMsg::Stargate {
                type_url: "/alliance.alliance.MsgClaimDelegationRewards".to_string(),
                value: Binary::from(msg.encode_to_vec()),
            };
            // Reply on error here is used to ignore errors from claiming rewards with validators that we did not delegate to
            SubMsg::reply_on_error(msg, CLAIM_REWARD_ERROR_REPLY_ID)
        })
        .collect();
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::UpdateRewardsCallback {}).unwrap(),
        funds: vec![],
    });

    Ok(Response::new()
        .add_attributes(vec![("action", "update_rewards")])
        .add_submessages(sub_msgs)
        .add_message(msg))
}

fn update_reward_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    let config = CONFIG.load(deps.storage)?;
    let reward_asset = AssetInfo::native(config.reward_denom);
    let current_balance = reward_asset.query_balance(&deps.querier, env.contract.address)?;
    let previous_balance = TEMP_BALANCE.load(deps.storage)?;
    let rewards_collected = current_balance - previous_balance;

    let asset_reward_distribution = ASSET_REWARD_DISTRIBUTION.load(deps.storage)?;
    let total_distribution = asset_reward_distribution
        .iter()
        .map(|a| a.distribution)
        .fold(Decimal::zero(), |acc, v| acc + v);

    for asset_distribution in asset_reward_distribution {
        let asset_key = AssetInfoKey::from(asset_distribution.asset);
        let total_reward_distributed = Decimal::from_atomics(rewards_collected, 0)?
            * asset_distribution.distribution
            / total_distribution;

        // If there are no balances, we stop updating the rate. This means that the emissions are not directed to any stakers.
        let total_balance = TOTAL_BALANCES
            .load(deps.storage, asset_key.clone())
            .unwrap_or(Uint128::zero());
        if !total_balance.is_zero() {
            let rate_to_update =
                total_reward_distributed / Decimal::from_atomics(total_balance, 0)?;
            if rate_to_update > Decimal::zero() {
                ASSET_REWARD_RATE.update(
                    deps.storage,
                    asset_key.clone(),
                    |rate| -> StdResult<_> { Ok(rate.unwrap_or(Decimal::zero()) + rate_to_update) },
                )?;
            }
        }
    }
    TEMP_BALANCE.remove(deps.storage);

    Ok(Response::new().add_attributes(vec![("action", "update_rewards_callback")]))
}

fn rebalance_emissions(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Allow execution only from the controller account
    let config = CONFIG.load(deps.storage)?;
    is_controller(&info, &config)?;
    // Before starting with the rebalance emission process
    // rewards must be updated to the current block height
    // Skip if no reward distribution in the first place
    let res = if ASSET_REWARD_DISTRIBUTION.load(deps.storage).is_ok() {
        update_rewards(deps, env.clone(), info)?
    } else {
        Response::new()
    };

    Ok(res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RebalanceEmissionsCallback {}).unwrap(),
        funds: vec![],
    })))
}

fn rebalance_emissions_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    let config = CONFIG.load(deps.storage)?;

    // This is the request that will be send to the oracle contract
    // on the QueryEmissionsDistributions entry point to recover
    // the assets_reward_distribution...
    let mut distr_req: HashMap<ChainId, Vec<AssetStaked>> = HashMap::new();

    let whitelist: Vec<(AssetInfoUnchecked, ChainId)> = WHITELIST
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.unwrap())
        .collect();
    for (asset, chain_id) in whitelist {
        let asset = asset.check(deps.api, None)?;
        let total_balance = TOTAL_BALANCES
            .load(deps.storage, AssetInfoKey::from(asset.clone()))
            .unwrap_or(Uint128::zero());

        // Oracle does not support non-native coins so skip if non-native
        if let AssetInfoBase::Native(denom) = asset {
            distr_req
                .entry(chain_id)
                .or_insert_with(Vec::new)
                .push(AssetStaked {
                    denom,
                    amount: total_balance,
                });
        }
    }

    // Query oracle contract for the new distribution
    let distr_res: Vec<EmissionsDistribution> = deps.querier.query_wasm_smart(
        config.oracle,
        &OracleQueryMsg::QueryEmissionsDistributions(distr_req),
    )?;

    let asset_reward_distribution: StdResult<Vec<AssetDistribution>> = distr_res
        .iter()
        .map(|d| -> StdResult<AssetDistribution> {
            let distribution = d.distribution.to_decimal()?;
            Ok(AssetDistribution {
                asset: AssetInfo::Native(d.denom.to_string()),
                distribution,
            })
        })
        .collect();
    let asset_reward_distribution = asset_reward_distribution?;
    ASSET_REWARD_DISTRIBUTION.save(deps.storage, &asset_reward_distribution)?;

    let mut attrs = vec![("action".to_string(), "rebalance_emissions".to_string())];
    for distribution in asset_reward_distribution {
        attrs.push((
            distribution.asset.to_string(),
            distribution.distribution.to_string(),
        ));
    }
    Ok(Response::new().add_attributes(attrs))
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
            let total_supply = Uint128::from(1_000_000_000_000_u128);
            let sub_msg_mint = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
                TokenExecuteMsg::MintTokens {
                    denom: denom.clone(),
                    amount: total_supply,
                    mint_to_address: env.contract.address.to_string(),
                },
            )));
            CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
                config.alliance_token_denom = denom.clone();
                config.alliance_token_supply = total_supply;
                Ok(config)
            })?;
            let symbol = "ALLIANCE";

            let sub_msg_metadata = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
                TokenExecuteMsg::SetMetadata {
                    denom: denom.clone(),
                    metadata: Metadata {
                        description: "Staking token for the alliance protocol".to_string(),
                        denom_units: vec![DenomUnit {
                            denom: denom.clone(),
                            exponent: 0,
                            aliases: vec![],
                        }],
                        base: denom.to_string(),
                        display: denom.to_string(),
                        name: "Alliance Token".to_string(),
                        symbol: symbol.to_string(),
                    },
                },
            )));
            Ok(Response::new()
                .add_attributes(vec![
                    ("alliance_token_denom", denom),
                    ("alliance_token_total_supply", total_supply.to_string()),
                ])
                .add_submessage(sub_msg_mint)
                .add_submessage(sub_msg_metadata))
        }
        CLAIM_REWARD_ERROR_REPLY_ID => {
            Ok(Response::new().add_attributes(vec![("action", "claim_reward_error")]))
        }
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

// Controller is used to perform administrative operations that deals with delegating the virtual
// tokens to the expected validators
fn is_controller(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.controller {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

// Only governance (through a on-chain prop) can change the whitelisted assets
fn is_governance(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.governance {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}
