use crate::error::ContractError;
use crate::msg::{Proposal, ProposalStatus};
use crate::state::{DAO_MEMBERS, PROPOSALS};
use cosmwasm_std::{Addr, Deps, Env, StdResult};

/// 检查地址是否为 DAO 成员
/// 这是一个辅助函数，用于在所有需要 DAO 成员身份验证的地方进行检查
pub fn is_dao_member(deps: Deps, address: &Addr) -> StdResult<bool> {
    DAO_MEMBERS
        .may_load(deps.storage, address.as_str())
        .map(|member| member.unwrap_or(false))
}

/// 验证调用者是否为 DAO 成员，如果不是则返回错误
/// 这个函数简化了在执行函数中进行成员身份检查的过程
pub fn ensure_dao_member(deps: Deps, address: &Addr) -> Result<(), ContractError> {
    if !is_dao_member(deps, address)? {
        return Err(ContractError::NotDaoMember {});
    }
    Ok(())
}

/// 检查提案是否存在
pub fn ensure_proposal_exists(deps: Deps, proposal_id: u64) -> Result<Proposal, ContractError> {
    PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound {})
}

/// 检查提案是否已过期
pub fn is_proposal_expired(env: &Env, proposal: &Proposal) -> bool {
    env.block.time.seconds() > proposal.voting_end
}

/// 检查提案是否处于活跃状态（可以投票）
pub fn is_proposal_active(proposal: &Proposal) -> bool {
    proposal.status == ProposalStatus::Active
}

/// 检查提案是否可以投票（活跃且未过期）
pub fn can_vote_on_proposal(env: &Env, proposal: &Proposal) -> bool {
    is_proposal_active(proposal) && !is_proposal_expired(env, proposal)
}

/// 验证提案可以投票，如果不能则返回相应错误
pub fn ensure_can_vote_on_proposal(env: &Env, proposal: &Proposal) -> Result<(), ContractError> {
    if !is_proposal_active(proposal) {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Proposal is not active",
        )));
    }

    if is_proposal_expired(env, proposal) {
        return Err(ContractError::ProposalExpired {});
    }

    Ok(())
}

pub fn update_proposal_status(proposal: &mut Proposal, env: &Env) {
    if proposal.status == ProposalStatus::Active && env.block.time.seconds() > proposal.voting_end {
        proposal.status = ProposalStatus::Expired;
    }
}

/// 检查提案是否已执行
pub fn is_proposal_executed(proposal: &Proposal) -> bool {
    proposal.status == ProposalStatus::Executed
}

/// 检查提案是否已通过
pub fn is_proposal_passed(proposal: &Proposal) -> bool {
    proposal.status == ProposalStatus::Passed
}

/// 验证提案可以执行，如果不能则返回相应错误
pub fn ensure_can_execute_proposal(env: &Env, proposal: &Proposal) -> Result<(), ContractError> {
    if !is_proposal_passed(proposal) {
        return Err(ContractError::ProposalDidNotPass {});
    }

    if is_proposal_executed(proposal) {
        return Err(ContractError::ProposalAlreadyExecuted {});
    }

    if is_proposal_expired(env, proposal) {
        return Err(ContractError::ProposalExpired {});
    }

    Ok(())
}

/// 验证提案状态转换是否有效
pub fn validate_proposal_status_transition(
    current_status: &ProposalStatus,
    new_status: &ProposalStatus,
) -> Result<(), ContractError> {
    match (current_status, new_status) {
        // 从 Active 可以转换到任何其他状态
        (ProposalStatus::Active, _) => Ok(()),

        // 从 Passed 只能转换到 Executed
        (ProposalStatus::Passed, ProposalStatus::Executed) => Ok(()),

        // 其他转换都是无效的
        _ => Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Invalid proposal status transition",
        ))),
    }
}

/// 获取当前 DAO 成员总数
pub fn get_dao_member_count(deps: Deps) -> StdResult<u64> {
    let members: StdResult<Vec<_>> = DAO_MEMBERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, is_member)) => {
                if is_member {
                    Some(Ok(()))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .collect();

    members.map(|m| m.len() as u64)
}

/// 验证 DAO 配置参数的有效性
pub fn validate_dao_config(
    voting_period: Option<u64>,
    approval_threshold: Option<u64>,
    min_members: Option<u64>,
) -> Result<(), ContractError> {
    if let Some(threshold) = approval_threshold {
        if threshold == 0 || threshold > 100 {
            return Err(ContractError::InvalidVotingThreshold {});
        }
    }

    if let Some(period) = voting_period {
        if period == 0 {
            return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                "Voting period must be greater than 0",
            )));
        }
    }

    if let Some(min) = min_members {
        if min == 0 {
            return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                "Minimum members must be greater than 0",
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ProposalType;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{Addr, Timestamp};

    #[test]
    fn test_is_dao_member() {
        let mut deps = mock_dependencies();
        let addr = Addr::unchecked("member1");

        // 测试不存在的成员
        assert_eq!(is_dao_member(deps.as_ref(), &addr).unwrap(), false);

        // 添加成员
        DAO_MEMBERS
            .save(deps.as_mut().storage, addr.as_str(), &true)
            .unwrap();
        assert_eq!(is_dao_member(deps.as_ref(), &addr).unwrap(), true);

        // 移除成员
        DAO_MEMBERS
            .save(deps.as_mut().storage, addr.as_str(), &false)
            .unwrap();
        assert_eq!(is_dao_member(deps.as_ref(), &addr).unwrap(), false);
    }

    #[test]
    fn test_ensure_dao_member() {
        let mut deps = mock_dependencies();
        let addr = Addr::unchecked("member1");

        // 测试非成员
        assert!(ensure_dao_member(deps.as_ref(), &addr).is_err());

        // 添加成员
        DAO_MEMBERS
            .save(deps.as_mut().storage, addr.as_str(), &true)
            .unwrap();
        assert!(ensure_dao_member(deps.as_ref(), &addr).is_ok());
    }

    #[test]
    fn test_proposal_time_validation() {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1000);

        let active_proposal = Proposal {
            id: 1,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 900,
            voting_end: 1100, // 未过期
            status: ProposalStatus::Active,
            execution_data: None,
        };

        let expired_proposal = Proposal {
            id: 2,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 800,
            voting_end: 900, // 已过期
            status: ProposalStatus::Active,
            execution_data: None,
        };

        // 测试活跃提案
        assert!(can_vote_on_proposal(&env, &active_proposal));
        assert!(ensure_can_vote_on_proposal(&env, &active_proposal).is_ok());

        // 测试过期提案
        assert!(!can_vote_on_proposal(&env, &expired_proposal));
        assert!(ensure_can_vote_on_proposal(&env, &expired_proposal).is_err());
    }

    #[test]
    fn test_proposal_status_validation() {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1000);

        let passed_proposal = Proposal {
            id: 1,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 900,
            voting_end: 1100,
            status: ProposalStatus::Passed,
            execution_data: None,
        };

        let executed_proposal = Proposal {
            id: 2,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 900,
            voting_end: 1100,
            status: ProposalStatus::Executed,
            execution_data: None,
        };

        // 测试可执行的提案
        assert!(ensure_can_execute_proposal(&env, &passed_proposal).is_ok());

        // 测试已执行的提案
        assert!(ensure_can_execute_proposal(&env, &executed_proposal).is_err());
    }

    #[test]
    fn test_validate_dao_config() {
        // 测试有效配置
        assert!(validate_dao_config(Some(86400), Some(51), Some(1)).is_ok());

        // 测试无效阈值
        assert!(validate_dao_config(None, Some(0), None).is_err());
        assert!(validate_dao_config(None, Some(101), None).is_err());

        // 测试无效投票期限
        assert!(validate_dao_config(Some(0), None, None).is_err());

        // 测试无效最小成员数
        assert!(validate_dao_config(None, None, Some(0)).is_err());
    }
}
/// 自动检查并更新过期提案的状态
/// 这个函数会检查提案是否过期，如果过期则更新状态为 Expired
pub fn check_and_expire_proposal(
    _deps: Deps,
    env: &Env,
    proposal: &mut Proposal,
) -> Result<bool, ContractError> {
    if is_proposal_expired(env, proposal) && proposal.status == ProposalStatus::Active {
        proposal.status = ProposalStatus::Expired;
        return Ok(true); // 表示状态已更新
    }
    Ok(false) // 表示状态未更新
}

/// 验证提案的投票期限设置是否合理
pub fn validate_voting_period(voting_period: u64) -> Result<(), ContractError> {
    if voting_period == 0 {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Voting period must be greater than 0",
        )));
    }

    // 设置最小投票期限为1小时（3600秒）
    if voting_period < 3600 {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Voting period must be at least 1 hour",
        )));
    }

    // 设置最大投票期限为30天（2592000秒）
    if voting_period > 2592000 {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Voting period cannot exceed 30 days",
        )));
    }

    Ok(())
}

/// 检查提案是否在有效的投票时间窗口内
pub fn is_within_voting_window(env: &Env, proposal: &Proposal) -> bool {
    let current_time = env.block.time.seconds();
    current_time >= proposal.created_at && current_time <= proposal.voting_end
}

/// 计算提案剩余的投票时间（秒）
pub fn get_remaining_voting_time(env: &Env, proposal: &Proposal) -> i64 {
    let current_time = env.block.time.seconds() as i64;
    let voting_end = proposal.voting_end as i64;
    voting_end - current_time
}

/// 检查提案是否即将过期（在指定时间内）
pub fn is_proposal_expiring_soon(env: &Env, proposal: &Proposal, warning_seconds: u64) -> bool {
    let remaining_time = get_remaining_voting_time(env, proposal);
    remaining_time > 0 && remaining_time <= warning_seconds as i64
}

/// 批量检查多个提案的状态并更新过期的提案
pub fn batch_check_proposal_expiration(
    deps: Deps,
    env: &Env,
    proposal_ids: &[u64],
) -> Result<Vec<(u64, bool)>, ContractError> {
    let mut results = Vec::new();

    for &proposal_id in proposal_ids {
        let proposal = ensure_proposal_exists(deps, proposal_id)?;
        let is_expired = is_proposal_expired(env, &proposal);
        results.push((proposal_id, is_expired));
    }

    Ok(results)
}

/// 验证提案状态转换的时间约束
pub fn validate_status_transition_timing(
    env: &Env,
    proposal: &Proposal,
    new_status: &ProposalStatus,
) -> Result<(), ContractError> {
    match new_status {
        ProposalStatus::Executed => {
            // 只有在投票期限内才能执行
            if is_proposal_expired(env, proposal) {
                return Err(ContractError::ProposalExpired {});
            }
        }
        ProposalStatus::Expired => {
            // 只有在投票期限过后才能标记为过期
            if !is_proposal_expired(env, proposal) {
                return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                    "Cannot mark proposal as expired before voting period ends",
                )));
            }
        }
        _ => {} // 其他状态转换不需要时间约束
    }

    Ok(())
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::msg::ProposalType;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::Timestamp;

    #[test]
    fn test_check_and_expire_proposal() {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1200);

        let mut active_proposal = Proposal {
            id: 1,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 900,
            voting_end: 1100, // 已过期
            status: ProposalStatus::Active,
            execution_data: None,
        };

        let deps = mock_dependencies();
        let result = check_and_expire_proposal(deps.as_ref(), &env, &mut active_proposal).unwrap();

        assert!(result); // 状态应该已更新
        assert_eq!(active_proposal.status, ProposalStatus::Expired);
    }

    #[test]
    fn test_validate_voting_period() {
        // 测试有效期限
        assert!(validate_voting_period(86400).is_ok()); // 1天

        // 测试无效期限
        assert!(validate_voting_period(0).is_err()); // 0秒
        assert!(validate_voting_period(1800).is_err()); // 30分钟，小于最小值
        assert!(validate_voting_period(3000000).is_err()); // 超过30天
    }

    #[test]
    fn test_voting_time_calculations() {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1000);

        let proposal = Proposal {
            id: 1,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 900,
            voting_end: 1200,
            status: ProposalStatus::Active,
            execution_data: None,
        };

        // 测试投票窗口检查
        assert!(is_within_voting_window(&env, &proposal));

        // 测试剩余时间计算
        assert_eq!(get_remaining_voting_time(&env, &proposal), 200);

        // 测试即将过期检查
        assert!(is_proposal_expiring_soon(&env, &proposal, 300)); // 300秒内过期
        assert!(!is_proposal_expiring_soon(&env, &proposal, 100)); // 100秒内不过期
    }

    #[test]
    fn test_status_transition_timing() {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1000);

        let active_proposal = Proposal {
            id: 1,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 900,
            voting_end: 1100, // 未过期
            status: ProposalStatus::Active,
            execution_data: None,
        };

        let expired_proposal = Proposal {
            id: 2,
            proposer: Addr::unchecked("proposer"),
            proposal_type: ProposalType::ArticlePublication,
            title: "Test".to_string(),
            description: "Test".to_string(),
            created_at: 800,
            voting_end: 900, // 已过期
            status: ProposalStatus::Active,
            execution_data: None,
        };

        // 测试在有效期内执行
        assert!(validate_status_transition_timing(
            &env,
            &active_proposal,
            &ProposalStatus::Executed
        )
        .is_ok());

        // 测试过期后执行
        assert!(validate_status_transition_timing(
            &env,
            &expired_proposal,
            &ProposalStatus::Executed
        )
        .is_err());

        // 测试标记为过期
        assert!(validate_status_transition_timing(
            &env,
            &expired_proposal,
            &ProposalStatus::Expired
        )
        .is_ok());
        assert!(validate_status_transition_timing(
            &env,
            &active_proposal,
            &ProposalStatus::Expired
        )
        .is_err());
    }
}
