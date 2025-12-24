# 测试规范指南

> 本文档定义了 Workflow CLI 项目的测试组织规范、命名约定和最佳实践。

---

## 📋 目录

- [概述](#-概述)
- [基本测试命令](#-基本测试命令)
- [测试类型](#-测试类型)
- [测试组织结构](#-测试组织结构)
- [测试文件命名约定](#-测试文件命名约定)
- [共享测试工具](#-共享测试工具)
- [测试数据管理](#-测试数据管理)
- [测试组织最佳实践](#-测试组织最佳实践)
- [测试覆盖率](#-测试覆盖率)
- [覆盖率测试指南](#-覆盖率测试指南)
- [测试编写规范](#-测试编写规范)
- [编写测试最佳实践](#-编写测试最佳实践)
- [测试工具](#-测试工具)
- [Mock 对象使用规范](#5-mock-对象使用规范)
- [测试数据管理最佳实践](#-测试数据管理最佳实践)
- [Mock 服务器使用指南](#-mock-服务器使用指南)
- [测试数据清理规则](#-测试数据清理规则)
- [集成测试环境配置](#-集成测试环境配置)
- [集成测试数据隔离](#-集成测试数据隔离)
- [集成测试清理机制](#-集成测试清理机制)
- [测试性能要求](#-测试性能要求)
- [性能测试指南](#-性能测试指南)
- [覆盖率提升技巧](#-覆盖率提升技巧)
- [相关文档](#-相关文档)
- [参考资源](#-参考资源)

---

## 📋 概述

本文档定义了 Workflow CLI 项目的测试组织规范、命名约定和最佳实践。

---

## 🚀 基本测试命令

### 运行测试

**运行所有测试**：
```bash
# 使用 Cargo
cargo test

# 使用 Makefile
make test
```

**运行特定测试**：
```bash
# 运行特定模块的测试
cargo test --lib 模块名

# 运行特定测试文件
cargo test --test 测试文件名

# 运行匹配模式的测试
cargo test test_parse_url

# 运行被忽略的测试
cargo test -- --ignored

# 运行所有测试（包括被忽略的）
make test-all
```

**测试输出选项**：
```bash
# 显示详细输出
cargo test -- --nocapture

# 显示测试执行时间
cargo test -- --nocapture --test-threads=1

# 只运行失败的测试（需要先运行一次）
cargo test -- --failed
```

### 测试类型命令

**单元测试**：
```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块的单元测试
cargo test --lib 模块名::函数名
```

**集成测试**：
```bash
# 运行所有集成测试
cargo test --test '*'

# 运行特定集成测试
cargo test --test integration_test
```

**文档测试**：
```bash
# 运行文档中的代码示例（doctest）
cargo test --doc

# 运行特定模块的文档测试
cargo test --doc 模块名
```

### Makefile 测试命令

项目提供了便捷的 Makefile 命令：

```bash
# 运行所有测试
make test

# 运行所有测试（包括被忽略的）
make test-all

# 生成覆盖率报告
make coverage

# 打开覆盖率报告
make coverage-open

# CI 环境覆盖率检查
make coverage-ci

# 查看覆盖率趋势
make coverage-trend
```

### 测试调试

**运行单个测试**：
```bash
# 运行单个测试函数
cargo test test_parse_url -- --nocapture

# 运行单个测试并显示详细输出
cargo test test_parse_url -- --nocapture --test-threads=1
```

**测试失败时调试**：
```bash
# 显示失败的测试输出
cargo test -- --nocapture

# 只运行失败的测试
cargo test -- --failed
```

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

## 📊 覆盖率测试指南

### 安装覆盖率工具

**cargo-tarpaulin**（推荐）：
```bash
# 安装 cargo-tarpaulin
cargo install cargo-tarpaulin

# 验证安装
cargo tarpaulin --version
```

### 生成覆盖率报告

**基本用法**：
```bash
# 生成 HTML 格式的覆盖率报告
make coverage
# 或
cargo tarpaulin --out Html --output-dir coverage \
    --exclude-files "src/bin/*" \
    --exclude-files "tests/*" \
    --exclude-files "benches/*" \
    --exclude-files "src/*/mod.rs"
```

**查看报告**：
```bash
# 打开覆盖率报告
make coverage-open
# 或手动打开
open coverage/tarpaulin-report.html
```

### CI 环境覆盖率检查

**生成 Lcov 格式报告**（适合 CI/CD）：
```bash
# CI 环境覆盖率检查
make coverage-ci
# 或
cargo tarpaulin --out Lcov --output-dir coverage
```

**覆盖率阈值检查**：
```bash
# 设置覆盖率阈值（例如 80%）
cargo tarpaulin --out Lcov --output-dir coverage --fail-under 80
```

### 覆盖率分析

**查看覆盖率趋势**：
```bash
# 查看覆盖率趋势（需要历史数据）
make coverage-trend
```

**排除文件**：
```bash
# 排除特定文件或目录
cargo tarpaulin --out Html \
    --exclude-files "src/bin/*" \
    --exclude-files "tests/*" \
    --exclude-files "src/*/mod.rs"
```

### 覆盖率报告解读

**HTML 报告**：
- **绿色**：已覆盖的代码行
- **红色**：未覆盖的代码行
- **黄色**：部分覆盖的代码行（条件分支）
- **覆盖率百分比**：显示每个文件和模块的覆盖率

**覆盖率指标**：
- **行覆盖率**：执行的代码行数 / 总代码行数
- **分支覆盖率**：执行的分支数 / 总分支数
- **函数覆盖率**：执行的函数数 / 总函数数

### 覆盖率提升策略

1. **识别低覆盖率模块**：查看报告，找出覆盖率低于目标的模块
2. **优先测试关键路径**：确保关键业务逻辑有充分的测试覆盖
3. **补充边界测试**：为边界条件和错误处理添加测试
4. **定期检查**：在每次功能开发后检查覆盖率变化

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

## ✍️ 编写测试最佳实践

### 1. 测试命名规范

**描述性命名**：
- ✅ 使用描述性的测试名称，说明测试的内容和预期结果
- ✅ 使用 `test_` 前缀或 `#[test]` 属性
- ✅ 测试名称应包含：被测试的功能、输入条件、预期结果

```rust
// ✅ 好的命名
#[test]
fn test_parse_ticket_id_with_valid_input() {}

#[test]
fn test_parse_ticket_id_with_invalid_input_returns_none() {}

// ❌ 不好的命名
#[test]
fn test1() {}

#[test]
fn test_parse() {}
```

### 2. 测试结构（AAA 模式）

**Arrange-Act-Assert 模式**：
```rust
#[test]
fn test_example() {
    // Arrange: 准备测试数据和环境
    let input = "PROJ-123";
    let expected = Some("PROJ-123");

    // Act: 执行被测试的功能
    let result = parse_ticket_id(input);

    // Assert: 验证结果
    assert_eq!(result, expected);
}
```

### 3. 测试独立性

**每个测试应独立**：
- ✅ 每个测试应独立运行，不依赖其他测试
- ✅ 每个测试应使用独立的数据和环境
- ✅ 测试之间不应共享状态

```rust
// ✅ 好的做法：每个测试独立
#[test]
fn test_parse_ticket_id_1() {
    let result = parse_ticket_id("PROJ-123");
    assert_eq!(result, Some("PROJ-123"));
}

#[test]
fn test_parse_ticket_id_2() {
    let result = parse_ticket_id("PROJ-456");
    assert_eq!(result, Some("PROJ-456"));
}

// ❌ 不好的做法：测试之间共享状态
static mut COUNTER: i32 = 0;

#[test]
fn test_1() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}

#[test]
fn test_2() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 2);  // 依赖 test_1
}
```

### 4. 测试覆盖原则

**测试覆盖重点**：
- ✅ **成功路径**：测试正常流程
- ✅ **错误路径**：测试错误处理和边界条件
- ✅ **边界条件**：测试边界值和极端情况
- ✅ **集成场景**：测试模块间交互

### 5. 测试数据管理

**使用 Fixtures**：
```rust
// ✅ 使用 fixtures 目录中的测试数据
use std::fs;

#[test]
fn test_parse_pr_response() {
    let data = fs::read_to_string("tests/fixtures/sample_github_pr.json")
        .expect("Failed to read fixture");
    // 使用测试数据
}
```

**使用测试数据工厂**：
```rust
// ✅ 使用测试数据工厂生成测试数据
use tests::common::test_data_factory::TestDataFactory;

#[test]
fn test_with_factory() {
    let pr = TestDataFactory::github_pr()
        .with_id(123)
        .with_title("Test PR")
        .build();
    // 使用生成的测试数据
}
```

### 6. Mock 使用原则

**何时使用 Mock**：
- ✅ 测试需要调用外部 API（GitHub、Jira 等）
- ✅ 测试需要模拟网络请求和响应
- ✅ 测试需要避免依赖外部服务
- ✅ 测试需要模拟错误情况

**Mock 使用规范**：
```rust
// ✅ 使用 MockServer 包装器
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

### 7. 断言最佳实践

**使用清晰的断言**：
```rust
// ✅ 使用描述性的断言消息
assert_eq!(result, expected, "Failed to parse ticket ID: {}", input);

// ✅ 使用专门的断言工具
use pretty_assertions::assert_eq;  // 显示彩色 diff

// ❌ 避免模糊的断言
assert!(result.is_some());  // 不够清晰
```

### 8. 测试文档

**为复杂测试添加注释**：
```rust
#[test]
fn test_complex_scenario() {
    // 测试场景：当用户输入无效的 ticket ID 时，
    // 系统应该返回 None 并记录错误日志

    let input = "INVALID";
    let result = parse_ticket_id(input);

    assert_eq!(result, None);
    // 验证错误日志已记录
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

## 📦 测试数据管理最佳实践

### 1. 测试数据组织

**Fixtures 目录结构**：
```
tests/
└── fixtures/
    ├── templates/              # 测试数据模板
    │   ├── github_pr.json
    │   └── jira_issue.json
    ├── scenarios/              # 测试场景数据
    │   ├── auth_failure.json
    │   └── network_timeout.json
    └── mock_responses/         # Mock 响应数据
        ├── github/
        └── jira/
```

### 2. 测试数据工厂

**使用测试数据工厂生成测试数据**：
```rust
use tests::common::test_data_factory::TestDataFactory;

#[test]
fn test_with_factory() {
    // 使用工厂创建测试数据
    let pr = TestDataFactory::github_pr()
        .with_id(123)
        .with_title("Test PR")
        .with_state("open")
        .build();

    // 使用生成的测试数据
    assert_eq!(pr.id, 123);
    assert_eq!(pr.title, "Test PR");
}
```

### 3. 测试数据复用

**创建可复用的测试数据构建器**：
```rust
// ✅ 创建可复用的构建器
struct GitHubPRBuilder {
    id: u64,
    title: String,
    state: String,
}

impl GitHubPRBuilder {
    fn new() -> Self {
        Self {
            id: 1,
            title: "Default PR".to_string(),
            state: "open".to_string(),
        }
    }

    fn with_id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    fn build(self) -> GitHubPR {
        GitHubPR {
            id: self.id,
            title: self.title,
            state: self.state,
        }
    }
}
```

### 4. 测试数据清理

**自动清理测试数据**：
```rust
// ✅ 使用实现了 Drop trait 的类型自动清理
use tempfile::TempDir;

#[test]
fn test_with_temp_data() {
    let temp_dir = TempDir::new().unwrap();
    // 使用临时目录进行测试
    // TempDir 会在 Drop 时自动删除
}
```

---

## 🔧 Mock 服务器使用指南

### 1. MockServer 基本使用

**创建 Mock 服务器**：
```rust
use crate::common::http_helpers::MockServer;

#[test]
fn test_api_call() {
    // 创建 Mock 服务器
    let mut mock_server = MockServer::new();

    // 设置 API 基础 URL
    mock_server.setup_github_base_url();
    // 或
    mock_server.setup_jira_base_url();
}
```

### 2. 创建 Mock 端点

**基本 Mock 端点**：
```rust
#[test]
fn test_get_request() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建 Mock 端点
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...
}
```

**带条件的 Mock 端点**：
```rust
use mockito::Matcher;

#[test]
fn test_with_conditions() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建带条件的 Mock
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .match_header("authorization", Matcher::Regex(r"token .+".to_string()))
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("key".to_string(), "value".to_string()),
        ]))
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...
}
```

### 3. 模拟错误情况

**模拟网络错误**：
```rust
#[test]
fn test_network_error() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 模拟 500 错误
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(500)
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    // 测试错误处理
    // ...
}
```

**模拟超时**：
```rust
#[test]
fn test_timeout() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 模拟延迟响应（测试超时处理）
    let _mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .with_header("content-type", "application/json")
        .create();

    // 测试超时处理逻辑
    // ...
}
```

### 4. 验证 Mock 调用

**验证 Mock 是否被调用**：
```rust
#[test]
fn test_verify_mock() {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // 创建 Mock 并保存引用
    let mock = mock_server
        .server
        .as_mut()
        .mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();

    // 执行测试
    // ...

    // 验证 Mock 被调用
    mock.assert();
}
```

### 5. Mock 服务器最佳实践

**每个测试独立 Mock**：
```rust
// ✅ 好的做法：每个测试创建独立的 Mock 服务器
#[test]
fn test_1() {
    let mut mock_server = MockServer::new();
    // ...
}

#[test]
fn test_2() {
    let mut mock_server = MockServer::new();
    // ...
}
```

**自动清理**：
```rust
// ✅ MockServer 会在 Drop 时自动清理环境变量
#[test]
fn test_with_auto_cleanup() {
    let mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    // 测试代码
    // MockServer 会在测试结束时自动清理环境变量
}
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

## ⚡ 性能测试指南

### 1. 性能基准测试（Benchmark）

**使用 Criterion 进行性能测试**：
```bash
# 安装 Criterion（如果未安装）
# Criterion 已在 Cargo.toml 中配置为 dev-dependency

# 运行所有基准测试
make bench
# 或
cargo bench

# 运行特定基准测试
make bench-cli        # CLI 性能测试
make bench-core       # 核心操作测试
make bench-network    # 网络操作测试
```

### 2. 创建基准测试

**基准测试文件结构**：
```rust
// benches/cli_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use workflow::commands::pr::create::create_pr;

fn bench_cli_command(c: &mut Criterion) {
    c.bench_function("cli_command", |b| {
        b.iter(|| {
            // 执行被测试的操作
            black_box(create_pr());
        });
    });
}

criterion_group!(benches, bench_cli_command);
criterion_main!(benches);
```

### 3. 性能测试报告

**查看性能报告**：
```bash
# 生成性能报告
make bench-report

# 打开性能报告
make bench-open
```

**性能报告位置**：
- CLI 性能：`target/criterion/cli_performance/index.html`
- 核心操作：`target/criterion/core_operations/index.html`
- 网络操作：`target/criterion/network_operations/index.html`

### 4. 性能对比和回归检测

**性能对比**：
```bash
# 对比当前结果与历史结果
make bench-compare
```

**性能回归检测**：
```bash
# 检测性能回归
make bench-regression
```

**CI 环境性能监控**：
```bash
# CI 环境性能监控
make bench-ci
```

### 5. 性能测试原则

**性能测试最佳实践**：
- ✅ **建立基线**：首次运行基准测试建立性能基线
- ✅ **定期运行**：在每次重要变更后运行性能测试
- ✅ **关注趋势**：关注性能趋势，及时发现性能回归
- ✅ **设置阈值**：为关键操作设置性能阈值
- ✅ **环境一致性**：在相同环境下运行性能测试，确保结果可比较

**性能测试注意事项**：
- ⚠️ **环境差异**：不同环境的性能测试结果可能不同
- ⚠️ **统计波动**：性能测试结果可能有统计波动，需要多次运行
- ⚠️ **资源限制**：注意测试环境的资源限制（CPU、内存等）

---

## 📈 覆盖率提升技巧

### 1. 识别低覆盖率区域

**查看覆盖率报告**：
```bash
# 生成覆盖率报告
make coverage

# 打开报告查看低覆盖率区域
make coverage-open
```

**重点关注**：
- 覆盖率低于目标的模块
- 关键业务逻辑模块
- 错误处理路径

### 2. 补充测试策略

**按优先级补充测试**：
1. **关键业务逻辑**：优先为关键业务逻辑添加测试
2. **错误处理**：为错误处理路径添加测试
3. **边界条件**：为边界条件和极端情况添加测试
4. **集成场景**：为模块间交互添加集成测试

### 3. 测试覆盖技巧

**使用参数化测试**：
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

**使用测试工具**：
```rust
// 使用 pretty_assertions 获得更好的错误信息
use pretty_assertions::assert_eq;

// 使用 insta 进行快照测试
use insta::assert_json_snapshot;
```

### 4. 覆盖率目标

**模块覆盖率目标**：
- **总体覆盖率**：> 80%
- **关键业务逻辑**：> 90%
- **工具函数**：> 70%
- **CLI 命令层**：> 75%

### 5. 持续改进

**定期检查**：
- 每次功能开发后检查覆盖率变化
- 每周检查覆盖率趋势
- 每月进行覆盖率审查

**覆盖率提升流程**：
1. 生成覆盖率报告
2. 识别低覆盖率区域
3. 制定测试补充计划
4. 实施测试补充
5. 验证覆盖率提升

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

**最后更新**: 2025-12-24
