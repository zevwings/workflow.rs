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
    fn test_parse_url() {
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
│   ├── llm_client.rs
│   ├── logger.rs
│   ├── settings.rs
│   ├── util_dialog.rs
│   └── util_platform.rs
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
│   ├── body_parser.rs
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
│   ├── sample_github_pr.json
│   ├── sample_jira_response.json
│   └── sample_pr_body.md
├── integration/        # 集成测试
│   ├── mod.rs
│   └── workflow.rs
└── integration_test.rs # 集成测试入口
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
src/lib/pr/body_parser.rs     → tests/pr/body_parser.rs
src/lib/completion/config.rs  → tests/completion/config.rs
```

### 不推荐的命名

- ❌ `lib_base_logger.rs` - 包含不必要的前缀
- ❌ `logger_test.rs` - 不够清晰，无法反映模块路径
- ❌ `logger.rs` - 可能与源代码混淆

---

## 🛠️ 共享测试工具

### 使用 common 模块

共享的测试工具应放在 `tests/common/` 目录：

```rust
// tests/common/mod.rs
pub mod helpers;

// tests/common/helpers.rs
pub fn setup_test_env() {
    // 设置测试环境
}

pub fn create_test_client() -> HttpClient {
    // 创建测试客户端
}
```

### 在测试中使用

```rust
// tests/base/http.rs
mod common;
use common::helpers::{setup_test_env, create_test_client};

#[test]
fn test_http_client() {
    setup_test_env();
    let client = create_test_client();
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
    ├── sample_github_pr.json
    ├── sample_jira_response.json
    └── sample_pr_body.md
```

### 使用 Fixtures

```rust
// tests/pr/github.rs
use std::fs;

#[test]
fn test_parse_pr_response() {
    let data = fs::read_to_string("tests/fixtures/sample_github_pr.json")
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
mod get_request {
    #[test]
    fn test_success() {}

    #[test]
    fn test_timeout() {}
}

mod post_request {
    #[test]
    fn test_success() {}
}
```

### 3. 测试函数命名

- 使用描述性的测试名称
- 使用 `test_` 前缀或 `#[test]` 属性
- 测试名称应说明测试的内容和预期结果

```rust
#[test]
fn test_parse_url_with_valid_input() {
    // ...
}

#[test]
fn test_parse_url_with_invalid_input() {
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
pub mod util_dialog;
pub mod util_platform;
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
fn test_parse_ticket_id() {
    // Arrange
    let input = "PROJ-123";

    // Act
    let result = parse_ticket_id(input);

    // Assert
    assert_eq!(result, Some("PROJ-123"));
}
```

### 2. 错误处理测试

为错误情况编写测试：

```rust
#[test]
fn test_parse_ticket_id_invalid() {
    assert_eq!(parse_ticket_id("invalid"), None);
    assert_eq!(parse_ticket_id(""), None);
}
```

### 3. 边界条件测试

测试边界条件和极端情况：

```rust
#[test]
fn test_parse_ticket_id_boundary() {
    // 最小长度
    assert_eq!(parse_ticket_id("A-1"), Some("A-1"));
    // 最大长度
    assert_eq!(parse_ticket_id("VERY-LONG-PROJECT-NAME-123"), Some("VERY-LONG-PROJECT-NAME-123"));
}
```

---

## 🔗 相关文档

- [开发规范](./DEVELOPMENT_GUIDELINES.md) - 包含测试规范的基础内容
- [PR 平台指南](./PR_PLATFORM_GUIDELINES.md) - PR 平台测试相关指南

---

## 📚 参考资源

- [The Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust API Guidelines - Testing](https://rust-lang.github.io/api-guidelines/documentation.html#c-test)
- [Cargo Book - Tests](https://doc.rust-lang.org/cargo/guide/tests.html)

---

**最后更新**: 2025-12-09
