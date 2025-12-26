# 测试工具指南

> 本文档介绍常用测试工具的使用方法。

---

## 📋 目录

- [pretty_assertions](#1-pretty_assertions)
- [rstest](#2-rstest)
- [mockito](#3-mockito)
- [Mock对象使用规范](#4-mock对象使用规范)
- [测试环境工具](#5-测试环境工具)
- [测试辅助工具](#6-测试辅助工具)

---

## 1. pretty_assertions

`pretty_assertions` 提供更清晰的断言输出，显示彩色 diff。

### 使用方式

```rust
use pretty_assertions::assert_eq;

#[test]
fn test_example() {
    let actual = "Hello";
    let expected = "World";
    assert_eq!(actual, expected);  // 会显示清晰的彩色 diff
}
```

### 效果

失败时会显示清晰的彩色 diff，更容易定位问题。

---

## 2. rstest

`rstest` 支持参数化测试和 fixtures，减少代码重复。

### 参数化测试

```rust
use rstest::rstest;

#[rstest]
#[case("input1", "output1")]
#[case("input2", "output2")]
#[case("input3", "output3")]
fn test_multiple_cases(#[case] input: &str, #[case] expected: &str) {
    let result = process(input);
    assert_eq!(result, expected);
}
```

### Fixtures

```rust
use rstest::{fixture, rstest};

#[fixture]
fn sample_data() -> Vec<i32> {
    vec![1, 2, 3, 4, 5]
}

#[rstest]
fn test_with_fixture(sample_data: Vec<i32>) {
    assert_eq!(sample_data.len(), 5);
}
```

### 优势

- 减少代码重复
- 更容易添加新的测试用例
- 测试用例更清晰

---

## 3. mockito

`mockito` 用于 HTTP API 的 Mock 测试，避免实际调用外部 API。

### 使用方式

```rust
use crate::common::http_helpers::MockServer;
use mockito::Matcher;

#[test]
fn test_api_call() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .match_header("authorization", Matcher::Regex(r"token .+".to_string()))
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // let result = client.call_api()?;
    // assert_eq!(result, "success");

    // 验证 Mock 被调用
    // _mock.assert();
}
```

### MockServer 工具

`tests/common/http_helpers.rs` 提供了 `MockServer` 包装器，简化 Mock 服务器的使用：

```rust
use crate::common::http_helpers::MockServer;

let mut mock_server = MockServer::new();
mock_server.setup_github_base_url();  // 设置 GitHub API Mock
mock_server.setup_jira_base_url();    // 设置 Jira API Mock
// MockServer 会在 Drop 时自动清理环境变量
```

### 优势

- 不依赖外部 API
- 测试执行速度快
- 可以模拟各种错误情况
- 测试更稳定

---

## 4. Mock对象使用规范

### 何时使用 Mock

- 测试需要调用外部 API（GitHub、Jira 等）
- 测试需要模拟网络请求和响应
- 测试需要避免依赖外部服务
- 测试需要模拟错误情况（网络超时、服务器错误等）

### Mock对象组织规范

```rust
// ✅ 推荐：使用 MockServer 包装器
use crate::common::http_helpers::MockServer;

#[test]
fn test_api_call() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...

    // MockServer 会在 Drop 时自动清理环境变量
}
```

### Mock使用规则

- **每个测试独立 Mock**：每个测试应创建自己的 Mock 服务器实例
- **自动清理**：使用 `MockServer` 包装器，它会自动清理环境变量
- **明确 Mock 范围**：每个 Mock 应明确指定请求方法和路径
- **验证 Mock 调用**：重要测试应验证 Mock 是否被正确调用（使用 `_mock.assert()`）

### 不推荐的用法

```rust
// ❌ 不推荐：手动管理环境变量，容易遗漏清理
env::set_var("GITHUB_API_URL", "http://localhost:1234");
// ... 测试代码 ...
env::remove_var("GITHUB_API_URL");  // 容易忘记

// ❌ 不推荐：在测试之间共享 Mock 服务器
static mut MOCK_SERVER: Option<MockServer> = None;
```

---

## 5. 测试环境工具

项目提供了统一的测试环境工具，基于 `TestIsolation` 构建，提供完全隔离的测试环境。

### 包含工具

- **TestIsolation**：统一测试隔离管理器，提供工作目录、环境变量、Git配置和Mock服务器的隔离
- **CliTestEnv**：CLI测试环境，提供便捷的文件和配置管理
- **GitTestEnv**：Git测试环境，自动初始化Git仓库并配置测试用户

### 快速使用

```rust
use tests::common::environments::CliTestEnv;

#[test]
fn test_cli_command() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    env.create_file("test.txt", "content")?;
    Ok(())
}
```

### 详细文档

更多详细信息和使用示例，请参考：
- [测试环境工具指南](./environments.md) - 完整的使用指南和API参考

---

## 6. 测试辅助工具

项目提供了测试辅助工具，简化测试代码编写。

### 包含工具

- **CliCommandBuilder**：CLI命令测试构建器，提供流畅的API
- **TestDataGenerator**：测试数据生成器，减少样板代码

### 快速使用

```rust
use tests::common::CliCommandBuilder;

#[test]
fn test_cli_command() {
    CliCommandBuilder::new()
        .arg("config")
        .arg("show")
        .assert_success();
}
```

### 详细文档

更多详细信息和使用示例，请参考：
- [测试辅助工具指南](./helpers.md) - 完整的使用指南和API参考

---

## 相关文档

- [测试环境工具指南](./environments.md) - 测试环境工具详细使用方法
- [测试辅助工具指南](./helpers.md) - 测试辅助工具详细使用方法
- [Mock服务器使用指南](./mock-server.md) - Mock服务器详细使用方法
- [测试编写规范](../writing.md) - 测试编写规范

---

**最后更新**: 2025-12-25

---

## 📝 变更历史

### 2025-12-25
- **移除 `insta` 快照测试**：不再需要额外的 JSON 结构验证
- **移除 `jsonschema` 依赖**：JSON 结构验证由 `extract_content()` 方法通过 serde 反序列化自动完成
- **原因**：`extract_content()` 方法使用 serde 反序列化验证 JSON 结构，额外的 Schema 验证是冗余的
- **新增测试环境工具文档**：添加 `environments.md` 和 `helpers.md` 文档

