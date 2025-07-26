# Research Data NFT Contract - Complete Test Commands

## Environment Variables
```bash
export YOUR_INT_ADDRESS="inj1z8zyrl0uaqc0ngj9dhafvuc9xzsfm737tv0mfx"
export CONTRACT_ADDRESS="inj1myyc49kthpzxhtqykh8j9vg2rwxs30r67876as"
export CHAIN_ID="injective-888"
export NODE="https://testnet.sentry.tm.injective.network:443"
export GAS="2000000"
export FEES="1000000000000000inj"
```

## 1. Contract Deployment

### Upload Wasm
```bash
yes 12345678 | injectived tx wasm store artifacts/bc.wasm \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--yes --fees=$FEES --gas=3000000 \
--node=$NODE
```

### Get Code ID
```bash
injectived query tx [TRANSACTION_HASH] --node=$NODE > tmp
# Extract code_id from the response
```

### Instantiate Contract
```bash
INIT='{"name":"Research Data NFT","symbol":"RDN","owner":"'$YOUR_INT_ADDRESS'"}'
yes password | injectived tx wasm instantiate [CODE_ID] "$INIT" \
--label="ResearchDataNFTInstance" \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--yes --fees=$FEES \
--gas=$GAS \
--no-admin \
--node=$NODE
```

## 2. Paper Management

### Create Paper Item (Direct)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"create_paper_item": {"ipfs_hash": "QmTestPaper123", "doi": "10.1000/test.2024", "metadata_uri": "https://example.com/metadata"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Create Another Paper
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"create_paper_item": {"ipfs_hash": "QmSecondPaper456", "doi": "10.1000/second.2024", "metadata_uri": "https://example.com/metadata2"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Cite Paper (with payment)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"cite_paper": {"paper_id": "0"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--amount=1000000000000000000inj \
--node=$NODE \
--yes
```

### Submit Correction
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"submit_correction": {"original_paper_id": "0", "new_ipfs_hash": "QmCorrectedPaper789"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

## 3. DAO Governance

### Submit Article Proposal
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"submit_article_proposal": {"ipfs_hash": "QmProposalPaper123","doi": "10.1000/proposal.2024","metadata_uri": "https://example.com/proposal.json","title": "New Research Paper","description": "Testing DAO governance for paper publication"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Vote on Proposal (Yes)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"vote_on_proposal": {"proposal_id": 0,"choice": "Yes"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Vote on Proposal (No)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"vote_on_proposal": {"proposal_id": 0,"choice": "No"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Vote on Proposal (Abstain)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"vote_on_proposal": {"proposal_id": 0,"choice": "Abstain"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Execute Proposal
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"execute_proposal": {"proposal_id": 0}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Submit Member Addition Proposal
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"submit_member_proposal": {"member_address": "inj1newmember123456789", "action": "Add", "title": "Add New DAO Member", "description": "Proposal to add a new member to the DAO"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Submit Member Removal Proposal
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"submit_member_proposal": {"member_address": "inj1oldmember123456789", "action": "Remove", "title": "Remove DAO Member", "description": "Proposal to remove a member from the DAO"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Update DAO Configuration
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"update_dao_config": {"voting_period": 1209600, "approval_threshold": 60, "min_members": 3}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

## 4. Data Management

### Update Data Item
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"update_data_item": {"token_id": "0", "new_ipfs_hash": "QmUpdatedHash123", "new_metadata_uri": "https://example.com/updated-metadata"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Freeze Data
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"freeze_data": {"token_id": "0", "freeze": true}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Unfreeze Data
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"freeze_data": {"token_id": "0", "freeze": false}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Grant Access (Read)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"grant_access": {"token_id": "0", "grantee": "inj1grantee123456789", "level": "Read"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Grant Access (Write)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"grant_access": {"token_id": "0", "grantee": "inj1grantee123456789", "level": "Write"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Request Access
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"request_access": {"token_id": "0"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

## 5. NFT Operations

### Transfer NFT
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"transfer_nft": {"recipient": "inj1recipient123456789", "token_id": "0"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Approve Spender
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"approve": {"spender": "inj1spender123456789", "token_id": "0"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Approve All (Operator)
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"approve_all": {"operator": "inj1operator123456789"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

### Revoke All
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"revoke_all": {"operator": "inj1operator123456789"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

## 6. Admin Functions

### Set Base Citation Fee
```bash
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"set_base_citation_fee": {"fee": "200000"}}' \
--from=$YOUR_INT_ADDRESS \
--chain-id=$CHAIN_ID \
--gas=$GAS \
--fees=$FEES \
--node=$NODE \
--yes
```

## 7. Query Commands

### Get Contract Info
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"contract_info": {}}' \
--node=$NODE \
--output json
```

### Get Token Owner
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"owner_of": {"token_id": "0"}}' \
--node=$NODE \
--output json
```

### Get Token Info
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"token_info": {"token_id": "0"}}' \
--node=$NODE \
--output json
```

### Get All Tokens
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"all_tokens": {"limit": 10}}' \
--node=$NODE \
--output json
```

### Get Number of Tokens
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"num_tokens": {}}' \
--node=$NODE \
--output json
```

### Get Data Item
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_data_item": {"token_id": "0"}}' \
--node=$NODE \
--output json
```

### Get Data Versions
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_data_versions": {"token_id": "0"}}' \
--node=$NODE \
--output json
```

### Get Authorized Users
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_authorized_users": {"token_id": "0"}}' \
--node=$NODE \
--output json
```

### Check Access Level
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"check_access_level": {"token_id": "0", "user": "'$YOUR_INT_ADDRESS'"}}' \
--node=$NODE \
--output json
```

### Get Citations
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_citations": {"paper_id": "0"}}' \
--node=$NODE \
--output json
```

### Get Paper DOI
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_paper_doi": {"paper_id": "0"}}' \
--node=$NODE \
--output json
```

### Get Base Citation Fee
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_base_citation_fee": {}}' \
--node=$NODE \
--output json
```

### Get DAO Members
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_dao_members": {}}' \
--node=$NODE \
--output json
```

### Get DAO Config
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_dao_config": {}}' \
--node=$NODE \
--output json
```

### Get Proposal
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_proposal": {"proposal_id": 0}}' \
--node=$NODE \
--output json
```

### Get All Proposals
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_proposals": {"limit": 10}}' \
--node=$NODE \
--output json
```

### Get Proposals by Status
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_proposals": {"limit": 10, "status_filter": "Active"}}' \
--node=$NODE \
--output json
```

### Get Vote
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_vote": {"proposal_id": 0, "voter": "'$YOUR_INT_ADDRESS'"}}' \
--node=$NODE \
--output json
```

### Get Vote Count
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_vote_count": {"proposal_id": 0}}' \
--node=$NODE \
--output json
```

### Get Member Voting Power
```bash
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"get_member_voting_power": {"member": "'$YOUR_INT_ADDRESS'"}}' \
--node=$NODE \
--output json
```

## 8. Complete Workflow Examples

### Complete Paper Publication Workflow (DAO)
```bash
# 1. Submit article proposal
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"submit_article_proposal": {"ipfs_hash": "QmWorkflowTest123","doi": "10.1000/workflow.test.2024","metadata_uri": "https://example.com/workflow.json","title": "Complete Workflow Test","description": "Testing complete DAO governance workflow"}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes

# 2. Vote on proposal
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"vote_on_proposal": {"proposal_id": 0,"choice": "Yes"}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes

# 3. Execute proposal (after voting period or enough votes)
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"execute_proposal": {"proposal_id": 0}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes

# 4. Cite the published paper
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"cite_paper": {"paper_id": "0"}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --amount=1000000000000000000inj --node=$NODE --yes
```

### Complete Data Management Workflow
```bash
# 1. Create paper directly
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"create_paper_item": {"ipfs_hash": "QmDataTest123", "doi": "10.1000/data.2024", "metadata_uri": "https://example.com/data"}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes

# 2. Grant access to another user
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"grant_access": {"token_id": "0", "grantee": "inj1collaborator123", "level": "Write"}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes

# 3. Update the data
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"update_data_item": {"token_id": "0", "new_ipfs_hash": "QmUpdatedData456", "new_metadata_uri": "https://example.com/updated"}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes

# 4. Freeze the data
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"freeze_data": {"token_id": "0", "freeze": true}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes
```

## 9. Testing Tips

1. **Check transaction status**: Always check if your transaction was successful before proceeding to the next step
2. **Wait for blocks**: Some operations might need a few blocks to be confirmed
3. **Query state**: Use query commands to verify the state changes after each operation
4. **Error handling**: If a command fails, check the error message for debugging
5. **Gas estimation**: Adjust gas limits if transactions fail due to out of gas errors

## 10. Common Error Scenarios to Test

### Test Unauthorized Access
```bash
# Try to update data from unauthorized account (should fail)
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"update_data_item": {"token_id": "0", "new_ipfs_hash": "QmUnauthorized", "new_metadata_uri": "https://unauthorized.com"}}' \
--from=inj1unauthorized123456789 --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --node=$NODE --yes
```

### Test Insufficient Payment for Citation
```bash
# Try to cite with insufficient payment (should fail)
injectived tx wasm execute $CONTRACT_ADDRESS \
'{"cite_paper": {"paper_id": "0"}}' \
--from=$YOUR_INT_ADDRESS --chain-id=$CHAIN_ID --gas=$GAS --fees=$FEES --amount=1inj --node=$NODE --yes
```

### Test Non-existent Token
```bash
# Try to query non-existent token (should fail)
injectived query wasm contract-state smart $CONTRACT_ADDRESS \
'{"token_info": {"token_id": "999"}}' \
--node=$NODE --output json
```