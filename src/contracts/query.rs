use cosmwasm_std::{entry_point, to_json_binary, Addr, Binary, Deps, Env, StdError, StdResult};

use crate::helpers::is_dao_member;
use crate::msg::{
    AccessLevel, BaseCitationFeeResponse, Citation, ContractInfoResponse, DataItem, DataVersion,
    NumTokensResponse, OwnerOfResponse, Proposal, ProposalStatus, QueryMsg, TokenInfoResponse,
    VoteChoice, VoteCount,
};
use crate::state::{
    ACCESS_CONTROLS, AUTHORIZED_USERS, BASE_CITATION_FEE, CITATIONS, CONTRACT_NAME, CONTRACT_OWNER,
    CONTRACT_SYMBOL, DAO_CONFIG, DAO_MEMBERS, DATA_ITEMS, DATA_VERSIONS, PAPER_DOIS, PROPOSALS,
    TOKEN_COUNT, TOKEN_OWNERS, VOTES, VOTE_COUNTS,
};

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OwnerOf { token_id } => to_json_binary(&query_owner_of(deps, token_id)?),
        QueryMsg::TokenInfo { token_id } => to_json_binary(&query_token_info(deps, token_id)?),
        QueryMsg::AllTokens { start_after, limit } => {
            to_json_binary(&query_all_tokens(deps, start_after, limit)?)
        }
        QueryMsg::NumTokens {} => to_json_binary(&query_num_tokens(deps)?),
        QueryMsg::ContractInfo {} => to_json_binary(&query_contract_info(deps)?),
        QueryMsg::GetDataItem { token_id } => to_json_binary(&query_data_item(deps, token_id)?),
        QueryMsg::GetDataVersions { token_id } => {
            to_json_binary(&query_data_versions(deps, token_id)?)
        }
        QueryMsg::GetAuthorizedUsers { token_id } => {
            to_json_binary(&query_authorized_users(deps, token_id)?)
        }
        QueryMsg::CheckAccessLevel { token_id, user } => {
            to_json_binary(&query_access_level(deps, token_id, user)?)
        }

        QueryMsg::GetCitations { paper_id } => to_json_binary(&query_citations(deps, paper_id)?),
        QueryMsg::GetPaperDoi { paper_id } => to_json_binary(&query_paper_doi(deps, paper_id)?),
        QueryMsg::GetBaseCitationFee {} => to_json_binary(&query_base_citation_fee(deps)?),

        // DAO queries
        QueryMsg::GetDaoMembers {} => to_json_binary(&query_dao_members(deps)?),
        QueryMsg::GetDaoConfig {} => to_json_binary(&query_dao_config(deps)?),
        QueryMsg::GetProposal { proposal_id } => {
            to_json_binary(&query_proposal(deps, proposal_id)?)
        }
        QueryMsg::GetProposals {
            start_after,
            limit,
            status_filter,
        } => to_json_binary(&query_proposals(deps, start_after, limit, status_filter)?),
        QueryMsg::GetVote { proposal_id, voter } => {
            to_json_binary(&query_vote(deps, proposal_id, voter)?)
        }
        QueryMsg::GetVoteCount { proposal_id } => {
            to_json_binary(&query_vote_count(deps, proposal_id)?)
        }
        QueryMsg::GetMemberVotingPower { member } => {
            to_json_binary(&query_member_voting_power(deps, member)?)
        }
    }
}

pub fn query_owner_of(deps: Deps, token_id: String) -> StdResult<OwnerOfResponse> {
    let owner = TOKEN_OWNERS.load(deps.storage, &token_id)?;
    Ok(OwnerOfResponse { owner })
}

pub fn query_token_info(deps: Deps, token_id: String) -> StdResult<TokenInfoResponse> {
    let owner = TOKEN_OWNERS.load(deps.storage, &token_id)?;
    let data_item = DATA_ITEMS.load(deps.storage, &token_id)?;

    Ok(TokenInfoResponse {
        token_id,
        owner,
        data_item,
    })
}

pub fn query_citations(deps: Deps, paper_id: String) -> StdResult<Vec<Citation>> {
    CITATIONS
        .may_load(deps.storage, &paper_id)
        .map(|citations| citations.unwrap_or_default())
}

pub fn query_all_tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<String>> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after
        .as_deref()
        .map(cw_storage_plus::Bound::exclusive);

    let tokens: StdResult<Vec<String>> = TOKEN_OWNERS
        .range(deps.storage, start, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| k))
        .collect();

    tokens
}

pub fn query_num_tokens(deps: Deps) -> StdResult<NumTokensResponse> {
    let count = TOKEN_COUNT.load(deps.storage)?;
    Ok(NumTokensResponse { count })
}

pub fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    let name = CONTRACT_NAME.load(deps.storage)?;
    let symbol = CONTRACT_SYMBOL.load(deps.storage)?;
    let owner = CONTRACT_OWNER.load(deps.storage)?;

    Ok(ContractInfoResponse {
        name,
        symbol,
        owner,
    })
}

pub fn query_data_item(deps: Deps, token_id: String) -> StdResult<DataItem> {
    DATA_ITEMS.load(deps.storage, &token_id)
}

pub fn query_data_versions(deps: Deps, token_id: String) -> StdResult<Vec<DataVersion>> {
    DATA_VERSIONS.load(deps.storage, &token_id)
}

pub fn query_authorized_users(deps: Deps, token_id: String) -> StdResult<Vec<Addr>> {
    AUTHORIZED_USERS
        .may_load(deps.storage, &token_id)
        .map(|users| users.unwrap_or_default())
}

pub fn query_access_level(deps: Deps, token_id: String, user: String) -> StdResult<AccessLevel> {
    ACCESS_CONTROLS
        .may_load(deps.storage, (&token_id, &user))
        .map(|level| level.unwrap_or(AccessLevel::None))
}

pub fn query_paper_doi(deps: Deps, paper_id: String) -> StdResult<String> {
    PAPER_DOIS.load(deps.storage, &paper_id)
}

pub fn query_base_citation_fee(deps: Deps) -> StdResult<BaseCitationFeeResponse> {
    let fee = BASE_CITATION_FEE.load(deps.storage)?;
    Ok(BaseCitationFeeResponse { fee })
}

// DAO 查询函数实现

/// 查询所有 DAO 成员
// pub fn query_dao_members(deps: Deps) -> StdResult<crate::msg::DaoMembersResponse> {
//     let members: StdResult<Vec<Addr>> = DAO_MEMBERS
//         .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
//         .filter_map(|item| match item {
//             Ok((addr_str, is_member)) => {
//                 if is_member {
//                     match deps.api.addr_validate(&addr_str) {
//                         Ok(addr) => Some(Ok(addr)),
//                         Err(e) => Some(Err(e)),
//                     }
//                 } else {
//                     None
//                 }
//             }
//             Err(e) => Some(Err(e)),
//         })
//         .collect();

//     let members = members?;
//     let total_count = members.len() as u64;

//     Ok(crate::msg::DaoMembersResponse {
//         members,
//         total_count,
//     })
//
pub fn query_dao_members(deps: Deps) -> StdResult<crate::msg::DaoMembersResponse> {
    let members: Result<Vec<Addr>, StdError> = DAO_MEMBERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((addr_str, is_member)) => {
                if is_member {
                    Some(deps.api.addr_validate(&addr_str))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .collect();

    let members = members?;
    let total_count = members.len() as u64;

    Ok(crate::msg::DaoMembersResponse {
        members,
        total_count,
    })
}

/// 查询 DAO 配置参数
pub fn query_dao_config(deps: Deps) -> StdResult<crate::msg::DaoConfigResponse> {
    let config = DAO_CONFIG.load(deps.storage)?;
    Ok(crate::msg::DaoConfigResponse { config })
}

/// 查询单个提案详情
pub fn query_proposal(deps: Deps, proposal_id: u64) -> StdResult<crate::msg::ProposalResponse> {
    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    Ok(crate::msg::ProposalResponse { proposal })
}

/// 查询提案列表，支持分页和状态过滤
pub fn query_proposals(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
    status_filter: Option<ProposalStatus>,
) -> StdResult<crate::msg::ProposalsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;

    // 设置起始点
    let start = start_after.map(cw_storage_plus::Bound::exclusive);

    // 获取所有提案并应用过滤器
    let proposals: StdResult<Vec<Proposal>> = PROPOSALS
        .range(deps.storage, start, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            match item {
                Ok((_, proposal)) => {
                    // 应用状态过滤器
                    if let Some(ref filter_status) = status_filter {
                        if proposal.status != *filter_status {
                            return None;
                        }
                    }
                    Some(Ok(proposal))
                }
                Err(e) => Some(Err(e)),
            }
        })
        .take(limit)
        .collect();

    let proposals = proposals?;

    // 获取总数（用于分页信息）
    let total_count = if let Some(ref filter_status) = status_filter {
        // 如果有状态过滤器，需要计算过滤后的总数
        PROPOSALS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .filter(|item| match item {
                Ok((_, proposal)) => proposal.status == *filter_status,
                Err(_) => false,
            })
            .count() as u64
    } else {
        // 没有过滤器，返回所有提案数量
        PROPOSALS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .count() as u64
    };

    Ok(crate::msg::ProposalsResponse {
        proposals,
        total_count,
    })
}

/// 查询特定投票记录
pub fn query_vote(
    deps: Deps,
    proposal_id: u64,
    voter: String,
) -> StdResult<crate::msg::VoteResponse> {
    let voter_addr = deps.api.addr_validate(&voter)?;
    let vote = VOTES.may_load(deps.storage, (proposal_id, voter_addr.as_str()))?;
    Ok(crate::msg::VoteResponse { vote })
}

/// 查询提案投票统计
pub fn query_vote_count(deps: Deps, proposal_id: u64) -> StdResult<crate::msg::VoteCountResponse> {
    // 首先检查提案是否存在
    PROPOSALS.load(deps.storage, proposal_id)?;

    // 尝试从缓存中加载投票统计
    let vote_count = VOTE_COUNTS.may_load(deps.storage, proposal_id)?;

    let vote_count = if let Some(count) = vote_count {
        count
    } else {
        // 如果缓存中没有，实时计算投票统计
        let mut yes = 0u64;
        let mut no = 0u64;
        let mut abstain = 0u64;

        // 遍历所有投票记录
        let votes: StdResult<Vec<_>> = VOTES
            .prefix(proposal_id)
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();

        for vote_result in votes? {
            let (_, vote) = vote_result;
            match vote.choice {
                VoteChoice::Yes => yes += 1,
                VoteChoice::No => no += 1,
                VoteChoice::Abstain => abstain += 1,
            }
        }

        // 获取 DAO 成员总数作为有资格投票的总数
        let total_eligible = query_dao_members(deps)?.total_count;

        VoteCount {
            yes,
            no,
            abstain,
            total_eligible,
        }
    };

    Ok(crate::msg::VoteCountResponse { vote_count })
}

/// 查询成员投票权力
pub fn query_member_voting_power(
    deps: Deps,
    member: String,
) -> StdResult<crate::msg::VotingPowerResponse> {
    let member_addr = deps.api.addr_validate(&member)?;
    let is_member = is_dao_member(deps, &member_addr)?;

    // 在当前实现中，每个 DAO 成员都有相等的投票权重（1 票）
    let power = if is_member { 1u64 } else { 0u64 };

    Ok(crate::msg::VotingPowerResponse { power, is_member })
}
