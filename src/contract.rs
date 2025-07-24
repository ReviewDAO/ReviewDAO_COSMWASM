use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{
    AccessLevel, BaseCitationFeeResponse, Citation, ContractInfoResponse, DataItem, DataVersion,
    ExecuteMsg, InstantiateMsg, NumTokensResponse, OwnerOfResponse, QueryMsg, TokenInfoResponse,
};
use crate::state::{
    ACCESS_CONTROLS, AUTHORIZED_USERS, BASE_CITATION_FEE, CITATIONS, CONTRACT_NAME, CONTRACT_OWNER,
    CONTRACT_SYMBOL, DATA_ITEMS, DATA_VERSIONS, OPERATOR_APPROVALS, PAPER_DOIS, TOKEN_APPROVALS,
    TOKEN_COUNT, TOKEN_ID_COUNTER, TOKEN_OWNERS,
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

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("name", msg.name)
        .add_attribute("symbol", msg.symbol)
        .add_attribute("owner", owner))
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
        authorized.retain(|addr| addr != &grantee_addr);
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
