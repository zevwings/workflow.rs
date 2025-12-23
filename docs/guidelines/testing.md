# 测试规范指南

> 本文档定义了 Workflow CLI 项目的测试组织规范、命名约定和最佳实践。

---

## 📋 目录

- [概述](#-概述)
- [测试类型](#-测试类型)
- [测试组织结构](#-测试组织结构)
- [测试文件命名约定](#-测试文件命名约定)
- [共享测试工具](#-共享测试工具)
- [测试数据管理](#-测试数据管理)
- [测试组织最佳实践](#-测试组织最佳实践)
- [测试覆盖率](#-测试覆盖率)
- [测试编写规范](#-测试编写规范)
- [测试工具](#-测试工具)
- [Mock 对象使用规范](#5-mock-对象使用规范)
- [测试数据清理规则](#-测试数据清理规则)
- [集成测试环境配置](#-集成测试环境配置)
- [集成测试数据隔离](#-集成测试数据隔离)
- [集成测试清理机制](#-集成测试清理机制)
- [测试性能要求](#-测试性能要求)
- [相关文档](#-相关文档)
- [参考资源](#-参考资源)

---

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

### 5. Mock 对象使用规范

**何时使用 Mock**：
- 测试需要调用外部 API（GitHub、Jira 等）
- 测试需要模拟网络请求和响应
- 测试需要避免依赖外部服务
- 测试需要模拟错误情况（网络超时、服务器错误等）

**Mock 对象组织规范**：

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

**Mock 使用规则**：
- **每个测试独立 Mock**：每个测试应创建自己的 Mock 服务器实例
- **自动清理**：使用 `MockServer` 包装器，它会自动清理环境变量
- **明确 Mock 范围**：每个 Mock 应明确指定请求方法和路径
- **验证 Mock 调用**：重要测试应验证 Mock 是否被正确调用（使用 `_mock.assert()`）

**不推荐的用法**：

```rust
// ❌ 不推荐：手动管理环境变量，容易遗漏清理
env::set_var("GITHUB_API_URL", "http://localhost:1234");
// ... 测试代码 ...
env::remove_var("GITHUB_API_URL");  // 容易忘记

// ❌ 不推荐：在测试之间共享 Mock 服务器
static mut MOCK_SERVER: Option<MockServer> = None;
```

---

## 🧹 测试数据清理规则

### 清理原则

1. **自动清理优先**：使用实现了 `Drop` trait 的类型自动清理资源
2. **测试隔离**：每个测试应使用独立的临时目录和数据
3. **失败时也清理**：即使测试失败，也应清理临时资源

### 临时文件管理

**使用 TempManager**：

```rust
use tests::utils::temp::TempManager;

#[test]
fn test_file_operations() -> Result<()> {
    let mut temp_manager = TempManager::new()?;

    // 创建临时文件
    let file_path = temp_manager.create_file("test.txt", "content")?;

    // 使用文件进行测试
    // ...

    // TempManager 会在 Drop 时自动清理所有临时文件
    Ok(())
}
```

**使用临时目录**：

```rust
use tests::common::helpers::{create_temp_test_dir, cleanup_temp_test_dir};

#[test]
fn test_directory_operations() {
    let test_dir = create_temp_test_dir("my_test");

    // 使用目录进行测试
    // ...

    // 手动清理（如果测试失败，可能需要手动清理）
    cleanup_temp_test_dir(&test_dir);
}
```

**使用 tempfile**：

```rust
use tempfile::TempDir;

#[test]
fn test_with_tempdir() {
    let temp_dir = TempDir::new().unwrap();

    // 使用临时目录进行测试
    // TempDir 会在 Drop 时自动删除目录
}
```

### 环境变量清理

**使用 MockServer**（自动清理）：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_with_mock() {
    let mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // MockServer 会在 Drop 时自动清理环境变量
}
```

**手动清理**（不推荐，仅在必要时使用）：

```rust
#[test]
fn test_with_env() {
    let original = env::var("TEST_VAR").ok();
    env::set_var("TEST_VAR", "test_value");

    // 使用环境变量进行测试
    // ...

    // 恢复原始值
    if let Some(val) = original {
        env::set_var("TEST_VAR", val);
    } else {
        env::remove_var("TEST_VAR");
    }
}
```

### 清理规则总结

- ✅ **优先使用**：实现了 `Drop` trait 的类型（`TempManager`、`TempDir`、`MockServer`）
- ✅ **测试隔离**：每个测试使用独立的临时目录和数据
- ✅ **清理时机**：在测试结束时自动清理，无需手动干预
- ❌ **避免**：在测试之间共享临时资源
- ❌ **避免**：手动管理环境变量（优先使用包装器）

---

## 🔧 集成测试环境配置

### 环境初始化

**使用 setup_test_env**：

```rust
use tests::common::helpers::setup_test_env;

#[test]
fn test_with_env_setup() {
    setup_test_env();  // 只会执行一次，即使多次调用

    // 测试代码
    // ...
}
```

**环境变量设置**：

- `RUST_LOG`：设置日志级别（测试中通常设置为 `debug`）
- `GITHUB_API_URL`：GitHub API Mock 地址（由 `MockServer` 设置）
- `JIRA_API_URL`：Jira API Mock 地址（由 `MockServer` 设置）

### CLI 测试环境

**使用 CliTestEnv**：

```rust
use tests::common::cli_helpers::CliTestEnv;

#[test]
fn test_cli_command() {
    let env = CliTestEnv::new()
        .init_git_repo()
        .create_file("test.txt", "content")
        .create_config("[jira]\nurl = \"https://test.atlassian.net\"");

    // 执行 CLI 命令测试
    // ...

    // CliTestEnv 会在 Drop 时自动清理临时目录
}
```

**CLI 测试环境功能**：
- 创建临时目录
- 初始化 Git 仓库
- 创建测试文件
- 创建配置文件
- 创建 Git 提交

### 环境配置最佳实践

- **每个测试独立环境**：每个测试应创建自己的测试环境
- **使用包装器**：优先使用 `CliTestEnv`、`MockServer` 等包装器
- **环境隔离**：确保测试之间不会相互影响
- **自动清理**：使用实现了 `Drop` trait 的类型自动清理

---

## 🔒 集成测试数据隔离

### 隔离原则

1. **独立数据**：每个测试使用独立的数据和资源
2. **唯一标识**：使用时间戳和随机字符串确保唯一性
3. **临时资源**：所有测试数据应存储在临时目录中

### 数据隔离实现

**临时目录隔离**：

```rust
use tests::common::helpers::create_temp_test_dir;

#[test]
fn test_isolation() {
    // 每个测试创建唯一的临时目录
    let test_dir = create_temp_test_dir("test_name");
    // 目录名格式：workflow_test_{prefix}_{timestamp}_{random}

    // 测试代码使用独立的目录
    // ...
}
```

**Git 仓库隔离**：

```rust
use tests::common::cli_helpers::CliTestEnv;

#[test]
fn test_git_operations() {
    let env = CliTestEnv::new().init_git_repo();

    // 每个测试有独立的 Git 仓库
    // 不会影响其他测试
    // ...
}
```

**Mock 服务器隔离**：

```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_api_call() {
    // 每个测试创建独立的 Mock 服务器
    let mut mock_server = MockServer::new();

    // Mock 服务器使用不同的端口
    // 不会与其他测试冲突
    // ...
}
```

### 隔离检查清单

- ✅ 每个测试使用独立的临时目录
- ✅ 每个测试创建独立的 Mock 服务器
- ✅ 每个测试使用独立的 Git 仓库（如需要）
- ✅ 测试之间不共享全局状态
- ✅ 测试之间不共享环境变量（使用 MockServer 自动管理）
- ❌ 避免在测试之间共享文件系统资源
- ❌ 避免在测试之间共享网络资源

---

## 🧼 集成测试清理机制

### 自动清理机制

**Drop trait 自动清理**：

项目中多个类型实现了 `Drop` trait，确保资源自动清理：

```rust
// MockServer - 自动清理环境变量
impl Drop for MockServer {
    fn drop(&mut self) {
        self.cleanup();  // 清理 GITHUB_API_URL、JIRA_API_URL
    }
}

// TempManager - 自动清理临时文件
impl Drop for TempManager {
    fn drop(&mut self) {
        let _ = self.cleanup_all_files();  // 清理所有临时文件
    }
}

// TempDir (tempfile) - 自动清理临时目录
// TempDir 实现了 Drop，会自动删除临时目录
```

**使用示例**：

```rust
#[test]
fn test_with_auto_cleanup() {
    // 创建资源
    let mock_server = MockServer::new();
    let mut temp_manager = TempManager::new().unwrap();

    // 使用资源进行测试
    // ...

    // 测试结束时，Drop trait 会自动清理：
    // - MockServer 清理环境变量
    // - TempManager 清理临时文件
    // - TempDir 清理临时目录
}
```

### 手动清理机制

**显式清理**（仅在必要时使用）：

```rust
use tests::common::helpers::cleanup_temp_test_dir;

#[test]
fn test_with_manual_cleanup() {
    let test_dir = create_temp_test_dir("test");

    // 测试代码
    // ...

    // 手动清理（如果测试失败，可能需要手动清理）
    cleanup_temp_test_dir(&test_dir);
}
```

**清理函数**：

- `cleanup_test_env()`：清理测试环境（当前为空实现）
- `cleanup_temp_test_dir(dir)`：清理临时测试目录
- `TempManager::cleanup_all_files()`：清理所有临时文件
- `MockServer::cleanup()`：清理环境变量

### 清理最佳实践

- ✅ **优先使用自动清理**：使用实现了 `Drop` trait 的类型
- ✅ **测试失败时也清理**：确保测试失败时资源也能被清理
- ✅ **清理顺序**：先清理文件，再清理目录，最后清理环境变量
- ❌ **避免手动管理**：除非必要，避免手动管理资源清理
- ❌ **避免全局状态**：避免使用全局状态，难以清理

---

## ⚡ 测试性能要求

### 测试执行时间要求

**单元测试**：
- **单个测试**：< 100ms
- **模块测试套件**：< 1s
- **所有单元测试**：< 10s

**集成测试**：
- **单个测试**：< 1s
- **模块测试套件**：< 10s
- **所有集成测试**：< 60s

**性能测试**：
- 使用 `#[ignore]` 标记长时间运行的测试
- 使用 `cargo test -- --ignored` 运行性能测试

```rust
#[test]
#[ignore]  // 标记为忽略，默认不运行
fn test_performance() {
    // 长时间运行的性能测试
    // ...
}
```

### 测试资源使用限制

**内存使用**：
- **单个测试**：< 100MB
- **测试套件**：< 500MB

**文件系统**：
- **临时文件**：使用临时目录，测试结束后自动清理
- **文件大小**：单个测试文件 < 10MB
- **文件数量**：单个测试创建的文件 < 100 个

**网络资源**：
- **Mock 服务器**：使用 Mock 服务器，避免实际网络请求
- **并发连接**：单个测试的并发连接 < 10 个

### 性能优化建议

- ✅ **使用 Mock**：避免实际网络请求，使用 Mock 服务器
- ✅ **并行执行**：Rust 测试默认并行执行，确保测试之间相互独立
- ✅ **减少 I/O**：减少文件系统操作，使用内存数据结构
- ✅ **避免阻塞**：避免长时间阻塞操作
- ❌ **避免实际 API 调用**：集成测试也应使用 Mock 服务器
- ❌ **避免大文件**：测试数据应尽量小，使用有代表性的样本

### 性能检查

**检查测试执行时间**：

```bash
# 运行测试并显示执行时间
cargo test -- --nocapture --test-threads=1

# 使用 cargo-nextest（如果已安装）
cargo nextest run
```

**检查资源使用**：

```bash
# 使用 time 命令检查执行时间
time cargo test

# 使用 valgrind 检查内存使用（Linux）
valgrind --leak-check=full cargo test
```

---

## 🔗 相关文档

- [开发规范索引](./development/README.md) - 开发规范总览
- [测试用例检查指南](./development/references/review-test-case.md) - 如何检查测试用例的覆盖情况、合理性和完整性
- [PR 平台指南](./pr-platform.md) - PR 平台测试相关指南
- 测试迁移指南文档已移除

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

**最后更新**: 2025-01-27
