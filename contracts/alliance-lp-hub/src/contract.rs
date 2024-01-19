use alliance_protocol::alliance_oracle_types::EmissionsDistribution;
use alliance_protocol::{
    alliance_protocol::{
        AllianceDelegateMsg, AllianceRedelegateMsg, AllianceUndelegateMsg, MigrateMsg,
    },
    error::ContractError,
    token_factory::{CustomExecuteMsg, DenomUnit, Metadata, TokenExecuteMsg},
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin as CwCoin, CosmosMsg, Decimal, DepsMut, Empty, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_asset::{Asset, AssetInfo, AssetInfoKey, AssetInfoUnchecked};
use cw_utils::parse_instantiate_response_data;
use std::str::FromStr;
use std::{collections::HashSet, env};
use terra_proto_rs::cosmos::evidence;
use terra_proto_rs::{
    alliance::alliance::{MsgClaimDelegationRewards, MsgDelegate, MsgRedelegate, MsgUndelegate},
    cosmos::base::v1beta1::Coin,
    traits::Message,
};

use crate::models::{AssetRewardRate, AssetUnclaimedRewards};
use crate::{
    astro_models::{Cw20Msg, ExecuteAstroMsg, QueryAstroMsg, RewardInfo},
    models::{Config, ExecuteMsg, InstantiateMsg, ModifyAsset, PendingRewardsRes},
    state::{
        ASSET_REWARD_RATE, BALANCES, CONFIG, TEMP_BALANCE, TOTAL_BALANCES, UNCLAIMED_REWARDS,
        USER_ASSET_REWARD_RATE, VALIDATORS, WHITELIST,
    },
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-alliance-lp-hub";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CREATE_REPLY_ID: u64 = 1;
const CLAIM_REWARD_ERROR_REPLY_ID: u64 = 2;
const CLAIM_ASTRO_REWARD_REPLY_ID: u64 = 3;

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
    deps.api.addr_validate(msg.astro_reward_denom.as_str())?;
    let astro_incentives_address = deps
        .api
        .addr_validate(msg.astro_incentives_address.as_str())?;
    let fee_collector_address = deps.api.addr_validate(msg.fee_collector_address.as_str())?;
    let create_msg = TokenExecuteMsg::CreateDenom {
        subdenom: "ualliancelp".to_string(),
    };
    let sub_msg = SubMsg::reply_on_success(
        CosmosMsg::Custom(CustomExecuteMsg::Token(create_msg)),
        CREATE_REPLY_ID,
    );
    let config = Config {
        governance: governance_address,
        controller: controller_address,
        astro_reward_denom: msg.astro_reward_denom,
        fee_collector: fee_collector_address,
        astro_incentives: astro_incentives_address,
        alliance_token_denom: "".to_string(),
        alliance_token_supply: Uint128::zero(),
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
        ExecuteMsg::ModifyAssets(assets) => modify_assets(deps, info, assets),

        ExecuteMsg::Receive(cw20_msg) => {
            let sender = deps.api.addr_validate(&cw20_msg.sender)?;
            let received_asset = Asset::cw20(info.sender.clone(), cw20_msg.amount);

            stake(deps, env, sender, received_asset)
        }
        ExecuteMsg::Stake {} => {
            if info.funds.len() != 1 {
                return Err(ContractError::OnlySingleAssetAllowed {});
            }
            let coin = info.funds[0].clone();
            if coin.amount.is_zero() {
                return Err(ContractError::AmountCannotBeZero {});
            }
            stake(deps, env, info.sender, coin.into())
        }

        ExecuteMsg::Unstake(asset) => unstake(deps, info, asset),
        ExecuteMsg::ClaimRewards(asset) => claim_rewards(deps, info, asset),

        ExecuteMsg::AllianceDelegate(msg) => alliance_delegate(deps, env, info, msg),
        ExecuteMsg::AllianceUndelegate(msg) => alliance_undelegate(deps, env, info, msg),
        ExecuteMsg::AllianceRedelegate(msg) => alliance_redelegate(deps, env, info, msg),

        ExecuteMsg::UpdateRewards {} => update_rewards(deps, env, info),
        ExecuteMsg::RebalanceEmissions(distributions) => {
            rebalance_emissions(deps, env, info, distributions)
        }

        ExecuteMsg::UpdateRewardsCallback {} => update_reward_callback(deps, env, info),
        ExecuteMsg::RebalanceEmissionsCallback(distributions) => {
            rebalance_emissions_callback(deps, env, info, distributions)
        }
    }
}

// This method iterate through the list of assets to be modified,
// for each asset it checks if it is being listed or delisted,
fn modify_assets(
    deps: DepsMut,
    info: MessageInfo,
    assets: Vec<ModifyAsset>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_governance(&info, &config)?;
    let mut attrs = vec![("action".to_string(), "modify_assets".to_string())];

    for asset in assets {
        if asset.delete {
            let asset_key = AssetInfoKey::from(asset.asset_info.clone());
            WHITELIST.remove(deps.storage, asset_key.clone());
            attrs.extend_from_slice(&[
                ("asset".to_string(), asset.asset_info.to_string()),
                ("to_remove".to_string(), asset.delete.to_string()),
            ]);
        } else {
            let asset_key = AssetInfoKey::from(asset.asset_info.clone());
            WHITELIST.save(deps.storage, asset_key.clone(), &Decimal::zero())?;
            ASSET_REWARD_RATE.update(
                deps.storage,
                asset_key,
                |asset_reward_rate| -> StdResult<_> {
                    Ok(asset_reward_rate.unwrap_or(AssetRewardRate::zero()))
                },
            )?;
            attrs.extend_from_slice(&[("asset".to_string(), asset.asset_info.to_string())]);
        }
    }

    Ok(Response::new().add_attributes(attrs))
}

// This method is used to stake both native and CW20 tokens,
// it checks if the asset is whitelisted and then proceeds to
// update the user balance and the total balance for the asset.
fn stake(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    received_asset: Asset,
) -> Result<Response, ContractError> {
    let asset_key = AssetInfoKey::from(&received_asset.info);
    WHITELIST
        .load(deps.storage, asset_key.clone())
        .map_err(|_| ContractError::AssetNotWhitelisted(received_asset.info.to_string()))?;

    let rewards = _claim_reward(deps.storage, sender.clone(), received_asset.info.clone())?;
    if !rewards.is_zero() {
        UNCLAIMED_REWARDS.update(
            deps.storage,
            (sender.clone(), asset_key.clone()),
            |balance| -> Result<_, ContractError> {
                let mut unclaimed_rewards = balance.unwrap_or(AssetUnclaimedRewards::zero());
                unclaimed_rewards.alliance_reward_rate += rewards;
                Ok(unclaimed_rewards)
            },
        )?;
    }
    let config = CONFIG.load(deps.storage)?;

    // Query astro incentives, to do so we must first remove the prefix
    // from the asset info e.g. cw20:asset1 -> asset1 or native:uluna -> uluna
    let lp_token = received_asset.info.to_string();
    let astro_incentives: Vec<RewardInfo> = deps
        .querier
        .query_wasm_smart(
            config.astro_incentives.to_string(),
            &QueryAstroMsg::RewardInfo {
                lp_token: lp_token.split(':').collect::<Vec<&str>>()[1].to_string(),
            },
        )
        .unwrap_or_default();

    let mut res = Response::new().add_attributes(vec![
        ("action", "stake"),
        ("user", sender.as_ref()),
        ("asset", &received_asset.info.to_string()),
        ("amount", &received_asset.amount.to_string()),
    ]);

    if !astro_incentives.is_empty() {
        let msg = match received_asset.info.clone() {
            AssetInfo::Native(native_asset) => {
                // If the asset is native, we need to send it to the astro incentives contract
                // using the ExecuteAstroMsg::Deposit message
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.astro_incentives.to_string(),
                    msg: to_json_binary(&ExecuteAstroMsg::Deposit { recipient: None })?,
                    funds: vec![CwCoin {
                        denom: native_asset,
                        amount: received_asset.amount,
                    }],
                })
            }
            AssetInfo::Cw20(cw20_contract_addr) => {
                // If the asset is a cw20 token, we need to send it to the astro incentives contract
                // using the ExecuteAstroMsg::Receive message
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: cw20_contract_addr.to_string(),
                    msg: to_json_binary(&Cw20ExecuteMsg::Send {
                        contract: config.astro_incentives.to_string(),
                        amount: received_asset.amount,
                        msg: to_json_binary(&Cw20ReceiveMsg {
                            sender: env.contract.address.to_string(),
                            amount: received_asset.amount,
                            msg: to_json_binary(&Cw20Msg::Deposit { recipient: None })?,
                        })?,
                    })?,
                    funds: vec![],
                })
            }
            _ => {
                return Err(ContractError::AssetNotWhitelisted(
                    received_asset.info.to_string(),
                ));
            }
        };

        res = res.add_message(msg);
    }

    BALANCES.update(
        deps.storage,
        (sender.clone(), asset_key.clone()),
        |balance| -> Result<_, ContractError> {
            match balance {
                Some(balance) => Ok(balance + received_asset.amount),
                None => Ok(received_asset.amount),
            }
        },
    )?;
    TOTAL_BALANCES.update(
        deps.storage,
        asset_key.clone(),
        |balance| -> Result<_, ContractError> {
            Ok(balance.unwrap_or(Uint128::zero()) + received_asset.amount)
        },
    )?;

    let asset_reward_rate = ASSET_REWARD_RATE
        .load(deps.storage, asset_key.clone())
        .unwrap_or(AssetRewardRate::zero());
    USER_ASSET_REWARD_RATE.save(
        deps.storage,
        (sender.clone(), asset_key),
        &asset_reward_rate,
    )?;

    Ok(res)
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
                let mut unclaimed_rewards = balance.unwrap_or(AssetUnclaimedRewards::zero());
                unclaimed_rewards.alliance_reward_rate += rewards;
                Ok(unclaimed_rewards)
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
        .unwrap_or(AssetUnclaimedRewards::zero());
    let final_rewards = rewards + unclaimed_rewards.alliance_reward_rate;
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
        let rewards = ((asset_reward_rate.alliance_reward_rate
            - user_reward_rate.alliance_reward_rate)
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
    let mut res = Response::new().add_attributes(vec![("action", "update_rewards")]);
    let reward_sent_in_tx: Option<&CwCoin> =
        info.funds.iter().find(|c| c.denom == config.reward_denom);
    let sent_balance = if let Some(coin) = reward_sent_in_tx {
        coin.amount
    } else {
        Uint128::zero()
    };

    let astr_incentives_rewards = _update_astro_rewards(
        &deps,
        env.contract.address.clone(),
        config.astro_incentives.clone(),
    )?;
    if let Some(msg) = astr_incentives_rewards {
        res = res.add_submessage(msg)
    };

    let reward_asset = AssetInfo::native(config.reward_denom.clone());
    let contract_balance =
        reward_asset.query_balance(&deps.querier, env.contract.address.clone())?;

    // Contract balance is guaranteed to be greater than sent balance
    // since contract balance = previous contract balance + sent balance > sent balance
    TEMP_BALANCE.save(
        deps.storage,
        AssetInfoKey::from(AssetInfo::Native(config.reward_denom.to_string())),
        &(contract_balance - sent_balance),
    )?;
    let validators = VALIDATORS.load(deps.storage)?;
    let alliance_sub_msg: Vec<SubMsg> = validators
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
        msg: to_json_binary(&ExecuteMsg::UpdateRewardsCallback {}).unwrap(),
        funds: vec![],
    });

    Ok(res.add_submessages(alliance_sub_msg).add_message(msg))
}

fn _update_astro_rewards(
    deps: &DepsMut,
    contract_addr: Addr,
    astro_incentives: Addr,
) -> Result<Option<SubMsg>, ContractError> {
    let whitelist = WHITELIST
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(a, d)| (AssetInfoKey(a), d)))
        .collect::<StdResult<Vec<(AssetInfoKey, Decimal)>>>()?;
    let mut lp_tokens_list: Vec<String> = vec![];

    for (asset_info, _) in whitelist {
        let lp_token = String::from_utf8(asset_info.0)?;
        let pending_rewards: Vec<PendingRewardsRes> = deps
            .querier
            .query_wasm_smart(
                astro_incentives.to_string(),
                &QueryAstroMsg::PendingRewards {
                    lp_token: lp_token.clone(),
                    user: contract_addr.to_string(),
                },
            )
            .unwrap_or_default();

        for pr in pending_rewards {
            if !pr.astro_rewards.is_zero() {
                lp_tokens_list.push(lp_token.clone())
            }
        }
    }

    if !lp_tokens_list.is_empty() {
        let msg: CosmosMsg<_> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: astro_incentives.to_string(),
            msg: to_json_binary(&ExecuteAstroMsg::ClaimRewards {
                lp_tokens: lp_tokens_list,
            })?,
            funds: vec![],
        });

        return Ok(Some(SubMsg::reply_always(msg, CLAIM_ASTRO_REWARD_REPLY_ID)));
    }

    Ok(None)
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
    let mut res = Response::new().add_attributes(vec![("action", "update_rewards_callback")]);
    // We only deal with alliance rewards here. Other rewards (e.g. ASTRO) needs to be dealt with separately
    // This is because the reward distribution only affects alliance rewards. LP rewards are directly distributed to LP holders
    // and not pooled together and shared
    let reward_asset = AssetInfo::native(config.reward_denom.clone());
    let current_balance = reward_asset.query_balance(&deps.querier, env.contract.address)?;
    let previous_balance = TEMP_BALANCE.load(
        deps.storage,
        AssetInfoKey::from(AssetInfo::Native(config.reward_denom.to_string())),
    )?;
    let rewards_collected = current_balance - previous_balance;

    let whitelist: StdResult<Vec<(AssetInfoKey, Decimal)>> = WHITELIST
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(a, d)| (AssetInfoKey(a), d)))
        .collect();
    let whitelist = whitelist?;

    let total_distribution = whitelist
        .iter()
        .fold(Decimal::zero(), |acc, (_, v)| acc + v);

    // Move all unallocated rewards to the unallocated rewards bucket
    if let Ok(unallocated_distribution) = Decimal::one().checked_sub(total_distribution) {
        let unallocated_rewards = (Decimal::from_atomics(rewards_collected, 0)?
            * unallocated_distribution)
            .to_uint_floor();
        if !unallocated_rewards.is_zero() {
            res = res.add_message(BankMsg::Send {
                to_address: config.fee_collector.to_string(),
                amount: vec![CwCoin::new(
                    unallocated_rewards.u128(),
                    config.reward_denom.clone(),
                )],
            })
        }
    } else {
        return Err(ContractError::InvalidTotalDistribution(total_distribution));
    }

    // Calculate the rewards for each asset
    for (asset_key, distribution) in whitelist {
        let total_reward_distributed: Decimal =
            Decimal::from_atomics(rewards_collected, 0)? * distribution;

        // If there are no balances, we stop updating the rate. This means that the emissions are not directed to any stakers.
        let total_balance = TOTAL_BALANCES
            .load(deps.storage, asset_key.clone())
            .unwrap_or(Uint128::zero());
        if total_balance.is_zero() {
            continue;
        }

        // Update reward rates for each asset
        let rate_to_update = total_reward_distributed / Decimal::from_atomics(total_balance, 0)?;
        if rate_to_update > Decimal::zero() {
            ASSET_REWARD_RATE.update(deps.storage, asset_key.clone(), |rate| -> StdResult<_> {
                let mut reward_rate = rate.unwrap_or(AssetRewardRate::zero());
                reward_rate.alliance_reward_rate =
                    reward_rate.alliance_reward_rate + rate_to_update;
                Ok(reward_rate)
            })?;
        }
    }
    TEMP_BALANCE.remove(
        deps.storage,
        AssetInfoKey::from(AssetInfo::Native(config.reward_denom.to_string())),
    );

    Ok(res)
}

fn rebalance_emissions(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    weights: Vec<EmissionsDistribution>,
) -> Result<Response, ContractError> {
    // Allow execution only from the controller account
    let config = CONFIG.load(deps.storage)?;
    is_controller(&info, &config)?;

    // // Before starting with the rebalance emission process
    // // rewards must be updated to the current block height
    // // Skip if no reward distribution in the first place
    // let res = if ASSET_REWARD_DISTRIBUTION.load(deps.storage).is_ok() {
    //     update_rewards(deps, env.clone(), info)?
    // } else {
    //     Response::new()
    // };

    let res = Response::new();
    Ok(res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&ExecuteMsg::RebalanceEmissionsCallback(weights)).unwrap(),
        funds: vec![],
    })))
}

fn rebalance_emissions_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    distributions: Vec<EmissionsDistribution>,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let total_distribution = distributions
        .iter()
        .map(|a| a.distribution.to_decimal().unwrap())
        .fold(Decimal::zero(), |acc, v| acc + v);
    if total_distribution > Decimal::one() {
        return Err(ContractError::InvalidTotalDistribution(total_distribution));
    }

    for distribution in distributions.iter() {
        let asset_info: AssetInfo =
            AssetInfoUnchecked::from_str(&distribution.denom)?.check(deps.api, None)?;
        let asset_key = AssetInfoKey::from(asset_info.clone());
        WHITELIST.update(
            deps.storage,
            asset_key,
            |current| -> Result<_, ContractError> {
                if let Some(current) = current {
                    Ok(current + distribution.distribution.to_decimal()?)
                } else {
                    Err(ContractError::AssetNotWhitelisted(asset_info.to_string()))
                }
            },
        )?;
    }

    let mut attrs = vec![("action".to_string(), "rebalance_emissions".to_string())];
    for distribution in distributions {
        attrs.push((
            distribution.denom.to_string(),
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

            let sub_msg_metadata = SubMsg::new(CosmosMsg::Custom(CustomExecuteMsg::Token(
                TokenExecuteMsg::SetMetadata {
                    denom: denom.clone(),
                    metadata: Metadata {
                        description: "Staking token for alliance protocol lp hub contract"
                            .to_string(),
                        denom_units: vec![DenomUnit {
                            denom: denom.clone(),
                            exponent: 0,
                            aliases: vec![],
                        }],
                        base: denom.to_string(),
                        display: denom.to_string(),
                        name: "Alliance LP Token".to_string(),
                        symbol: "ALLIANCE_LP".to_string(),
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
        CLAIM_ASTRO_REWARD_REPLY_ID => reply_astro_rewards(deps, reply),
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

// Example of a response https://terrasco.pe/testnet/tx/EC20D82F519B8B76EBFF1DDB75592330CA5A1CACE21943B22BAA4F46468AB5E7
fn reply_astro_rewards(
    deps: DepsMut,
    reply: Reply,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let result = reply.result.unwrap();
    let event = result
        .events
        .iter()
        .find(|event| event.ty == "wasm")
        .ok_or_else(|| StdError::generic_err("cannot find `wasm` event"))?;

    let first_attr = event.attributes[0].clone();
    let event_key = first_attr.key.clone();
    let even_value = first_attr.value.clone();

    if event_key != "_contract_address" && even_value != config.astro_incentives {
        return Err(ContractError::InvalidContractCallback(
            event_key, even_value,
        ));
    }
    
    for attr in event.attributes.clone() {
        if attr.key == "claimed_position" {
            // TODO : find the claimed_rewards
        }
    }

    Ok(Response::new().add_attributes(vec![("action", "reply_astro_rewards")]))
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
