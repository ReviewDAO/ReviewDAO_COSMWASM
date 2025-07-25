use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::helpers::{
    ensure_can_execute_proposal, ensure_can_vote_on_proposal, ensure_dao_member,
    ensure_proposal_exists, is_dao_member, validate_dao_config, validate_voting_period,
};
use crate::msg::{
    AccessLevel, BaseCitationFeeResponse, Citation, ContractInfoResponse, DaoConfig, DataItem,
    DataVersion, ExecuteMsg, ExecutionData, InstantiateMsg, MemberAction, NumTokensResponse,
    OwnerOfResponse, Proposal, ProposalStatus, ProposalType, QueryMsg, TokenInfoResponse, Vote,
    VoteChoice, VoteCount,
};
use crate::state::{
    ACCESS_CONTROLS, AUTHORIZED_USERS, BASE_CITATION_FEE, CITATIONS, CONTRACT_NAME, CONTRACT_OWNER,
    CONTRACT_SYMBOL, DAO_CONFIG, DAO_MEMBERS, DATA_ITEMS, DATA_VERSIONS, OPERATOR_APPROVALS,
    PAPER_DOIS, PROPOSALS, PROPOSAL_COUNTER, TOKEN_APPROVALS, TOKEN_COUNT, TOKEN_ID_COUNTER,
    TOKEN_OWNERS, VOTES, VOTE_COUNTS,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = deps.api.addr_validate(&msg.owner)?;

    CONTRACT_NAME.save(deps.storage, &msg.name)?;
    CONTRACT_SYMBOL.save(deps.storage, &msg.symbol)?;
    CONTRACT_OWNER.save(deps.storage, &owner)?;
    TOKEN_ID_COUNTER.save(deps.storage, &0u64)?;
    TOKEN_COUNT.save(deps.storage, &0u64)?;

    BASE_CITATION_FEE.save(deps.storage, &Uint128::new(100_000))?; // 0.1 token

    // 初始化 DAO
    // 将合约创建者设置为第一个 DAO 成员
    DAO_MEMBERS.save(deps.storage, owner.as_str(), &true)?;

    // 初始化默认的 DAO 配置参数
    let dao_config = DaoConfig {
        voting_period: 604800,  // 7 天 (7 * 24 * 60 * 60 秒)
        approval_threshold: 51, // 51% 通过阈值
        min_members: 1,         // 最小成员数量为 1
    };
    DAO_CONFIG.save(deps.storage, &dao_config)?;

    // 设置提案计数器为 0
    PROPOSAL_COUNTER.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("name", msg.name)
        .add_attribute("symbol", msg.symbol)
        .add_attribute("owner", owner.to_string())
        .add_attribute("dao_initialized", "true")
        .add_attribute("first_dao_member", owner.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateDataItem {
            ipfs_hash,
            price,
            is_public,
            metadata_uri,
        } => execute_create_data_item(deps, env, info, ipfs_hash, price, is_public, metadata_uri),

        ExecuteMsg::RequestAccess { token_id } => execute_request_access(deps, env, info, token_id),

        ExecuteMsg::UpdateDataItem {
            token_id,
            new_ipfs_hash,
            new_metadata_uri,
        } => execute_update_data_item(deps, env, info, token_id, new_ipfs_hash, new_metadata_uri),

        ExecuteMsg::FreezeData { token_id, freeze } => {
            execute_freeze_data(deps, env, info, token_id, freeze)
        }

        ExecuteMsg::GrantAccess {
            token_id,
            grantee,
            level,
        } => execute_grant_access(deps, env, info, token_id, grantee, level),

        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),

        ExecuteMsg::Approve { spender, token_id } => {
            execute_approve(deps, env, info, spender, token_id)
        }

        ExecuteMsg::ApproveAll { operator } => execute_approve_all(deps, env, info, operator),

        ExecuteMsg::RevokeAll { operator } => execute_revoke_all(deps, env, info, operator),

        ExecuteMsg::CreatePaperItem {
            ipfs_hash,
            doi,
            metadata_uri,
        } => execute_create_paper_item(deps, env, info, ipfs_hash, doi, metadata_uri),

        ExecuteMsg::CitePaper { paper_id } => execute_cite_paper(deps, env, info, paper_id),

        ExecuteMsg::SubmitCorrection {
            original_paper_id,
            new_ipfs_hash,
        } => execute_submit_correction(deps, env, info, original_paper_id, new_ipfs_hash),

        ExecuteMsg::SetBaseCitationFee { fee } => {
            execute_set_base_citation_fee(deps, env, info, fee)
        }

        // DAO 治理消息
        ExecuteMsg::SubmitArticleProposal {
            ipfs_hash,
            doi,
            metadata_uri,
            title,
            description,
        } => execute_submit_article_proposal(
            deps,
            env,
            info,
            ipfs_hash,
            doi,
            metadata_uri,
            title,
            description,
        ),

        ExecuteMsg::SubmitMemberProposal {
            member_address,
            action,
            title,
            description,
        } => execute_submit_member_proposal(
            deps,
            env,
            info,
            member_address,
            action,
            title,
            description,
        ),

        ExecuteMsg::VoteOnProposal {
            proposal_id,
            choice,
        } => execute_vote_on_proposal(deps, env, info, proposal_id, choice),

        ExecuteMsg::ExecuteProposal { proposal_id } => {
            execute_proposal(deps, env, info, proposal_id)
        }

        ExecuteMsg::UpdateDaoConfig {
            voting_period,
            approval_threshold,
            min_members,
        } => execute_update_dao_config(
            deps,
            env,
            info,
            voting_period,
            approval_threshold,
            min_members,
        ),
    }
}

pub fn execute_cite_paper(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    paper_id: String,
) -> Result<Response, ContractError> {
    // 检查论文是否存在
    let paper_owner = TOKEN_OWNERS
        .load(deps.storage, &paper_id)
        .map_err(|_| ContractError::TokenNotFound {})?;

    let base_fee = BASE_CITATION_FEE.load(deps.storage)?;

    // 检查付款
    let payment = info
        .funds
        .iter()
        .find(|coin| coin.denom == "utoken")
        .map(|coin| coin.amount)
        .unwrap_or_else(Uint128::zero);

    if payment < base_fee {
        return Err(ContractError::InsufficientPayment {});
    }

    // 添加引用记录
    let citation = Citation {
        citer: info.sender.clone(),
        amount: payment,
        timestamp: env.block.time.seconds(),
    };

    let mut citations = CITATIONS
        .may_load(deps.storage, &paper_id)?
        .unwrap_or_default();
    citations.push(citation);
    CITATIONS.save(deps.storage, &paper_id, &citations)?;

    let mut response = Response::new()
        .add_attribute("method", "cite_paper")
        .add_attribute("paper_id", paper_id)
        .add_attribute("citer", info.sender.to_string())
        .add_attribute("amount", payment.to_string());

    // 分配费用：95% 给作者，5% 给 DAO
    if payment > Uint128::zero() {
        let dao_share = payment * Uint128::new(5) / Uint128::new(100);
        let author_share = payment - dao_share;

        // 发送给作者
        let author_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: paper_owner.to_string(),
            amount: vec![Coin {
                denom: "utoken".to_string(),
                amount: author_share,
            }],
        });

        // 发送给 DAO (合约所有者)
        let contract_owner = CONTRACT_OWNER.load(deps.storage)?;
        let dao_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: contract_owner.to_string(),
            amount: vec![Coin {
                denom: "utoken".to_string(),
                amount: dao_share,
            }],
        });

        response = response.add_messages(vec![author_msg, dao_msg]);
    }

    Ok(response)
}

pub fn execute_submit_correction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    original_paper_id: String,
    new_ipfs_hash: String,
) -> Result<Response, ContractError> {
    // 检查对原论文的授权
    if !is_approved_or_owner(deps.as_ref(), &info.sender, &original_paper_id)? {
        return Err(ContractError::NotAuthorized {});
    }

    let original_data = DATA_ITEMS.load(deps.storage, &original_paper_id)?;
    let original_doi = PAPER_DOIS.load(deps.storage, &original_paper_id)?;
    let original_versions = DATA_VERSIONS.load(deps.storage, &original_paper_id)?;

    // 创建修正版本的 DOI (添加版本号)
    let correction_doi = format!("{}-v{}", original_doi, original_versions.len() + 1);

    // 获取下一个 token ID
    let token_id = TOKEN_ID_COUNTER.load(deps.storage)?;
    let token_id_str = token_id.to_string();

    // 检查 token 是否已存在
    if TOKEN_OWNERS.has(deps.storage, &token_id_str) {
        return Err(ContractError::TokenExists {});
    }

    // 创建 NFT token
    TOKEN_OWNERS.save(deps.storage, &token_id_str, &info.sender)?;

    // 创建修正版本的数据项
    let data_item = DataItem {
        owner: info.sender.clone(),
        ipfs_hash: new_ipfs_hash.clone(),
        price: Uint128::zero(),
        is_public: true,
        total_earned: Uint128::zero(),
        created_at: env.block.time.seconds(),
        last_updated: env.block.time.seconds(),
        metadata_uri: original_data.metadata_uri,
        is_frozen: false,
    };
    DATA_ITEMS.save(deps.storage, &token_id_str, &data_item)?;

    // 创建初始版本
    let version = DataVersion {
        ipfs_hash: new_ipfs_hash.clone(),
        timestamp: env.block.time.seconds(),
    };
    DATA_VERSIONS.save(deps.storage, &token_id_str, &vec![version])?;

    // 保存修正版本的 DOI
    PAPER_DOIS.save(deps.storage, &token_id_str, &correction_doi)?;

    // 更新计数器
    TOKEN_ID_COUNTER.save(deps.storage, &(token_id + 1))?;
    let count = TOKEN_COUNT.load(deps.storage)?;
    TOKEN_COUNT.save(deps.storage, &(count + 1))?;

    Ok(Response::new()
        .add_attribute("method", "submit_correction")
        .add_attribute("token_id", &token_id_str)
        .add_attribute("original_paper_id", original_paper_id)
        .add_attribute("correction_id", token_id_str)
        .add_attribute("correction_doi", correction_doi)
        .add_attribute("correction_type", "paper_correction"))
}

pub fn execute_set_base_citation_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee: Uint128,
) -> Result<Response, ContractError> {
    // 只有合约所有者可以设置费用
    let contract_owner = CONTRACT_OWNER.load(deps.storage)?;
    if info.sender != contract_owner {
        return Err(ContractError::NotAuthorized {});
    }

    BASE_CITATION_FEE.save(deps.storage, &fee)?;

    Ok(Response::new()
        .add_attribute("method", "set_base_citation_fee")
        .add_attribute("new_fee", fee.to_string()))
}

pub fn execute_create_data_item(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ipfs_hash: String,
    price: Uint128,
    is_public: bool,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    // 获取下一个 token ID
    let token_id = TOKEN_ID_COUNTER.load(deps.storage)?;
    let token_id_str = token_id.to_string();

    // 检查 token 是否已存在
    if TOKEN_OWNERS.has(deps.storage, &token_id_str) {
        return Err(ContractError::TokenExists {});
    }

    // 创建 NFT token
    TOKEN_OWNERS.save(deps.storage, &token_id_str, &info.sender)?;

    // 创建数据项
    let data_item = DataItem {
        owner: info.sender.clone(),
        ipfs_hash: ipfs_hash.clone(),
        price,
        is_public,
        total_earned: Uint128::zero(),
        created_at: env.block.time.seconds(),
        last_updated: env.block.time.seconds(),
        metadata_uri,
        is_frozen: false,
    };
    DATA_ITEMS.save(deps.storage, &token_id_str, &data_item)?;

    // 创建初始版本
    let version = DataVersion {
        ipfs_hash: ipfs_hash.clone(),
        timestamp: env.block.time.seconds(),
    };
    DATA_VERSIONS.save(deps.storage, &token_id_str, &vec![version])?;

    // 更新计数器
    TOKEN_ID_COUNTER.save(deps.storage, &(token_id + 1))?;
    let count = TOKEN_COUNT.load(deps.storage)?;
    TOKEN_COUNT.save(deps.storage, &(count + 1))?;

    Ok(Response::new()
        .add_attribute("method", "create_data_item")
        .add_attribute("token_id", token_id_str)
        .add_attribute("owner", info.sender)
        .add_attribute("ipfs_hash", ipfs_hash))
}

pub fn execute_request_access(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    // 检查 token 是否存在
    let owner = TOKEN_OWNERS
        .load(deps.storage, &token_id)
        .map_err(|_| ContractError::TokenNotFound {})?;

    let mut data_item = DATA_ITEMS.load(deps.storage, &token_id)?;

    let mut response = Response::new()
        .add_attribute("method", "request_access")
        .add_attribute("token_id", token_id.clone())
        .add_attribute("requester", info.sender.to_string());

    if !data_item.is_public {
        // 检查授权
        let access_level = ACCESS_CONTROLS
            .may_load(deps.storage, (&token_id, info.sender.as_str()))?
            .unwrap_or(AccessLevel::None);

        let is_owner = owner == info.sender;
        let is_approved = TOKEN_APPROVALS
            .may_load(deps.storage, &token_id)?
            .map(|approved| approved == info.sender)
            .unwrap_or(false);
        let is_operator = OPERATOR_APPROVALS
            .may_load(deps.storage, (owner.as_str(), info.sender.as_str()))?
            .unwrap_or(false);

        if !is_owner && !is_approved && !is_operator && access_level == AccessLevel::None {
            return Err(ContractError::NotAuthorized {});
        }

        // 处理付款
        if !info.funds.is_empty() {
            let payment = info
                .funds
                .iter()
                .find(|coin| coin.denom == "utoken") // 假设使用原生代币
                .map(|coin| coin.amount)
                .unwrap_or_else(Uint128::zero);

            if payment < data_item.price {
                return Err(ContractError::InsufficientPayment {});
            }

            // 发送付款给所有者
            if payment > Uint128::zero() {
                let send_msg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: owner.to_string(),
                    amount: vec![Coin {
                        denom: "utoken".to_string(),
                        amount: payment,
                    }],
                });
                response = response.add_message(send_msg);

                // 更新总收入
                data_item.total_earned += payment;
                DATA_ITEMS.save(deps.storage, &token_id, &data_item)?;
            }
        }
    }

    Ok(response)
}

pub fn execute_update_data_item(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    new_ipfs_hash: String,
    new_metadata_uri: String,
) -> Result<Response, ContractError> {
    let mut data_item = DATA_ITEMS
        .load(deps.storage, &token_id)
        .map_err(|_| ContractError::TokenNotFound {})?;

    // 检查授权 - 必须是所有者或被批准的用户
    if !is_approved_or_owner(deps.as_ref(), &info.sender, &token_id)? {
        return Err(ContractError::NotAuthorized {});
    }

    if data_item.is_frozen {
        return Err(ContractError::DataFrozen {});
    }

    // 更新数据项
    data_item.ipfs_hash = new_ipfs_hash.clone();
    data_item.metadata_uri = new_metadata_uri;
    data_item.last_updated = env.block.time.seconds();
    DATA_ITEMS.save(deps.storage, &token_id, &data_item)?;

    // 添加新版本
    let mut versions = DATA_VERSIONS.load(deps.storage, &token_id)?;
    versions.push(DataVersion {
        ipfs_hash: new_ipfs_hash.clone(),
        timestamp: env.block.time.seconds(),
    });
    DATA_VERSIONS.save(deps.storage, &token_id, &versions)?;

    Ok(Response::new()
        .add_attribute("method", "update_data_item")
        .add_attribute("token_id", token_id)
        .add_attribute("new_ipfs_hash", new_ipfs_hash))
}

pub fn execute_freeze_data(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    freeze: bool,
) -> Result<Response, ContractError> {
    let mut data_item = DATA_ITEMS
        .load(deps.storage, &token_id)
        .map_err(|_| ContractError::TokenNotFound {})?;

    // 检查授权
    if !is_approved_or_owner(deps.as_ref(), &info.sender, &token_id)? {
        return Err(ContractError::NotAuthorized {});
    }

    data_item.is_frozen = freeze;
    DATA_ITEMS.save(deps.storage, &token_id, &data_item)?;

    Ok(Response::new()
        .add_attribute("method", "freeze_data")
        .add_attribute("token_id", token_id)
        .add_attribute("frozen", freeze.to_string()))
}

pub fn execute_grant_access(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    grantee: String,
    level: AccessLevel,
) -> Result<Response, ContractError> {
    // 检查授权 - 必须是所有者
    if !is_approved_or_owner(deps.as_ref(), &info.sender, &token_id)? {
        return Err(ContractError::NotAuthorized {});
    }

    let grantee_addr = deps.api.addr_validate(&grantee)?;

    // 设置访问级别
    ACCESS_CONTROLS.save(deps.storage, (&token_id, &grantee), &level)?;

    // 更新授权用户列表
    if !matches!(level, AccessLevel::None) {
        let mut authorized = AUTHORIZED_USERS
            .may_load(deps.storage, &token_id)?
            .unwrap_or_default();

        if !authorized.contains(&grantee_addr) {
            authorized.push(grantee_addr.clone());
            AUTHORIZED_USERS.save(deps.storage, &token_id, &authorized)?;
        }
    } else {
        // 如果设置为 None，从授权列表中移除
        let mut authorized = AUTHORIZED_USERS
            .may_load(deps.storage, &token_id)?
            .unwrap_or_default();
        authorized.retain(|addr| addr != grantee_addr);
        AUTHORIZED_USERS.save(deps.storage, &token_id, &authorized)?;
    }

    Ok(Response::new()
        .add_attribute("method", "grant_access")
        .add_attribute("token_id", token_id)
        .add_attribute("grantee", grantee)
        .add_attribute("level", format!("{:?}", level)))
}

pub fn execute_transfer_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let owner = TOKEN_OWNERS
        .load(deps.storage, &token_id)
        .map_err(|_| ContractError::TokenNotFound {})?;

    // 检查授权
    if !is_approved_or_owner(deps.as_ref(), &info.sender, &token_id)? {
        return Err(ContractError::NotAuthorized {});
    }

    let recipient_addr = deps.api.addr_validate(&recipient)?;

    // 转移所有权
    TOKEN_OWNERS.save(deps.storage, &token_id, &recipient_addr)?;

    // 更新数据项中的所有者
    let mut data_item = DATA_ITEMS.load(deps.storage, &token_id)?;
    data_item.owner = recipient_addr.clone();
    DATA_ITEMS.save(deps.storage, &token_id, &data_item)?;

    // 清除批准
    TOKEN_APPROVALS.remove(deps.storage, &token_id);

    Ok(Response::new()
        .add_attribute("method", "transfer_nft")
        .add_attribute("token_id", token_id)
        .add_attribute("from", owner)
        .add_attribute("to", recipient_addr))
}

pub fn execute_approve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let owner = TOKEN_OWNERS
        .load(deps.storage, &token_id)
        .map_err(|_| ContractError::TokenNotFound {})?;

    // 只有所有者可以批准
    if owner != info.sender {
        return Err(ContractError::NotAuthorized {});
    }

    let spender_addr = deps.api.addr_validate(&spender)?;
    TOKEN_APPROVALS.save(deps.storage, &token_id, &spender_addr)?;

    Ok(Response::new()
        .add_attribute("method", "approve")
        .add_attribute("token_id", token_id)
        .add_attribute("spender", spender_addr))
}

pub fn execute_approve_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATOR_APPROVALS.save(deps.storage, (info.sender.as_str(), &operator), &true)?;

    Ok(Response::new()
        .add_attribute("method", "approve_all")
        .add_attribute("owner", info.sender)
        .add_attribute("operator", operator_addr))
}

pub fn execute_revoke_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATOR_APPROVALS.remove(deps.storage, (info.sender.as_str(), &operator));

    Ok(Response::new()
        .add_attribute("method", "revoke_all")
        .add_attribute("owner", info.sender)
        .add_attribute("operator", operator_addr))
}

pub fn execute_create_paper_item(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ipfs_hash: String,
    doi: String,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    // 获取下一个 token ID
    let token_id = TOKEN_ID_COUNTER.load(deps.storage)?;
    let token_id_str = token_id.to_string();

    // 检查 token 是否已存在
    if TOKEN_OWNERS.has(deps.storage, &token_id_str) {
        return Err(ContractError::TokenExists {});
    }

    // 创建 NFT token
    TOKEN_OWNERS.save(deps.storage, &token_id_str, &info.sender)?;

    // 创建数据项 (论文默认公开，价格为 0)
    let data_item = DataItem {
        owner: info.sender.clone(),
        ipfs_hash: ipfs_hash.clone(),
        price: Uint128::zero(),
        is_public: true, // 论文默认公开
        total_earned: Uint128::zero(),
        created_at: env.block.time.seconds(),
        last_updated: env.block.time.seconds(),
        metadata_uri: metadata_uri.clone(),
        is_frozen: false,
    };
    DATA_ITEMS.save(deps.storage, &token_id_str, &data_item)?;

    // 创建初始版本
    let version = DataVersion {
        ipfs_hash: ipfs_hash.clone(),
        timestamp: env.block.time.seconds(),
    };
    DATA_VERSIONS.save(deps.storage, &token_id_str, &vec![version])?;

    // 保存 DOI
    PAPER_DOIS.save(deps.storage, &token_id_str, &doi)?;

    // 更新计数器
    TOKEN_ID_COUNTER.save(deps.storage, &(token_id + 1))?;
    let count = TOKEN_COUNT.load(deps.storage)?;
    TOKEN_COUNT.save(deps.storage, &(count + 1))?;

    Ok(Response::new()
        .add_attribute("method", "create_paper_item")
        .add_attribute("token_id", token_id_str)
        .add_attribute("owner", info.sender)
        .add_attribute("ipfs_hash", ipfs_hash)
        .add_attribute("paper_doi", doi)
        .add_attribute("paper_type", "academic_paper"))
}

// 辅助函数
fn is_approved_or_owner(deps: Deps, spender: &Addr, token_id: &str) -> StdResult<bool> {
    let owner = TOKEN_OWNERS.load(deps.storage, token_id)?;

    if owner == *spender {
        return Ok(true);
    }

    // 检查单个 token 批准
    if let Ok(approved) = TOKEN_APPROVALS.load(deps.storage, token_id) {
        if approved == *spender {
            return Ok(true);
        }
    }

    // 检查操作员批准
    if let Ok(is_operator) =
        OPERATOR_APPROVALS.load(deps.storage, (owner.as_str(), spender.as_str()))
    {
        if is_operator {
            return Ok(true);
        }
    }

    Ok(false)
}

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
pub fn query_dao_members(deps: Deps) -> StdResult<crate::msg::DaoMembersResponse> {
    let members: StdResult<Vec<Addr>> = DAO_MEMBERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((addr_str, is_member)) => {
                if is_member {
                    match deps.api.addr_validate(&addr_str) {
                        Ok(addr) => Some(Ok(addr)),
                        Err(e) => Some(Err(e)),
                    }
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

// DAO 成员管理功能实现

// /// 检查地址是否为 DAO 成员
// fn is_dao_member(deps: Deps, address: &Addr) -> StdResult<bool> {
//     DAO_MEMBERS
//         .may_load(deps.storage, address.as_str())
//         .map(|member| member.unwrap_or(false))
// }

/// 提交文章发布提案
pub fn execute_submit_article_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ipfs_hash: String,
    doi: String,
    metadata_uri: String,
    title: String,
    description: String,
) -> Result<Response, ContractError> {
    // 验证文章信息的完整性
    if ipfs_hash.trim().is_empty() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "IPFS hash cannot be empty",
        )));
    }

    if doi.trim().is_empty() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "DOI cannot be empty",
        )));
    }

    if metadata_uri.trim().is_empty() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Metadata URI cannot be empty",
        )));
    }

    if title.trim().is_empty() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Title cannot be empty",
        )));
    }

    // 验证 IPFS 哈希格式（基本检查）
    if !ipfs_hash.starts_with("Qm") && !ipfs_hash.starts_with("bafy") {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Invalid IPFS hash format",
        )));
    }

    // 验证 DOI 格式（基本检查）
    if !doi.contains('/') {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Invalid DOI format",
        )));
    }

    // 获取 DAO 配置
    let dao_config = DAO_CONFIG.load(deps.storage)?;

    // 获取下一个提案 ID
    let proposal_id = PROPOSAL_COUNTER.load(deps.storage)?;

    // 设置提案的投票截止时间
    let voting_end = env.block.time.seconds() + dao_config.voting_period;

    // 创建文章发布提案
    let proposal = Proposal {
        id: proposal_id,
        proposer: info.sender.clone(),
        proposal_type: ProposalType::ArticlePublication,
        title,
        description,
        created_at: env.block.time.seconds(),
        voting_end,
        status: ProposalStatus::Active,
        execution_data: Some(ExecutionData::ArticlePublication {
            ipfs_hash: ipfs_hash.clone(),
            doi: doi.clone(),
            metadata_uri: metadata_uri.clone(),
        }),
    };

    // 保存提案
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    // 初始化投票统计
    let total_members = count_dao_members(deps.as_ref())?;
    let vote_count = VoteCount {
        yes: 0,
        no: 0,
        abstain: 0,
        total_eligible: total_members,
    };
    VOTE_COUNTS.save(deps.storage, proposal_id, &vote_count)?;

    // 更新提案计数器
    PROPOSAL_COUNTER.save(deps.storage, &(proposal_id + 1))?;

    // 返回唯一的提案 ID
    Ok(Response::new()
        .add_attribute("method", "submit_article_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("proposer", info.sender.to_string())
        .add_attribute("article_ipfs_hash", ipfs_hash)
        .add_attribute("article_doi", doi)
        .add_attribute("voting_end", voting_end.to_string())
        .add_attribute("proposal_type", "article_publication"))
}

/// 提交成员管理提案
pub fn execute_submit_member_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    member_address: String,
    action: MemberAction,
    title: String,
    description: String,
) -> Result<Response, ContractError> {
    // 验证提案者是 DAO 成员
    ensure_dao_member(deps.as_ref(), &info.sender)?;

    // 验证目标地址
    let target_addr = deps.api.addr_validate(&member_address)?;

    // 根据操作类型进行验证
    match action {
        MemberAction::Add => {
            // 检查成员是否已存在
            if is_dao_member(deps.as_ref(), &target_addr)? {
                return Err(ContractError::MemberAlreadyExists {});
            }
        }
        MemberAction::Remove => {
            // 检查成员是否存在
            if !is_dao_member(deps.as_ref(), &target_addr)? {
                return Err(ContractError::MemberDoesNotExist {});
            }

            // 检查是否为最后一个成员
            let member_count = count_dao_members(deps.as_ref())?;
            if member_count <= 1 {
                return Err(ContractError::CannotRemoveLastMember {});
            }
        }
    }

    // 获取 DAO 配置
    let dao_config = DAO_CONFIG.load(deps.storage)?;

    // 获取下一个提案 ID
    let proposal_id = PROPOSAL_COUNTER.load(deps.storage)?;

    // 创建提案
    let proposal = Proposal {
        id: proposal_id,
        proposer: info.sender.clone(),
        proposal_type: match action {
            MemberAction::Add => ProposalType::AddMember,
            MemberAction::Remove => ProposalType::RemoveMember,
        },
        title,
        description,
        created_at: env.block.time.seconds(),
        voting_end: env.block.time.seconds() + dao_config.voting_period,
        status: ProposalStatus::Active,
        execution_data: Some(ExecutionData::MemberChange {
            member_address: target_addr.to_string(),
            action: action.clone(),
        }),
    };

    // 保存提案
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    // 初始化投票统计
    let total_members = count_dao_members(deps.as_ref())?;
    let vote_count = VoteCount {
        yes: 0,
        no: 0,
        abstain: 0,
        total_eligible: total_members,
    };
    VOTE_COUNTS.save(deps.storage, proposal_id, &vote_count)?;

    // 更新提案计数器
    PROPOSAL_COUNTER.save(deps.storage, &(proposal_id + 1))?;

    Ok(Response::new()
        .add_attribute("method", "submit_member_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("proposer", info.sender.to_string())
        .add_attribute("target_member", target_addr.to_string())
        .add_attribute("action", format!("{:?}", action))
        .add_attribute("voting_end", proposal.voting_end.to_string()))
}

/// 执行提案
pub fn execute_proposal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    // 验证执行者是 DAO 成员
    ensure_dao_member(deps.as_ref(), &info.sender)?;

    // 加载提案并验证可以执行
    let mut proposal = ensure_proposal_exists(deps.as_ref(), proposal_id)?;
    ensure_can_execute_proposal(&env, &proposal)?;

    let mut response = Response::new()
        .add_attribute("method", "execute_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("executor", info.sender.to_string());

    // 根据提案类型执行相应逻辑
    if let Some(execution_data) = &proposal.execution_data {
        match execution_data {
            ExecutionData::MemberChange {
                member_address,
                action,
            } => {
                let target_addr = deps.api.addr_validate(member_address)?;

                match action {
                    MemberAction::Add => {
                        // 添加成员
                        DAO_MEMBERS.save(deps.storage, target_addr.as_str(), &true)?;
                        response = response
                            .add_attribute("action", "member_added")
                            .add_attribute("new_member", target_addr.to_string());
                    }
                    MemberAction::Remove => {
                        // 再次检查是否为最后一个成员（防止竞态条件）
                        let member_count = count_dao_members(deps.as_ref())?;
                        if member_count <= 1 {
                            return Err(ContractError::CannotRemoveLastMember {});
                        }

                        // 处理被移除成员的历史投票记录
                        // 注意：我们保持历史投票记录有效，但需要更新活跃提案的投票统计
                        update_vote_counts_for_removed_member(deps.storage, &target_addr)?;

                        // 移除成员
                        DAO_MEMBERS.remove(deps.storage, target_addr.as_str());
                        response = response
                            .add_attribute("action", "member_removed")
                            .add_attribute("removed_member", target_addr.to_string());
                    }
                }
            }
            ExecutionData::ArticlePublication {
                ipfs_hash,
                doi,
                metadata_uri,
            } => {
                // 执行文章发布提案的自动执行逻辑
                match execute_article_publication_proposal(
                    deps.branch(),
                    env.clone(),
                    &proposal,
                    ipfs_hash.clone(),
                    doi.clone(),
                    metadata_uri.clone(),
                ) {
                    Ok(article_response) => {
                        // 处理执行成功的状态更新
                        response = response
                            .add_attribute("action", "article_published")
                            .add_attribute("article_ipfs_hash", ipfs_hash)
                            .add_attribute("article_doi", doi)
                            .add_attribute("original_proposer", proposal.proposer.to_string())
                            .add_attribute("execution_status", "success");

                        // 合并来自文章创建的属性和消息
                        response = response
                            .add_attributes(article_response.attributes)
                            .add_submessages(article_response.messages);
                    }
                    Err(e) => {
                        // 处理执行失败的状态更新
                        proposal.status = ProposalStatus::Rejected;
                        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

                        let _response = response
                            .add_attribute("action", "article_publication_failed")
                            .add_attribute("execution_status", "failed")
                            .add_attribute("failure_reason", e.to_string());

                        return Err(e);
                    }
                }
            }
            ExecutionData::ConfigUpdate { new_config } => {
                // 验证新配置的有效性
                if new_config.approval_threshold == 0 || new_config.approval_threshold > 100 {
                    return Err(ContractError::InvalidVotingThreshold {});
                }

                if new_config.min_members == 0 {
                    return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                        "Minimum members must be at least 1",
                    )));
                }

                // 验证新的投票期限
                validate_voting_period(new_config.voting_period)?;

                // 检查当前 DAO 成员数量是否满足新的最小成员要求
                let current_member_count = count_dao_members(deps.as_ref())?;
                if current_member_count < new_config.min_members {
                    return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                        format!(
                            "Current member count ({}) is less than required minimum ({})",
                            current_member_count, new_config.min_members
                        ),
                    )));
                }

                // 保存旧配置用于日志记录
                let old_config = DAO_CONFIG.load(deps.storage)?;

                // 更新 DAO 配置
                DAO_CONFIG.save(deps.storage, new_config)?;

                response = response
                    .add_attribute("action", "config_updated")
                    .add_attribute("old_voting_period", old_config.voting_period.to_string())
                    .add_attribute("new_voting_period", new_config.voting_period.to_string())
                    .add_attribute(
                        "old_approval_threshold",
                        old_config.approval_threshold.to_string(),
                    )
                    .add_attribute(
                        "new_approval_threshold",
                        new_config.approval_threshold.to_string(),
                    )
                    .add_attribute("old_min_members", old_config.min_members.to_string())
                    .add_attribute("new_min_members", new_config.min_members.to_string())
                    .add_attribute("execution_status", "success");
            }
        }
    }

    // 更新提案状态为已执行
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(response.add_attribute("status", "executed"))
}

/// 计算 DAO 成员数量
fn count_dao_members(deps: Deps) -> StdResult<u64> {
    let members: StdResult<Vec<_>> = DAO_MEMBERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();

    match members {
        Ok(member_list) => Ok(member_list.len() as u64),
        Err(e) => Err(e),
    }
}
/// 检查提案是否通过并更新状态
/// 处理提案状态的自动更新（通过/拒绝）
/// 这个函数还会在提案通过时自动触发执行（对于文章发布提案）
pub fn check_and_update_proposal_status(
    mut deps: DepsMut,
    env: Env,
    proposal_id: u64,
) -> Result<ProposalStatus, ContractError> {
    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound {})?;

    // 如果提案已经不是活跃状态，直接返回当前状态
    if proposal.status != ProposalStatus::Active {
        return Ok(proposal.status);
    }

    // 检查是否过期
    if env.block.time.seconds() > proposal.voting_end {
        proposal.status = ProposalStatus::Expired;
        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;
        return Ok(ProposalStatus::Expired);
    }

    // 使用新的辅助函数检查通过阈值
    if check_approval_threshold(deps.as_ref(), proposal_id)? {
        proposal.status = ProposalStatus::Passed;
        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

        // 对于文章发布提案，自动执行
        if proposal.proposal_type == ProposalType::ArticlePublication {
            match try_auto_execute_article_proposal(deps.branch(), env.clone(), &proposal) {
                Ok(_) => {
                    // 自动执行成功，更新状态为已执行
                    let mut updated_proposal = PROPOSALS.load(deps.storage, proposal_id)?;
                    updated_proposal.status = ProposalStatus::Executed;
                    PROPOSALS.save(deps.storage, proposal_id, &updated_proposal)?;
                    return Ok(ProposalStatus::Executed);
                }
                Err(_) => {
                    // 自动执行失败，保持通过状态，可以手动执行
                    // 这里不返回错误，因为提案确实通过了，只是自动执行失败
                }
            }
        }

        return Ok(ProposalStatus::Passed);
    }

    // 使用新的辅助函数检查是否不可能通过
    if check_impossible_to_pass(deps.as_ref(), proposal_id)? {
        proposal.status = ProposalStatus::Rejected;
        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;
        return Ok(ProposalStatus::Rejected);
    }

    // 提案仍在活跃状态
    Ok(ProposalStatus::Active)
}

/// 尝试自动执行文章发布提案
/// 这个函数在提案通过时被调用，尝试自动执行文章发布
fn try_auto_execute_article_proposal(
    deps: DepsMut,
    env: Env,
    proposal: &Proposal,
) -> Result<(), ContractError> {
    if let Some(ExecutionData::ArticlePublication {
        ipfs_hash,
        doi,
        metadata_uri,
    }) = &proposal.execution_data
    {
        // 尝试执行文章发布
        execute_article_publication_proposal(
            deps,
            env,
            proposal,
            ipfs_hash.clone(),
            doi.clone(),
            metadata_uri.clone(),
        )?;

        Ok(())
    } else {
        Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Invalid execution data for article publication proposal",
        )))
    }
}
/// 执行文章发布提案的自动执行逻辑
/// 这个函数专门处理通过的文章发布提案的执行，集成现有的 create_paper_item 功能
pub fn execute_article_publication_proposal(
    deps: DepsMut,
    env: Env,
    proposal: &Proposal,
    ipfs_hash: String,
    doi: String,
    metadata_uri: String,
) -> Result<Response, ContractError> {
    // 验证文章信息的完整性（再次验证以确保数据一致性）
    if ipfs_hash.trim().is_empty() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "IPFS hash cannot be empty during execution",
        )));
    }

    if doi.trim().is_empty() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "DOI cannot be empty during execution",
        )));
    }

    if metadata_uri.trim().is_empty() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Metadata URI cannot be empty during execution",
        )));
    }

    // 检查 DOI 是否已经存在（防止重复发布）
    let existing_papers: StdResult<Vec<_>> = PAPER_DOIS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();

    if let Ok(papers) = existing_papers {
        for (_, existing_doi) in papers {
            if existing_doi == doi {
                return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                    format!("DOI {} already exists", doi),
                )));
            }
        }
    }

    // 集成现有的 create_paper_item 功能
    // 使用提案者作为文章的所有者
    let paper_creation_info = MessageInfo {
        sender: proposal.proposer.clone(),
        funds: vec![], // 文章发布不需要资金
    };

    // 调用现有的 create_paper_item 函数
    match execute_create_paper_item(
        deps,
        env,
        paper_creation_info,
        ipfs_hash.clone(),
        doi.clone(),
        metadata_uri.clone(),
    ) {
        Ok(mut paper_response) => {
            // 添加 DAO 执行相关的属性
            paper_response = paper_response
                .add_attribute("dao_approved", "true")
                .add_attribute("proposal_id", proposal.id.to_string())
                .add_attribute("execution_method", "dao_proposal");

            Ok(paper_response)
        }
        Err(e) => {
            // 记录执行失败的详细信息
            Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                format!("Failed to create paper item: {}", e),
            )))
        }
    }
}

/// 实现投票功能
/// 编写 execute_vote_on_proposal 函数
/// 验证投票者的 DAO 成员身份
/// 处理重复投票的更新逻辑
/// 实时更新投票统计
pub fn execute_vote_on_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    choice: VoteChoice,
) -> Result<Response, ContractError> {
    // 验证投票者的 DAO 成员身份
    ensure_dao_member(deps.as_ref(), &info.sender)?;

    // 检查提案是否存在并验证可以投票
    let proposal = ensure_proposal_exists(deps.as_ref(), proposal_id)?;
    ensure_can_vote_on_proposal(&env, &proposal)?;

    // 创建投票记录
    let vote = Vote {
        voter: info.sender.clone(),
        choice: choice.clone(),
        timestamp: env.block.time.seconds(),
    };

    // 检查是否已经投过票（处理重复投票的更新逻辑）
    let vote_key = (proposal_id, info.sender.as_str());
    let previous_vote = VOTES.may_load(deps.storage, vote_key)?;

    // 保存新的投票记录
    VOTES.save(deps.storage, vote_key, &vote)?;

    // 实时更新投票统计
    let mut vote_count = VOTE_COUNTS.load(deps.storage, proposal_id)?;

    // 如果是重复投票，先减去之前的投票
    if let Some(ref prev_vote) = previous_vote {
        match prev_vote.choice {
            VoteChoice::Yes => vote_count.yes -= 1,
            VoteChoice::No => vote_count.no -= 1,
            VoteChoice::Abstain => vote_count.abstain -= 1,
        }
    }

    // 添加新的投票
    match choice {
        VoteChoice::Yes => vote_count.yes += 1,
        VoteChoice::No => vote_count.no += 1,
        VoteChoice::Abstain => vote_count.abstain += 1,
    }

    // 保存更新后的投票统计
    VOTE_COUNTS.save(deps.storage, proposal_id, &vote_count)?;

    // 检查并更新提案状态（如果达到通过阈值或不可能通过）
    let updated_status = check_and_update_proposal_status(deps, env, proposal_id)?;

    let mut response = Response::new()
        .add_attribute("method", "vote_on_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("voter", info.sender.to_string())
        .add_attribute("choice", format!("{:?}", choice))
        .add_attribute("proposal_status", format!("{:?}", updated_status));

    // 如果是重复投票，添加相应属性
    if previous_vote.is_some() {
        response = response.add_attribute("vote_updated", "true");
    } else {
        response = response.add_attribute("vote_updated", "false");
    }

    // 添加当前投票统计信息
    response = response
        .add_attribute("yes_votes", vote_count.yes.to_string())
        .add_attribute("no_votes", vote_count.no.to_string())
        .add_attribute("abstain_votes", vote_count.abstain.to_string())
        .add_attribute("total_eligible", vote_count.total_eligible.to_string());

    Ok(response)
}

/// 计算投票统计
/// 编写投票统计计算函数
pub fn calculate_vote_statistics(deps: Deps, proposal_id: u64) -> Result<VoteCount, ContractError> {
    // 直接从存储中加载投票统计
    let vote_count = VOTE_COUNTS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound {})?;

    Ok(vote_count)
}

/// 实现半数以上通过阈值的检查逻辑
/// 检查提案是否达到通过阈值
pub fn check_approval_threshold(deps: Deps, proposal_id: u64) -> Result<bool, ContractError> {
    let vote_count = VOTE_COUNTS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound {})?;

    let dao_config = DAO_CONFIG.load(deps.storage)?;

    // 计算通过阈值（向上取整）
    let required_yes_votes = (vote_count.total_eligible * dao_config.approval_threshold).div_ceil(100);

    // 检查是否达到通过阈值
    Ok(vote_count.yes >= required_yes_votes)
}

/// 检查提案是否不可能通过
/// 即使剩余所有成员都投赞成票也无法达到阈值
pub fn check_impossible_to_pass(deps: Deps, proposal_id: u64) -> Result<bool, ContractError> {
    let vote_count = VOTE_COUNTS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound {})?;

    let dao_config = DAO_CONFIG.load(deps.storage)?;

    // 计算通过阈值
    let required_yes_votes = (vote_count.total_eligible * dao_config.approval_threshold).div_ceil(100);

    // 计算已投票数
    let total_voted = vote_count.yes + vote_count.no + vote_count.abstain;
    let remaining_votes = vote_count.total_eligible - total_voted;

    // 计算最大可能的赞成票数
    let max_possible_yes = vote_count.yes + remaining_votes;

    // 如果最大可能的赞成票数仍然小于所需阈值，则不可能通过
    Ok(max_possible_yes < required_yes_votes)
}

/// 获取提案的详细投票统计信息
pub fn get_detailed_vote_statistics(
    deps: Deps,
    proposal_id: u64,
) -> Result<(VoteCount, bool, bool, u64), ContractError> {
    let vote_count = calculate_vote_statistics(deps, proposal_id)?;
    let passed = check_approval_threshold(deps, proposal_id)?;
    let impossible = check_impossible_to_pass(deps, proposal_id)?;

    let dao_config = DAO_CONFIG.load(deps.storage)?;
    let required_yes_votes = (vote_count.total_eligible * dao_config.approval_threshold).div_ceil(100);

    Ok((vote_count, passed, impossible, required_yes_votes))
}

/// 更新活跃提案的投票统计，当成员被移除时
fn update_vote_counts_for_removed_member(
    storage: &mut dyn cosmwasm_std::Storage,
    _removed_member: &Addr,
) -> Result<(), ContractError> {
    // 获取所有活跃提案并更新其投票统计中的 total_eligible 数量
    let proposals: StdResult<Vec<_>> = PROPOSALS
        .range(storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();

    match proposals {
        Ok(proposal_list) => {
            for (proposal_id, proposal) in proposal_list {
                // 只处理活跃状态的提案
                if proposal.status == ProposalStatus::Active {
                    if let Ok(mut vote_count) = VOTE_COUNTS.load(storage, proposal_id) {
                        // 减少总有效投票者数量
                        if vote_count.total_eligible > 0 {
                            vote_count.total_eligible -= 1;
                            VOTE_COUNTS.save(storage, proposal_id, &vote_count)?;
                        }
                    }
                }
            }
            Ok(())
        }
        Err(e) => Err(ContractError::Std(e)),
    }
}

/// 更新 DAO 配置
/// 只有 DAO 成员可以提交配置更新提案
pub fn execute_update_dao_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    voting_period: Option<u64>,
    approval_threshold: Option<u64>,
    min_members: Option<u64>,
) -> Result<Response, ContractError> {
    // 验证提案者是 DAO 成员
    ensure_dao_member(deps.as_ref(), &info.sender)?;

    // 验证配置参数的有效性
    validate_dao_config(voting_period, approval_threshold, min_members)?;

    // 获取当前配置
    let current_config = DAO_CONFIG.load(deps.storage)?;

    // 创建新配置，使用提供的值或保持当前值
    let new_config = DaoConfig {
        voting_period: voting_period.unwrap_or(current_config.voting_period),
        approval_threshold: approval_threshold.unwrap_or(current_config.approval_threshold),
        min_members: min_members.unwrap_or(current_config.min_members),
    };

    // 验证新的投票期限
    validate_voting_period(new_config.voting_period)?;

    // 检查当前 DAO 成员数量是否满足新的最小成员要求
    let current_member_count = count_dao_members(deps.as_ref())?;
    if current_member_count < new_config.min_members {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            format!(
                "Current member count ({}) is less than required minimum ({})",
                current_member_count, new_config.min_members
            ),
        )));
    }

    // 获取下一个提案 ID
    let proposal_id = PROPOSAL_COUNTER.load(deps.storage)?;

    // 设置提案的投票截止时间
    let voting_end = env.block.time.seconds() + current_config.voting_period;

    // 创建配置更新提案
    let proposal = Proposal {
        id: proposal_id,
        proposer: info.sender.clone(),
        proposal_type: ProposalType::UpdateConfig,
        title: "DAO Configuration Update".to_string(),
        description: format!(
            "Update DAO configuration - Voting Period: {} -> {}, Approval Threshold: {}% -> {}%, Min Members: {} -> {}",
            current_config.voting_period,
            new_config.voting_period,
            current_config.approval_threshold,
            new_config.approval_threshold,
            current_config.min_members,
            new_config.min_members
        ),
        created_at: env.block.time.seconds(),
        voting_end,
        status: ProposalStatus::Active,
        execution_data: Some(ExecutionData::ConfigUpdate {
            new_config: new_config.clone(),
        }),
    };

    // 保存提案
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    // 初始化投票统计
    let member_count = count_dao_members(deps.as_ref())?;
    let vote_count = VoteCount {
        yes: 0,
        no: 0,
        abstain: 0,
        total_eligible: member_count,
    };
    VOTE_COUNTS.save(deps.storage, proposal_id, &vote_count)?;

    // 更新提案计数器
    PROPOSAL_COUNTER.save(deps.storage, &(proposal_id + 1))?;

    Ok(Response::new()
        .add_attribute("method", "update_dao_config")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("proposer", info.sender.to_string())
        .add_attribute(
            "old_voting_period",
            current_config.voting_period.to_string(),
        )
        .add_attribute("new_voting_period", new_config.voting_period.to_string())
        .add_attribute(
            "old_approval_threshold",
            current_config.approval_threshold.to_string(),
        )
        .add_attribute(
            "new_approval_threshold",
            new_config.approval_threshold.to_string(),
        )
        .add_attribute("old_min_members", current_config.min_members.to_string())
        .add_attribute("new_min_members", new_config.min_members.to_string())
        .add_attribute("voting_end", voting_end.to_string())
        .add_attribute("proposal_type", "config_update"))
}
