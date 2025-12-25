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

所有Builder都基于JSON模板文件生成数据：
- `tests/fixtures/templates/git_commit.json`
- `tests/fixtures/templates/github_pr.json`
- `tests/fixtures/templates/jira_issue.json`

---

**最后更新**: 2025-12-25
