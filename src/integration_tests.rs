#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VoteChoice};
    use crate::state::{DAO_CONFIG, DAO_MEMBERS, PROPOSAL_COUNTER};
    use crate::{contract::*, ContractError};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, Uint128};

    // ===== 基础功能测试 =====

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::ContractInfo {}).unwrap();
        let value: crate::msg::ContractInfoResponse = from_json(&res).unwrap();
        assert_eq!("Research Data NFT", value.name);
        assert_eq!("RDN", value.symbol);
    }

    #[test]
    fn dao_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Verify DAO initialization
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "dao_initialized" && attr.value == "true"));
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "first_dao_member" && attr.value == "creator"));

        // Verify creator is DAO member
        let is_member = DAO_MEMBERS.load(deps.as_ref().storage, "creator").unwrap();
        assert_eq!(true, is_member);

        // Verify DAO config defaults
        let dao_config = DAO_CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(604800, dao_config.voting_period); // 7 days
        assert_eq!(51, dao_config.approval_threshold); // 51%
        assert_eq!(1, dao_config.min_members);

        let proposal_counter = PROPOSAL_COUNTER.load(deps.as_ref().storage).unwrap();
        assert_eq!(0, proposal_counter);
    }

    // ===== NFT基础功能测试 =====

    #[test]
    fn create_data_item() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::CreateDataItem {
            ipfs_hash: "QmTest".to_string(),
            price: Uint128::new(1000),
            is_public: false,
            metadata_uri: "https://example.com/metadata.json".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetDataItem {
                token_id: "0".to_string(),
            },
        )
        .unwrap();
        let data_item: crate::msg::DataItem = from_json(&res).unwrap();
        assert_eq!("QmTest", data_item.ipfs_hash);
        assert_eq!(Uint128::new(1000), data_item.price);
        assert_eq!(false, data_item.is_public);
    }

    #[test]
    fn create_paper_item() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("author", &coins(2, "token"));
        let msg = ExecuteMsg::CreatePaperItem {
            ipfs_hash: "QmPaperTest".to_string(),
            doi: "10.1000/test.paper".to_string(),
            metadata_uri: "https://example.com/paper.json".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetPaperDoi {
                paper_id: "0".to_string(),
            },
        )
        .unwrap();
        let doi: String = from_json(&res).unwrap();
        assert_eq!("10.1000/test.paper", doi);
    }

    #[test]
    fn cite_paper() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("author", &coins(2, "token"));
        let msg = ExecuteMsg::CreatePaperItem {
            ipfs_hash: "QmPaperTest".to_string(),
            doi: "10.1000/test.paper".to_string(),
            metadata_uri: "https://example.com/paper.json".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("citer", &coins(100_000, "utoken"));
        let msg = ExecuteMsg::CitePaper {
            paper_id: "0".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len()); // Payment to author and DAO

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCitations {
                paper_id: "0".to_string(),
            },
        )
        .unwrap();
        let citations: Vec<crate::msg::Citation> = from_json(&res).unwrap();
        assert_eq!(1, citations.len());
        assert_eq!(Uint128::new(100_000), citations[0].amount);
    }

    // ===== DAO核心功能测试 =====

    #[test]
    fn test_article_proposal_and_voting() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Submit article proposal
        let info = mock_info("author", &coins(2, "token"));
        let msg = ExecuteMsg::SubmitArticleProposal {
            ipfs_hash: "QmTestArticle123".to_string(),
            doi: "10.1000/test.article.2024".to_string(),
            metadata_uri: "https://example.com/article.json".to_string(),
            title: "Test Article".to_string(),
            description: "A test article for DAO approval".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "method" && attr.value == "submit_article_proposal"));
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "proposal_id" && attr.value == "0"));

        // Vote on proposal
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::VoteOnProposal {
            proposal_id: 0,
            choice: VoteChoice::Yes,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "method" && attr.value == "vote_on_proposal"));
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "voter" && attr.value == "creator"));
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "choice" && attr.value == "Yes"));

        // Since there's only 1 member and they voted Yes, proposal should be executed automatically
        // Verify the paper was created
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetPaperDoi {
                paper_id: "0".to_string(),
            },
        )
        .unwrap();
        let doi: String = from_json(&res).unwrap();
        assert_eq!("10.1000/test.article.2024", doi);
    }

    #[test]
    fn test_dao_config_update() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Submit config update proposal
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateDaoConfig {
            voting_period: Some(1209600), // 14 days
            approval_threshold: Some(60), // 60%
            min_members: Some(1),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "method" && attr.value == "update_dao_config"));
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "proposal_id" && attr.value == "0"));
    }

    #[test]
    fn test_member_management() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Submit member addition proposal
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::SubmitMemberProposal {
            member_address: "new_member".to_string(),
            action: crate::msg::MemberAction::Add,
            title: "Add New Member".to_string(),
            description: "Adding a new member to the DAO".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "method" && attr.value == "submit_member_proposal"));
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "action" && attr.value == "Add"));
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "target_member" && attr.value == "new_member"));
    }

    // ===== 错误处理测试 =====

    #[test]
    fn test_non_dao_member_cannot_vote() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Submit article proposal
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::SubmitArticleProposal {
            ipfs_hash: "QmNonMemberTest123".to_string(),
            doi: "10.1000/non.member.2024".to_string(),
            metadata_uri: "https://example.com/nonmember.json".to_string(),
            title: "Non-member Test".to_string(),
            description: "Testing non-member voting restriction".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to vote as non-DAO member
        let info = mock_info("non_member", &coins(2, "token"));
        let msg = ExecuteMsg::VoteOnProposal {
            proposal_id: 0,
            choice: VoteChoice::Yes,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::NotDaoMember {} => {}
            _ => panic!("Expected NotDaoMember error"),
        }
    }

    #[test]
    fn test_config_validation() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Test invalid approval threshold (0%)
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateDaoConfig {
            voting_period: None,
            approval_threshold: Some(0),
            min_members: None,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        // Test invalid min_members (0)
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateDaoConfig {
            voting_period: None,
            approval_threshold: None,
            min_members: Some(0),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());
    }

    #[test]
    fn test_cannot_remove_last_dao_member() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to remove the only member (creator)
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::SubmitMemberProposal {
            member_address: "creator".to_string(),
            action: crate::msg::MemberAction::Remove,
            title: "Remove Last Member".to_string(),
            description: "Attempting to remove the last member".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::CannotRemoveLastMember {} => {}
            _ => panic!("Expected CannotRemoveLastMember error"),
        }
    }

    // ===== 集成测试 =====

    #[test]
    fn test_complete_workflow() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // 1. Submit article proposal
        let info = mock_info("author", &coins(2, "token"));
        let msg = ExecuteMsg::SubmitArticleProposal {
            ipfs_hash: "QmWorkflowTest123".to_string(),
            doi: "10.1000/workflow.test.2024".to_string(),
            metadata_uri: "https://example.com/workflow.json".to_string(),
            title: "Complete Workflow Test".to_string(),
            description: "Testing complete DAO governance workflow".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "proposal_id" && attr.value == "0"));

        // 2. Vote on proposal
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::VoteOnProposal {
            proposal_id: 0,
            choice: VoteChoice::Yes,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Should be auto-executed since only 1 member voted Yes (100% > 51%)
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "proposal_status"
                && (attr.value == "Passed" || attr.value == "Executed")));

        // 3. Verify NFT was created
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetPaperDoi {
                paper_id: "0".to_string(),
            },
        )
        .unwrap();
        let doi: String = from_json(&res).unwrap();
        assert_eq!("10.1000/workflow.test.2024", doi);

        // 4. Test citation functionality on published article
        let info = mock_info("citer", &coins(100_000, "utoken"));
        let msg = ExecuteMsg::CitePaper {
            paper_id: "0".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len()); // Payment to author and DAO

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCitations {
                paper_id: "0".to_string(),
            },
        )
        .unwrap();
        let citations: Vec<crate::msg::Citation> = from_json(&res).unwrap();
        assert_eq!(1, citations.len());
    }
}
