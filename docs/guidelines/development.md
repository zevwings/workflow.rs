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
- [发布前检查](#-发布前检查)
- [定期检查机制](#-定期检查机制)
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

- **模块名**：`snake-_case`（如 `jira-_logs`、`pr-_helpers`）
- **函数名**：`snake-_case`（如 `download-_logs`、`create-_pr`）
- **变量名**：`snake-_case`（如 `api-_token`、`response-_data`）
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

pub fn download-_logs(ticket-_id: &str) -> Result<Vec<u8>> {
    // 实现
}
```

### 错误信息

提供清晰、有上下文的错误信息：

```rust
// ✅ 好的做法
use anyhow::{Context, Result};

pub fn parse-_config(path: &Path) -> Result<Config> {
    let content = fs::read-_to-_string(path)
        .with-_context(|| format!("Failed to read config file: {}", path.display()))?;

    toml::from-_str(&content)
        .context("Failed to parse TOML config")?;
}

// ❌ 不好的做法
pub fn parse-_config(path: &Path) -> Result<Config> {
    let content = fs::read-_to-_string(path)?;  // 错误信息不清晰
    toml::from-_str(&content)?;
}
```

### 错误处理模式

#### 1. 使用 `Context` 添加上下文

```rust
use anyhow::{Context, Result};

let result = operation()
    .with-_context(|| format!("Failed to perform operation with id: {}", id))?;
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
    status-_code < 400,
    "HTTP request failed with status: {}",
    status-_code
);
```

### 分层错误处理

不同层级使用不同的错误处理策略：

1. **CLI 层**：参数验证错误，使用 `clap` 自动处理
2. **命令层**：用户交互错误、业务逻辑错误，提供友好的错误提示
3. **库层**：底层操作错误（文件、网络、API），提供详细的错误信息

```rust
// 命令层：提供友好的错误提示
pub fn download-_command(ticket-_id: Option<&str>) -> Result<()> {
    let id = ticket-_id
        .map(|s| s.to-_string())
        .or-_else(|| {
            Input::new()
                .with-_prompt("Enter JIRA ticket ID")
                .interact-_text()
                .ok()
        })
        .ok-_or-_else(|| anyhow::anyhow!("JIRA ticket ID is required"))?;

    // 调用库层，传递详细错误
    JiraLogs::new()?.download-_from-_jira(&id)?;
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
/// * `ticket-_id` - JIRA ticket ID（如 "PROJ-123"）
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
/// let data = logs.download-_from-_jira("PROJ-123")?;
/// ```
pub fn download-_from-_jira(&self, ticket-_id: &str) -> Result<Vec<u8>> {
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
let delay = (1 << retry-_count).min(60);
```

### 文档同步要求

为确保架构文档与代码实现始终保持一致，所有代码变更必须同步更新相关文档。

#### 基本原则

**核心原则**：代码变更与文档更新必须同步进行，PR 必须包含相关文档的更新。

#### 具体要求

1. **新增模块时**：
   - **必须**同步创建对应的架构文档
   - 文档位置：
     - Lib 层模块 → `docs/architecture/lib/{MODULE}_architecture.md`
     - 命令层模块 → `docs/architecture/commands/{MODULE}_COMMAND_architecture.md`
   - 文档内容：参考 [文档编写指南](./document.md) 创建完整的架构文档
   - 更新索引：更新 `docs/README.md` 中的文档索引（如适用）

2. **重构模块时**：
   - **必须**同步更新对应的架构文档
   - 更新内容：
     - 模块结构变化
     - API 接口变更
     - 功能描述更新
     - 依赖关系变化
   - 验证一致性：使用 [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) 验证文档与代码的一致性

3. **API 变更时**：
   - **必须**更新文档中的接口描述
   - 更新内容：
     - 函数签名变更
     - 参数类型或名称变更
     - 返回值类型变更
     - 错误类型变更
   - 更新位置：对应的架构文档中的"API 接口"或"核心组件"章节

4. **功能变更时**：
   - **必须**更新文档中的功能说明
   - 更新内容：
     - 功能描述更新
     - 使用示例更新（如适用）
     - 配置项说明更新（如适用）
   - 更新位置：对应的架构文档中的"功能描述"章节

5. **配置变更时**：
   - **必须**更新文档中的配置项说明
   - 更新内容：
     - 配置项新增或删除
     - 配置项类型、默认值、必填性变更
   - 更新位置：
     - 架构文档中的配置说明
     - README.md 中的配置说明（如适用）
     - 迁移文档（如有破坏性变更）

6. **命令变更时**：
   - **必须**更新 README.md 中的命令清单
   - 更新内容：
     - 新增命令
     - 命令参数变更
     - 命令行为变更
   - 验证完整性：确保所有命令都在命令清单中列出

#### PR 要求

**重要**：所有 PR 必须包含相关文档的更新。

- **新增模块的 PR**：必须包含新模块的架构文档
- **重构模块的 PR**：必须包含架构文档的更新
- **API 变更的 PR**：必须包含接口描述的更新
- **功能变更的 PR**：必须包含功能说明的更新
- **配置变更的 PR**：必须包含配置说明的更新
- **命令变更的 PR**：必须包含 README.md 的更新

**代码审查时**：审查者应检查 PR 是否包含必要的文档更新（见 [代码审查清单](#-代码审查)）。

#### 文档检查

在提交 PR 前，使用以下方法检查文档是否已同步更新：

1. **使用检查清单**：
   - 参考 [代码审查清单](#-代码审查) 中的"文档更新检查"项
   - 确保所有相关文档都已更新

2. **使用检查指南**：
   - 参考 [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) 进行系统化检查
   - 验证文档与代码的一致性

3. **快速验证**：
   ```bash
   # 检查新增的文件是否有对应的架构文档
   # 检查修改的模块是否有架构文档更新
   git diff master --name-only | grep -E "\.(rs|md)$"
   ```

#### 相关文档

- [文档编写指南](./document.md) - 架构文档编写规范和模板
- [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) - 详细的架构文档检查方法和流程
- [代码审查清单](#-代码审查) - 包含文档更新检查项

---

## 🏷️ 命名规范

### 文件命名

- **模块文件**：`snake-_case.rs`（如 `jira-_client.rs`、`pr-_helpers.rs`）
- **测试文件**：与源文件同名，放在 `tests/` 目录或使用 `#[cfg(test)]` 模块
- **文档文件**：`SCREAMING_SNAKE_CASE.md`（如 `development.md`、`pr.md`）
  - **架构文档**：`{MODULE}_architecture.md`（如 `pr.md`、`git.md`）
  - **命令文档**：`{MODULE}_COMMAND_architecture.md`（如 `pr.md`、`LOG_COMMAND_architecture.md`）
  - **指南文档**：`{TOPIC}_GUIDELINES.md`（如 `development.md`、`document.md`）
  - **需求文档**：`{FEATURE}_REQUIREMENT.md` 或 `{FEATURE}_REQUIREMENTS.md`（如 `GITHUB_BRANCH_PREFIX_REPO_BASED_REQUIREMENT.md`）
  - **待办文档**：`{MODULE}_TODO.md` 或 `{TOPIC}_TODO.md`（如 `jira.md`、`integration.md`）

### 函数命名

- **动作函数**：使用动词（如 `download`、`create`、`merge`）
- **查询函数**：使用 `get_` 前缀（如 `get-_status`、`get-_info`）
- **检查函数**：使用 `is_` 或 `has_` 前缀（如 `is-_valid`、`has-_permission`）
- **转换函数**：使用 `to_` 或 `into_` 前缀（如 `to-_string`、`into-_json`）

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

### CLI 参数命名规范

CLI 参数命名需要遵循以下规范，确保一致性和可维护性。

#### 结构体字段名

- 使用 `snake-_case`（如 `jira-_id`、`dry-_run`、`output-_format`）

```rust
#[derive(Args, Debug, Clone)]
pub struct JiraIdArg {
    pub jira-_id: Option<String>,  // ✅ snake-_case
}
```

#### value-_name 规范

- 使用 `SCREAMING_SNAKE_CASE`（如 `JIRA_ID`、`DRY_RUN`、`PR_ID`）
- 用于在帮助信息中显示参数值的占位符

```rust
/// Jira ticket ID (optional, will prompt interactively if not provided)
#[arg(value-_name = "JIRA_ID")]  // ✅ SCREAMING_SNAKE_CASE
pub jira-_id: Option<String>,
```

#### 参数长名规范

- 使用 `kebab-case`（clap 自动从字段名转换，如 `--jira-id`、`--dry-run`）
- 字段名使用 `snake-_case`，clap 会自动转换为 `kebab-case`

```rust
#[arg(long)]  // 自动生成 --jira-id
pub jira-_id: Option<String>,
```

#### 参数短名规范

- 使用单个字符（如 `-n`、`-f`、`-v`）
- 优先使用常见的短名（如 `-n` 用于 dry-run，`-f` 用于 force）

```rust
#[arg(long, short = 'n', action = clap::ArgAction::SetTrue)]
pub dry-_run: bool,  // --dry-run 或 -n
```

#### 参数类型规范

- **可选参数**：使用 `Option<T>`
- **必需参数**：直接使用类型（如 `String`、`usize`）
- **布尔标志**：使用 `bool` + `action = clap::ArgAction::SetTrue`

```rust
// ✅ 可选参数
#[arg(value-_name = "JIRA_ID")]
pub jira-_id: Option<String>,

// ✅ 必需参数
#[arg(value-_name = "BRANCH_NAME")]
pub branch-_name: String,

// ✅ 布尔标志
#[arg(long, short = 'f', action = clap::ArgAction::SetTrue)]
pub force: bool,
```

#### 文档注释规范

所有参数必须有文档注释，说明参数的用途、格式和默认行为：

```rust
/// Jira ticket ID (optional, will prompt interactively if not provided)
///
/// Examples:
///   workflow jira info PROJ-123
///   workflow jira info  # Will prompt for JIRA ID
#[arg(value-_name = "JIRA_ID")]
pub jira-_id: Option<String>,
```

#### 命名一致性规范

- **相同语义的参数必须使用相同的命名**：
  - ✅ 统一使用 `jira-_id`（而不是 `jira-_ticket`、`jira-id` 等）
  - ✅ 统一使用 `dry-_run`（而不是 `dry-run`、`dryrun` 等）
  - ✅ 统一使用 `output-_format`（而不是 `format`、`output` 等）

- **value-_name 必须与字段名语义一致**：
  - 字段名：`jira-_id` → value-_name：`JIRA_ID`
  - 字段名：`dry-_run` → value-_name：`DRY_RUN`（但通常布尔标志不需要 value-_name）

#### 共用参数规范

对于在多个命令中重复使用的参数，应该提取为共用参数组（见 [CLI 检查指南](./reviews/review-cli.md)）：

```rust
// src/lib/cli/args.rs
/// 可选 JIRA ID 参数
#[derive(Args, Debug, Clone)]
pub struct JiraIdArg {
    /// Jira ticket ID (optional, will prompt interactively if not provided)
    #[arg(value-_name = "JIRA_ID")]
    pub jira-_id: Option<String>,
}

// 在命令中使用
use super::args::JiraIdArg;

#[derive(Subcommand)]
pub enum MySubcommand {
    Info {
        #[command(flatten)]
        jira-_id: JiraIdArg,  // ✅ 使用共用参数
    },
}
```

#### 示例对比

```rust
// ❌ 不好的做法
Create {
    #[arg(value-_name = "jira-_ticket")]  // value-_name 应该大写
    jira-_ticket: Option<String>,  // 命名不一致（应该用 jira-_id）
}

// ✅ 好的做法
Create {
    #[command(flatten)]
    jira-_id: JiraIdArg,  // 使用共用参数，命名一致
}
```

**参考**：
- [CLI 检查指南](./reviews/review-cli.md) - 参数复用检查和参数提取指南
- [clap 文档](https://docs.rs/clap/) - clap 参数定义规范

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

**注意**：Workflow CLI 支持通过模板系统自定义分支命名格式。详细配置方法请参考 [模板配置指南](./TEMPLATE_GUIDELINES.md#分支命名模板-templatebranch)。

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

**注意**：Workflow CLI 支持通过模板系统自定义提交消息格式，包括是否使用 Conventional Commits 格式。详细配置方法请参考 [模板配置指南](./TEMPLATE_GUIDELINES.md#提交消息模板-templatecommit)。

---

## 🧪 测试规范

> **详细测试规范**：请参考 [测试规范指南](./testing.md)

### 单元测试

为所有公共函数编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test-_parse-_ticket-_id() {
        assert-_eq!(parse-_ticket-_id("PROJ-123"), Some("PROJ-123"));
        assert-_eq!(parse-_ticket-_id("invalid"), None);
    }
}
```

### 测试组织

- 测试模块放在源文件底部，使用 `#[cfg(test)]`
- 测试函数使用 `test_` 前缀或 `#[test]` 属性
- 使用描述性的测试名称
- 集成测试使用目录结构组织（详见 [测试规范指南](./testing.md)）

### 测试覆盖率

- 目标覆盖率：> 80%
- 关键业务逻辑：> 90%
- 使用 `cargo tarpaulin` 检查覆盖率

### 集成测试

对于 CLI 命令，编写集成测试：

```rust
// tests/integration-_test.rs
#[test]
fn test-_pr-_create-_command() {
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
- [ ] **文档更新检查**（如适用）：
  - [ ] 新增模块：已创建/更新对应的架构文档（`docs/architecture/`）
  - [ ] 重构模块：已同步更新架构文档
  - [ ] API 变更：已更新文档中的接口描述
  - [ ] 功能变更：已更新文档中的功能说明
  - [ ] 配置变更：已更新文档中的配置项说明
  - [ ] 命令变更：已更新 README.md 中的命令清单
  - [ ] 已更新 CHANGELOG.md（如适用）

### 审查重点

- **功能正确性**：代码是否实现了预期功能
- **代码质量**：是否遵循了代码风格和最佳实践
- **错误处理**：是否正确处理了错误情况
- **性能**：是否有性能问题
- **安全性**：是否有安全漏洞
- **可维护性**：代码是否易于理解和维护
- **文档完整性**：相关文档是否已同步更新（架构文档、README、CHANGELOG）

---

## 🚀 发布前检查

### 检查清单

在发布新版本前，必须完成以下检查：

#### 代码质量检查

- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy -- -D warnings`）
- [ ] 所有测试通过（`cargo test`）
- [ ] 没有引入新的警告
- [ ] 代码审查已完成

#### 版本管理检查

- [ ] `Cargo.toml` 中的版本号已更新
- [ ] `CHANGELOG.md` 已更新，包含本次发布的所有变更
- [ ] 版本号格式符合语义化版本规范（如 `1.6.7`）
- [ ] 变更内容分类正确（Added、Changed、Fixed、Removed 等）

#### 文档检查

**架构文档与代码一致性**：
- [ ] 所有架构文档已与代码实现同步
  - [ ] 使用 [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) 进行全面检查
  - [ ] 检查范围：`docs/architecture/` 目录下的所有文档（约 30+ 个文档）
  - [ ] 模块结构与实际代码结构一致
  - [ ] API 接口描述与代码实现一致
  - [ ] 功能描述与实际实现一致
  - [ ] 配置项说明与代码结构一致
  - [ ] 依赖关系与实际代码依赖一致

**README.md 检查**：
- [ ] README.md 中的命令清单完整
- [ ] 所有命令都有说明和使用示例
- [ ] 版本号与 `Cargo.toml` 一致
- [ ] 所有文档链接有效

**其他文档检查**：
- [ ] CHANGELOG.md 已更新
- [ ] 所有文档链接有效
- [ ] 文档索引（`docs/README.md`）已更新（如适用）

#### 测试检查

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 补全脚本完整性测试通过（`cargo test --test completeness`）
- [ ] 测试覆盖率满足要求（> 80%，关键业务逻辑 > 90%）

#### 构建检查

- [ ] 项目可以成功编译（`cargo build --release`）
- [ ] 所有平台构建通过（如适用）
- [ ] 二进制文件大小合理

### 检查方法

#### 架构文档检查

使用架构文档审查指南进行全面检查：

```bash
# 参考架构文档审查指南
# docs/guidelines/workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md

# 快速检查命令示例：
# 1. 检查模块结构
MODULE=pr
find src/lib/$MODULE -name "*.rs" -type f | grep -v mod.rs | sort

# 2. 统计代码行数
find src/lib/$MODULE -name "*.rs" -type f | xargs wc -l | tail -1

# 3. 提取公共 API
grep -r "pub fn\|pub struct\|pub enum\|pub trait" src/lib/$MODULE/ | head -20
```

#### 文档链接检查

```bash
# 检查文档中的链接（手动检查或使用工具）
# 确保所有内部链接指向的文件存在
# 确保所有外部链接可访问（可选）
```

### 检查要求

- **必须完成**：发布前必须完成所有架构文档检查
- **检查范围**：`docs/architecture/` 目录下的所有文档（约 30+ 个文档）
- **检查工具**：使用 [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) 进行全面检查
- **检查记录**：建议记录检查结果到 `docs/architecture/CHECK_LOG.md`（如已创建）

### 相关文档

- [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) - 详细的架构文档检查方法和流程
- [文档审查指南](./workflows/references/review-document.md) - 完整的文档检查指南
- [深入检查指南](./workflows/review.md) - 综合深入检查流程

---

## 🔄 定期检查机制

### 检查计划

为确保架构文档与代码实现始终保持一致，建立以下定期检查机制：

#### 检查频率

1. **每次发布前**：全面检查所有架构文档
   - 使用 [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) 进行全面检查
   - 检查范围：`docs/architecture/` 目录下的所有文档（约 30+ 个文档）
   - 检查内容：模块结构、API 接口、功能描述、配置项、依赖关系、错误处理
   - 检查结果：记录到 `docs/architecture/CHECK_LOG.md`

2. **每月**：抽查部分模块的文档准确性
   - 随机选择 5-10 个模块进行抽查
   - 重点关注最近有代码变更的模块
   - 检查内容：模块结构、API 接口、功能描述
   - 检查结果：记录到 `docs/architecture/CHECK_LOG.md`

3. **每季度**：全面审查所有架构文档
   - 使用 [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) 进行全面检查
   - 检查范围：所有架构文档
   - 检查内容：所有检查项（模块结构、统计、API、功能、依赖、错误处理）
   - 检查结果：记录到 `docs/architecture/CHECK_LOG.md`

#### 检查责任人

- **代码审查者**：在代码审查时检查相关模块的文档是否已更新
- **文档维护者**：负责定期检查计划的执行和检查记录的维护
- **发布负责人**：负责发布前的全面文档检查

#### 检查方法

1. **使用检查指南**：
   - 参考 [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) 进行系统化检查
   - 使用快速检查清单进行快速验证

2. **记录检查结果**：
   - 在 `docs/architecture/CHECK_LOG.md` 中记录每次检查的结果
   - 记录发现的问题和修复状态
   - 跟踪问题的修复进度

3. **问题修复**：
   - 发现问题后，及时创建 issue 或任务
   - 优先修复高优先级问题（公共 API 不一致、核心模块结构变化等）
   - 定期回顾检查记录，确保问题得到及时修复

### 检查记录

所有检查结果应记录到 `docs/architecture/CHECK_LOG.md` 文件中。

**记录格式**：
- 检查日期
- 检查范围（全部/部分模块）
- 检查人员
- 发现的问题列表
- 修复状态（待修复/已修复）
- 检查结果（通过/需要更新）

**记录模板**：参考 `docs/architecture/CHECK_LOG.md` 文件中的记录格式。

### 相关文档

- [架构文档审查指南](./workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) - 详细的架构文档检查方法和流程
- [检查记录文件](../architecture/CHECK_LOG.md) - 架构文档检查记录

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

- [文档编写指南](./document.md) - 架构文档编写规范
- [模板配置指南](./TEMPLATE_GUIDELINES.md) - 模板系统配置和使用方法
- [主架构文档](../architecture/architecture.md) - 项目总体架构
- [Rust 官方文档](https://doc.rust-lang.org/) - Rust 语言文档
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/) - Rust API 设计指南

---

---

**最后更新**: 2025-12-16
