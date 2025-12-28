# TestDataFactory 使用指南

> 本文档详细介绍 `TestDataFactory` 的使用方法和最佳实践。

---

## 快速开始

```rust
use tests::common::test_data_factory::TestDataFactory;

let factory = TestDataFactory::new();
let pr = factory.github_pr().number(123).title("Test PR").build()?;
```

## 支持的构建器

### 1. GitHub PR Builder

创建 GitHub Pull Request 测试数据：

```rust
let factory = TestDataFactory::new();

// 使用默认值
let pr = factory.github_pr().build()?;

// 自定义值
let pr = factory
    .github_pr()
    .number(456)
    .title("Custom PR")
    .state("closed")
    .merged(true)
    .head_ref("feature/custom")
    .base_ref("main")
    .build()?;
```

**可用方法**：
- `number(u64)` - PR 编号
- `title(&str)` - PR 标题
- `body(&str)` - PR 正文
- `state(&str)` - PR 状态（"open" 或 "closed"）
- `head_ref(&str)` - 源分支
- `base_ref(&str)` - 目标分支
- `merged(bool)` - 是否已合并

### 2. Jira Issue Builder

创建 Jira Issue 测试数据：

```rust
let factory = TestDataFactory::new();

// 使用默认值
let issue = factory.jira_issue().build()?;

// 自定义值
let issue = factory
    .jira_issue()
    .key("PROJ-456")
    .summary("Custom Issue")
    .status("Done")
    .issue_type("Feature")
    .description("Issue description")
    .build()?;
```

**可用方法**：
- `key(&str)` - Issue Key
- `summary(&str)` - Issue 摘要
- `description(&str)` - Issue 描述
- `status(&str)` - Issue 状态
- `issue_type(&str)` - Issue 类型

### 3. Git Commit Builder

创建 Git Commit 测试数据：

```rust
let factory = TestDataFactory::new();

// 使用默认值
let commit = factory.git_commit().build()?;

// 自定义值
let commit = factory
    .git_commit()
    .sha("custom123sha")
    .message("feat: add feature")
    .author_name("John Doe")
    .author_email("john@example.com")
    .author_date("2024-01-01T10:00:00Z")
    .build()?;
```

**可用方法**：
- `sha(&str)` - Commit SHA
- `message(&str)` - Commit 消息
- `author_name(&str)` - 作者名称
- `author_email(&str)` - 作者邮箱
- `author_date(&str)` - 作者日期
- `committer_name(&str)` - 提交者名称
- `committer_email(&str)` - 提交者邮箱
- `committer_date(&str)` - 提交者日期
- `parent_sha(&str)` - 父提交 SHA

### 4. Branch Builder

创建分支测试数据：

```rust
let factory = TestDataFactory::new();

// 获取分支名称字符串
let branch_name = factory.branch().name("feature/test").build_name();

// 获取分支 JSON 数据
let branch_data = factory
    .branch()
    .name("feature/test")
    .prefix("feature")
    .jira_key("PROJ-123")
    .build();
```

**可用方法**：
- `name(&str)` - 分支名称
- `prefix(&str)` - 分支前缀（feature, bugfix, hotfix等）
- `jira_key(&str)` - 关联的 Jira Key
- `build_name()` - 返回分支名称字符串
- `build()` - 返回分支 JSON 数据

### 5. User Builder

创建用户测试数据：

```rust
let factory = TestDataFactory::new();

// GitHub 用户
let github_user = factory
    .user()
    .github()
    .login("testuser")
    .id("12345")
    .name("Test User")
    .email("test@example.com")
    .build();

// Jira 用户
let jira_user = factory
    .user()
    .jira()
    .account_id("test-account-id-123")
    .name("Test User")
    .email("test@example.com")
    .build();
```

**可用方法**：
- `github()` - 设置为 GitHub 用户
- `jira()` - 设置为 Jira 用户
- `login(&str)` - 登录名（GitHub）
- `id(&str)` - 用户 ID
- `name(&str)` - 显示名称
- `email(&str)` - 邮箱
- `account_id(&str)` - 账户 ID（Jira）

### 6. Config Builder

创建配置测试数据：

```rust
let factory = TestDataFactory::new();

// 使用默认配置
let config = factory.config().build();

// 自定义日志配置
let config = factory
    .config()
    .log(json!({
        "level": "debug",
        "method": "file"
    }))
    .build();
```

**可用方法**：
- `github(Value)` - GitHub 配置
- `jira(Value)` - Jira 配置
- `llm(Value)` - LLM 配置
- `log(Value)` - 日志配置

## 数据验证

使用验证器确保测试数据的正确性：

```rust
use tests::common::validators::{
    GitHubPRValidator, JiraIssueValidator, GitCommitValidator, TestDataValidator
};

let factory = TestDataFactory::new();

// 验证 GitHub PR
let pr = factory.github_pr().build()?;
pr.validate()?;
GitHubPRValidator::validate(&pr)?;

// 验证 Jira Issue
let issue = factory.jira_issue().build()?;
JiraIssueValidator::validate(&issue)?;

// 验证 Git Commit
let commit = factory.git_commit().build()?;
GitCommitValidator::validate(&commit)?;
```

## 最佳实践

### 1. 重用 Factory 实例

```rust
// ✅ 推荐：重用 factory 实例
let factory = TestDataFactory::new();
let pr1 = factory.github_pr().number(1).build()?;
let pr2 = factory.github_pr().number(2).build()?;

// ❌ 不推荐：每次都创建新实例
let pr1 = TestDataFactory::new().github_pr().number(1).build()?;
let pr2 = TestDataFactory::new().github_pr().number(2).build()?;
```

### 2. 使用默认值

```rust
// ✅ 推荐：只设置必要的字段
let pr = factory.github_pr().number(123).build()?;

// ❌ 不推荐：设置所有字段（即使使用默认值）
let pr = factory
    .github_pr()
    .number(123)
    .title("Test PR")  // 默认值，不需要设置
    .state("open")    // 默认值，不需要设置
    .build()?;
```

### 3. 链式调用提高可读性

```rust
// ✅ 推荐：链式调用，清晰易读
let pr = factory
    .github_pr()
    .number(123)
    .title("Feature: Add new functionality")
    .state("open")
    .build()?;

// ❌ 不推荐：分多行设置
let mut builder = factory.github_pr();
builder = builder.number(123);
builder = builder.title("Feature: Add new functionality");
let pr = builder.build()?;
```

### 4. 在测试中使用验证器

```rust
#[test]
fn test_pr_creation() -> Result<()> {
    let factory = TestDataFactory::new();
    let pr = factory.github_pr().build()?;

    // 验证数据完整性
    GitHubPRValidator::validate(&pr)?;

    // 执行测试逻辑
    // ...

    Ok(())
}
```

### 5. 使用 rstest fixture

```rust
use rstest::fixture;

#[fixture]
fn test_factory() -> TestDataFactory {
    TestDataFactory::new()
}

#[rstest]
fn test_with_factory(test_factory: TestDataFactory) -> Result<()> {
    let pr = test_factory.github_pr().build()?;
    // ...
    Ok(())
}
```

## 应用场景

### 场景 1: 替换硬编码 JSON

**迁移前**:
```rust
#[test]
fn test_pr_creation() -> Result<()> {
    let pr_data = json!({
        "number": 123,
        "title": "Test PR",
        "state": "open",
        "head": {"ref": "feature/test"},
        "base": {"ref": "main"}
    });

    // 使用 pr_data...
    Ok(())
}
```

**迁移后**:
```rust
#[test]
fn test_pr_creation() -> Result<()> {
    let factory = TestDataFactory::new();
    let pr_data = factory.github_pr()
        .number(123)
        .title("Test PR")
        .state("open")
        .head_ref("feature/test")
        .base_ref("main")
        .build()?;

    // 使用 pr_data...
    Ok(())
}
```

**优势**:
- ✅ 类型安全（Builder 模式）
- ✅ 可读性更好
- ✅ 易于维护和修改
- ✅ 自动生成默认值

### 场景 2: 与 MockServer 结合使用

**迁移前**:
```rust
#[test]
fn test_github_api() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 硬编码响应体
    let response_body = r#"{
        "number": 123,
        "title": "Test PR",
        "state": "open"
    }"#;

    mock_server.server.as_mut()
        .mock("GET", "/repos/owner/repo/pulls/123")
        .with_body(response_body)
        .create();

    Ok(())
}
```

**迁移后**:
```rust
use crate::common::http_helpers::MockServer;
use std::collections::HashMap;

#[test]
fn test_github_api() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 使用 TestDataFactory 生成响应数据
    let factory = TestDataFactory::new();
    let pr_data = factory.github_pr()
        .number(123)
        .title("Test PR")
        .state("open")
        .build()?;

    // 转换为字符串用于 MockServer
    let response_body = serde_json::to_string(&pr_data)?;

    mock_server.server.as_mut()
        .mock("GET", "/repos/owner/repo/pulls/123")
        .with_body(&response_body)
        .create();

    Ok(())
}
```

**或者使用 mock_with_template**:
```rust
#[test]
fn test_github_api() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    let factory = TestDataFactory::new();
    let pr_data = factory.github_pr()
        .number(123)
        .title("Test PR")
        .build()?;

    // 使用 mock_with_template（推荐）
    let mut vars = HashMap::new();
    vars.insert("pr_number".to_string(), "123".to_string());

    mock_server.mock_with_template(
        "GET",
        "/repos/{owner}/repo/pulls/{pr_number}",
        &serde_json::to_string(&pr_data)?,
        vars,
        200,
    );

    Ok(())
}
```

### 场景 3: 批量生成测试数据

**迁移前**:
```rust
#[test]
fn test_multiple_prs() -> Result<()> {
    let mut prs = Vec::new();
    for i in 1..=10 {
        prs.push(json!({
            "number": i,
            "title": format!("PR {}", i),
            "state": "open"
        }));
    }

    // 使用 prs...
    Ok(())
}
```

**迁移后**:
```rust
#[test]
fn test_multiple_prs() -> Result<()> {
    let factory = TestDataFactory::new();

    // 批量生成
    let prs = factory.build_batch(10, |f| {
        f.github_pr().build()
    });

    // 或者自定义每个数据项
    let prs: Vec<Value> = (1..=10)
        .map(|n| factory.github_pr().number(n).title(&format!("PR {}", n)).build().unwrap())
        .collect();

    assert_eq!(prs.len(), 10);
    Ok(())
}
```

## 常见用例

### 创建多个相似数据

```rust
let factory = TestDataFactory::new();
let prs: Vec<Value> = (1..=5)
    .map(|n| factory.github_pr().number(n).build().unwrap())
    .collect();
```

### 创建关联数据

```rust
let factory = TestDataFactory::new();
let jira_key = "PROJ-123";
let branch = factory.branch().name("feature/PROJ-123").jira_key(jira_key).build();
let issue = factory.jira_issue().key(jira_key).build()?;
```

### 组合使用多个构建器

```rust
let factory = TestDataFactory::new();
let user = factory.user().github().login("testuser").build();
let pr = factory.github_pr().build()?;
let commit = factory.git_commit().author_name("testuser").build()?;
```

## 迁移步骤

### 步骤 1: 识别可迁移的代码

查找以下模式：
- `json!({...})` - 硬编码 JSON 对象
- `serde_json::json!({...})` - 硬编码 JSON 对象
- 字符串形式的 JSON（`r#"{"key": "value"}"#`）
- 重复的 JSON 结构

### 步骤 2: 确定对应的 Builder

| 数据类型 | Builder 方法 |
|---------|------------|
| GitHub PR | `factory.github_pr()` |
| Jira Issue | `factory.jira_issue()` |
| Git Commit | `factory.git_commit()` |
| Config | `factory.config()` |
| Branch | `factory.branch()` |
| User | `factory.user()` |

### 步骤 3: 替换硬编码数据

**示例**:
```rust
// 迁移前
let pr = json!({
    "number": 123,
    "title": "Test",
    "state": "open"
});

// 迁移后
let factory = TestDataFactory::new();
let pr = factory.github_pr()
    .number(123)
    .title("Test")
    .state("open")
    .build()?;
```

### 步骤 4: 更新导入

```rust
// 添加导入
use tests::common::test_data_factory::TestDataFactory;
```

### 步骤 5: 测试验证

运行测试确保功能正常：
```bash
cargo test
```

## 常见问题

### Q1: TestDataFactory 和 MockServer 的区别？

**A**:
- **TestDataFactory**: 生成测试数据结构（`serde_json::Value`）
- **MockServer**: 模拟 HTTP 服务器响应（字符串）

**使用方式**:
```rust
// 1. 使用 TestDataFactory 生成数据
let factory = TestDataFactory::new();
let pr_data = factory.github_pr().build()?;

// 2. 转换为字符串用于 MockServer
let response_body = serde_json::to_string(&pr_data)?;

// 3. 使用 MockServer 设置响应
mock_server.server.as_mut()
    .mock("GET", "/api/endpoint")
    .with_body(&response_body)
    .create();
```

### Q2: 如何自定义数据？

**A**: 使用 Builder 模式的方法：

```rust
let pr = factory.github_pr()
    .number(123)           // 自定义 PR 编号
    .title("Custom PR")    // 自定义标题
    .state("closed")       // 自定义状态
    .merged(true)          // 自定义合并状态
    .head_ref("feature/x") // 自定义源分支
    .base_ref("main")      // 自定义目标分支
    .build()?;
```

### Q3: 如何处理复杂嵌套结构？

**A**: TestDataFactory 的 Builder 已经处理了常见的嵌套结构。如果需要更复杂的自定义，可以：

1. 使用 Builder 生成基础数据
2. 手动修改特定字段

```rust
let mut pr = factory.github_pr().build()?;
pr["head"]["sha"] = json!("custom-sha");
pr["base"]["sha"] = json!("base-sha");
```

### Q4: 什么时候应该迁移到 TestDataFactory？

**A**: 以下情况建议迁移：

- ✅ **重复的硬编码 JSON** - 多个测试中使用相同的 JSON 结构
- ✅ **与 MockServer 结合的场景** - 使用硬编码 JSON 作为 Mock 响应
- ✅ **批量数据生成** - 循环生成多个数据项
- ✅ **需要类型安全** - Builder 模式提供编译时检查

以下情况可以不迁移：

- ⚠️ **一次性使用的简单 JSON** - 如果 JSON 非常简单（如 `{}`），可以不迁移
- ⚠️ **临时测试数据** - 如果数据只在一个测试中使用且结构简单

## 实际迁移示例

### 示例 1: 简单 JSON 对象迁移

**文件**: `tests/base/http/client.rs`

#### 迁移前

```rust
#[test]
fn test_post_request_with_json_body() -> Result<()> {
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test", mock_server.base_url);

    // 硬编码 JSON
    let body_data = serde_json::json!({"name": "test", "value": 123});

    // ... 使用 body_data
    Ok(())
}
```

#### 迁移后

```rust
use tests::common::test_data_factory::TestDataFactory;

#[test]
fn test_post_request_with_json_body() -> Result<()> {
    let mut mock_server = setup_mock_server();
    let url = format!("{}/test", mock_server.base_url);

    // 使用 TestDataFactory（如果这是配置数据）
    let factory = TestDataFactory::new();
    let body_data = factory.config().build();

    // 或者对于简单对象，可以直接使用 json!（如果 TestDataFactory 没有对应 Builder）
    // 但对于复杂对象（PR、Issue等），应该使用 TestDataFactory

    Ok(())
}
```

### 示例 2: GitHub PR 数据迁移（与 MockServer 结合）

**文件**: `tests/base/http/github.rs`

#### 迁移前

```rust
#[test]
fn test_create_pull_request_success() {
    let mut mock_server = setup_mock_server();

    // 硬编码模板字符串
    let mut vars = HashMap::new();
    vars.insert("html_url".to_string(), "https://github.com/owner/repo/pull/123".to_string());

    mock_server.mock_with_template(
        "POST",
        "/repos/{owner}/{repo}/pulls",
        r#"{"html_url":"{{html_url}}"}"#,
        vars,
        201,
    );
}
```

#### 迁移后

```rust
use tests::common::test_data_factory::TestDataFactory;
use crate::common::http_helpers::MockServer;
use std::collections::HashMap;

#[test]
fn test_create_pull_request_success() -> Result<()> {
    let mut mock_server = setup_mock_server();

    // 使用 TestDataFactory 生成 PR 数据
    let factory = TestDataFactory::new();
    let pr_data = factory.github_pr()
        .number(123)
        .title("Test PR")
        .html_url("https://github.com/owner/repo/pull/123")
        .build()?;

    // 转换为字符串用于模板
    let template = serde_json::to_string(&pr_data)?;

    let mut vars = HashMap::new();
    vars.insert("owner".to_string(), "owner".to_string());
    vars.insert("repo".to_string(), "repo".to_string());

    mock_server.mock_with_template(
        "POST",
        "/repos/{owner}/{repo}/pulls",
        &template,
        vars,
        201,
    );

    Ok(())
}
```

### 示例 3: Jira Issue 数据迁移

**文件**: `tests/integration/jira.rs`

#### 迁移前

```rust
#[test]
fn test_jira_issue_creation() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // 从文件加载（已是不错的方式，但可以改进）
    let response_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mock_responses")
        .join("jira")
        .join("issue.json");
    mock_server.mock_jira_issue_from_file("GET", "/rest/api/3/issue/PROJ-123", &response_file, 200);

    Ok(())
}
```

#### 迁移后

```rust
use tests::common::test_data_factory::TestDataFactory;
use crate::common::http_helpers::MockServer;

#[test]
fn test_jira_issue_creation() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // 使用 TestDataFactory 生成 Issue 数据
    let factory = TestDataFactory::new();
    let issue_data = factory.jira_issue()
        .key("PROJ-123")
        .summary("Test Issue")
        .description("Test description")
        .status("In Progress")
        .build()?;

    // 转换为字符串用于 Mock
    let response_body = serde_json::to_string(&issue_data)?;

    mock_server.server.as_mut()
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response_body)
        .create();

    Ok(())
}
```

### 示例 4: 批量数据生成迁移

#### 迁移前

```rust
#[test]
fn test_multiple_prs() -> Result<()> {
    let mut prs = Vec::new();
    for i in 1..=10 {
        prs.push(json!({
            "number": i,
            "title": format!("PR {}", i),
            "state": "open"
        }));
    }

    // 使用 prs...
    Ok(())
}
```

#### 迁移后

```rust
use tests::common::test_data_factory::TestDataFactory;

#[test]
fn test_multiple_prs() -> Result<()> {
    let factory = TestDataFactory::new();

    // 批量生成（使用默认值）
    let prs = factory.build_batch(10, |f| {
        f.github_pr().build()
    });

    // 或者自定义每个数据项
    let prs: Vec<_> = (1..=10)
        .map(|i| {
            factory.github_pr()
                .number(i as u64)
                .title(&format!("PR {}", i))
                .state("open")
                .build()
        })
        .collect::<Result<Vec<_>>>()?;

    assert_eq!(prs.len(), 10);
    Ok(())
}
```

### 示例 5: 与 MockServer 模板结合

#### 迁移前

```rust
#[test]
fn test_github_pr_info() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    let mut vars = HashMap::new();
    vars.insert("pr_number".to_string(), "123".to_string());
    vars.insert("title".to_string(), "Test PR".to_string());

    mock_server.mock_with_template(
        "GET",
        "/repos/{owner}/repo/pulls/{pr_number}",
        r#"{
            "number": {{pr_number}},
            "title": "{{title}}",
            "state": "open"
        }"#,
        vars,
        200,
    );

    Ok(())
}
```

#### 迁移后（推荐方式）

```rust
use tests::common::test_data_factory::TestDataFactory;
use crate::common::http_helpers::MockServer;

#[test]
fn test_github_pr_info() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 使用 TestDataFactory 生成数据
    let factory = TestDataFactory::new();
    let pr_data = factory.github_pr()
        .number(123)
        .title("Test PR")
        .state("open")
        .build()?;

    // 直接使用数据作为响应体（不需要模板变量）
    let response_body = serde_json::to_string(&pr_data)?;

    mock_server.server.as_mut()
        .mock("GET", "/repos/owner/repo/pulls/123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response_body)
        .create();

    Ok(())
}
```

## 迁移检查清单

### 步骤 1: 识别可迁移的代码

- [ ] 查找 `json!({...})` 或 `serde_json::json!({...})`
- [ ] 查找硬编码的 JSON 字符串
- [ ] 查找重复的 JSON 结构

### 步骤 2: 确定对应的 Builder

- [ ] GitHub PR → `factory.github_pr()`
- [ ] Jira Issue → `factory.jira_issue()`
- [ ] Git Commit → `factory.git_commit()`
- [ ] Config → `factory.config()`
- [ ] Branch → `factory.branch()`
- [ ] User → `factory.user()`

### 步骤 3: 执行迁移

- [ ] 添加导入：`use tests::common::test_data_factory::TestDataFactory;`
- [ ] 替换硬编码 JSON
- [ ] 使用 Builder 模式设置字段
- [ ] 调用 `.build()?` 生成数据

### 步骤 4: 测试验证

- [ ] 运行测试确保功能正常
- [ ] 检查编译错误
- [ ] 验证数据格式正确
- [ ] 使用了合适的默认值
- [ ] 添加了数据验证（如需要）
- [ ] 代码可读性提高

## 迁移优先级

### 高优先级（立即迁移）

1. **重复的 PR/Issue 数据**
   - 多个测试中使用相同的结构
   - 迁移收益：减少重复，提高可维护性

2. **与 MockServer 结合的场景**
   - 使用 JSON 作为 Mock 响应
   - 迁移收益：统一数据生成方式

### 中优先级（逐步迁移）

3. **批量数据生成**
   - 循环生成多个数据项
   - 迁移收益：使用 `build_batch` 提升性能

4. **复杂嵌套结构**
   - 多层嵌套的 JSON
   - 迁移收益：Builder 模式更清晰

### 低优先级（可选）

5. **简单的一次性 JSON**
   - 如果 JSON 非常简单（如 `{"key": "value"}`）
   - 迁移收益：较小，可以不迁移

## 相关资源

- [Mock服务器使用指南](./mock-server.md) - MockServer 使用方法
- [测试工具指南](./tools.md) - 其他测试工具

---

**最后更新**: 2025-01-27

