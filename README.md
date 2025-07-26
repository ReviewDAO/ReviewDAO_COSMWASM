# 学术研究数据管理与DAO治理智能合约

这是一个基于CosmWasm构建的学术研究数据管理和去中心化自治组织(DAO)治理智能合约。该合约结合了NFT技术、数据版本控制、学术论文发布和DAO治理机制，为学术研究社区提供了一个完整的去中心化数据管理解决方案。

## 项目特性

### 核心功能
- **研究数据NFT化**: 将研究数据转换为NFT，确保数据所有权和可追溯性
- **版本控制**: 支持数据版本管理，记录每次更新的历史
- **访问权限管理**: 灵活的权限控制系统，支持公开/私有数据访问
- **学术论文发布**: 专门的学术论文创建和管理功能
- **引用系统**: 内置的论文引用机制，支持引用费用分配

### DAO治理功能
- **提案系统**: 支持文章发布、成员管理、配置更新等多种提案类型
- **投票机制**: 基于成员身份的投票系统，支持是/否/弃权投票
- **自动执行**: 提案通过后可自动执行相应操作
- **成员管理**: 动态的DAO成员添加和移除机制
- **配置管理**: 可通过治理流程调整DAO参数

### 经济模型
- **引用费用**: 论文引用需要支付费用，95%给作者，5%给DAO
- **数据访问费用**: 私有数据访问需要支付设定的费用
- **灵活定价**: 数据所有者可自由设定访问价格

## 代码结构

```
src/
├── lib.rs              # 库入口文件，模块声明
├── contract.rs         # 主合约逻辑，包含所有执行和查询函数
├── msg.rs              # 消息定义，包含执行消息、查询消息和响应结构
├── state.rs            # 状态存储定义，定义所有存储项和映射
├── error.rs            # 错误类型定义
├── helpers.rs          # 辅助函数，包含DAO治理相关的验证和检查函数
└── integration_tests.rs # 集成测试，覆盖主要功能流程

schema/                 # JSON Schema文件
├── execute_msg.json    # 执行消息schema
├── query_msg.json      # 查询消息schema
├── instantiate_msg.json # 初始化消息schema
└── ...                 # 其他响应类型schema

examples/
└── schema.rs           # Schema生成示例

artifacts/              # 编译产物
├── bc.wasm            # 编译后的WASM文件
└── checksums.txt      # 校验和文件
```

### 核心模块说明

#### contract.rs
主合约逻辑文件，包含：
- `instantiate`: 合约初始化，设置基础参数和DAO配置
- `execute`: 处理所有执行消息，包括数据管理、NFT操作、DAO治理等
- `query`: 处理所有查询请求，提供数据访问接口

主要执行函数：
- 数据管理: `execute_create_data_item`, `execute_update_data_item`, `execute_freeze_data`
- 权限管理: `execute_grant_access`, `execute_request_access`
- 论文管理: `execute_create_paper_item`, `execute_cite_paper`, `execute_submit_correction`
- DAO治理: `execute_submit_*_proposal`, `execute_vote_on_proposal`, `execute_proposal`

#### msg.rs
消息和数据结构定义：
- `ExecuteMsg`: 所有执行操作的消息枚举
- `QueryMsg`: 所有查询操作的消息枚举
- `DataItem`: 研究数据项结构
- `Proposal`: DAO提案结构
- `DaoConfig`: DAO配置参数
- 各种响应结构体

#### state.rs
状态存储定义：
- 基础存储: 合约信息、Token计数器
- NFT存储: Token所有权、批准关系
- 数据存储: 数据项、版本历史、访问控制
- 论文存储: 引用记录、DOI映射
- DAO存储: 成员列表、提案、投票记录

#### helpers.rs
DAO治理辅助函数：
- 成员身份验证: `is_dao_member`, `ensure_dao_member`
- 提案状态管理: `ensure_proposal_exists`, `can_vote_on_proposal`
- 时间验证: `is_proposal_expired`, `validate_voting_period`
- 配置验证: `validate_dao_config`

#### error.rs
自定义错误类型：
- 基础错误: `Unauthorized`, `TokenNotFound`, `InsufficientPayment`
- DAO错误: `NotDaoMember`, `ProposalNotFound`, `VotingPeriodActive`

## 环境设置

### rust环境
```
rustc 1.87.0 (17067e9ac 2025-05-09)
binary: rustc
commit-hash: 17067e9ac6d7ecb70e50f92c1944e545188d2359
commit-date: 2025-05-09
host: x86_64-unknown-linux-gnu
release: 1.87.0
LLVM version: 20.1.1
```

### 编译合约
```sh
# 开发编译
make build

# 优化编译（用于部署）
make artifacts
```

### 运行测试
```sh
# 集成测试
make test
```

### 生成Schema
```sh
make schema
```

## 部署和使用

### 本地测试网部署
1. 创建`injective`账号
2. 编译合约: `make artifacts`
3. 部署合约到测试网
4. 初始化合约参数

### Injective Testnet (最新部署 - 2025-07-26)
`hash`: `inj1crxs4sfy9smufn7dq63ph064ac5ku8ppfqr69s`

### 主要使用流程

1. **初始化DAO**: 合约部署时自动创建DAO，部署者成为首个成员
2. **创建研究数据**: 研究者可以创建数据NFT，设置访问权限和价格
3. **发布学术论文**: 通过DAO提案流程发布学术论文
4. **数据访问**: 其他用户可以请求访问数据，支付相应费用
5. **论文引用**: 引用论文时支付引用费用，自动分配给作者和DAO
6. **DAO治理**: 成员可以提交各类提案，通过投票决定DAO事务

### 一个简单的例子
#### Upload Wasm
```
yes 12345678 | injectived tx wasm store artifacts/bc.wasm \
--from=$YOUR_INT_ADDRESS$ \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=3000000 \
--node=https://testnet.sentry.tm.injective.network:443
```

#### Get The Code Of Contract
```
injectived query tx traction_code --node=https://testnet.sentry.tm.injective.network:443 > tmp
```

#### Instantiate Contract
```
INIT='{"name":"Research Data NFT","symbol":"RDN","owner":$YOUR_INT_ADDRESS$}'
yes password | injectived tx wasm instantiate the_code_of_contract "$INIT" \
  --label="ResearchDataNFTInstance" \
  --from=$YOUR_INT_ADDRESS$ \
  --chain-id="injective-888" \
  --yes --fees=1000000000000000inj \
  --gas=2000000 \
  --no-admin \
  --node=https://testnet.sentry.tm.injective.network:443
```

#### Submit Article Proposal
```
injectived tx wasm execute $CONTRACT_ADDRESS$ \
'{
  "submit_article_proposal": {
    "ipfs_hash": "QmWorkflowTest123",
    "doi": "10.1000/workflow.test.2024",
    "metadata_uri": "https://example.com/workflow.json",
    "title": "Complete Workflow Test",
    "description": "Testing complete DAO governance workflow"
  }
}' \
--from=$YOUR_INT_ADDRESS$ \
--chain-id=injective-888 \
--gas=2000000 \
--fees=1000000000000000inj \
--node=https://testnet.sentry.tm.injective.network:443 \
--yes

```

#### Vote On Proposal(Only DAO)
```
injectived tx wasm execute $CONTRACT_ADDRESS$ \
'{
  "vote_on_proposal": {
    "proposal_id": 0,
    "choice": "Yes"
  }
}' \
--from=$YOUR_INT_ADDRESS$ \
--chain-id=injective-888 \
--gas=2000000 \
--fees=1000000000000000inj \
--node=https://testnet.sentry.tm.injective.network:443 \
--yes

```

#### Get Doi
```
injectived query wasm contract-state smart $CONTRACT_ADDRESS$ \               
'{
  "get_paper_doi": {
    "paper_id": "0"
  }
}' \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json
```

#### Cite Paper
```
injectived tx wasm execute $CONTRACT_ADDRESS$ \
'{
  "cite_paper": {
    "paper_id": "0"
  }
}' \
--from=$YOUR_INT_ADDRESS$ \
--chain-id=injective-888 \
--gas=2000000 \
--fees=1000000000000000inj \
--node=https://testnet.sentry.tm.injective.network:443 \
--yes

```

#### Check Cite
```
 injectived query wasm contract-state smart $CONTRACT_ADDRESS$ \
'{
  "get_citations": {
    "paper_id": "0"
  }
}' \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json
```

for more, click [here](./test_commands.md)


## 技术特点

- **模块化设计**: 清晰的模块分离，便于维护和扩展
- **完整的测试覆盖**: 包含单元测试和集成测试
- **灵活的权限系统**: 支持多级访问控制
- **自动化治理**: 提案自动执行机制
- **经济激励**: 内置的费用分配和激励机制

## 贡献指南

欢迎提交Issue和Pull Request

## 未来计划

计划在后面加入agent功能，在论文审批通过之后可以可以通过oracle，调用agent来评定整篇论文的整体质量（综合选题，热度等等），传入智能合约之后重新计算作者的影响因子，当达到特定比重的时候就可以加入DAO，参与整体学术社区的治理，并且在DAO中影响因子更高的作者对应的投票比重也将更高（也就是更有说服力和话语权）
