use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    Coin, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, WasmQuery, Addr, Decimal, from_json, to_json_binary,
};

use crate::astro_models::{QueryAstroMsg, RewardInfo, AstroRewardType, AstroAssetInfo};

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies.
/// This uses the Astroport CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: Default::default(),
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
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
            QueryRequest::Wasm(WasmQuery::Smart {contract_addr: _, msg})
                => match from_json(msg).unwrap() {
                    QueryAstroMsg::RewardInfo { lp_token } => {
                        if lp_token == "astro_existent_cw20" || lp_token == "astro_existent_native_coin" {
                            let msg = vec![RewardInfo {
                                reward: AstroRewardType::Int(AstroAssetInfo::Token {
                                    contract_addr: Addr::unchecked(lp_token),
                                }),
                                rps: Decimal::zero(),
                                index: Decimal::zero(),
                                orphaned: Decimal::zero(),
                            }];
                            return SystemResult::Ok(to_json_binary(&msg).into());
                        } 
                        panic!("The only mocked tokens are 'astro_existent_cw20' and 'astro_existent_native_coin' you send {}",lp_token)
                    }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
        }
    }
}
