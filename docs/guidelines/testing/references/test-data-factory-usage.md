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

## 相关资源

- [TestDataFactory 迁移指南](./test-data-factory-migration.md)
- [测试数据工厂架构文档](./data-factory.md)

---

**最后更新**: 2025-01-XX

