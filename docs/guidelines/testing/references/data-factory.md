# 测试数据工厂指南

> 本文档介绍测试数据工厂的使用和扩展。

---

测试数据工厂提供了Builder模式的测试数据生成接口，简化测试中的数据创建和维护。

## 架构设计

**Builder模式优势**：
- **流畅的API**：链式调用，代码可读性强
- **默认值支持**：提供合理的默认值，减少样板代码
- **类型安全**：编译时检查，避免运行时错误
- **易于维护**：集中管理测试数据模板

**支持的数据类型**：
1. **GitCommitBuilder** - Git提交数据
2. **GitHubPRBuilder** - GitHub Pull Request数据
3. **JiraIssueBuilder** - Jira Issue数据
4. **ConfigBuilder** - 配置数据
5. **BranchBuilder** - 分支数据（名称和JSON）
6. **UserBuilder** - 用户数据（GitHub和Jira）

## 使用示例

更多使用示例和API详细信息，请参考项目测试代码。

### Git Commit 数据生成

```rust
use tests::common::test_data_factory::TestDataFactory;

#[test]
fn test_git_commit_example() {
    let factory = TestDataFactory::new();
    let commit = factory
        .git_commit()
        .sha("abc123def456")
        .message("feat: add new feature")
        .build();
}
```

### 模板文件

部分Builder基于JSON模板文件生成数据：
- `tests/fixtures/templates/git_commit.json`
- `tests/fixtures/templates/github_pr.json`
- `tests/fixtures/templates/jira_issue.json`

其他Builder（BranchBuilder、UserBuilder、ConfigBuilder）直接生成JSON数据。

## 数据验证

使用 `TestDataValidator` trait 和专用验证器确保测试数据的正确性：

```rust
use tests::common::validators::{GitHubPRValidator, TestDataValidator};

let factory = TestDataFactory::new();
let pr = factory.github_pr().build()?;

// 基础验证
pr.validate()?;

// 专用验证
GitHubPRValidator::validate(&pr)?;
```

可用的验证器：
- `GitHubPRValidator` - 验证 GitHub PR 数据
- `JiraIssueValidator` - 验证 Jira Issue 数据
- `GitCommitValidator` - 验证 Git Commit 数据

## 相关文档

- [TestDataFactory 使用指南](./test-data-factory-usage.md) - 详细的使用方法和最佳实践
- [TestDataFactory 迁移指南](./test-data-factory-migration.md) - 从硬编码数据迁移到 Factory

---

**最后更新**: 2025-01-XX
