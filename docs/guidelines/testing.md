# 测试规范指南

## 📋 概述

本文档定义了 Workflow CLI 项目的测试组织规范、命名约定和最佳实践。

---

## 🎯 测试类型

### 1. 单元测试 (Unit Tests)

- **位置**：与源代码在同一文件中
- **特点**：测试私有函数，快速执行
- **组织方式**：使用 `#[cfg(test)]` 模块

```rust
// src/lib/base/http.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test-_parse-_url() {
        // 测试私有函数
    }
}
```

### 2. 集成测试 (Integration Tests)

- **位置**：`tests/` 目录
- **特点**：测试公共 API，独立编译
- **组织方式**：使用目录结构组织

---

## 📁 测试组织结构

### 当前测试结构

本项目采用**目录结构**（Directory Structure）组织测试：

```
tests/
├── base/              # Base 模块测试
│   ├── mod.rs
│   ├── llm-_client.rs
│   ├── logger.rs
│   ├── settings.rs
│   ├── util-_dialog.rs
│   └── util-_platform.rs
├── cli/                # CLI 命令层测试
│   ├── mod.rs
│   ├── github.rs
│   ├── jira.rs
│   ├── llm.rs
│   ├── log.rs
│   ├── pr.rs
│   └── proxy.rs
├── completion/         # Completion 模块测试
│   ├── mod.rs
│   ├── completeness.rs
│   ├── config.rs
│   ├── generate.rs
│   └── helpers.rs
├── git/                # Git 模块测试
│   └── mod.rs
├── jira/               # Jira 模块测试
│   ├── mod.rs
│   ├── history.rs
│   ├── logs.rs
│   └── status.rs
├── pr/                 # PR 模块测试
│   ├── mod.rs
│   ├── body-_parser.rs
│   ├── github.rs
│   └── table.rs
├── proxy/              # Proxy 模块测试
│   └── mod.rs
├── rollback/           # Rollback 模块测试
│   └── mod.rs
├── common/             # 共享测试工具
│   ├── mod.rs
│   └── helpers.rs
├── fixtures/           # 测试数据
│   ├── .gitkeep
│   ├── sample-_github-_pr.json
│   ├── sample-_jira-_response.json
│   └── sample-_pr-_body.md
├── integration/        # 集成测试
│   ├── mod.rs
│   └── workflow.rs
└── integration-_test.rs # 集成测试入口
```

### 结构说明

- **模块对应**：测试目录结构与源代码模块结构对应
- **每个目录**：包含 `mod.rs` 文件用于模块声明
- **共享工具**：`common/` 目录存放共享的测试辅助函数
- **测试数据**：`fixtures/` 目录存放测试用的示例数据
- **集成测试**：`integration/` 目录存放端到端测试

---

## 📝 测试文件命名约定

### 命名规则

1. **反映模块路径**：测试文件名应反映对应的源代码模块路径
2. **使用下划线分隔**：使用下划线（`_`）分隔路径组件
3. **保持简洁**：避免不必要的 `lib_` 前缀

### 命名示例

```rust
// 源代码路径 → 测试文件路径
src/lib/base/http.rs          → tests/base/http.rs
src/lib/base/logger.rs        → tests/base/logger.rs
src/lib/pr/body-_parser.rs     → tests/pr/body-_parser.rs
src/lib/completion/config.rs  → tests/completion/config.rs
```

### 不推荐的命名

- ❌ `lib-_base-_logger.rs` - 包含不必要的前缀
- ❌ `logger-_test.rs` - 不够清晰，无法反映模块路径
- ❌ `logger.rs` - 可能与源代码混淆

---

## 🛠️ 共享测试工具

### 使用 common 模块

共享的测试工具应放在 `tests/common/` 目录：

```rust
// tests/common/mod.rs
pub mod helpers;

// tests/common/helpers.rs
pub fn setup-_test-_env() {
    // 设置测试环境
}

pub fn create-_test-_client() -> HttpClient {
    // 创建测试客户端
}
```

### 在测试中使用

```rust
// tests/base/http.rs
mod common;
use common::helpers::{setup-_test-_env, create-_test-_client};

#[test]
fn test-_http-_client() {
    setup-_test-_env();
    let client = create-_test-_client();
    // ...
}
```

---

## 📦 测试数据管理

### Fixtures 目录

测试数据应放在 `tests/fixtures/` 目录：

```
tests/
└── fixtures/
    ├── sample-_github-_pr.json
    ├── sample-_jira-_response.json
    └── sample-_pr-_body.md
```

### 使用 Fixtures

```rust
// tests/pr/github.rs
use std::fs;

#[test]
fn test-_parse-_pr-_response() {
    let data = fs::read-_to-_string("tests/fixtures/sample-_github-_pr.json")
        .expect("Failed to read fixture");
    // 使用测试数据
}
```

---

## 📋 测试组织最佳实践

### 1. 单元测试 vs 集成测试

- **单元测试**：放在源代码文件中，测试私有函数和内部逻辑
- **集成测试**：放在 `tests/` 目录，测试公共 API 和模块间交互

### 2. 测试分组

使用模块组织相关测试：

```rust
// tests/base/http.rs
mod get-_request {
    #[test]
    fn test-_success() {}

    #[test]
    fn test-_timeout() {}
}

mod post-_request {
    #[test]
    fn test-_success() {}
}
```

### 3. 测试函数命名

- 使用描述性的测试名称
- 使用 `test_` 前缀或 `#[test]` 属性
- 测试名称应说明测试的内容和预期结果

```rust
#[test]
fn test-_parse-_url-_with-_valid-_input() {
    // ...
}

#[test]
fn test-_parse-_url-_with-_invalid-_input() {
    // ...
}
```

### 4. 模块声明

每个测试目录应包含 `mod.rs` 文件：

```rust
// tests/base/mod.rs
pub mod http;
pub mod logger;
pub mod settings;
pub mod util-_dialog;
pub mod util-_platform;
```

---

## 🎯 测试覆盖率

### 覆盖率目标

- **总体覆盖率**：> 80%
- **关键业务逻辑**：> 90%
- **工具函数**：> 70%

### 覆盖率检查

使用 `cargo tarpaulin` 检查覆盖率：

```bash
# 安装
cargo install cargo-tarpaulin

# 运行覆盖率检查
cargo tarpaulin --out Html
```

---

## ✅ 测试编写规范

### 1. 测试结构

每个测试应包含：
- **Arrange**：准备测试数据和环境
- **Act**：执行被测试的功能
- **Assert**：验证结果

```rust
#[test]
fn test-_parse-_ticket-_id() {
    // Arrange
    let input = "PROJ-123";

    // Act
    let result = parse-_ticket-_id(input);

    // Assert
    assert-_eq!(result, Some("PROJ-123"));
}
```

### 2. 错误处理测试

为错误情况编写测试：

```rust
#[test]
fn test-_parse-_ticket-_id-_invalid() {
    assert-_eq!(parse-_ticket-_id("invalid"), None);
    assert-_eq!(parse-_ticket-_id(""), None);
}
```

### 3. 边界条件测试

测试边界条件和极端情况：

```rust
#[test]
fn test-_parse-_ticket-_id-_boundary() {
    // 最小长度
    assert-_eq!(parse-_ticket-_id("A-1"), Some("A-1"));
    // 最大长度
    assert-_eq!(parse-_ticket-_id("VERY-LONG-PROJECT-NAME-123"), Some("VERY-LONG-PROJECT-NAME-123"));
}
```

---

## 🛠️ 测试工具

### 1. pretty_assertions

`pretty_assertions` 提供更清晰的断言输出，显示彩色 diff。

**使用方式**：

```rust
use pretty_assertions::assert-_eq;

#[test]
fn test-_example() {
    let actual = "Hello";
    let expected = "World";
    assert-_eq!(actual, expected);  // 会显示清晰的彩色 diff
}
```

**效果**：失败时会显示清晰的彩色 diff，更容易定位问题。

### 2. rstest

`rstest` 支持参数化测试和 fixtures，减少代码重复。

**参数化测试**：

```rust
use rstest::rstest;

#[rstest]
#[case("input1", "output1")]
#[case("input2", "output2")]
#[case("input3", "output3")]
fn test-_multiple-_cases(#[case] input: &str, #[case] expected: &str) {
    let result = process(input);
    assert-_eq!(result, expected);
}
```

**Fixtures**：

```rust
use rstest::{fixture, rstest};

#[fixture]
fn sample-_data() -> Vec<i32> {
    vec![1, 2, 3, 4, 5]
}

#[rstest]
fn test-_with-_fixture(sample-_data: Vec<i32>) {
    assert-_eq!(sample-_data.len(), 5);
}
```

**优势**：
- 减少代码重复
- 更容易添加新的测试用例
- 测试用例更清晰

### 3. insta

`insta` 提供快照测试功能，特别适合测试 JSON 响应和复杂数据结构。

**使用方式**：

```rust
use insta::assert-_json-_snapshot;

#[test]
fn test-_json-_response() {
    let json = json!({
        "id": 123,
        "name": "Test",
        "data": [1, 2, 3]
    });

    // 首次运行会创建快照文件
    // 后续运行会与快照对比
    assert-_json-_snapshot!("test-_response", json);
}
```

**快照管理**：

```bash
# 首次运行会创建快照文件
cargo test

# 如果快照需要更新
INSTA_UPDATE=1 cargo test

# 或者使用 cargo-insta
cargo install cargo-insta
cargo insta review
```

**快照文件位置**：`tests/__snapshots__/` 或 `tests/{module}/snapshots/`

**注意事项**：
- 快照文件需要提交到版本控制
- 更新快照时要谨慎，确保变更是正确的
- 适合测试稳定的数据结构

### 4. mockito

`mockito` 用于 HTTP API 的 Mock 测试，避免实际调用外部 API。

**使用方式**：

```rust
use crate::common::http-_helpers::MockServer;
use mockito::Matcher;

#[test]
fn test-_api-_call() {
    let mut mock-_server = MockServer::new();
    mock-_server.setup-_github-_base-_url();

    // 创建 Mock
    let _mock = mock-_server
        .server
        .as-_mut()
        .mock("GET", "/api/endpoint")
        .match-_header("authorization", Matcher::Regex(r"token .+".to-_string()))
        .with-_status(200)
        .with-_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // let result = client.call-_api()?;
    // assert-_eq!(result, "success");

    // 验证 Mock 被调用
    // _mock.assert();
}
```

**MockServer 工具**：

`tests/common/http-_helpers.rs` 提供了 `MockServer` 包装器，简化 Mock 服务器的使用：

```rust
use crate::common::http-_helpers::MockServer;

let mut mock-_server = MockServer::new();
mock-_server.setup-_github-_base-_url();  // 设置 GitHub API Mock
mock-_server.setup-_jira-_base-_url();    // 设置 Jira API Mock
// MockServer 会在 Drop 时自动清理环境变量
```

**优势**：
- 不依赖外部 API
- 测试执行速度快
- 可以模拟各种错误情况
- 测试更稳定

---

## 🔗 相关文档

- [开发规范](./development.md) - 包含测试规范的基础内容
- [测试用例检查指南](./reviews/review-test-case.md) - 如何检查测试用例的覆盖情况、合理性和完整性
- [PR 平台指南](./PR_PLATFORM_GUIDELINES.md) - PR 平台测试相关指南
- [测试迁移指南](../requirements/TESTING_MIGRATION_GUIDE.md) - 详细的测试工具迁移指南

---

## 📚 参考资源

- [The Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust API Guidelines - Testing](https://rust-lang.github.io/api-guidelines/documentation.html#c-test)
- [Cargo Book - Tests](https://doc.rust-lang.org/cargo/guide/tests.html)
- [pretty_assertions 文档](https://docs.rs/pretty_assertions/)
- [rstest 文档](https://docs.rs/rstest/)
- [insta 文档](https://docs.rs/insta/)
- [mockito 文档](https://docs.rs/mockito/)

---

**最后更新**: 2025-12-12
