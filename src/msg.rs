use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // 核心功能
    CreateDataItem {
        ipfs_hash: String,
        price: Uint128,
        is_public: bool,
        metadata_uri: String,
    },
    RequestAccess {
        token_id: String,
    },

    // 数据管理功能
    UpdateDataItem {
        token_id: String,
        new_ipfs_hash: String,
        new_metadata_uri: String,
    },
    FreezeData {
        token_id: String,
        freeze: bool,
    },

    // 权限管理
    GrantAccess {
        token_id: String,
        grantee: String,
        level: AccessLevel,
    },

    // NFT 转移功能
    TransferNft {
        recipient: String,
        token_id: String,
    },
    Approve {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
    },
    RevokeAll {
        operator: String,
    },

    CreatePaperItem {
        ipfs_hash: String,
        doi: String,
        metadata_uri: String,
    },
    SetBaseCitationFee {
        fee: Uint128,
    },

    CitePaper {
        paper_id: String,
    },
    SubmitCorrection {
        original_paper_id: String,
        new_ipfs_hash: String,
    },

    // DAO 治理消息
    SubmitArticleProposal {
        ipfs_hash: String,
        doi: String,
        metadata_uri: String,
        title: String,
        description: String,
    },
    SubmitMemberProposal {
        member_address: String,
        action: MemberAction,
        title: String,
        description: String,
    },
    VoteOnProposal {
        proposal_id: u64,
        choice: VoteChoice,
    },
    ExecuteProposal {
        proposal_id: u64,
    },
    UpdateDaoConfig {
        voting_period: Option<u64>,
        approval_threshold: Option<u64>,
        min_members: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // NFT 基础查询
    OwnerOf {
        token_id: String,
    },
    TokenInfo {
        token_id: String,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},

    // 研究数据特定查询
    GetDataItem {
        token_id: String,
    },
    GetDataVersions {
        token_id: String,
    },
    GetAuthorizedUsers {
        token_id: String,
    },
    CheckAccessLevel {
        token_id: String,
        user: String,
    },
    GetCitations {
        paper_id: String,
    },
    GetPaperDoi {
        paper_id: String,
    },
    GetBaseCitationFee {},

    // DAO 查询
    GetDaoMembers {},
    GetDaoConfig {},
    GetProposal {
        proposal_id: u64,
    },
    GetProposals {
        start_after: Option<u64>,
        limit: Option<u32>,
        status_filter: Option<ProposalStatus>,
    },
    GetVote {
        proposal_id: u64,
        voter: String,
    },
    GetVoteCount {
        proposal_id: u64,
    },
    GetMemberVotingPower {
        member: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum AccessLevel {
    None,
    Read,
    Write,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DataItem {
    pub owner: Addr,
    pub ipfs_hash: String,
    pub price: Uint128,
    pub is_public: bool,
    pub total_earned: Uint128,
    pub created_at: u64,
    pub last_updated: u64,
    pub metadata_uri: String,
    pub is_frozen: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DataVersion {
    pub ipfs_hash: String,
    pub timestamp: u64,
}

// 查询响应类型
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfoResponse {
    pub token_id: String,
    pub owner: Addr,
    pub data_item: DataItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfoResponse {
    pub name: String,
    pub symbol: String,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerOfResponse {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NumTokensResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Citation {
    pub citer: Addr,
    pub amount: Uint128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BaseCitationFeeResponse {
    pub fee: Uint128,
}

// DAO 相关数据结构
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DaoConfig {
    pub voting_period: u64,      // 投票期限（秒）
    pub approval_threshold: u64, // 通过阈值（百分比，如51表示51%）
    pub min_members: u64,        // 最小成员数量
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Addr,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub created_at: u64,
    pub voting_end: u64,
    pub status: ProposalStatus,
    pub execution_data: Option<ExecutionData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProposalType {
    ArticlePublication,
    AddMember,
    RemoveMember,
    UpdateConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProposalStatus {
    Active,   // 投票中
    Passed,   // 通过
    Rejected, // 拒绝
    Executed, // 已执行
    Expired,  // 已过期
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vote {
    pub voter: Addr,
    pub choice: VoteChoice,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteCount {
    pub yes: u64,
    pub no: u64,
    pub abstain: u64,
    pub total_eligible: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecutionData {
    ArticlePublication {
        ipfs_hash: String,
        doi: String,
        metadata_uri: String,
    },
    MemberChange {
        member_address: String,
        action: MemberAction,
    },
    ConfigUpdate {
        new_config: DaoConfig,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum MemberAction {
    Add,
    Remove,
}

// DAO 查询响应结构
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DaoMembersResponse {
    pub members: Vec<Addr>,
    pub total_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalsResponse {
    pub proposals: Vec<Proposal>,
    pub total_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotingPowerResponse {
    pub power: u64,
    pub is_member: bool,
}

// Additional DAO query response structures
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DaoConfigResponse {
    pub config: DaoConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalResponse {
    pub proposal: Proposal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteResponse {
    pub vote: Option<Vote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteCountResponse {
    pub vote_count: VoteCount,
}
