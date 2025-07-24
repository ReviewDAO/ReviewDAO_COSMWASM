use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Token does not exist")]
    TokenNotFound {},

    #[error("Data is frozen")]
    DataFrozen {},

    #[error("Insufficient payment")]
    InsufficientPayment {},

    #[error("Payment failed")]
    PaymentFailed {},

    #[error("Token already exists")]
    TokenExists {},

    #[error("Not authorized")]
    NotAuthorized {},
}
