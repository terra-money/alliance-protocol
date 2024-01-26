use crate::{
    astro_models::{Cw20Msg, ExecuteAstroMsg, QueryAstroMsg, RewardInfo},
    helpers::{is_controller, is_governance},
    models::{
        from_string_to_asset_info, AstroClaimRewardsPosition, Config, ExecuteMsg, InstantiateMsg,
        ModifyAssetPair,
    },
    state::{
        ASSET_REWARD_RATE, BALANCES, CONFIG, TEMP_BALANCE, TOTAL_BALANCES, UNCLAIMED_REWARDS,
        USER_ASSET_REWARD_RATE, VALIDATORS, WHITELIST,
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
    to_json_binary, Addr, BankMsg, Binary, Coin as CwCoin, CosmosMsg, Decimal, DepsMut, Empty, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_asset::{Asset, AssetInfo, AssetInfoKey};
use cw_utils::parse_instantiate_response_data;
use std::{collections::HashSet, env, str::FromStr};
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
    let governance_address = deps.api.addr_validate(msg.governance.as_str())?;
    let controller_address = deps.api.addr_validate(msg.controller.as_str())?;
    let astro_incentives_addr = deps.api.addr_validate(msg.astro_incentives_addr.as_str())?;
    let fee_collector_addr = deps.api.addr_validate(msg.fee_collector_addr.as_str())?;
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
        fee_collector_addr: fee_collector_addr,

        astro_incentives_addr: astro_incentives_addr,
        astro_reward_denom: msg.astro_reward_denom,

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
        // Used to whitelist, modify or delete assets from the allowed list
        ExecuteMsg::ModifyAssetPairs(assets) => modify_asset(deps, info, assets),

        // User interactions Stake, Unstake and ClaimRewards
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

// This method iterate through the list of assets to be modified,
// for each asset it checks if it is being listed or delisted,
fn modify_asset(
    deps: DepsMut,
    info: MessageInfo,
    assets: Vec<ModifyAssetPair>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    is_governance(&info, &config)?;
    let mut attrs = vec![("action".to_string(), "modify_asset".to_string())];

    for asset in assets {
        let reward_asset_info_key = match asset.reward_asset_info {
            Some(reward_asset_info) => AssetInfoKey::from(reward_asset_info),
            None => {
                return Err(ContractError::MissingRewardAsset(
                    asset.asset_info.to_string(),
                ))
            }
        };
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
                (asset_key, reward_asset_info_key),
                |asset_reward_rate| -> StdResult<_> {
                    Ok(asset_reward_rate.unwrap_or(Decimal::zero()))
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
    let deposit_asset_key = AssetInfoKey::from(&received_asset.info);
    WHITELIST
        .load(deps.storage, deposit_asset_key.clone())
        .map_err(|_| ContractError::AssetNotWhitelisted(received_asset.info.to_string()))?;
    let config = CONFIG.load(deps.storage)?;
    let reward_asset_key = AssetInfoKey::from(AssetInfo::Native(config.alliance_reward_denom));
    let rewards = _claim_alliance_rewards(
        deps.storage,
        sender.clone(),
        AssetInfoKey::from(received_asset.info.clone()),
        reward_asset_key.clone(),
    )?;
    if !rewards.is_zero() {
        UNCLAIMED_REWARDS.update(
            deps.storage,
            (
                sender.clone(),
                deposit_asset_key.clone(),
                reward_asset_key.clone(),
            ),
            |balance| -> Result<_, ContractError> {
                let mut unclaimed_rewards = balance.unwrap_or_default();
                unclaimed_rewards += rewards;
                Ok(unclaimed_rewards)
            },
        )?;
    }

    // Query astro incentives, to do so we must first remove the prefix
    // from the asset info e.g. cw20:asset1 -> asset1 or native:uluna -> uluna
    let lp_token = received_asset.info.to_string();
    let astro_incentives: Vec<RewardInfo> = deps.querier.query_wasm_smart(
        config.astro_incentives_addr.to_string(),
        &QueryAstroMsg::RewardInfo {
            lp_token: lp_token.split(':').collect::<Vec<&str>>()[1].to_string(),
        },
    )?;

    let mut res = Response::new().add_attributes(vec![
        ("action", "stake"),
        ("user", sender.as_ref()),
        ("asset", &received_asset.info.to_string()),
        ("amount", &received_asset.amount.to_string()),
    ]);

    // When Astro incentives is not empty it means that the asset
    // is whitelisted in astro incentives, so we need to send the
    // tokens to the astro incentives contract and then stake them.
    // Besides that we also claim rewards from astro incentives if any
    if !astro_incentives.is_empty() {
        let msg = _create_astro_deposit_msg(
            received_asset.clone(),
            config.astro_incentives_addr.to_string(),
            env,
        )?;
        res = res.add_message(msg);
        let astro_reward_token =
            AssetInfoKey::from(AssetInfo::Native(config.astro_reward_denom.clone()));
        let received_asset_key = AssetInfoKey::from(received_asset.info.clone());
        let astro_rewards = _claim_astro_rewards(
            deps.storage,
            sender.clone(),
            received_asset_key.clone(),
            astro_reward_token.clone(),
        )?;

        if !astro_rewards.is_zero() {
            let key = (sender.clone(), received_asset_key, astro_reward_token);
            UNCLAIMED_REWARDS.update(deps.storage, key, |balance| -> Result<_, ContractError> {
                let mut unclaimed_rewards = balance.unwrap_or_default();
                unclaimed_rewards += rewards;
                Ok(unclaimed_rewards)
            })?;
        }
    }

    BALANCES.update(
        deps.storage,
        (sender.clone(), deposit_asset_key.clone()),
        |balance| -> Result<_, ContractError> {
            match balance {
                Some(balance) => Ok(balance + received_asset.amount),
                None => Ok(received_asset.amount),
            }
        },
    )?;
    TOTAL_BALANCES.update(
        deps.storage,
        deposit_asset_key.clone(),
        |balance| -> Result<_, ContractError> {
            Ok(balance.unwrap_or(Uint128::zero()) + received_asset.amount)
        },
    )?;

    let asset_reward_rate = ASSET_REWARD_RATE
        .load(
            deps.storage,
            (deposit_asset_key.clone(), reward_asset_key.clone()),
        )
        .unwrap_or(Decimal::zero());
    USER_ASSET_REWARD_RATE.save(
        deps.storage,
        (sender, deposit_asset_key, reward_asset_key),
        &asset_reward_rate,
    )?;
    Ok(res)
}

fn _create_astro_deposit_msg(
    received_asset: Asset,
    astro_incentives_addr: String,
    env: Env,
) -> Result<CosmosMsg, ContractError> {
    let msg = match received_asset.info.clone() {
        AssetInfo::Native(native_asset) => {
            // If the asset is native, we need to send it to the astro incentives contract
            // using the ExecuteAstroMsg::Deposit message
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: astro_incentives_addr.to_string(),
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
    let reward_asset_token = AssetInfoKey::from(AssetInfo::Native(config.alliance_token_denom));
    let rewards = _claim_alliance_rewards(
        deps.storage,
        sender.clone(),
        AssetInfoKey::from(asset.info.clone()),
        reward_asset_token.clone(),
    )?;
    if !rewards.is_zero() {
        let key = (sender.clone(), deposit_asset.clone(), reward_asset_token);
        UNCLAIMED_REWARDS.update(deps.storage, key, |balance| -> Result<_, ContractError> {
            let mut unclaimed_rewards = balance.unwrap_or_default();
            unclaimed_rewards += rewards;
            Ok(unclaimed_rewards)
        })?;
    }

    let mut res = Response::new().add_attributes(vec![
        ("action", "unstake_alliance_lp"),
        ("user", sender.clone().as_ref()),
        ("asset", &asset.info.to_string()),
        ("amount", &asset.amount.to_string()),
    ]);

    // Query astro incentives to check if there are enough staked tokens
    // transforming the lp_token to the following formats cw20:asset1 -> asset1,
    // or native:uluna -> uluna
    let lp_token: String = asset.info.to_string();
    let lp_token = lp_token.split(':').collect::<Vec<&str>>()[1].to_string();
    let astro_incentives_staked: Uint128 = deps.querier.query_wasm_smart(
        config.astro_incentives_addr.to_string(),
        &QueryAstroMsg::Deposit {
            lp_token: lp_token.to_string(),
            user: env.contract.address.to_string(),
        },
    )?;

    // If there are enough tokens staked in astro incentives,
    // it means that we should withdraw tokens from astro
    // incentives otherwise the contract will endup having
    // less balance than the user is trying to unstake.
    if astro_incentives_staked >= asset.amount {
        let astro_reward_token =
            AssetInfoKey::from(AssetInfo::Native(config.astro_reward_denom.clone()));
        let astro_rewards = _claim_astro_rewards(
            deps.storage,
            sender.clone(),
            AssetInfoKey::from(deposit_asset.clone()),
            astro_reward_token.clone(),
        )?;

        if !astro_rewards.is_zero() {
            let key = (sender.clone(), deposit_asset.clone(), astro_reward_token);
            UNCLAIMED_REWARDS.update(deps.storage, key, |balance| -> Result<_, ContractError> {
                let mut unclaimed_rewards = balance.unwrap_or_default();
                unclaimed_rewards += rewards;
                Ok(unclaimed_rewards)
            })?;
        }

        let withdraw_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.astro_incentives_addr.to_string(),
            msg: to_json_binary(&ExecuteAstroMsg::Withdraw {
                lp_token: lp_token,
                amount: asset.amount,
            })?,
            funds: vec![],
        });
        res = res.add_message(withdraw_msg);
    }

    // Subtract the amount from the user balance and the total balance
    // since these tokens will be send to the user on the callback function
    let balance_key = (sender.clone(), deposit_asset.clone());
    BALANCES.update(deps.storage, balance_key, |b| -> Result<_, ContractError> {
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
        deposit_asset.clone(),
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

fn claim_rewards(
    deps: DepsMut,
    info: MessageInfo,
    deposit_asset: AssetInfo,
) -> Result<Response, ContractError> {
    let user = info.sender;
    let config = CONFIG.load(deps.storage)?;
    let mut res = Response::new().add_attribute("action", "claim_alliance_lp_rewards");

    // Claim alliance rewards, add the rewards to the response,
    // create the transfer msg and send the tokens to the user.
    let alliance_reward_token_key =
        AssetInfoKey::from(AssetInfo::Native(config.alliance_reward_denom.clone()));
    let alliance_rewards = _claim_alliance_rewards(
        deps.storage,
        user.clone(),
        AssetInfoKey::from(deposit_asset.clone()),
        alliance_reward_token_key.clone(),
    )?;
    let unclaimed_rewards = UNCLAIMED_REWARDS
        .load(
            deps.storage,
            (
                user.clone(),
                AssetInfoKey::from(deposit_asset.clone()),
                alliance_reward_token_key.clone(),
            ),
        )
        .unwrap_or_default();
    let final_alliance_rewards = alliance_rewards + unclaimed_rewards;
    UNCLAIMED_REWARDS.remove(
        deps.storage,
        (
            user.clone(),
            AssetInfoKey::from(deposit_asset.clone()),
            alliance_reward_token_key.clone(),
        ),
    );

    res = res.add_attributes(vec![
        ("user", user.as_ref()),
        ("asset", &deposit_asset.to_string()),
        (
            "alliance_reward_amount",
            &final_alliance_rewards.to_string(),
        ),
    ]);
    if !final_alliance_rewards.is_zero() {
        let rewards_asset = Asset {
            info: AssetInfo::Native(config.alliance_reward_denom),
            amount: final_alliance_rewards,
        };
        res = res.add_message(rewards_asset.transfer_msg(&user)?)
    }

    let astro_reward_token =
        AssetInfoKey::from(AssetInfo::Native(config.astro_reward_denom.clone()));
    let astro_rewards = _claim_astro_rewards(
        deps.storage,
        user.clone(),
        AssetInfoKey::from(deposit_asset.clone()),
        astro_reward_token.clone(),
    )?;
    let unclaimed_rewards = UNCLAIMED_REWARDS
        .load(
            deps.storage,
            (
                user.clone(),
                AssetInfoKey::from(deposit_asset.clone()),
                astro_reward_token.clone(),
            ),
        )
        .unwrap_or_default();
    let final_astro_rewards = astro_rewards + unclaimed_rewards;
    UNCLAIMED_REWARDS.remove(
        deps.storage,
        (
            user.clone(),
            AssetInfoKey::from(deposit_asset.clone()),
            astro_reward_token,
        ),
    );
    res = res.add_attribute("astro_reward_amount", &final_astro_rewards.to_string());
    if !final_astro_rewards.is_zero() {
        let info = match deps.api.addr_validate(&config.astro_reward_denom) {
            Ok(addr) => AssetInfo::Cw20(addr),
            Err(_) => AssetInfo::Native(config.astro_reward_denom.clone()),
        };

        let rewards_asset = Asset {
            info: info,
            amount: final_astro_rewards,
        };
        res = res.add_message(rewards_asset.transfer_msg(&user)?)
    }

    Ok(res)
}

fn _claim_alliance_rewards(
    storage: &mut dyn Storage,
    user: Addr,
    staked_asset: AssetInfoKey,
    reward_denom: AssetInfoKey,
) -> Result<Uint128, ContractError> {
    let state_key = (user.clone(), staked_asset.clone(), reward_denom.clone());
    let user_reward_rate = USER_ASSET_REWARD_RATE.load(storage, state_key.clone());
    let asset_reward_rate = ASSET_REWARD_RATE
        .load(storage, (staked_asset.clone(), reward_denom.clone()))
        .unwrap_or_default();

    if let Ok(user_reward_rate) = user_reward_rate {
        let user_staked = BALANCES
            .load(storage, (user.clone(), staked_asset.clone()))
            .unwrap_or_default();
        let user_staked = Decimal::from_atomics(user_staked, 0)?;
        let rewards = ((asset_reward_rate - user_reward_rate) * user_staked).to_uint_floor();

        if rewards.is_zero() {
            Ok(Uint128::zero())
        } else {
            USER_ASSET_REWARD_RATE.save(storage, state_key, &asset_reward_rate)?;
            Ok(rewards)
        }
    } else {
        USER_ASSET_REWARD_RATE.save(storage, state_key, &asset_reward_rate)?;
        Ok(Uint128::zero())
    }
}

fn _claim_astro_rewards(
    storage: &mut dyn Storage,
    user: Addr,
    staked_asset: AssetInfoKey,
    reward_denom: AssetInfoKey,
) -> Result<Uint128, ContractError> {
    let state_key: (Addr, AssetInfoKey, AssetInfoKey) =
        (user.clone(), staked_asset.clone(), reward_denom.clone());
    let user_reward_rate = USER_ASSET_REWARD_RATE.load(storage, state_key.clone());
    let asset_reward_rate = ASSET_REWARD_RATE
        .load(storage, (staked_asset.clone(), reward_denom.clone()))
        .unwrap_or_default();

    if let Ok(user_reward_rate) = user_reward_rate {
        let total_staked = TOTAL_BALANCES
            .load(storage, staked_asset.clone())
            .unwrap_or_default();
        let user_staked = Decimal::from_atomics(total_staked, 0)?;
        let rewards = ((asset_reward_rate - user_reward_rate) * user_staked).to_uint_floor();

        if rewards.is_zero() {
            Ok(Uint128::zero())
        } else {
            USER_ASSET_REWARD_RATE.save(storage, state_key, &asset_reward_rate)?;
            Ok(rewards)
        }
    } else {
        USER_ASSET_REWARD_RATE.save(storage, state_key, &asset_reward_rate)?;
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
) -> Result<Option<SubMsg>, ContractError> {
    let mut whitelist: Vec<String> = Vec::new();

    for f in WHITELIST.range(deps.storage, None, None, Order::Ascending) {
        let (asset_info, _) = f?;
        let asset_info = asset_info.check(deps.api, None)?;
        let asset_string = asset_info.to_string();
        let asset_denom = asset_string.split(":").collect::<Vec<&str>>()[1].to_string();

        whitelist.push(asset_denom);
    }

    let mut lp_tokens_list: Vec<String> = vec![];

    for lp_token in whitelist {
        let pending_rewards: Vec<Asset> = deps.querier.query_wasm_smart(
            astro_incentives.to_string(),
            &QueryAstroMsg::PendingRewards {
                lp_token: lp_token.clone(),
                user: contract_addr.to_string(),
            },
        )?;

        for pr in pending_rewards {
            if !pr.amount.is_zero() {
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

        return Ok(Some(SubMsg::reply_always(
            msg,
            CLAIM_ASTRO_REWARDS_REPLY_ID,
        )));
    }

    Ok(None)
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
        let asset_info = AssetInfo::Native(config.alliance_reward_denom.clone());
        let contract_balance =
            asset_info.query_balance(&deps.querier, env.contract.address.clone())?;

        TEMP_BALANCE.save(
            deps.storage,
            AssetInfoKey::from(asset_info),
            &contract_balance,
        )?
    }

    let astr_incentives_rewards = _update_astro_rewards(
        &deps,
        env.contract.address.clone(),
        config.astro_incentives_addr.clone(),
    )?;
    if let Some(msg) = astr_incentives_rewards {
        res = res.add_submessage(msg)
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
    // We only deal with alliance rewards here. Other rewards (e.g. ASTRO) needs to be dealt with separately
    // This is because the reward distribution only affects alliance rewards. LP rewards are directly distributed to LP holders
    // and not pooled together and shared
    let reward_asset = AssetInfo::native(config.alliance_reward_denom.clone());
    let reward_asset_info_key =
        AssetInfoKey::from(AssetInfo::Native(config.alliance_reward_denom.to_string()));
    let current_balance = reward_asset.query_balance(&deps.querier, env.contract.address)?;
    let previous_balance = TEMP_BALANCE.load(deps.storage, reward_asset_info_key.clone())?;
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
                to_address: config.fee_collector_addr.to_string(),
                amount: vec![CwCoin::new(
                    unallocated_rewards.u128(),
                    config.alliance_reward_denom.clone(),
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
            ASSET_REWARD_RATE.update(
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
        AssetInfoKey::from(AssetInfo::Native(config.alliance_reward_denom.to_string())),
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
    let even_value = first_attr.value.clone();
    if event_key != "_contract_address" && even_value != config.astro_incentives_addr {
        return Err(ContractError::InvalidContractCallback(
            event_key, even_value,
        ));
    }

    // In the list of attributes we need to find the **claimed_positions**,
    // and group all the **claimed_reward** for each claimed position.
    let mut astro_claims: Vec<AstroClaimRewardsPosition> = vec![];
    let mut astro_claim = AstroClaimRewardsPosition::default();
    let mut count_claim_position = true;
    for attr in event.attributes.iter() {
        if attr.key == "claimed_position" && count_claim_position {
            astro_claim = AstroClaimRewardsPosition::default();
            astro_claim.deposited_asset = attr.value.clone();
            count_claim_position = false;
        }

        if attr.key == "claimed_reward" {
            let coin = CwCoin::from_str(&attr.value)?;
            astro_claim.rewards.add(coin)?;
        }

        if attr.key == "claimed_reward" && !count_claim_position {
            astro_claims.push(astro_claim.clone());
            count_claim_position = true
        }
    }

    // Given the claimed rewards account them to the state of the contract
    // dividing the reward_amount per total_lp_staked summing the already
    // accounted asset_reward_rate.
    for claim in astro_claims {
        let deposit_asset_key = from_string_to_asset_info(claim.deposited_asset)?;
        let total_lp_staked = TOTAL_BALANCES.load(deps.storage, deposit_asset_key.clone())?;
        let total_lp_staked = Decimal::from_atomics(total_lp_staked, 0)?;

        for reward in claim.rewards {
            let reward_asset_key = from_string_to_asset_info(reward.denom)?;
            let reward_ammount = Decimal::from_atomics(reward.amount, 0)?;
            let mut asset_reward_rate = Decimal::zero();

            ASSET_REWARD_RATE.update(
                deps.storage,
                (deposit_asset_key.clone(), reward_asset_key),
                |a| -> StdResult<_> {
                    asset_reward_rate = a.unwrap_or_default();
                    asset_reward_rate = (reward_ammount / total_lp_staked) + asset_reward_rate;
                    Ok(asset_reward_rate)
                },
            )?;
        }
    }

    Ok(res.add_attribute("action", "claim_alliance_lp_astro_rewards_success"))
}
