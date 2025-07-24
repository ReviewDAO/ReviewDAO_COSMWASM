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
