# 测试组织规范

> 本文档定义测试组织结构、命名约定和共享工具使用规范。

---

## 📋 目录

- [测试类型](#-测试类型)
- [测试组织结构](#-测试组织结构)
- [测试文件命名约定](#-测试文件命名约定)
- [共享测试工具](#-共享测试工具)
- [测试数据管理](#-测试数据管理)
- [测试组织最佳实践](#-测试组织最佳实践)

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
├── cli/               # CLI 命令层测试
│   ├── mod.rs
│   ├── github.rs
│   ├── jira.rs
│   ├── llm.rs
│   ├── log.rs
│   ├── pr.rs
│   └── proxy.rs
├── completion/        # Completion 模块测试
│   ├── mod.rs
│   ├── completeness.rs
│   ├── config.rs
│   ├── generate.rs
│   └── helpers.rs
├── git/               # Git 模块测试
│   └── mod.rs
├── jira/              # Jira 模块测试
│   ├── mod.rs
│   ├── history.rs
│   ├── logs.rs
│   └── status.rs
├── pr/                # PR 模块测试
│   ├── mod.rs
│   ├── body_parser.rs
│   ├── github.rs
│   └── table.rs
├── proxy/             # Proxy 模块测试
│   └── mod.rs
├── rollback/          # Rollback 模块测试
│   └── mod.rs
├── common/            # 共享测试工具
│   ├── mod.rs
│   └── helpers.rs
├── fixtures/          # 测试数据
│   ├── .gitkeep
│   ├── sample_github_pr.json
│   ├── sample_jira_response.json
│   └── sample_pr_body.md
├── integration/       # 集成测试
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

### tests/common 目录结构

共享的测试工具应放在 `tests/common/` 目录。该目录采用模块化组织，按功能分类：

```
tests/common/
├── environments/          # 测试环境封装
│   ├── cli_test_env.rs   # CLI 测试环境
│   └── git_test_env.rs   # Git 测试环境
├── guards/               # 守卫模式实现
│   ├── env_guard.rs      # 环境变量守卫
│   └── git_config_guard.rs # Git 配置守卫
├── mock/                 # Mock 相关模块
│   ├── server.rs         # MockServer 核心实现
│   ├── templates.rs      # Mock 模板系统
│   ├── validators.rs     # Mock 请求验证器
│   └── scenarios.rs      # Mock 场景预设库
├── test_data/            # 测试数据管理模块
│   ├── factory.rs        # TestDataFactory 核心
│   ├── cache.rs          # 测试数据缓存
│   ├── cleanup.rs        # 测试数据清理
│   └── version.rs        # 测试数据版本管理
├── isolation.rs          # 测试隔离管理器（TestIsolation）
├── fixtures.rs           # 测试 Fixtures
├── helpers.rs            # 通用辅助函数
├── cli_helpers.rs        # CLI 辅助函数
├── macros.rs             # 测试辅助宏
├── validators.rs         # 数据验证器
├── cache.rs              # 缓存工具
├── performance.rs        # 性能测量工具
├── reporter.rs           # 测试报告生成器
├── snapshot.rs           # 测试环境快照
└── integration_examples.rs # 集成示例
```

### 核心模块说明

#### 1. 测试环境模块 (`environments/`)

提供测试环境的封装：

- **`CliTestEnv`**: CLI 测试环境，提供 CLI 命令测试辅助
- **`GitTestEnv`**: Git 测试环境，自动初始化 Git 仓库，提供分支和提交操作

#### 2. Mock 模块 (`mock/`)

提供 HTTP Mock 功能：

- **`MockServer`**: HTTP Mock 服务器核心实现，支持 GitHub/Jira API Mock
- **`templates`**: Mock 响应模板系统，支持变量替换和路径参数
- **`scenarios`**: Mock 场景预设库，支持从文件加载预设场景
- **`validators`**: Mock 请求验证器，验证请求参数和格式

#### 3. 测试数据模块 (`test_data/`)

提供测试数据生成和管理：

- **`TestDataFactory`**: 测试数据工厂，使用 Builder 模式生成测试数据
- **`cache`**: 测试数据缓存，提高测试性能
- **`cleanup`**: 测试数据清理，自动清理测试数据
- **`version`**: 测试数据版本管理，管理测试数据版本

#### 4. 测试隔离 (`isolation.rs`)

**`TestIsolation`**: 统一测试隔离管理器，提供：
- 独立工作目录（使用绝对路径，避免竞态条件）
- 环境变量隔离（EnvGuard）
- Git 配置隔离（GitConfigGuard）
- Mock 服务器集成
- RAII 模式自动清理

### 使用示例

#### 使用 TestIsolation

```rust
use tests::common::TestIsolation;

#[test]
fn test_with_isolation() -> color_eyre::Result<()> {
    let isolation = TestIsolation::new()?
        .with_git_config()?
        .with_mock_server()?;

    let work_dir = isolation.work_dir(); // 绝对路径
    // 测试代码...
    Ok(())
    // 自动清理
}
```

#### 使用 MockServer

```rust
use tests::common::mock::MockServer;
use std::collections::HashMap;

#[test]
fn test_mock_server() -> Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    let mut vars = HashMap::new();
    vars.insert("pr_number".to_string(), "123".to_string());

    mock_server.mock_with_template(
        "GET",
        "/repos/{owner}/repo/pulls/{pr_number}",
        r#"{"number": {{pr_number}}}"#,
        vars,
        200,
    );
    // 测试代码...
    Ok(())
}
```

#### 使用 TestDataFactory

```rust
use tests::common::test_data::TestDataFactory;

#[test]
fn test_data_factory() -> Result<()> {
    let factory = TestDataFactory::new();
    let pr = factory.github_pr()
        .number(123)
        .title("Test PR")
        .build()?;
    // 测试代码...
    Ok(())
}
```

#### 使用 GitTestEnv

```rust
use tests::common::environments::GitTestEnv;

#[test]
fn test_git_env() -> Result<()> {
    let env = GitTestEnv::new()?;
    env.create_file("test.txt", "content")?;
    env.git_add("test.txt")?;
    env.git_commit("Initial commit")?;
    // 测试代码...
    Ok(())
}
```

### 模块导入路径

拆分后的模块导入路径：

```rust
// Mock 相关
use tests::common::mock::MockServer;

// 测试数据相关
use tests::common::test_data::TestDataFactory;

// 测试环境相关
use tests::common::environments::{CliTestEnv, GitTestEnv};

// 测试隔离
use tests::common::TestIsolation;

// 其他工具
use tests::common::{helpers, fixtures, macros};
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
- **CLI 命令层**：> 75%

### 覆盖率检查

使用 `cargo tarpaulin` 检查覆盖率：

```bash
# 安装
cargo install cargo-tarpaulin

# 运行覆盖率检查
cargo tarpaulin --out Html
```

---

## 相关文档

- [测试编写规范](./writing.md) - 测试编写的具体规范
- [测试命令参考](./commands.md) - 常用测试命令
- [测试工具指南](./references/tools.md) - 测试工具使用指南

---

**最后更新**: 2025-12-25

