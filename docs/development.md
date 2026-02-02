# 开发规范

> Workflow CLI 项目的开发规范和最佳实践

---

## 📋 目录

- [开发环境设置](#-开发环境设置)
- [代码风格规范](#-代码风格规范)
- [错误处理规范](#-错误处理规范)
- [命名规范](#-命名规范)
- [模块组织规范](#-模块组织规范)
- [文档规范](#-文档规范)
- [提交规范](#-提交规范)
- [检查流程](#-检查流程)
- [开发工具使用](#-开发工具使用)

---

## 🛠️ 开发环境设置

### 工具安装

首次开发前，运行以下命令安装所需的开发工具：

```bash
make setup
```

这会自动安装：
- `rustfmt` - 代码格式化工具
- `clippy` - 代码检查工具
- `rust-analyzer` - 语言服务器（从源码构建）
- `cargo-bloat` - 二进制大小分析工具

> **注意**：如果您的平台没有预编译的 rust-analyzer 二进制文件，`make setup` 会自动从源码构建安装。这可能需要几分钟时间。

### 开发工具说明

| 工具 | 用途 | 命令 |
|------|------|------|
| `rustfmt` | 代码格式化 | `cargo fmt` |
| `clippy` | 代码质量检查 | `cargo clippy` |
| `rust-analyzer` | IDE 语言服务器 | 自动集成到编辑器 |
| `cargo-bloat` | 二进制大小分析 | `make bloat` |

### 验证安装

安装完成后，可以运行以下命令验证工具是否已正确安装：

```bash
# 检查 rustfmt
cargo fmt --version

# 检查 clippy
cargo clippy --version

# 检查 rust-analyzer
rust-analyzer --version

# 检查 cargo-bloat
cargo bloat --version
```

---

## 🎨 代码风格规范

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

---

## ⚠️ 错误处理规范

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
use anyhow::{Context, Result};

pub fn parse_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    toml::from_str(&content)
        .context("Failed to parse TOML config")?;
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

---

## 🏷️ 命名规范

### 文件命名

- **模块文件**：`snake_case.rs`（如 `jira_client.rs`、`pr_helpers.rs`）
- **测试文件**：与源文件同名，放在 `tests/` 目录或使用 `#[cfg(test)]` 模块
- **文档文件**：参考文档使用 `kebab-case.md`（如 `architecture.md`、`development.md`）

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
pub const MAX_DOWNLOAD_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

### CLI 参数命名

- **结构体字段名**：使用 `snake_case`（如 `jira_id`、`dry_run`）
- **value_name**：使用 `SCREAMING_SNAKE_CASE`（如 `JIRA_ID`、`DRY_RUN`）
- **参数长名**：使用 `kebab-case`（clap 自动从字段名转换，如 `--jira-id`）
- **参数短名**：使用单个字符（如 `-n`、`-f`）

对于在多个命令中重复使用的参数，应该提取为共用参数组：

```rust
// crates/app/src/cli/args.rs
#[derive(Args, Debug, Clone)]
pub struct JiraIdArg {
    /// Jira ticket ID
    #[arg(value_name = "JIRA_ID")]
    pub jira_id: Option<String>,
}

// 在命令中使用
#[command(flatten)]
jira_id: JiraIdArg,
```

---

## 📁 模块组织规范

### 目录结构（v2 workspace）

遵循 workspace 多 crate 结构：

```
crates/
├── app/                 # CLI 入口与命令
│   ├── src/bin/         # workflow、install 二进制
│   ├── cli/             # 参数与子命令定义
│   ├── commands/        # 命令实现
│   └── workflows/       # 工作流编排
├── domain/              # 领域模型与仓储 trait
├── storage/             # Git/CNB 等存储实现
├── services/            # 应用服务
├── toolkit/             # HTTP、日志、路径、模板等
├── prompt/              # 交互与输出
└── registry/             # 依赖注入
```

### 模块职责

- **`app`**：CLI 入口、命令封装与工作流编排
- **`domain`**：领域实体与仓储接口
- **`storage`** / **`services`**：存储与业务实现
- **`toolkit`** / **`prompt`**：通用能力与交互

### 模块依赖规则

- **app** 依赖 domain、storage、services、toolkit、prompt、registry
- **domain** 不依赖 app、storage 的具体实现
- **storage** 实现 domain 中的仓储接口

---

## 📝 文档规范

### 代码文档注释

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
/// 参见各 crate 的公共 API（如 `app::commands::*`、`domain::*`）。
pub fn download_from_jira(&self, ticket_id: &str) -> Result<Vec<u8>> {
    // 实现
}
```

### 文档注释格式

- 使用 `///` 为公共项添加文档
- 使用 `//!` 为模块添加文档
- 包含参数说明、返回值说明、错误说明、使用示例

### 文档命名规范

- **参考文档**：使用 `kebab-case.md`（如 `architecture.md`、`development.md`、`testing.md`）
- **临时文档**：不做要求，可随意命名

### 文档格式规范

- Markdown 基本格式
- 代码块格式
- 列表格式

### 文档维护

- 代码变更时同步更新文档注释
- 运行 `cargo doc --open` 查看生成的文档

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
```

### 提交信息要求

- **主题行**：不超过 50 个字符，使用祈使语气
- **正文**：详细说明变更原因和方式，每行不超过 72 个字符
- **页脚**：引用相关 issue（如 `Closes #123`）

---

## 🔍 检查流程

### 提交前检查（pre-commit）

提交代码前需要完成的检查（5-15 分钟）：

#### 核心检查（必做）

```bash
# 1. 代码格式化检查
cargo fmt --check

# 2. Clippy 检查
cargo clippy -- -D warnings

# 3. 测试执行
cargo test

# 4. 编译检查
cargo check

# 5. 运行测试
cargo test
```

#### 一键检查命令

```bash
# 快速检查
make lint && make test
```

#### 版本检查（如需要）

如果提交包含新功能（feat）或 bug 修复（fix），需要检查：

```bash
# 检查版本号是否已更新（v2 在 [workspace.package] 下）
grep -A1 "\[workspace.package\]" Cargo.toml | grep version

# 检查 CHANGELOG.md 是否已更新
head -30 CHANGELOG.md
```

### 综合检查（review）

功能完成后、定期审查、重大重构前、发布前需要进行的综合检查（1-2 小时）：

#### 基础检查

```bash
# 确保代码可以编译
cargo check

# 确保测试通过
cargo test

# 确保格式化通过
cargo fmt --check

# 确保 Clippy 通过
cargo clippy -- -D warnings
```

#### 代码质量检查

- 检查代码重复
- 检查工具函数复用
- 检查配置管理统一性

#### 测试覆盖检查

- 检查测试覆盖情况
- 检查测试用例合理性
- 识别缺失测试

---

## 🔧 开发工具使用

### Makefile 命令

项目提供了 Makefile 来简化常用开发任务。详细命令列表请运行 `make help` 查看。

### CI/CD 配置

项目使用 GitHub Actions 进行持续集成和发布。

#### CI 流程（`.github/workflows/ci.yml`）

**触发条件**：
- Pull Request 到 `master` 或 `main` 分支
- 手动触发（`workflow_dispatch`）

**检查步骤**：
1. **代码格式化检查**：`cargo fmt --check`
2. **Clippy 检查**：`cargo clippy -- -D warnings`
3. **测试执行**：`cargo test`
4. **多平台构建验证**：在多个平台上验证代码可以编译

**平台支持**：
- Ubuntu Latest
- macOS Latest
- Windows Latest

#### 发布流程（`.github/workflows/release.yml`）

**触发条件**：
- 代码合并到 `master` 分支
- 手动触发（`workflow_dispatch`）

**发布步骤**：
1. **代码质量检查**：格式化、Clippy、测试
2. **自动创建 Tag**：根据 `Cargo.toml` 中的版本号创建 tag
3. **多平台构建**：为多个平台构建 release 二进制文件
4. **创建 Release**：在 GitHub 上创建 Release，并上传构建产物
5. **更新 Homebrew Formula**：自动更新 `homebrew-workflow` 仓库中的 Formula 文件

**平台支持**：
- Linux (x86_64, ARM64)
- macOS (x86_64, ARM64)
- Windows (x86_64)

**配置要求**：
- 需要配置 `HOMEBREW_TAP_TOKEN` secret 用于更新 Homebrew Formula
- 需要配置 `WORKFLOW_USER_NAME` secret 用于创建版本更新 PR

**详细配置**：
- CI 配置：`.github/workflows/ci.yml`
- 发布配置：`.github/workflows/release.yml`

---

## 📚 相关文档

- [架构设计](./architecture.md) - 项目整体架构设计
- [测试规范](./testing.md) - 测试组织、编写、命令参考
- [迁移文档](./migration/README.md) - 版本迁移指南

**API 文档**：运行 `cargo doc --open` 查看完整的 API 文档。

---

**最后更新**: 2025-01-27
