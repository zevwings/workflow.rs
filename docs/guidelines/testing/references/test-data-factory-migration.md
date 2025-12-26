# TestDataFactory 迁移指南

> 本指南帮助你将硬编码的测试数据迁移到使用 `TestDataFactory`。

---

## 为什么使用 TestDataFactory？

**优势**：
- ✅ **统一管理**：所有测试数据集中在一个地方
- ✅ **默认值支持**：减少样板代码
- ✅ **类型安全**：编译时检查
- ✅ **易于维护**：修改模板文件即可更新所有测试数据
- ✅ **可读性强**：Builder 模式使代码更清晰

## 迁移步骤

### 1. GitHub PR 数据迁移

**迁移前**：
```rust
let pr_data = json!({
    "number": 123,
    "title": "Test PR",
    "state": "open",
    "merged": false,
});
```

**迁移后**：
```rust
use tests::common::test_data_factory::TestDataFactory;

let factory = TestDataFactory::new();
let pr_data = factory
    .github_pr()
    .number(123)
    .title("Test PR")
    .state("open")
    .merged(false)
    .build()?;
```

### 2. Jira Issue 数据迁移

**迁移前**：
```rust
let issue_data = json!({
    "key": "PROJ-123",
    "fields": {
        "summary": "Test Issue",
        "status": {"name": "In Progress"},
    },
});
```

**迁移后**：
```rust
let factory = TestDataFactory::new();
let issue_data = factory
    .jira_issue()
    .key("PROJ-123")
    .summary("Test Issue")
    .status("In Progress")
    .build()?;
```

### 3. Git Commit 数据迁移

**迁移前**：
```rust
let commit_data = json!({
    "sha": "abc123",
    "commit": {
        "message": "feat: add feature",
        "author": {
            "name": "John Doe",
            "email": "john@example.com",
        },
    },
});
```

**迁移后**：
```rust
let factory = TestDataFactory::new();
let commit_data = factory
    .git_commit()
    .sha("abc123")
    .message("feat: add feature")
    .author_name("John Doe")
    .author_email("john@example.com")
    .build()?;
```

### 4. 分支数据迁移

**迁移前**：
```rust
let branch_name = "feature/test";
let branch_data = json!({
    "name": branch_name,
    "ref": format!("refs/heads/{}", branch_name),
});
```

**迁移后**：
```rust
let factory = TestDataFactory::new();
let branch_name = factory.branch().name("feature/test").build_name();
let branch_data = factory.branch().name("feature/test").build();
```

### 5. 用户数据迁移

**迁移前（GitHub 用户）**：
```rust
let user_data = json!({
    "login": "testuser",
    "id": "12345",
    "name": "Test User",
    "email": "test@example.com",
});
```

**迁移后**：
```rust
let factory = TestDataFactory::new();
let user_data = factory
    .user()
    .github()
    .login("testuser")
    .id("12345")
    .name("Test User")
    .email("test@example.com")
    .build();
```

**迁移前（Jira 用户）**：
```rust
let user_data = json!({
    "accountId": "test-123",
    "displayName": "Test User",
    "emailAddress": "test@example.com",
});
```

**迁移后**：
```rust
let factory = TestDataFactory::new();
let user_data = factory
    .user()
    .jira()
    .account_id("test-123")
    .name("Test User")
    .email("test@example.com")
    .build();
```

### 6. 配置数据迁移

**迁移前**：
```rust
let config_data = json!({
    "github": {
        "accounts": [],
        "current_account": null,
    },
    "jira": {
        "service_address": null,
        "username": null,
        "api_token": null,
    },
});
```

**迁移后**：
```rust
let factory = TestDataFactory::new();
let config_data = factory.config().build();
```

## 使用验证器

迁移后，建议使用验证器确保数据正确性：

```rust
use tests::common::validators::{GitHubPRValidator, TestDataValidator};

let factory = TestDataFactory::new();
let pr = factory.github_pr().build()?;

// 验证数据
pr.validate()?;
GitHubPRValidator::validate(&pr)?;
```

## 常见问题

### Q: 如果只需要部分字段怎么办？

A: Factory 提供默认值，你只需要设置需要的字段：

```rust
// 只设置必要的字段，其他使用默认值
let pr = factory.github_pr().number(456).title("Custom").build()?;
```

### Q: 如何创建多个相似的数据？

A: 可以重用 factory 实例：

```rust
let factory = TestDataFactory::new();
let pr1 = factory.github_pr().number(1).build()?;
let pr2 = factory.github_pr().number(2).build()?;
```

### Q: 如何验证自定义数据结构？

A: 实现 `TestDataValidator` trait：

```rust
impl TestDataValidator for MyCustomType {
    fn validate(&self) -> Result<()> {
        // 自定义验证逻辑
        Ok(())
    }
}
```

## 检查清单

迁移时请检查：

- [ ] 所有硬编码的 JSON 数据已替换为 Factory
- [ ] 测试仍然通过
- [ ] 使用了合适的默认值
- [ ] 添加了数据验证（如需要）
- [ ] 代码可读性提高

---

**最后更新**: 2025-01-XX

