use crate::error::ContractError;
use crate::msg::{DaoConfig, InstantiateMsg};
use crate::state::{
    BASE_CITATION_FEE, CONTRACT_NAME, CONTRACT_OWNER, CONTRACT_SYMBOL, DAO_CONFIG, DAO_MEMBERS,
    PROPOSAL_COUNTER, TOKEN_COUNT, TOKEN_ID_COUNTER,
};
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, Uint128};

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
        .add_attribute("action", "instantiate")
        .add_attribute("name", &msg.name)
        .add_attribute("symbol", &msg.symbol)
        .add_attribute("owner", owner.as_str())
        .add_attribute("dao_initialized", "true")
        .add_attribute("first_dao_member", owner.as_str())
        .add_attribute("base_citation_fee", "1000000"))
}
