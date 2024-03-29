use cosmwasm_std::{Decimal, DecimalRangeExceeded, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Only a single asset is allowed")]
    OnlySingleAssetAllowed {},

    #[error("Asset '{0}' not whitelisted")]
    AssetNotWhitelisted(String),

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Amount cannot be zero")]
    AmountCannotBeZero {},

    #[error("Invalid reply id {0}")]
    InvalidReplyId(u64),

    #[error("Empty delegation")]
    EmptyDelegation {},

    #[error("Invalid reward rate '{0}' for denom '{1}'")]
    InvalidRewardRate(Decimal, String),

    #[error("Invalid total distribution: {0}")]
    InvalidTotalDistribution(Decimal),

    #[error("Asset not staked")]
    AssetNotStaked {},
}
