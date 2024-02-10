use crate::{
    astro_models::{Cw20Msg, ExecuteAstroMsg, PendingAssetRewards, QueryAstroMsg, RewardInfo},
    helpers::{from_string_to_asset_info, get_string_without_prefix, is_controller, is_governance},
    models::{Config, ExecuteMsg, InstantiateMsg, ModifyAssetPair, PendingRewards},
    state::{
        CONFIG, TEMP_BALANCE, TOTAL_ASSET_REWARD_RATE, TOTAL_BALANCES, UNCLAIMED_REWARDS,
        USER_ASSET_REWARD_RATE, USER_BALANCES, VALIDATORS, WHITELIST,
    },
};
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
    to_json_binary, Addr, BankMsg, Binary, Coin as CwCoin, Coins, CosmosMsg, Decimal, DepsMut,
    Empty, Env, MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoKey};
use cw_utils::parse_instantiate_response_data;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    vec,
};
use terra_proto_rs::{
    alliance::alliance::{MsgClaimDelegationRewards, MsgDelegate, MsgRedelegate, MsgUndelegate},
    cosmos::base::v1beta1::Coin,
    traits::Message,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra-alliance-lp-hub";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CREATE_REPLY_ID: u64 = 1;
const CLAIM_ALLIANCE_REWARDS_ERROR_REPLY_ID: u64 = 2;
const CLAIM_ASTRO_REWARDS_REPLY_ID: u64 = 3;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let governance_addr = deps.api.addr_validate(msg.governance_addr.as_str())?;
    let controller_addr = deps.api.addr_validate(msg.controller_addr.as_str())?;
    let astro_incentives_addr = deps.api.addr_validate(msg.astro_incentives_addr.as_str())?;
    let create_msg = TokenExecuteMsg::CreateDenom {
        subdenom: msg.alliance_token_subdenom,
    };
    let sub_msg = SubMsg::reply_on_success(
        CosmosMsg::Custom(CustomExecuteMsg::Token(create_msg)),
        CREATE_REPLY_ID,
    );
    let config = Config {
        governance_addr,
        controller_addr,
        astro_incentives_addr,
        alliance_token_denom: "".to_string(),
        alliance_token_supply: Uint128::zero(),
        alliance_reward_denom: msg.alliance_reward_denom,
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
        // Used to whitelist, modify or delete assets from the whitelist list
        ExecuteMsg::ModifyAssetPairs(assets) => modify_asset(deps, info, assets),

        // User interactions Stake, Unstake and ClaimRewards
        ExecuteMsg::Receive(cw20_msg) => {
            let sender = deps.api.addr_validate(&cw20_msg.sender)?;
            let received_asset = Asset::cw20(info.sender, cw20_msg.amount);

            stake(deps, sender, received_asset)
        }
        ExecuteMsg::Stake {} => {
            if info.funds.len() != 1 {
                return Err(ContractError::OnlySingleAssetAllowed {});
            }
            let coin = info.funds[0].clone();
            if coin.amount.is_zero() {
                return Err(ContractError::AmountCannotBeZero {});
            }
            stake(deps, info.sender, coin.into())
        }
        ExecuteMsg::Unstake(asset) => unstake(deps, env, info, asset),
        ExecuteMsg::UnstakeCallback(asset, addr) => unstake_callback(env, info, asset, addr),
        ExecuteMsg::ClaimRewards(asset) => claim_rewards(deps, info, asset),

        // Alliance interactions Delegate, undelegate and redelegate
        ExecuteMsg::AllianceDelegate(msg) => alliance_delegate(deps, env, info, msg),
        ExecuteMsg::AllianceUndelegate(msg) => alliance_undelegate(deps, env, info, msg),
        ExecuteMsg::AllianceRedelegate(msg) => alliance_redelegate(deps, env, info, msg),

        // Rewards related messages
        ExecuteMsg::UpdateRewards {} => update_rewards(deps, env, info),
        ExecuteMsg::UpdateAllianceRewardsCallback {} => {
            update_alliance_reward_callback(deps, env, info)
        }
    }
}

// Function that allows execution only though the governance address.
// It has two types of executions,
// - first execution type is to remove assets from the whitelist defined by asset.delete,
// - second execution type is to overwrite the asset in the whitelist with the new asset_distribution,
//   and it will also initiate the ASSET_REWARD_RATE to ZERO if it was not already initiated.
fn modify_asset(
    deps: DepsMut,
    info: MessageInfo,
    assets: Vec<ModifyAssetPair>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_governance(&info, &config)?;
    let mut attrs = vec![("action".to_string(), "modify_asset".to_string())];

    for asset in assets {
        if asset.delete {
            let deposit_asset_key = AssetInfoKey::from(asset.asset_info.clone());
            WHITELIST.remove(deps.storage, deposit_asset_key.clone());
            attrs.extend_from_slice(&[
                ("asset".to_string(), asset.asset_info.to_string()),
                ("to_remove".to_string(), asset.delete.to_string()),
            ]);
        } else {
            let deposit_asset_key = AssetInfoKey::from(asset.asset_info.clone());
            let reward_asset_key = asset
                .clone()
                .reward_asset_info
                .ok_or_else(|| ContractError::MissingRewardAsset(asset.asset_info.to_string()))
                .map(AssetInfoKey::from)?;
            WHITELIST.save(
                deps.storage,
                deposit_asset_key.clone(),
                &Decimal::new(asset.asset_distribution),
            )?;
            TOTAL_ASSET_REWARD_RATE.update(
                deps.storage,
                (deposit_asset_key, reward_asset_key),
                |asset_reward_rate| -> StdResult<_> { Ok(asset_reward_rate.unwrap_or_default()) },
            )?;
            attrs.extend_from_slice(&[("asset".to_string(), asset.asset_info.to_string())]);
        }
    }

    Ok(Response::new().add_attributes(attrs))
}

// This function is used to stake both native and CW20 tokens,
// it checks if the asset is whitelisted, if it is it will
// claim rewards and check if the asset is whitelisted in astro
// incentives update the balances incrementing both the
// USER_BALANCES and TOTAL_BALANCES. If the asset is whitelisted
// in astro incentives it will send the tokens to the astro incentives.
fn stake(deps: DepsMut, sender: Addr, received_asset: Asset) -> Result<Response, ContractError> {
    let deposit_asset_key = AssetInfoKey::from(&received_asset.info);
    WHITELIST
        .load(deps.storage, deposit_asset_key.clone())
        .map_err(|_| ContractError::AssetNotWhitelisted(received_asset.info.to_string()))?;

    let rewards_list = _claim_rewards(&deps, sender.clone(), received_asset.info.clone())?;
    for f in rewards_list.into_iter() {
        let key = (
            sender.clone(),
            AssetInfoKey::from(f.deposit_asset.clone().unwrap()),
            AssetInfoKey::from(f.reward_asset.clone().unwrap()),
        );

        // If the rewards are zero, it means that the asset is
        // staked for the first time or that there were no rewards
        // distributed to this asset, so to make it easy on future
        // operations we assign the total asset reward rate to the
        // user asset reward rate even if both values are the same.
        if f.rewards.is_zero() {
            let asset_reward_rate =
                TOTAL_ASSET_REWARD_RATE.load(deps.storage, (key.1.clone(), key.2.clone()))?;
            USER_ASSET_REWARD_RATE.save(deps.storage, key.clone(), &asset_reward_rate)?;
        } else {
            // When rewards are not zero, it means that the user has unclaimed
            // rewards, so we keep track of them in the UNCLAIMED_REWARDS map.
            UNCLAIMED_REWARDS.update(
                deps.storage,
                key.clone(),
                |b| -> Result<_, ContractError> { Ok(b.unwrap_or_default() + f.rewards) },
            )?;
        }
    }

    // Query astro incentives contract to check if the asset is whitelisted
    let config = CONFIG.load(deps.storage)?;
    let astro_incentives: Vec<RewardInfo> = deps
        .querier
        .query_wasm_smart(
            config.astro_incentives_addr.to_string(),
            &QueryAstroMsg::RewardInfo {
                lp_token: get_string_without_prefix(received_asset.info.clone()),
            },
        )
        .unwrap_or_default();

    let mut res = Response::new().add_attributes(vec![
        ("action", "stake"),
        ("user", sender.as_ref()),
        ("asset", &received_asset.info.to_string()),
        ("amount", &received_asset.amount.to_string()),
    ]);

    // When Astro incentives list is not empty it means that the asset
    // is whitelisted in astro incentives, so we need to send the
    // tokens to the astro incentives contract and then stake them.
    if !astro_incentives.is_empty() {
        let msg = _create_astro_deposit_msg(
            received_asset.clone(),
            config.astro_incentives_addr.to_string(),
        )?;
        res = res.add_message(msg);
    }

    TOTAL_BALANCES.update(
        deps.storage,
        deposit_asset_key.clone(),
        |b| -> Result<_, ContractError> { Ok(b.unwrap_or_default() + received_asset.amount) },
    )?;
    USER_BALANCES.update(
        deps.storage,
        (sender.clone(), deposit_asset_key.clone()),
        |b| -> Result<_, ContractError> { Ok(b.unwrap_or_default() + received_asset.amount) },
    )?;

    Ok(res)
}

fn _create_astro_deposit_msg(
    received_asset: Asset,
    astro_incentives_addr: String,
) -> Result<CosmosMsg, ContractError> {
    let msg = match received_asset.info.clone() {
        AssetInfo::Native(native_asset) => {
            // If the asset is native, we need to send it to the astro incentives contract
            // using the ExecuteAstroMsg::Deposit message
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: astro_incentives_addr,
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
                    contract: astro_incentives_addr,
                    amount: received_asset.amount,
                    msg: to_json_binary(&Cw20Msg::Deposit { recipient: None })?,
                })?,
                funds: vec![],
            })
        }
        _ => {
            return Err(ContractError::InvalidDenom(received_asset.info.to_string()));
        }
    };
    Ok(msg)
}

// This function unstake the alliance and astro deposits,
// if there are any astro deposits it will withdraw them
// from the astro incentives contract. This function also
// has a callback function that will send the tokens to the
// user after the astro incentives contract has withdrawn
fn unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Asset,
) -> Result<Response, ContractError> {
    if asset.amount.is_zero() {
        return Err(ContractError::AmountCannotBeZero {});
    }
    let config = CONFIG.load(deps.storage)?;
    let sender = info.sender.clone();
    let deposit_asset = AssetInfoKey::from(asset.info.clone());
    let rewards_list = _claim_rewards(&deps, sender.clone(), asset.info.clone())?;
    for f in rewards_list.into_iter() {
        let key = (
            sender.clone(),
            AssetInfoKey::from(f.deposit_asset.unwrap().clone()),
            AssetInfoKey::from(f.reward_asset.unwrap().clone()),
        );

        if f.rewards.is_zero() {
            let asset_reward_rate =
                TOTAL_ASSET_REWARD_RATE.load(deps.storage, (key.1.clone(), key.2.clone()))?;
            USER_ASSET_REWARD_RATE.save(deps.storage, key.clone(), &asset_reward_rate)?;
        } else {
            let unclaimed_rewards = UNCLAIMED_REWARDS
                .load(deps.storage, key.clone())
                .unwrap_or_default();
            let final_rewards = f.rewards + unclaimed_rewards;
            UNCLAIMED_REWARDS.save(deps.storage, key, &final_rewards)?
        }
    }

    let mut res = Response::new().add_attributes(vec![
        ("action", "unstake_alliance_lp"),
        ("user", sender.as_ref()),
        ("asset", &asset.info.to_string()),
        ("amount", &asset.amount.to_string()),
    ]);

    // Query astro incentives to check if there are enough staked tokens
    let astro_incentives_staked: Uint128 = deps
        .querier
        .query_wasm_smart(
            config.astro_incentives_addr.to_string(),
            &QueryAstroMsg::Deposit {
                lp_token: get_string_without_prefix(asset.info.clone()),
                user: env.contract.address.to_string(),
            },
        )
        .unwrap();

    // If there are enough tokens staked in astro incentives,
    // it means that we should withdraw tokens from astro
    // incentives otherwise the contract will endup having
    // less balance than the user is trying to unstake.
    if astro_incentives_staked >= asset.amount {
        let withdraw_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.astro_incentives_addr.to_string(),
            msg: to_json_binary(&ExecuteAstroMsg::Withdraw {
                lp_token: get_string_without_prefix(asset.info.clone()),
                amount: asset.amount,
            })?,
            funds: vec![],
        });
        res = res.add_message(withdraw_msg);
    }

    // Subtract the amount from the user balance and the total balance
    // since these tokens will be send to the user on the callback function
    let balance_key = (sender, deposit_asset.clone());
    USER_BALANCES.update(deps.storage, balance_key, |b| -> Result<_, ContractError> {
        match b {
            Some(b) => {
                if b < asset.amount {
                    Err(ContractError::InsufficientBalance {})
                } else {
                    Ok(b - asset.amount)
                }
            }
            None => Err(ContractError::InsufficientBalance {}),
        }
    })?;
    TOTAL_BALANCES.update(
        deps.storage,
        deposit_asset,
        |b| -> Result<_, ContractError> {
            let b = b.unwrap_or(Uint128::zero());
            if b < asset.amount {
                Err(ContractError::InsufficientBalance {})
            } else {
                Ok(b - asset.amount)
            }
        },
    )?;

    // Use a callback function to send staked tokens back
    // to the user after the possible executio of astro incentives
    // withdraw function execution is done. That way we can be sure
    // that the tokens are available to be sent back to the user.
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&ExecuteMsg::UnstakeCallback(asset, info.sender))?,
        funds: vec![],
    });

    Ok(res.add_message(msg))
}

// Callback that can be used by the contract itself to send
// tokens back to the user when unstaking. This function
// needs to be a callback because we need to be sure that
// the tokens are available to be sent back to the user
// after unstaking from astro incentives
fn unstake_callback(
    env: Env,
    info: MessageInfo,
    asset: Asset,
    usr: Addr,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_attribute("action", "unstake_alliance_lp_callback")
        .add_message(asset.transfer_msg(usr)?))
}

// Claiming rewards process iterates over the array of rewards
// and for each reward it will check if the user has any unclaimed
// rewards, if it does it will sum the unclaimed rewards with the
// rewards from the current iteration and send the tokens to the user.
fn claim_rewards(
    deps: DepsMut,
    info: MessageInfo,
    deposit_asset: AssetInfo,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let mut res = Response::new()
        .add_attribute("action", "claim_alliance_lp_rewards")
        .add_attribute("sender", sender.as_ref());

    let rewards_list = _claim_rewards(&deps, sender.clone(), deposit_asset.clone())?;
    for f in rewards_list.into_iter() {
        let deposit_asset_key = AssetInfoKey::from(f.deposit_asset.clone().unwrap());
        let reward_asset_key = AssetInfoKey::from(f.reward_asset.clone().unwrap());
        let key = (sender.clone(), deposit_asset_key, reward_asset_key);

        let unclaimed_rewards = UNCLAIMED_REWARDS
            .load(deps.storage, key.clone())
            .unwrap_or_default();
        UNCLAIMED_REWARDS.remove(deps.storage, key);

        let final_rewards = f.rewards + unclaimed_rewards;
        if !final_rewards.is_zero() {
            let rewards_asset = Asset {
                info: f.reward_asset.clone().unwrap(),
                amount: final_rewards,
            };
            res = res
                .add_attribute("deposit_asset", &deposit_asset.to_string())
                .add_attribute("reward_asset", rewards_asset.info.to_string())
                .add_attribute("rewards_amount", &final_rewards.to_string())
                .add_message(rewards_asset.transfer_msg(&sender)?)
        }
    }

    Ok(res)
}

fn _claim_rewards(
    deps: &DepsMut,
    user: Addr,
    deposit_asset: AssetInfo,
) -> Result<Vec<PendingRewards>, ContractError> {
    let deposit_asset_key = AssetInfoKey::from(deposit_asset.clone());
    let user_balance = USER_BALANCES
        .load(deps.storage, (user.clone(), deposit_asset_key.clone()))
        .unwrap_or_default();
    let user_balance = Decimal::from_atomics(user_balance, 0)?;

    let claimed_rewards = TOTAL_ASSET_REWARD_RATE
        .prefix(deposit_asset_key.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|f| {
            let (reward_asset, total_asset_reward_rate) = f?;
            let reward_asset = reward_asset.check(deps.api, None)?;
            let reward_asset_info_key = AssetInfoKey::from(reward_asset.clone());

            let user_asset_reward_rate = USER_ASSET_REWARD_RATE.load(
                deps.storage,
                (
                    user.clone(),
                    deposit_asset_key.clone(),
                    reward_asset_info_key.clone(),
                ),
            );

            if let Ok(user_asset_reward_rate) = user_asset_reward_rate {
                if user_balance.is_zero() {
                    return Ok(PendingRewards::new(
                        deposit_asset.clone(),
                        reward_asset.clone(),
                        Uint128::zero(),
                    ));
                } else {
                    let rewards = ((total_asset_reward_rate - user_asset_reward_rate)
                        * user_balance)
                        .to_uint_floor();

                    return Ok(PendingRewards::new(
                        deposit_asset.clone(),
                        reward_asset.clone(),
                        rewards,
                    ));
                }
            }
            Ok(PendingRewards::new(
                deposit_asset.clone(),
                reward_asset.clone(),
                Uint128::zero(),
            ))
        })
        .collect();

    claimed_rewards
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
        .add_attributes(vec![("action", "alliance_lp_delegate")])
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
        .add_attributes(vec![("action", "alliance_lp_undelegate")])
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
        .add_attributes(vec![("action", "alliance_lp_redelegate")])
        .add_messages(msgs))
}

fn _update_astro_rewards(
    deps: &DepsMut,
    contract_addr: Addr,
    astro_incentives: Addr,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut whitelist: Vec<String> = Vec::new();

    for f in WHITELIST.range(deps.storage, None, None, Order::Ascending) {
        let (asset_info, _) = f?;
        let asset_info = asset_info.check(deps.api, None)?;
        let asset_denom = get_string_without_prefix(asset_info);

        whitelist.push(asset_denom);
    }

    let mut lp_tokens_list: Vec<String> = vec![];

    for lp_token in whitelist {
        let pending_rewards: Vec<PendingAssetRewards> = deps
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
            if !pr.amount.is_zero() {
                lp_tokens_list.push(lp_token.clone())
            }
        }
    }

    let mut sub_msgs: Vec<SubMsg> = vec![];
    for lp_token in lp_tokens_list.clone() {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: astro_incentives.to_string(),
            msg: to_json_binary(&ExecuteAstroMsg::ClaimRewards {
                lp_tokens: vec![lp_token],
            })?,
            funds: vec![],
        });

        sub_msgs.push(SubMsg::reply_always(msg, CLAIM_ASTRO_REWARDS_REPLY_ID));
    }

    Ok(sub_msgs)
}

fn update_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut res = Response::new().add_attributes(vec![("action", "update_alliance_lp_rewards")]);

    // Iterate over received funds, check the balance of the smart contract
    // and finally update the temp balance with the difference between the
    // contract balance and coin received.
    if !info.funds.is_empty() {
        info.funds.iter().try_for_each(|coin| {
            let asset_info = AssetInfo::Native(coin.denom.clone());
            let contract_balance =
                asset_info.query_balance(&deps.querier, env.contract.address.clone())?;

            TEMP_BALANCE.save(
                deps.storage,
                AssetInfoKey::from(asset_info),
                &(contract_balance - coin.amount),
            )
        })?;
    } else {
        let contract_balance = config
            .alliance_reward_denom
            .query_balance(&deps.querier, env.contract.address.clone())?;

        TEMP_BALANCE.save(
            deps.storage,
            AssetInfoKey::from(config.alliance_reward_denom.clone()),
            &contract_balance,
        )?
    }

    let astr_incentives_rewards = _update_astro_rewards(
        &deps,
        env.contract.address.clone(),
        config.astro_incentives_addr.clone(),
    )?;
    if !astr_incentives_rewards.is_empty() {
        res = res.add_submessages(astr_incentives_rewards)
    };

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
            SubMsg::reply_on_error(msg, CLAIM_ALLIANCE_REWARDS_ERROR_REPLY_ID)
        })
        .collect();
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&ExecuteMsg::UpdateAllianceRewardsCallback {})?,
        funds: vec![],
    });

    Ok(res.add_submessages(alliance_sub_msg).add_message(msg))
}

// This function only deals with alliance rewards. Other rewards (e.g. ASTRO)
// needs to be dealt with separately This is because the reward distribution
// only affects alliance rewards. LP rewards are directly distributed to LP
// holders and not pooled together and shared
fn update_alliance_reward_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    let config = CONFIG.load(deps.storage)?;
    let mut res =
        Response::new().add_attributes(vec![("action", "update_alliance_rewards_callback")]);
    let reward_asset_info_key = AssetInfoKey::from(config.alliance_reward_denom.clone());
    let current_balance = config
        .alliance_reward_denom
        .clone()
        .query_balance(&deps.querier, env.contract.address)?;
    let previous_balance = TEMP_BALANCE.load(deps.storage, reward_asset_info_key.clone())?;
    let rewards_collected = current_balance - previous_balance;

    let whitelist: Vec<(AssetInfoKey, Decimal)> = WHITELIST
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(a, d)| (AssetInfoKey(a), d)))
        .collect::<StdResult<Vec<(AssetInfoKey, Decimal)>>>()?;

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
                to_address: config.controller_addr.to_string(),
                amount: vec![CwCoin::new(
                    unallocated_rewards.u128(),
                    get_string_without_prefix(config.alliance_reward_denom.clone()),
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
            TOTAL_ASSET_REWARD_RATE.update(
                deps.storage,
                (asset_key.clone(), reward_asset_info_key.clone()),
                |rate| -> StdResult<_> {
                    let mut reward_rate = rate.unwrap_or_default();
                    reward_rate += rate_to_update;
                    Ok(reward_rate)
                },
            )?;
        }
    }
    TEMP_BALANCE.remove(
        deps.storage,
        AssetInfoKey::from(config.alliance_reward_denom),
    );

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
    match reply.id {
        CREATE_REPLY_ID => reply_instantiate(deps, env, reply),
        CLAIM_ALLIANCE_REWARDS_ERROR_REPLY_ID => {
            Ok(Response::new().add_attributes(vec![("action", "claim_alliance_lp_rewards_error")]))
        }
        CLAIM_ASTRO_REWARDS_REPLY_ID => reply_claim_astro_rewards(deps, reply),
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

fn reply_instantiate(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
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
                description: "Staking token for alliance protocol lp hub contract".to_string(),
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

// This is an example of a response from claiming astro rewards
// https://terrasco.pe/testnet/tx/EC20D82F519B8B76EBFF1DDB75592330CA5A1CACE21943B22BAA4F46468AB5E7
fn reply_claim_astro_rewards(
    deps: DepsMut,
    reply: Reply,
) -> Result<Response<CustomExecuteMsg>, ContractError> {
    let mut res = Response::new();

    // Check if the reply result returns an error and
    // if it does so wrap the error in the response
    // attribute and return the response.
    if reply.result.is_err() {
        res = res.add_attributes(vec![
            ("action", "claim_alliance_lp_astro_rewards_error"),
            ("error", &reply.result.unwrap_err()),
        ]);

        return Ok(res);
    }

    // If the request has been successful, we need to check
    // if the contract address and the event attributes are
    // correct and account for the rewards.
    let config = CONFIG.load(deps.storage)?;
    let result = reply.result.unwrap();
    let event = result
        .events
        .iter()
        .find(|event| event.ty == "wasm")
        .ok_or_else(|| StdError::generic_err("cannot find `wasm` event on astro reply"))?;

    // Check if the callback comes from the correct contract
    let first_attr = event.attributes[0].clone();
    let event_key = first_attr.key.clone();
    let event_value = first_attr.value;
    if event_key != "_contract_address" && event_value != config.astro_incentives_addr {
        return Err(ContractError::InvalidContractCallback(
            event_key,
            event_value,
        ));
    }

    // Map the event attributes to a hashmap to make it easier
    // to group the rewards by the claimed position and then
    // account for the rewards.
    let mut astro_claims: HashMap<String, Coins> = HashMap::new();
    let mut current_position: Option<String> = None;
    for attr in event.attributes.iter() {
        match attr.key.as_str() {
            "claimed_position" => {
                current_position = Some(attr.value.clone());
                astro_claims.entry(attr.value.clone()).or_default();
            }
            "claimed_reward" => {
                if let Some(position) = &current_position {
                    if let Some(astro_claim) = astro_claims.get_mut(position) {
                        let coin = CwCoin::from_str(&attr.value)?;
                        astro_claim.add(coin)?;
                    }
                }
            }
            _ => {}
        }
    }

    // Given the claimed rewards account them to the state of the contract
    // dividing the reward_amount per total_lp_staked summing the already
    // accounted asset_reward_rate.
    for (deposited_asset, claim) in astro_claims {
        let deposit_asset_key = from_string_to_asset_info(deposited_asset.clone())?;
        let total_lp_staked = TOTAL_BALANCES.load(deps.storage, deposit_asset_key.clone())?;
        let total_lp_staked = Decimal::from_atomics(total_lp_staked, 0)?;

        for reward in claim {
            let reward_asset_key = from_string_to_asset_info(reward.denom.clone())?;
            let reward_ammount = Decimal::from_atomics(reward.amount, 0)?;
            let asset_reward_rate_key = (deposit_asset_key.clone(), reward_asset_key);
            let mut asset_reward_rate = TOTAL_ASSET_REWARD_RATE
                .load(deps.storage, asset_reward_rate_key.clone())
                .unwrap_or_default();

            asset_reward_rate = (reward_ammount / total_lp_staked) + asset_reward_rate;

            TOTAL_ASSET_REWARD_RATE.save(
                deps.storage,
                asset_reward_rate_key,
                &asset_reward_rate,
            )?;
        }
    }

    Ok(res.add_attribute("action", "claim_alliance_lp_astro_rewards_success"))
}
