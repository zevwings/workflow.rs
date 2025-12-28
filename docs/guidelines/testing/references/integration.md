# 集成测试指南

> 本文档介绍集成测试的环境配置和最佳实践。

---

## 集成测试环境配置

### 环境初始化

```rust
use tests::common::helpers::setup_test_env;

#[test]
fn test_with_env_setup() {
    setup_test_env();  // 只会执行一次
    // 测试代码
}
```

### CLI 测试环境

```rust
use tests::common::cli_helpers::CliTestEnv;

#[test]
fn test_cli_command() {
    let env = CliTestEnv::new()
        .init_git_repo()
        .create_file("test.txt", "content")
        .create_config("[jira]\nurl = \"https://test.atlassian.net\"");
}
```

## 数据隔离规则

1. **独立数据**：每个测试使用独立的数据和资源
2. **唯一标识**：使用时间戳和随机字符串确保唯一性
3. **临时资源**：所有测试数据应存储在临时目录中

## 清理机制

### Drop trait 自动清理

```rust
#[test]
fn test_with_auto_cleanup() {
    let mock_server = MockServer::new();
    let mut temp_manager = TempManager::new().unwrap();
    
    // 测试代码
    
    // Drop trait 会自动清理资源
}
```

更多详细配置和使用方法，请参考实际测试代码。

---

**最后更新**: 2025-12-25
