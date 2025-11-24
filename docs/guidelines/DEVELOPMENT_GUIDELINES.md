# 开发规范文档

> 本文档定义了 Workflow CLI 项目的开发规范和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [代码风格](#-代码风格)
- [错误处理](#-错误处理)
- [文档规范](#-文档规范)
- [命名规范](#-命名规范)
- [模块组织](#-模块组织)
- [Git 工作流](#-git-工作流)
- [提交规范](#-提交规范)
- [测试规范](#-测试规范)
- [代码审查](#-代码审查)
- [依赖管理](#-依赖管理)
- [开发工具](#-开发工具)

---

## 🎨 代码风格

### 代码格式化

所有代码必须使用 `rustfmt` 进行格式化：

```bash
# 自动格式化代码
cargo fmt

# 检查代码格式（CI/CD 中使用）
cargo fmt --check
```

**规则**：
- 提交前必须运行 `cargo fmt`
- CI/CD 会检查代码格式，格式不正确会导致构建失败
- 使用默认的 `rustfmt` 配置（项目根目录的 `rustfmt.toml` 如果存在）

### Lint 检查

使用 `clippy` 进行代码质量检查：

```bash
# 运行 Clippy 检查
cargo clippy -- -D warnings

# 或使用 Makefile
make lint
```

**规则**：
- 所有警告必须修复（`-D warnings` 会将警告视为错误）
- 禁止使用 `#[allow(clippy::xxx)]` 除非有充分理由，并添加注释说明
- 定期运行 `cargo clippy` 检查代码质量

### Rust 命名约定

遵循 Rust 官方命名约定：

- **模块名**：`snake_case`（如 `jira_logs`、`pr_helpers`）
- **函数名**：`snake_case`（如 `download_logs`、`create_pr`）
- **变量名**：`snake_case`（如 `api_token`、`response_data`）
- **常量名**：`SCREAMING_SNAKE_CASE`（如 `MAX_RETRIES`、`DEFAULT_TIMEOUT`）
- **类型名**：`PascalCase`（如 `HttpClient`、`JiraTicket`）
- **Trait 名**：`PascalCase`（如 `PlatformProvider`、`ResponseParser`）
- **枚举变体**：`PascalCase`（如 `GitHub`、`Codeup`）

### 代码组织

#### 导入顺序

1. 标准库导入
2. 第三方库导入
3. 项目内部导入

```rust
// 标准库
use std::path::PathBuf;
use std::fs;

// 第三方库
use anyhow::Result;
use serde::Deserialize;

// 项目内部
use crate::base::http::HttpClient;
use crate::jira::client::JiraClient;
```

#### 模块声明

- 使用 `mod.rs` 文件管理模块声明
- 按功能分组组织模块
- 使用 `pub use` 重新导出常用的公共 API

```rust
// src/lib/jira/mod.rs
mod client;
mod config;
mod ticket;

pub use client::JiraClient;
pub use ticket::JiraTicket;
```

---

## ⚠️ 错误处理

### 错误类型

统一使用 `anyhow::Result<T>` 作为函数返回类型：

```rust
use anyhow::Result;

pub fn download_logs(ticket_id: &str) -> Result<Vec<u8>> {
    // 实现
}
```

### 错误信息

提供清晰、有上下文的错误信息：

```rust
// ✅ 好的做法
use anyhow::{Context, Result};

pub fn parse_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    toml::from_str(&content)
        .context("Failed to parse TOML config")?;
}

// ❌ 不好的做法
pub fn parse_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?;  // 错误信息不清晰
    toml::from_str(&content)?;
}
```

### 错误处理模式

#### 1. 使用 `Context` 添加上下文

```rust
use anyhow::{Context, Result};

let result = operation()
    .with_context(|| format!("Failed to perform operation with id: {}", id))?;
```

#### 2. 使用 `bail!` 快速返回错误

```rust
use anyhow::{bail, Result};

if value < 0 {
    bail!("Value must be non-negative, got: {}", value);
}
```

#### 3. 使用 `ensure!` 进行断言

```rust
use anyhow::{ensure, Result};

ensure!(
    status_code < 400,
    "HTTP request failed with status: {}",
    status_code
);
```

### 分层错误处理

不同层级使用不同的错误处理策略：

1. **CLI 层**：参数验证错误，使用 `clap` 自动处理
2. **命令层**：用户交互错误、业务逻辑错误，提供友好的错误提示
3. **库层**：底层操作错误（文件、网络、API），提供详细的错误信息

```rust
// 命令层：提供友好的错误提示
pub fn download_command(ticket_id: Option<&str>) -> Result<()> {
    let id = ticket_id
        .map(|s| s.to_string())
        .or_else(|| {
            Input::new()
                .with_prompt("Enter JIRA ticket ID")
                .interact_text()
                .ok()
        })
        .ok_or_else(|| anyhow::anyhow!("JIRA ticket ID is required"))?;

    // 调用库层，传递详细错误
    JiraLogs::new()?.download_from_jira(&id)?;
    Ok(())
}
```

---

## 📝 文档规范

### 公共 API 文档

所有公共函数、结构体、枚举、Trait 必须添加文档注释：

```rust
/// 下载指定 JIRA ticket 的日志文件
///
/// # 参数
///
/// * `ticket_id` - JIRA ticket ID（如 "PROJ-123"）
///
/// # 返回
///
/// 返回下载的日志文件字节数据
///
/// # 错误
///
/// 如果下载失败，返回错误信息
///
/// # 示例
///
/// ```rust
/// use workflow::jira::logs::JiraLogs;
///
/// let logs = JiraLogs::new()?;
/// let data = logs.download_from_jira("PROJ-123")?;
/// ```
pub fn download_from_jira(&self, ticket_id: &str) -> Result<Vec<u8>> {
    // 实现
}
```

### 文档注释格式

- 使用 `///` 为公共项添加文档
- 使用 `//!` 为模块添加文档
- 包含参数说明、返回值说明、错误说明、使用示例

### 内部文档

对于复杂的实现逻辑，添加内部注释：

```rust
// 使用指数退避策略进行重试
// 初始延迟 1 秒，每次重试延迟翻倍，最大延迟 60 秒
let delay = (1 << retry_count).min(60);
```

---

## 🏷️ 命名规范

### 文件命名

- **模块文件**：`snake_case.rs`（如 `jira_client.rs`、`pr_helpers.rs`）
- **测试文件**：与源文件同名，放在 `tests/` 目录或使用 `#[cfg(test)]` 模块

### 函数命名

- **动作函数**：使用动词（如 `download`、`create`、`merge`）
- **查询函数**：使用 `get_` 前缀（如 `get_status`、`get_info`）
- **检查函数**：使用 `is_` 或 `has_` 前缀（如 `is_valid`、`has_permission`）
- **转换函数**：使用 `to_` 或 `into_` 前缀（如 `to_string`、`into_json`）

### 结构体命名

- 使用名词或名词短语（如 `HttpClient`、`JiraTicket`）
- 避免使用 `Data`、`Info`、`Manager` 等泛化名称，使用具体名称

### 常量命名

- 使用 `SCREAMING_SNAKE_CASE`
- 放在模块顶层或专门的常量模块中

```rust
// src/lib/jira/logs/constants.rs
pub const MAX_DOWNLOAD_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

---

## 📁 模块组织

### 目录结构

遵循项目的三层架构：

```
src/
├── main.rs              # CLI 入口
├── lib.rs               # 库入口
├── bin/                 # 独立可执行文件
│   └── install.rs
├── commands/            # 命令封装层
│   ├── pr/
│   ├── log/
│   └── ...
└── lib/                 # 核心业务逻辑层
    ├── base/           # 基础模块
    ├── pr/             # PR 模块
    ├── jira/           # Jira 模块
    └── ...
```

### 模块职责

- **`commands/`**：CLI 命令封装，处理用户交互、参数解析
- **`lib/`**：核心业务逻辑，可复用的功能模块
- **`bin/`**：独立的可执行文件入口

### 模块依赖规则

- **命令层** → **库层**：命令层可以依赖库层，但不能反向依赖
- **库层内部**：可以相互依赖，但避免循环依赖
- **基础模块**：`lib/base/` 不依赖其他业务模块

---

## 🔀 Git 工作流

### 分支策略

- **`master`**：主分支，保持稳定，只接受合并请求
- **`feature/*`**：功能分支，从 `master` 创建，完成后合并回 `master`
- **`fix/*`**：修复分支，从 `master` 创建，用于修复 bug
- **`hotfix/*`**：热修复分支，用于紧急修复生产问题

### 分支命名

- 功能分支：`feature/jira-attachments`
- 修复分支：`fix/pr-merge-error`
- 热修复分支：`hotfix/critical-bug`

### 工作流程

1. **创建分支**：从 `master` 创建新分支
2. **开发**：在分支上进行开发
3. **提交**：遵循提交规范（见下方）
4. **推送**：推送到远程仓库
5. **创建 PR**：创建 Pull Request 到 `master`
6. **代码审查**：等待代码审查
7. **合并**：审查通过后合并到 `master`

---

## 📋 提交规范

### Conventional Commits

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 提交类型

- **`feat`**：新功能
- **`fix`**：修复 bug
- **`docs`**：文档更新
- **`style`**：代码格式调整（不影响功能）
- **`refactor`**：代码重构
- **`test`**：测试相关
- **`chore`**：构建过程或辅助工具的变动
- **`perf`**：性能优化
- **`ci`**：CI/CD 配置变更

### 提交示例

```bash
# 功能提交
feat(jira): add attachments download command

Add new command to download all attachments from a JIRA ticket.
The command supports filtering by file type and size.

Closes #123

# 修复提交
fix(pr): handle merge conflict error

Fix the issue where PR merge fails silently when there's a merge conflict.
Now the command will display a clear error message.

Fixes #456

# 文档提交
docs: update development guidelines

Add error handling best practices section.

# 重构提交
refactor(http): simplify retry logic

Extract retry logic into a separate module for better maintainability.
```

### 提交信息要求

- **主题行**：不超过 50 个字符，使用祈使语气
- **正文**：详细说明变更原因和方式，每行不超过 72 个字符
- **页脚**：引用相关 issue（如 `Closes #123`）

---

## 🧪 测试规范

### 单元测试

为所有公共函数编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ticket_id() {
        assert_eq!(parse_ticket_id("PROJ-123"), Some("PROJ-123"));
        assert_eq!(parse_ticket_id("invalid"), None);
    }
}
```

### 测试组织

- 测试模块放在源文件底部，使用 `#[cfg(test)]`
- 测试函数使用 `test_` 前缀或 `#[test]` 属性
- 使用描述性的测试名称

### 测试覆盖率

- 目标覆盖率：> 80%
- 关键业务逻辑：> 90%
- 使用 `cargo tarpaulin` 检查覆盖率

### 集成测试

对于 CLI 命令，编写集成测试：

```rust
// tests/integration_test.rs
#[test]
fn test_pr_create_command() {
    // 测试 CLI 命令
}
```

---

## 👀 代码审查

### 审查清单

提交 PR 前，确保：

- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy`）
- [ ] 所有测试通过（`cargo test`）
- [ ] 添加了必要的文档注释
- [ ] 遵循了错误处理规范
- [ ] 提交信息符合规范
- [ ] 没有引入新的警告

### 审查重点

- **功能正确性**：代码是否实现了预期功能
- **代码质量**：是否遵循了代码风格和最佳实践
- **错误处理**：是否正确处理了错误情况
- **性能**：是否有性能问题
- **安全性**：是否有安全漏洞
- **可维护性**：代码是否易于理解和维护

---

## 📦 依赖管理

### 添加依赖

使用 `cargo add` 添加依赖：

```bash
# 添加依赖
cargo add serde --features derive

# 添加开发依赖
cargo add --dev mockito
```

### 依赖原则

- **最小化依赖**：只添加必要的依赖
- **版本管理**：使用语义化版本，避免使用 `*` 通配符
- **功能标志**：使用 feature flags 控制可选功能
- **定期更新**：定期更新依赖到最新稳定版本

### 依赖审查

添加新依赖前，考虑：

- 是否真的需要这个依赖？
- 是否有更轻量的替代方案？
- 依赖的维护状态如何？
- 依赖的许可证是否兼容？

---

## 🛠️ 开发工具

### 必需工具

安装开发工具：

```bash
make setup
```

这会安装：
- `rustfmt` - 代码格式化
- `clippy` - 代码检查
- `rust-analyzer` - 语言服务器

### 常用命令

```bash
# 构建
cargo build
make release

# 测试
cargo test
make test

# 代码检查
cargo fmt
cargo clippy
make lint

# 运行 CLI
cargo run -- --help
```

### IDE 配置

推荐使用支持 Rust 的 IDE：
- **VS Code** + rust-analyzer 扩展
- **IntelliJ IDEA** + Rust 插件
- **CLion** + Rust 插件

### 预提交钩子

建议配置 Git 预提交钩子，自动运行代码检查：

```bash
# .git/hooks/pre-commit
#!/bin/sh
cargo fmt --check && cargo clippy -- -D warnings
```

---

## 📚 相关文档

- [文档编写指南](./DOCUMENT_GUIDELINES.md) - 架构文档编写规范
- [主架构文档](../architecture/ARCHITECTURE.md) - 项目总体架构
- [Rust 官方文档](https://doc.rust-lang.org/) - Rust 语言文档
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/) - Rust API 设计指南

---

## 🔄 更新记录

### v1.0.0 (2024-12)

- 初始版本
- 包含代码风格、错误处理、文档、命名、模块组织、Git 工作流、提交、测试、代码审查、依赖管理等规范

---

*最后更新：2024-12*

