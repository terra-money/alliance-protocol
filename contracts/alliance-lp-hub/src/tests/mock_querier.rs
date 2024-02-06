use crate::astro_models::{
    AstroAssetInfo, AstroRewardType, PendingAssetRewards, QueryAstroMsg, RewardInfo,
};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Coin, Decimal, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};

const ASTRO_MOCK_CONTRACT_ADDR: &str = "astro_incentives";

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies.
/// This uses the Astroport CustomQuerier.
pub fn mock_dependencies(
    balance: Option<&[Coin]>,
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = match balance {
        Some(b) => {
            let balances = vec![(ASTRO_MOCK_CONTRACT_ADDR, b), (MOCK_CONTRACT_ADDR, b)];

            WasmMockQuerier::new(MockQuerier::new(&balances))
        }
        None => WasmMockQuerier::new(MockQuerier::new(&[(ASTRO_MOCK_CONTRACT_ADDR, &[])])),
    };
    // MockQuerier::default()
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: Default::default(),
    }
}

pub struct WasmMockQuerier {
    pub base: MockQuerier<Empty>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely
        let request: QueryRequest<Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => match from_json(msg).unwrap() {
                QueryAstroMsg::RewardInfo { lp_token } => {
                    if lp_token == "terra_astro_cw20" || lp_token == "factory/astro_native" {
                        let msg = vec![RewardInfo {
                            reward: AstroRewardType::Int(AstroAssetInfo::Token {
                                contract_addr: Addr::unchecked(lp_token),
                            }),
                            rps: Decimal::one(),
                            index: Decimal::one(),
                            orphaned: Decimal::one(),
                        }];
                        return SystemResult::Ok(to_json_binary(&msg).into());
                    }
                    let msg: Vec<RewardInfo> = vec![];
                    SystemResult::Ok(to_json_binary(&msg).into())
                }
                QueryAstroMsg::PendingRewards { lp_token, user: _ } => {
                    if lp_token == "factory/astro_native" {
                        let msg = vec![PendingAssetRewards {
                            info: AstroAssetInfo::NativeToken { denom: lp_token },
                            amount: Uint128::one(),
                        }];
                        return SystemResult::Ok(to_json_binary(&msg).into());
                    } else if lp_token == "terra_astro_cw20" {
                        let msg = vec![PendingAssetRewards {
                            info: AstroAssetInfo::Token {
                                contract_addr: Addr::unchecked(lp_token),
                            },
                            amount: Uint128::one(),
                        }];
                        return SystemResult::Ok(to_json_binary(&msg).into());
                    }

                    let msg = vec![PendingAssetRewards {
                        info: AstroAssetInfo::Token {
                            contract_addr: Addr::unchecked(lp_token),
                        },
                        amount: Uint128::zero(),
                    }];
                    SystemResult::Ok(to_json_binary(&msg).into())
                }
                QueryAstroMsg::Deposit { lp_token, user: _ } => {
                    if lp_token == "factory/astro_native" {
                        return SystemResult::Ok(to_json_binary(&Uint128::one()).into());
                    } else if lp_token == "terra_astro_cw20" {
                        return SystemResult::Ok(to_json_binary(&Uint128::new(50)).into());
                    }
                    SystemResult::Ok(to_json_binary(&Uint128::zero()).into())
                }
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier { base }
    }
}
