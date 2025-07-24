#[cfg(test)]
mod tests {
    use crate::contract::*;
    use crate::msg::{AccessLevel, ExecuteMsg, InstantiateMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, Uint128};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::ContractInfo {}).unwrap();
        let value: crate::msg::ContractInfoResponse = from_json(&res).unwrap();
        assert_eq!("Research Data NFT", value.name);
        assert_eq!("RDN", value.symbol);
    }

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

        // Create data item
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::CreateDataItem {
            ipfs_hash: "QmTest".to_string(),
            price: Uint128::new(1000),
            is_public: false,
            metadata_uri: "https://example.com/metadata.json".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Query the data item
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
    fn grant_access() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Research Data NFT".to_string(),
            symbol: "RDN".to_string(),
            owner: "creator".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Create data item
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::CreateDataItem {
            ipfs_hash: "QmTest".to_string(),
            price: Uint128::new(1000),
            is_public: false,
            metadata_uri: "https://example.com/metadata.json".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Grant access
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::GrantAccess {
            token_id: "0".to_string(),
            grantee: "user1".to_string(),
            level: AccessLevel::Read,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Check access level
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CheckAccessLevel {
                token_id: "0".to_string(),
                user: "user1".to_string(),
            },
        )
        .unwrap();
        let access_level: AccessLevel = from_json(&res).unwrap();
        assert_eq!(AccessLevel::Read, access_level);
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

        // Create paper item
        let info = mock_info("author", &coins(2, "token"));
        let msg = ExecuteMsg::CreatePaperItem {
            ipfs_hash: "QmPaperTest".to_string(),
            doi: "10.1000/test.paper".to_string(),
            metadata_uri: "https://example.com/paper.json".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Query the paper DOI
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

        // Create paper item
        let info = mock_info("author", &coins(2, "token"));
        let msg = ExecuteMsg::CreatePaperItem {
            ipfs_hash: "QmPaperTest".to_string(),
            doi: "10.1000/test.paper".to_string(),
            metadata_uri: "https://example.com/paper.json".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Cite paper with payment
        let info = mock_info("citer", &coins(100_000, "utoken"));
        let msg = ExecuteMsg::CitePaper {
            paper_id: "0".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len()); // Payment to author and DAO

        // Query citations
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
}
