use cosmwasm_std::StdError;
use thiserror::Error;

/// This enum describes errors for the generator_proxy_template contract
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Incorrect CW20 hook message variant!")]
    IncorrectCw20HookMessageVariant {},
}
