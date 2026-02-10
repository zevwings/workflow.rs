# 测试架构改进 - 实施计划

> 基于方案 A 的各 crate 测试模块实施指南

## 📊 总体概览

| Crate | 优先级 | 状态 | testing 模块内容 | 理由 |
|-------|--------|------|-----------------|------|
| **http** | P0 | ✅ 已完成 | MockServer, MockServerManager, TestDataFactory | HTTP 测试工具基础 |
| **domain** | P1 | 📝 建议 | 测试数据工厂 | 领域对象构建器 |
| **storage** | P1 | 🔧 重组扩展 | Git 测试辅助 + API 响应工厂 | 统一测试工具位置 |
| **services** | P2 | 📝 建议 | Service Mock + 业务数据工厂 | 简化 app 集成测试 |
| **llm** | P3 | 📝 可选 | LLM 响应 Mock | 加速测试，避免真实调用 |
| **toolkit** | ❌ | 不需要 | - | 简单工具库 |
| **prompt** | ❌ | 不需要 | - | UI 库 |
| **di** | ❌ | 不需要 | - | 依赖注入容器 |
| **app** | ❌ | 不需要 | - | 使用其他 crate 的测试工具 |

---

## 1️⃣ domain - 领域对象测试数据工厂 (P1)

### 📦 Crate 职责
- 领域层（DDD Domain Layer）
- 包含实体、值对象、仓储接口、领域服务接口

### 🎯 为什么需要 testing 模块
- 其他 crate（storage、services、app）的测试需要创建领域对象
- 领域对象构造复杂，有验证规则
- 避免测试代码重复创建相似的测试数据

### 📁 建议的目录结构

```bash
crates/domain/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── testing/
│       ├── mod.rs
│       ├── entity_factory.rs    # 实体构建器
│       └── value_object_factory.rs  # 值对象构建器
```

### 🔧 Cargo.toml 配置

```toml
[features]
default = []
testing = []

# 无需额外依赖，因为只创建领域对象
```

### 💻 实现示例

```rust
// crates/domain/src/testing/entity_factory.rs

use crate::entities::*;

pub struct TestEntityFactory;

impl TestEntityFactory {
    /// 创建分支实体构建器
    pub fn branch() -> BranchBuilder {
        BranchBuilder::default()
    }

    /// 创建提交实体构建器
    pub fn commit() -> CommitBuilder {
        CommitBuilder::default()
    }

    /// 创建 Pull Request 实体构建器
    pub fn pull_request() -> PullRequestBuilder {
        PullRequestBuilder::default()
    }
}

#[derive(Default)]
pub struct BranchBuilder {
    name: Option<String>,
    is_current: bool,
    upstream: Option<String>,
}

impl BranchBuilder {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn as_current(mut self) -> Self {
        self.is_current = true;
        self
    }

    pub fn with_upstream(mut self, upstream: impl Into<String>) -> Self {
        self.upstream = Some(upstream.into());
        self
    }

    pub fn build(self) -> Branch {
        Branch {
            name: self.name.unwrap_or_else(|| "test-branch".to_string()),
            is_current: self.is_current,
            upstream: self.upstream,
        }
    }
}
```

### 📝 使用场景

```rust
// 在 storage 的测试中
use domain::testing::TestEntityFactory;

#[test]
fn test_save_branch() {
    let branch = TestEntityFactory::branch()
        .with_name("feature/new-feature")
        .as_current()
        .build();

    // 使用 branch 进行测试...
}
```

---

## 2️⃣ storage - 重组和扩展现有测试工具 (P1)

### 📦 Crate 职责
- 存储实现层（Storage Adapters）
- 实现仓储接口，提供数据持久化和外部服务调用

### ✅ 现状
- ✅ 已有 `testing` feature
- ✅ 已有 `git/testing.rs`（包含 Git 测试辅助函数）
- ❌ 测试工具分散，缺少统一入口

### 🎯 改进目标
1. 将 `git/testing.rs` 迁移到 `src/testing/git.rs`
2. 添加 GitHub/Jira API 响应工厂
3. 统一测试工具导出

### 📁 建议的目录结构

```bash
crates/storage/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── git/
│   │   └── (移除 testing.rs)
│   └── testing/
│       ├── mod.rs
│       ├── git.rs              # 从 git/testing.rs 迁移
│       ├── github_fixtures.rs  # GitHub API 响应数据
│       └── jira_fixtures.rs    # Jira API 响应数据
```

### 🔧 Cargo.toml 配置（已存在，无需修改）

```toml
[features]
default = []
testing = ["dep:tempfile"]

[dependencies]
tempfile = { workspace = true, optional = true }

[dev-dependencies]
tempfile.workspace = true
```

### 💻 实现示例

```rust
// crates/storage/src/testing/mod.rs
//! Storage 测试辅助工具
//!
//! 提供 Git 测试辅助函数和外部 API 响应 fixtures。

pub mod git;
pub mod github_fixtures;
pub mod jira_fixtures;

// 重新导出常用函数
pub use git::{
    noop_hook_service,
    setup_repo,
    setup_repo_with_branches,
    setup_repo_with_file,
};

pub use github_fixtures::GitHubFixtures;
pub use jira_fixtures::JiraFixtures;
```

```rust
// crates/storage/src/testing/git.rs
// (从 git/testing.rs 迁移过来的内容)
// ... 保持现有代码不变 ...
```

```rust
// crates/storage/src/testing/github_fixtures.rs

use serde_json::Value;

/// GitHub API 响应 Fixtures
///
/// 提供预定义的 GitHub API 响应数据，用于测试。
pub struct GitHubFixtures;

impl GitHubFixtures {
    /// 获取示例 Pull Request 响应
    pub fn sample_pull_request() -> Value {
        serde_json::json!({
            "id": 1,
            "number": 123,
            "state": "open",
            "title": "Add new feature",
            "body": "This PR adds a new feature",
            "user": {
                "login": "octocat",
                "id": 1
            },
            "head": {
                "ref": "feature-branch",
                "sha": "abc123"
            },
            "base": {
                "ref": "main",
                "sha": "def456"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })
    }

    /// 获取示例 Pull Request 列表响应
    pub fn sample_pull_request_list() -> Value {
        serde_json::json!([
            Self::sample_pull_request(),
        ])
    }

    /// 获取示例 Issue 响应
    pub fn sample_issue() -> Value {
        serde_json::json!({
            "id": 1,
            "number": 456,
            "state": "open",
            "title": "Bug report",
            "body": "Found a bug",
            "user": {
                "login": "octocat",
                "id": 1
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })
    }
}
```

```rust
// crates/storage/src/testing/jira_fixtures.rs

use serde_json::Value;

/// Jira API 响应 Fixtures
pub struct JiraFixtures;

impl JiraFixtures {
    /// 获取示例 Issue 响应
    pub fn sample_issue() -> Value {
        serde_json::json!({
            "key": "PROJ-123",
            "id": "10001",
            "fields": {
                "summary": "Test Issue",
                "description": "Test description",
                "issuetype": {
                    "name": "Task"
                },
                "status": {
                    "name": "To Do"
                },
                "project": {
                    "key": "PROJ"
                }
            }
        })
    }

    /// 获取示例转换响应
    pub fn sample_transitions() -> Value {
        serde_json::json!({
            "transitions": [
                {
                    "id": "11",
                    "name": "In Progress",
                    "to": {
                        "name": "In Progress"
                    }
                },
                {
                    "id": "21",
                    "name": "Done",
                    "to": {
                        "name": "Done"
                    }
                }
            ]
        })
    }
}
```

### 🔄 迁移步骤

1. ✅ 创建 `src/testing/` 目录
2. ✅ 创建 `src/testing/mod.rs`
3. ✅ 将 `git/testing.rs` 内容移动到 `src/testing/git.rs`
4. ✅ 创建 `github_fixtures.rs` 和 `jira_fixtures.rs`
5. ✅ 更新 `src/lib.rs` 导出
6. ✅ 更新所有使用 `git::testing` 的测试代码
7. ✅ 删除旧的 `git/testing.rs`

### 📝 使用场景

```rust
// 在测试中使用
use storage::testing::{setup_repo, GitHubFixtures};

#[test]
fn test_github_integration() {
    let (_tmp, ctx) = setup_repo();
    let pr_data = GitHubFixtures::sample_pull_request();

    // 使用 pr_data 进行测试...
}
```

---

## 3️⃣ services - 业务服务测试工具 (P2)

### 📦 Crate 职责
- 服务层（Application Service Layer）
- 组合 storage，实现业务用例

### 🎯 为什么需要 testing 模块
- app 层需要 Mock 业务服务进行集成测试
- 业务用例复杂，需要预定义的测试场景
- 避免在测试中重复创建相似的 Mock

### 📁 建议的目录结构

```bash
crates/services/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── branch/
│   ├── commit/
│   ├── pull_request/
│   └── testing/
│       ├── mod.rs
│       ├── mock_services.rs    # Service Mock
│       └── builders.rs         # 测试数据构建器
```

### 🔧 Cargo.toml 配置

```toml
[features]
default = []
testing = ["dep:once_cell"]

[dependencies]
once_cell = { workspace = true, optional = true }

[dev-dependencies]
once_cell.workspace = true
tempfile.workspace = true
```

### 💻 实现示例

```rust
// crates/services/src/testing/mod.rs

pub mod mock_services;
pub mod builders;

pub use mock_services::{MockBranchService, MockCommitService, MockPullRequestService};
pub use builders::{BranchServiceTestData, CommitServiceTestData};
```

```rust
// crates/services/src/testing/mock_services.rs

use crate::branch::traits::BranchService;
use domain::{Branch, GitError};
use std::sync::{Arc, Mutex};

/// Mock 分支服务
///
/// 用于测试，不执行真实的 Git 操作。
pub struct MockBranchService {
    branches: Arc<Mutex<Vec<Branch>>>,
    current_branch: Arc<Mutex<Option<String>>>,
}

impl MockBranchService {
    /// 创建新的 Mock 服务
    pub fn new() -> Self {
        Self {
            branches: Arc::new(Mutex::new(Vec::new())),
            current_branch: Arc::new(Mutex::new(Some("main".to_string()))),
        }
    }

    /// 添加分支到 Mock 数据
    pub fn add_branch(&self, branch: Branch) {
        self.branches.lock().unwrap().push(branch);
    }

    /// 设置当前分支
    pub fn set_current_branch(&self, name: impl Into<String>) {
        *self.current_branch.lock().unwrap() = Some(name.into());
    }
}

impl BranchService for MockBranchService {
    fn list_branches(&self) -> Result<Vec<Branch>, GitError> {
        Ok(self.branches.lock().unwrap().clone())
    }

    fn get_current_branch(&self) -> Result<Option<String>, GitError> {
        Ok(self.current_branch.lock().unwrap().clone())
    }

    fn create_branch(&self, name: &str, _base: Option<&str>) -> Result<(), GitError> {
        let branch = Branch {
            name: name.to_string(),
            is_current: false,
            upstream: None,
        };
        self.add_branch(branch);
        Ok(())
    }

    // 实现其他方法...
}

impl Default for MockBranchService {
    fn default() -> Self {
        Self::new()
    }
}
```

```rust
// crates/services/src/testing/builders.rs

use crate::branch::traits::BranchService;
use domain::Branch;
use super::MockBranchService;

/// 分支服务测试数据构建器
pub struct BranchServiceTestData {
    service: MockBranchService,
}

impl BranchServiceTestData {
    /// 创建新的测试数据构建器
    pub fn new() -> Self {
        Self {
            service: MockBranchService::new(),
        }
    }

    /// 添加分支
    pub fn with_branch(self, branch: Branch) -> Self {
        self.service.add_branch(branch);
        self
    }

    /// 设置当前分支
    pub fn with_current_branch(self, name: impl Into<String>) -> Self {
        self.service.set_current_branch(name);
        self
    }

    /// 添加多个默认分支
    pub fn with_default_branches(self) -> Self {
        self.with_branch(Branch {
            name: "main".to_string(),
            is_current: true,
            upstream: Some("origin/main".to_string()),
        })
        .with_branch(Branch {
            name: "develop".to_string(),
            is_current: false,
            upstream: Some("origin/develop".to_string()),
        })
    }

    /// 构建并返回 Mock 服务
    pub fn build(self) -> MockBranchService {
        self.service
    }
}

impl Default for BranchServiceTestData {
    fn default() -> Self {
        Self::new()
    }
}
```

### 📝 使用场景

```rust
// 在 app 的测试中
use services::testing::{MockBranchService, BranchServiceTestData};
use domain::testing::TestEntityFactory;

#[test]
fn test_branch_listing_command() {
    // 准备测试数据
    let branch_service = BranchServiceTestData::new()
        .with_default_branches()
        .build();

    // 使用 branch_service 进行测试...
    let branches = branch_service.list_branches().unwrap();
    assert_eq!(branches.len(), 2);
}
```

---

## 4️⃣ llm - LLM 响应 Mock (P3 - 可选)

### 📦 Crate 职责
- LLM 客户端与对话抽象
- 提供配置驱动的 API 调用

### 🎯 为什么需要 testing 模块
- LLM 调用慢且不稳定
- 避免测试依赖外部 API
- 测试不同的 LLM 响应场景（成功、失败、超时）

### 📁 建议的目录结构

```bash
crates/llm/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── testing/
│       ├── mod.rs
│       ├── mock_client.rs      # Mock LLM 客户端
│       └── fixtures.rs         # 预定义响应
```

### 🔧 Cargo.toml 配置

```toml
[features]
default = []
testing = []

# 无需额外依赖
```

### 💻 实现示例

```rust
// crates/llm/src/testing/mod.rs

pub mod mock_client;
pub mod fixtures;

pub use mock_client::MockLLMClient;
pub use fixtures::LLMFixtures;
```

```rust
// crates/llm/src/testing/mock_client.rs

use crate::{LLMClient, LLMRequest, LLMResponse, LLMError};
use std::sync::{Arc, Mutex};

/// Mock LLM 客户端
///
/// 不发送真实的 API 请求，而是返回预定义的响应。
pub struct MockLLMClient {
    responses: Arc<Mutex<Vec<String>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockLLMClient {
    /// 创建新的 Mock 客户端
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// 添加预定义响应
    pub fn add_response(&self, response: impl Into<String>) {
        self.responses.lock().unwrap().push(response.into());
    }

    /// 获取调用次数
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

impl LLMClient for MockLLMClient {
    fn send(&self, _request: &LLMRequest) -> Result<LLMResponse, LLMError> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        let responses = self.responses.lock().unwrap();
        let index = (*count - 1) % responses.len();

        if let Some(content) = responses.get(index) {
            Ok(LLMResponse {
                content: content.clone(),
                model: "mock-model".to_string(),
            })
        } else {
            Err(LLMError::NoResponse)
        }
    }
}

impl Default for MockLLMClient {
    fn default() -> Self {
        Self::new()
    }
}
```

```rust
// crates/llm/src/testing/fixtures.rs

/// LLM 响应 Fixtures
pub struct LLMFixtures;

impl LLMFixtures {
    /// 获取示例提交消息生成响应
    pub fn commit_message() -> &'static str {
        "feat: add new user authentication feature\n\n\
         - Implement JWT-based authentication\n\
         - Add user login and registration endpoints\n\
         - Update database schema for user table"
    }

    /// 获取示例 PR 描述生成响应
    pub fn pr_description() -> &'static str {
        "## Summary\n\
         This PR adds a new user authentication feature.\n\n\
         ## Changes\n\
         - Add JWT library\n\
         - Implement login endpoint\n\
         - Add tests\n\n\
         ## Test Plan\n\
         - Unit tests pass\n\
         - Integration tests pass"
    }

    /// 获取错误响应
    pub fn error_response() -> &'static str {
        "Error: Unable to generate response"
    }
}
```

### 📝 使用场景

```rust
// 在测试中使用
use llm::testing::{MockLLMClient, LLMFixtures};
use llm::LLMClient;

#[test]
fn test_commit_message_generation() {
    let client = MockLLMClient::new();
    client.add_response(LLMFixtures::commit_message());

    let request = create_commit_message_request();
    let response = client.send(&request).unwrap();

    assert!(response.content.starts_with("feat:"));
    assert_eq!(client.call_count(), 1);
}
```

---

## 📋 实施优先级和顺序

### Phase 1: 基础设施 (已完成)
- ✅ **http** - HTTP 测试工具（MockServer, TestDataFactory）

### Phase 2: 数据层 (建议下一步)
1. **domain** (P1) - 领域对象工厂
   - 影响范围：所有其他 crate
   - 实施难度：低
   - 预计时间：2-3 小时

2. **storage** (P1) - 重组和扩展
   - 影响范围：services, app
   - 实施难度：中（需要迁移现有代码）
   - 预计时间：3-4 小时

### Phase 3: 业务层 (根据需求)
3. **services** (P2) - Service Mock
   - 影响范围：app
   - 实施难度：中高
   - 预计时间：4-6 小时

### Phase 4: 辅助层 (可选)
4. **llm** (P3) - LLM Mock
   - 影响范围：services, app
   - 实施难度：低
   - 预计时间：1-2 小时

---

## 🎯 预期收益

### 1. 测试代码简化
**Before:**
```rust
#[test]
fn test_something() {
    // 30 行代码创建测试数据
    let pr = json!({
        "title": "Test",
        "body": "Body",
        "number": 1,
        // ... 很多字段
    });

    // 20 行代码设置 Mock
    let mut server = MockServer::new();
    server.mock("GET", "/api/pr").create();

    // 实际测试逻辑
    // ...
}
```

**After:**
```rust
#[test]
fn test_something() {
    // 3 行代码准备测试数据
    let pr = TestDataFactory::github_pr()
        .with_title("Test")
        .build();

    let mut manager = MockServerManager::new();
    manager.setup_github_pr_list(vec![pr]);

    // 实际测试逻辑
    // ...
}
```

### 2. 测试可维护性提升
- ✅ 统一的测试数据创建方式
- ✅ 预定义的测试场景
- ✅ 清晰的 API 文档

### 3. 测试速度提升
- ✅ Mock 替代真实的外部调用
- ✅ 避免重复的设置代码

---

## 📚 参考文档

- [06-test-architecture-improvement.md](./06-test-architecture-improvement.md) - 架构方案
- [06-test-architecture-improvement-appendix.md](./06-test-architecture-improvement-appendix.md) - 技术细节
- [http/TESTING.md](../../crates/http/TESTING.md) - HTTP 测试工具使用指南（参考模板）

---

## ✅ 检查清单

每个 testing 模块实施时，确保：

- [ ] 添加 `testing` feature 到 `Cargo.toml`
- [ ] 创建 `src/testing/` 目录和 `mod.rs`
- [ ] 在 `lib.rs` 中使用 `#[cfg(any(test, feature = "testing"))]` 导出
- [ ] 编写完整的文档注释和使用示例
- [ ] 添加单元测试验证测试工具本身
- [ ] 更新 README 或创建 TESTING.md 文档
- [ ] 在实际测试中使用，验证易用性
- [ ] 运行 `cargo test` 确保所有测试通过
