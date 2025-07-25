use crate::msg::{
    AccessLevel, Citation, DaoConfig, DataItem, DataVersion, Proposal, Vote, VoteCount,
};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

// 合约基础信息
pub const CONTRACT_NAME: Item<String> = Item::new("contract_name");
pub const CONTRACT_SYMBOL: Item<String> = Item::new("contract_symbol");
pub const CONTRACT_OWNER: Item<Addr> = Item::new("contract_owner");

// Token 计数器
pub const TOKEN_ID_COUNTER: Item<u64> = Item::new("token_id_counter");
pub const TOKEN_COUNT: Item<u64> = Item::new("token_count");

// NFT 所有权和批准
pub const TOKEN_OWNERS: Map<&str, Addr> = Map::new("token_owners");
pub const TOKEN_APPROVALS: Map<&str, Addr> = Map::new("token_approvals");
pub const OPERATOR_APPROVALS: Map<(&str, &str), bool> = Map::new("operator_approvals");

// 研究数据特定存储
pub const DATA_ITEMS: Map<&str, DataItem> = Map::new("data_items");
pub const DATA_VERSIONS: Map<&str, Vec<DataVersion>> = Map::new("data_versions");
pub const ACCESS_CONTROLS: Map<(&str, &str), AccessLevel> = Map::new("access_controls");
pub const AUTHORIZED_USERS: Map<&str, Vec<Addr>> = Map::new("authorized_users");

// 论文特定存储
pub const CITATIONS: Map<&str, Vec<Citation>> = Map::new("citations");
pub const PAPER_DOIS: Map<&str, String> = Map::new("paper_dois");
pub const BASE_CITATION_FEE: Item<Uint128> = Item::new("base_citation_fee");

// DAO 存储
pub const DAO_MEMBERS: Map<&str, bool> = Map::new("dao_members");
pub const DAO_CONFIG: Item<DaoConfig> = Item::new("dao_config");
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
pub const PROPOSAL_COUNTER: Item<u64> = Item::new("proposal_counter");
pub const VOTES: Map<(u64, &str), Vote> = Map::new("votes");
pub const VOTE_COUNTS: Map<u64, VoteCount> = Map::new("vote_counts");
