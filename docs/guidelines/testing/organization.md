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
- **测试对象**：**仅测试私有函数和内部逻辑**
- **特点**：快速执行，最小依赖
- **组织方式**：使用 `#[cfg(test)]` 模块

```rust
// src/lib/git/auth.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_username_from_url() {
        // 测试私有方法 extract_username_from_url
        assert_eq!(
            GitAuth::extract_username_from_url("https://user@github.com/owner/repo.git"),
            Some("user")
        );
    }
}
```

**重要规则**：
- ✅ **只测试 private 方法和内部逻辑**
- ❌ **不测试 public 方法**（public 方法测试应放在 `tests/` 目录）
- ✅ 快速执行，不依赖外部环境
- ✅ 只测试模块内部的实现细节

### 2. 集成测试 (Integration Tests)

- **位置**：`tests/` 目录
- **测试对象**：**public API 和模块间交互**
- **特点**：测试公共 API，独立编译
- **组织方式**：使用目录结构组织

```rust
// tests/base/format/message.rs
use workflow::base::format::MessageFormatter;

#[test]
fn test_error_formatting() {
    // 测试 public 方法 MessageFormatter::error
    let msg = MessageFormatter::error("read", "config.toml", "Permission denied");
    assert_eq!(msg, "Failed to read config.toml: Permission denied");
}
```

**重要规则**：
- ✅ **测试所有 public 方法**
- ✅ **测试模块间交互和集成场景**
- ✅ 可以使用测试环境、Mock 服务器等工具
- ✅ 验证公共 API 的完整功能

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

**严格的测试组织规则**：

- **单元测试（`src/` 中的 `#[cfg(test)]`）**：
  - ✅ **只测试私有函数和内部逻辑**
  - ❌ **不测试 public 方法**
  - ✅ 快速执行，最小依赖
  - ✅ 只测试模块内部的实现细节

  ```rust
  // src/lib/git/auth.rs
  #[cfg(test)]
  mod tests {
      #[test]
      fn test_extract_username_from_url() {
          // ✅ 测试私有方法
      }
  }
  ```

- **集成测试（`tests/` 目录）**：
  - ✅ **测试所有 public 方法**
  - ✅ **测试模块间交互和集成场景**
  - ✅ 可以使用测试环境、Mock 服务器等工具
  - ✅ 独立编译，验证公共 API

  ```rust
  // tests/base/format/message.rs
  #[test]
  fn test_error_formatting() {
      // ✅ 测试 public 方法
      let msg = MessageFormatter::error("read", "config.toml", "Permission denied");
      assert_eq!(msg, "Failed to read config.toml: Permission denied");
  }
  ```

**为什么要严格分离？**

1. **清晰的测试边界**：单元测试只关注内部实现，集成测试只关注公共接口
2. **更好的封装**：public 方法的测试在 `tests/` 中，不依赖内部实现细节
3. **独立编译**：集成测试作为独立的 crate 编译，确保 public API 的正确性
4. **测试覆盖率**：避免在两个地方重复测试 public 方法
5. **重构友好**：重构内部实现时，只需更新 `src/` 中的单元测试

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

## 📦 测试迁移指南

如果你在 `src/` 中发现了测试 public 方法的测试，请按以下步骤迁移：

### 迁移步骤

1. **识别 public 方法测试**：
   - 检查测试是否调用了 public 方法（如 `pub fn method_name()`）
   - 如果测试只调用 private 方法，应保留在 `src/` 中

2. **创建对应的测试文件**：
   - 在 `tests/` 目录中创建对应的测试文件
   - 例如：`src/lib/base/format/message.rs` → `tests/base/format/message.rs`

3. **迁移测试代码**：
   - 将 public 方法测试复制到 `tests/` 目录
   - 更新 import 语句（使用 `use workflow::...`）
   - 添加适当的测试文档注释

4. **清理源文件**：
   - 从 `src/` 中删除已迁移的 public 方法测试
   - 保留 private 方法测试
   - 添加注释说明：`// 注意：public 方法的测试已迁移到 tests/...`

5. **更新模块声明**：
   - 在 `tests/*/mod.rs` 中添加新测试模块的声明
   - 例如：`pub mod message;`

6. **验证测试**：
   - 运行 `cargo test` 确保所有测试通过
   - 检查测试覆盖率没有下降

### 迁移示例

```rust
// ❌ 错误：在 src/ 中测试 public 方法
// src/lib/base/format/message.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_error_formatting() {
        let msg = MessageFormatter::error("read", "file", "error");
        assert_eq!(msg, "Failed to read file: error");
    }
}

// ✅ 正确：在 tests/ 中测试 public 方法
// tests/base/format/message.rs
use workflow::base::format::MessageFormatter;

#[test]
fn test_error_formatting() {
    let msg = MessageFormatter::error("read", "file", "error");
    assert_eq!(msg, "Failed to read file: error");
}

// ✅ 正确：在 src/ 中测试 private 方法
// src/lib/git/auth.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_username_from_url() {
        // 测试私有方法 extract_username_from_url
        assert_eq!(
            GitAuth::extract_username_from_url("https://user@github.com/repo.git"),
            Some("user")
        );
    }
}
```

---

## 相关文档

- [测试编写规范](./writing.md) - 测试编写的具体规范
- [测试命令参考](./commands.md) - 常用测试命令
- [测试工具指南](./references/tools.md) - 测试工具使用指南

---

**最后更新**: 2025-12-30

