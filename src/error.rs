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

    // DAO 相关错误
    #[error("Not a DAO member")]
    NotDaoMember {},

    #[error("Proposal not found")]
    ProposalNotFound {},

    #[error("Proposal has expired")]
    ProposalExpired {},

    #[error("Proposal already executed")]
    ProposalAlreadyExecuted {},

    #[error("Voting period has not ended")]
    VotingPeriodActive {},

    #[error("Proposal did not pass")]
    ProposalDidNotPass {},

    #[error("Cannot remove last DAO member")]
    CannotRemoveLastMember {},

    #[error("Member already exists")]
    MemberAlreadyExists {},

    #[error("Member does not exist")]
    MemberDoesNotExist {},

    #[error("Invalid voting threshold")]
    InvalidVotingThreshold {},

    #[error("Feature not implemented yet")]
    NotImplemented {},
}
