# 开发规范文档

> 本文档定义了 Workflow CLI 项目的开发规范和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [概述](#-概述)
- [代码风格](#-代码风格)
- [错误处理](#-错误处理)
- [日志和调试](#-日志和调试)
- [文档规范](#-文档规范)
- [API 设计规范](#-api-设计规范)
- [代码重构规则](#-代码重构规则)
- [命名规范](#-命名规范)
- [模块组织](#-模块组织)
- [配置管理](#️-配置管理)
- [Git 工作流](#-git-工作流)
- [提交规范](#-提交规范)
- [测试规范](#-测试规范)
- [代码审查](#-代码审查)
- [安全性规则](#-安全性规则)
- [发布前检查](#-发布前检查)
- [定期检查机制](#-定期检查机制)
- [依赖管理](#-依赖管理)
- [性能优化](#-性能优化)
- [开发工具](#-开发工具)

---

## 📋 概述

本文档定义了 Workflow CLI 项目的开发规范和最佳实践，所有贡献者都应遵循这些规范。

### 文档结构

本文档包含以下主要内容：

- **代码风格**：代码格式化、Lint 检查、Rust 命名约定、代码组织
- **错误处理**：错误类型、错误信息、错误处理模式、分层错误处理
- **日志和调试**：日志系统架构、日志级别使用、敏感信息过滤、日志输出规则
- **文档规范**：公共 API 文档、文档注释格式、内部文档
- **命名规范**：文件命名、函数命名、结构体命名、常量命名
- **模块组织**：目录结构、模块职责、模块依赖规则
- **Git 工作流**：分支策略、分支命名、工作流程
- **提交规范**：Conventional Commits 格式、提交类型、提交示例
- **测试规范**：单元测试、测试组织、测试覆盖率、集成测试
- **代码审查**：审查清单、审查重点
- **依赖管理**：添加依赖、依赖原则、依赖审查
- **开发工具**：必需工具、常用命令、IDE 配置、预提交钩子

### 使用建议

- 新加入项目的开发者应完整阅读本文档
- 提交代码前，请确保遵循所有相关规范
- 如有疑问，请参考具体章节的详细说明

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
use color_eyre::Result;
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

### color-eyre 配置要求

在 `main()` 函数中最早调用 `color_eyre::install()?` 启用错误报告功能：

```rust
fn main() -> Result<()> {
    // 安装 color-eyre（最早调用）
    color_eyre::install()?;

    // ... 其他初始化代码
}
```

color-eyre 会自动提供：
- 颜色输出：错误消息以彩色显示，提高可读性
- 错误堆栈跟踪：显示完整的错误链和调用栈
- 错误报告格式化：结构化的错误信息展示

### 错误类型

统一使用 `color_eyre::Result<T>` 作为函数返回类型：

```rust
use color_eyre::Result;

pub fn download_logs(ticket_id: &str) -> Result<Vec<u8>> {
    // 实现
}
```

### 错误信息

提供清晰、有上下文的错误信息：

```rust
// ✅ 好的做法
use color_eyre::{eyre::WrapErr, Result};

pub fn parse_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read config file: {}", path.display()))?;

    toml::from_str(&content)
        .wrap_err("Failed to parse TOML config")?;
}

// ❌ 不好的做法
pub fn parse_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?;  // 错误信息不清晰
    toml::from_str(&content)?;
}
```

### 错误消息格式规范

#### 用户友好的错误消息格式

错误消息应遵循以下格式：

1. **包含操作上下文**：说明在做什么操作时出错
2. **包含目标信息**：文件路径、URL、ID 等
3. **包含可操作的指导**：告诉用户如何解决问题

```rust
// ✅ 好的错误消息格式
color_eyre::eyre::bail!(
    "Failed to read configuration file at {}. Please check file permissions or run 'workflow setup' to create it.",
    path.display()
);

// ❌ 不好的错误消息格式
color_eyre::eyre::bail!("Error: Failed");
```

#### 使用统一的错误消息格式

使用 `MessageFormatter::error()` 格式化常见错误消息：

```rust
use crate::base::format::MessageFormatter;

let error_msg = MessageFormatter::error("read", "config.toml", "Permission denied");
// 输出: "Failed to read config.toml: Permission denied"
```

### 错误消息内容要求

#### 避免技术术语

错误消息应使用用户可理解的语言：

```rust
// ✅ 好的做法：用户友好的语言
color_eyre::eyre::bail!(
    "Configuration file not found. Please run 'workflow setup' to create it."
);

// ❌ 不好的做法：技术术语
color_eyre::eyre::bail!("FileNotFoundError: Config file missing");
```

#### 提供解决方案

错误消息应包含解决方案或下一步操作建议：

```rust
// ✅ 好的做法：提供解决方案
color_eyre::eyre::bail!(
    "Invalid JIRA ID format: {}. Expected format: PROJ-123",
    input
);

// ❌ 不好的做法：只说明问题
color_eyre::eyre::bail!("Invalid JIRA ID format");
```

#### 区分用户错误和系统错误

- **用户错误**：输入验证失败、配置错误等，应提供清晰的指导
- **系统错误**：网络错误、文件系统错误等，应提供详细的错误信息

```rust
// 用户错误：提供格式说明
if !is_valid_jira_id(&input) {
    color_eyre::eyre::bail!(
        "Invalid JIRA ID format: {}\n\nExpected formats:\n  • Ticket ID: PROJ-123\n  • Project name: PROJ",
        input
    );
}

// 系统错误：提供详细错误信息
let response = client.get(url)
    .wrap_err_with(|| format!("Failed to fetch data from {}", url))?;
```

### 错误消息管理

#### 使用错误消息常量

使用错误消息常量统一管理，避免硬编码：

```rust
use crate::base::constants::errors::file_operations::READ_CONFIG_FAILED;

// ✅ 好的做法：使用常量
color_eyre::eyre::bail!("{}: {}", READ_CONFIG_FAILED, path.display());

// ❌ 不好的做法：硬编码字符串
color_eyre::eyre::bail!("Failed to read config file: {}", path.display());
```

#### 错误消息模板

错误消息模板应包含格式说明：

```rust
use crate::base::constants::errors::validation_errors::JIRA_ID_FORMAT_HELP;

color_eyre::eyre::bail!(
    "Invalid JIRA ID format.\n{}\n\nError details: {}",
    JIRA_ID_FORMAT_HELP,
    input
);
```

### 错误处理模式

#### 1. 使用 `WrapErr` 添加上下文

```rust
use color_eyre::{eyre::WrapErr, Result};

let result = operation()
    .wrap_err_with(|| format!("Failed to perform operation with id: {}", id))?;
```

#### 2. 使用 `ContextCompat` 添加上下文

```rust
use color_eyre::{eyre::ContextCompat, Result};

let result = operation()
    .context("Failed to perform operation")?;
```

#### 3. 使用 `eyre!` 创建错误

```rust
use color_eyre::eyre::eyre;

if condition {
    return Err(eyre!("Error message with context: {}", value));
}
```

#### 4. 使用 `bail!` 快速返回错误

```rust
use color_eyre::eyre::bail;

if value < 0 {
    bail!("Value must be non-negative, got: {}", value);
}
```

#### 5. 使用 `ensure!` 进行断言

```rust
use color_eyre::eyre::ensure;

ensure!(
    status_code < 400,
    "HTTP request failed with status: {}",
    status_code
);
```

### 分层错误处理

不同层级使用不同的错误处理策略：

1. **CLI 层**：参数验证错误，使用 `clap` 自动处理
2. **命令层**：用户交互错误、业务逻辑错误，提供友好的错误提示，可使用 `log_error!` 宏输出
3. **库层**：底层操作错误（文件、网络、API），提供详细的错误信息，使用 `WrapErr` 添加上下文

```rust
use color_eyre::{eyre::WrapErr, Result};
use workflow::log_error;

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
        .ok_or_else(|| color_eyre::eyre::eyre!("JIRA ticket ID is required"))?;

    // 调用库层，传递详细错误
    match JiraLogs::new()?.download_from_jira(&id) {
        Ok(_) => Ok(()),
        Err(e) => {
            log_error!("Failed to download logs: {}", e);
            Err(e)
        }
    }
}

// 库层：提供详细的错误信息
pub fn download_from_jira(&self, ticket_id: &str) -> Result<Vec<u8>> {
    let url = format!("{}/api/ticket/{}", self.base_url, ticket_id);
    let response = self.client
        .get(&url)
        .wrap_err_with(|| format!("Failed to fetch ticket {} from JIRA", ticket_id))?;

    response
        .bytes()
        .wrap_err("Failed to read response body")?
        .to_vec()
        .wrap_err("Failed to convert response to bytes")
}
```

### 错误消息结构化

对于 API 错误等复杂错误，应进行结构化格式化：

```rust
use crate::pr::github::errors::format_error;

// 格式化 GitHub API 错误
let error = format_error(&github_error, &response);
// 输出结构化的错误信息，包含：
// - 错误消息
// - HTTP 状态码
// - 错误详情列表
// - 完整的错误响应（用于调试）
```

---

## 📝 日志和调试

### 日志系统架构

项目采用**分离的日志系统设计**，实现了职责分离：

- **Commands 层**：使用 `log_*!` 宏进行用户友好的控制台输出（带颜色、Emoji）
- **Lib 层**：使用 `trace_*!` 宏进行结构化日志记录（默认不输出到控制台，可配置启用）

#### Commands 层日志

Commands 层使用 `log_*!` 宏，直接输出到控制台，用户可见：

```rust
use workflow::{log_success, log_error, log_warning, log_info, log_debug};

// 成功消息（总是输出）
log_success!("Operation completed");

// 错误消息（仅在日志级别 >= ERROR 时输出）
log_error!("Operation failed: {}", error_msg);

// 警告消息（仅在日志级别 >= WARN 时输出）
log_warning!("Retrying operation");

// 信息消息（仅在日志级别 >= INFO 时输出）
log_info!("Processing data");

// 调试消息（仅在日志级别 >= DEBUG 时输出）
log_debug!("Debug information: {}", data);
```

#### Lib 层日志

Lib 层使用 `trace_*!` 宏，默认输出到日志文件，不输出到控制台：

```rust
use workflow::{trace_debug, trace_info, trace_warn, trace_error};

// 调试信息（输出到日志文件）
trace_debug!("Processing data: {}", data);

// 信息日志（输出到日志文件）
trace_info!("Operation completed");

// 警告日志（输出到日志文件）
trace_warn!("Retrying operation");

// 错误日志（输出到日志文件）
trace_error!("Operation failed: {}", error);
```

**重要规则**：
- ❌ **禁止**在 Lib 层使用 `log_*!` 宏（会直接输出到控制台，影响用户体验）
- ✅ **必须**在 Lib 层使用 `trace_*!` 宏（输出到日志文件）

### 日志级别使用规则

#### Commands 层日志级别

- **`log_success!`**：成功消息，总是输出，不受日志级别限制
- **`log_error!`**：系统错误，需要立即关注，仅在日志级别 >= ERROR 时输出
- **`log_warning!`**：警告信息，可能的问题，仅在日志级别 >= WARN 时输出
- **`log_info!`**：重要操作信息（如命令执行、配置加载），仅在日志级别 >= INFO 时输出
- **`log_debug!`**：调试信息，仅在开发时使用，仅在日志级别 >= DEBUG 时输出
- **`log_message!`**：说明信息，总是输出，不受日志级别限制（用于 setup/check 等命令）

#### Lib 层日志级别

- **`trace_error!`**：系统错误，需要立即关注
- **`trace_warn!`**：警告信息，可能的问题
- **`trace_info!`**：重要操作信息（如 API 调用、文件操作）
- **`trace_debug!`**：调试信息，仅在开发时使用
- **`trace!`**：详细跟踪信息，仅在深度调试时使用

### 日志配置

#### 日志级别配置

日志级别从配置文件 `~/.workflow/config/workflow.toml` 中的 `log.level` 字段读取：

```toml
[log]
level = "info"  # 可选值：off, error, warn, info, debug, trace
enable_trace_console = false  # 是否同时输出 trace_*! 日志到控制台
```

#### 日志文件位置

Lib 层的 `trace_*!` 日志默认输出到日志文件：
- 路径：`~/.workflow/logs/tracing/workflow-YYYY-MM-DD.log`
- 格式：按日期分割，每天一个文件
- 存储：强制本地存储（不使用 iCloud 同步）

#### 初始化

在 `main()` 函数中初始化日志系统：

```rust
use workflow::{LogLevel, Tracer};

fn main() -> Result<()> {
    // 安装 color-eyre（最早调用）
    color_eyre::install()?;

    // 初始化日志级别（从配置文件读取）
    let config_level = Settings::get()
        .log
        .level
        .as_ref()
        .and_then(|s| s.parse::<LogLevel>().ok());
    LogLevel::init(config_level);

    // 初始化 tracing（从配置文件读取）
    Tracer::init();

    // ... 其他初始化代码
}
```

### 敏感信息过滤规则

**重要**：所有日志输出前必须过滤敏感信息。

#### 使用 `mask_sensitive_value` 函数

```rust
use crate::mask_sensitive_value;

// ❌ 不安全
trace_info!("API token: {}", token);

// ✅ 安全
trace_info!("API token: {}", mask_sensitive_value(token));
```

#### 使用 `Sensitive` trait

```rust
use crate::base::util::string::Sensitive;

// ❌ 不安全
log_info!("API token: {}", token);

// ✅ 安全
log_info!("API token: {}", token.mask());
```

#### 敏感信息类型

以下信息必须过滤：
- API Token（GitHub、Jira、LLM 等）
- 密码和密钥
- 用户输入中的敏感信息
- 配置文件路径中的敏感信息（如包含用户名的路径）

### 日志输出规则

#### Commands 层规则

- 使用 `log_*!` 宏，直接输出到控制台，用户可见
- 成功消息和说明信息总是输出，不受日志级别限制
- 其他消息根据日志级别决定是否输出
- 所有敏感信息必须过滤

#### Lib 层规则

- 使用 `trace_*!` 宏，默认输出到日志文件
- 不输出到控制台（除非配置 `enable_trace_console = true`）
- 禁止使用 `log_*!` 宏（会直接输出到控制台，影响用户体验）
- 所有敏感信息必须过滤

### 最佳实践

#### 1. 选择合适的日志级别

```rust
// ✅ 好的做法：使用合适的日志级别
trace_info!("API request sent");  // 重要操作
trace_debug!("Request payload: {}", payload);  // 调试信息

// ❌ 不好的做法：过度使用 debug 级别
trace_debug!("API request sent");  // 应该是 info 级别
```

#### 2. 提供有意义的日志消息

```rust
// ✅ 好的做法：提供上下文信息
trace_info!("Downloading file from {} to {}", url, path);

// ❌ 不好的做法：日志消息不清晰
trace_info!("Downloading");
```

#### 3. 过滤敏感信息

```rust
// ✅ 好的做法：过滤敏感信息
trace_info!("API token: {}", mask_sensitive_value(token));
trace_info!("User: {}", user.mask());

// ❌ 不好的做法：直接输出敏感信息
trace_info!("API token: {}", token);
```

#### 4. 避免过度日志记录

```rust
// ✅ 好的做法：只在关键点记录日志
trace_info!("Starting operation");
let result = perform_operation()?;
trace_info!("Operation completed");

// ❌ 不好的做法：记录过多细节
trace_debug!("Step 1");
trace_debug!("Step 2");
trace_debug!("Step 3");
// ... 太多日志
```

### 相关文档

- [Logger 模块架构文档](../architecture/logger.md) - 详细的日志系统架构说明
- `src/lib/base/logger/tracing.rs` - Tracing 封装实现
- `src/lib/base/logger/console.rs` - 控制台日志输出
- `src/lib/base/logger/log_level.rs` - 日志级别管理

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
     - 模块架构文档 → `docs/architecture/{module}.md`（包含 Lib 层和 Commands 层两部分）
   - 文档内容：参考 [文档编写指南](./document.md) 创建完整的架构文档
   - 更新索引：更新 `docs/README.md` 中的文档索引（如适用）

2. **重构模块时**：
   - **必须**同步更新对应的架构文档
   - 更新内容：
     - 模块结构变化
     - API 接口变更
     - 功能描述更新
     - 依赖关系变化
   - 验证一致性：使用 [架构文档审查指南](./development/references/review-architecture-consistency.md) 验证文档与代码的一致性

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
   - 参考 [架构文档审查指南](./development/references/review-architecture-consistency.md) 进行系统化检查
   - 验证文档与代码的一致性

3. **快速验证**：
   ```bash
   # 检查新增的文件是否有对应的架构文档
   # 检查修改的模块是否有架构文档更新
   git diff master --name-only | grep -E "\.(rs|md)$"
   ```

#### 相关文档

- [文档编写指南](./document.md) - 架构文档编写规范和模板
- [架构文档审查指南](./development/references/review-architecture-consistency.md) - 详细的架构文档检查方法和流程
- [代码审查清单](#-代码审查) - 包含文档更新检查项

---

## 🔌 API 设计规范

### 公共 API 设计原则

设计公共 API 时，应遵循以下原则：

1. **简洁性和一致性**：
   - API 应保持简洁，避免过度设计
   - 相似的 API 应遵循一致的命名和结构
   - 使用清晰的命名，避免歧义

2. **避免破坏性变更**：
   - 公共 API 的变更应尽可能保持向后兼容
   - 避免删除或重命名公共函数、结构体、枚举等
   - 如需重大变更，应提供迁移路径

3. **清晰的错误处理**：
   - 使用 `color_eyre::Result<T>` 作为返回类型
   - 提供清晰的错误消息和上下文
   - 错误类型应明确且可操作

4. **完整的文档**：
   - 所有公共 API 必须包含完整的文档注释
   - 文档应包含参数说明、返回值说明、错误说明和使用示例
   - 参考 [文档规范](#-文档规范) 中的公共 API 文档要求

### 向后兼容性要求

**核心原则**：公共 API 变更必须保持向后兼容，除非有充分的理由进行破坏性变更。

#### 向后兼容的变更

以下变更被认为是向后兼容的：

1. **添加新的公共函数、结构体、枚举等**：
   ```rust
   // ✅ 向后兼容：添加新函数
   pub fn new_function() -> Result<()> { ... }
   ```

2. **为现有结构体添加新字段（使用 `Default` 或 `Option`）**：
   ```rust
   // ✅ 向后兼容：新字段使用 Option
   pub struct Config {
       pub existing_field: String,
       pub new_field: Option<String>,  // 新字段
   }
   ```

3. **为现有枚举添加新变体**：
   ```rust
   // ✅ 向后兼容：添加新变体
   pub enum Status {
       Active,
       Inactive,
       Pending,  // 新变体
   }
   ```

4. **使用 `pub use` 重新导出**：
   ```rust
   // ✅ 向后兼容：重新导出保持 API 可用
   pub use internal_module::PublicType;
   ```

5. **使用类型别名提供向后兼容**：
   ```rust
   // ✅ 向后兼容：类型别名
   pub type OldName = NewName;
   ```

#### 破坏性变更的处理

如果必须进行破坏性变更，应遵循以下流程：

1. **使用 `#[deprecated]` 标记即将废弃的 API**：
   ```rust
   /// 旧的 API（已废弃）
   ///
   /// # 废弃说明
   ///
   /// 此函数已废弃，请使用 `new_function()` 替代。
   #[deprecated(note = "使用 new_function() 替代")]
   pub fn old_function() -> Result<()> {
       // 实现可以委托给新函数
       new_function()
   }
   ```

2. **提供迁移指南**：
   - 在文档中说明如何从旧 API 迁移到新 API
   - 提供迁移示例代码
   - 在 CHANGELOG.md 中记录破坏性变更

3. **保持旧 API 至少一个主版本**：
   - 标记为 `deprecated` 后，至少保持一个主版本周期
   - 给用户足够的时间进行迁移

4. **创建迁移文档**（如适用）：
   - 如果破坏性变更影响配置格式，创建迁移文档
   - 参考 `docs/migration/` 目录中的迁移文档模板

### API 版本管理规则

#### 语义化版本

项目遵循语义化版本（Semantic Versioning）：

- **Major**（主版本号）：破坏性变更，不向后兼容
- **Minor**（次版本号）：新功能，向后兼容
- **Patch**（补丁版本号）：Bug 修复，向后兼容

#### API 废弃流程

1. **标记废弃**：
   - 使用 `#[deprecated]` 属性标记即将废弃的 API
   - 在文档中说明废弃原因和替代方案

2. **通知用户**：
   - 在 CHANGELOG.md 中记录废弃信息
   - 在文档中标注废弃状态

3. **保持兼容**：
   - 废弃的 API 应继续工作，至少保持一个主版本周期

4. **移除废弃 API**：
   - 在下一个主版本中移除废弃的 API
   - 确保已提供足够的迁移时间

### API 设计最佳实践

#### 使用 `pub use` 重新导出

使用 `pub use` 重新导出常用的公共 API，保持 API 的简洁性：

```rust
// src/lib/mod.rs
mod internal_module;

// 重新导出公共 API
pub use internal_module::{PublicType, PublicFunction};
```

#### 使用类型别名保持向后兼容

当重构导致类型名称变更时，使用类型别名保持向后兼容：

```rust
// 新类型
pub struct NewClient { ... }

// 向后兼容的类型别名
pub type OldClient = NewClient;
```

#### 使用包装器函数保持向后兼容

当函数签名需要变更时，可以保留旧函数作为包装器：

```rust
// 新函数
pub fn new_function(param: NewType) -> Result<()> { ... }

// 旧函数（向后兼容包装器）
pub fn old_function(param: OldType) -> Result<()> {
    let new_param = convert(param);
    new_function(new_param)
}
```

#### 模块化设计

- 将相关功能组织到同一模块中
- 使用 `mod.rs` 文件管理模块声明和公共 API 导出
- 保持模块之间的依赖关系清晰

### API 变更时的文档更新

**重要**：API 变更时必须同步更新相关文档。

1. **更新文档注释**：
   - 更新函数、结构体等的文档注释
   - 如果 API 被废弃，添加废弃说明和迁移指南

2. **更新架构文档**：
   - 更新对应模块的架构文档中的 API 接口描述
   - 参考 [文档同步要求](#-文档同步要求) 中的"API 变更时"部分

3. **更新 CHANGELOG.md**：
   - 记录 API 变更（新增、废弃、移除）
   - 说明变更原因和影响

4. **更新迁移文档**（如适用）：
   - 如果有破坏性变更，创建或更新迁移文档

### 相关文档

- [文档规范](#-文档规范) - 公共 API 文档要求
- [文档同步要求](#-文档同步要求) - API 变更时的文档更新要求
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/) - Rust API 设计最佳实践

---

## 🔧 代码重构规则

### 概述

代码重构是改进代码结构、提高代码质量的重要手段。本规则定义了重构前的检查清单、重构时的测试要求、重构影响范围评估、重构步骤和流程，以及重构最佳实践。

**核心原则**：
- 重构不应改变代码的外部行为
- 重构前必须有完整的测试覆盖
- 重构应分步骤进行，避免大范围变更
- 公共 API 重构必须保持向后兼容

---

### 重构前的检查清单

#### 重构范围评估

在进行重构之前，必须明确以下内容：

1. **明确重构目标**：
   - 重构的目的是什么？（提高可读性、消除重复、优化性能等）
   - 重构要解决什么问题？
   - 重构后的预期效果是什么？

2. **识别重构范围**：
   - 哪些模块需要重构？
   - 哪些函数或结构体需要重构？
   - 重构的范围有多大？

3. **评估复杂度和风险**：
   - 重构的复杂度如何？
   - 重构的风险有多大？
   - 是否有回滚方案？

4. **确定优先级**：
   - 重构的优先级如何？
   - 是否应该立即进行，还是可以延后？

#### 影响分析

在开始重构之前，必须进行全面的影响分析：

1. **识别受影响的模块和依赖关系**：
   - 哪些模块会受到影响？
   - 哪些函数或结构体会被修改？
   - 依赖关系会发生什么变化？

2. **评估对公共 API 的影响**：
   - 重构是否会影响公共 API？
   - 是否需要保持向后兼容？
   - 是否需要提供迁移路径？

3. **评估对性能的影响**：
   - 重构是否会影响性能？
   - 关键路径的性能是否会下降？
   - 是否需要性能测试？

4. **评估对文档的影响**：
   - 哪些文档需要更新？
   - 架构文档是否需要更新？
   - API 文档是否需要更新？

5. **评估对测试的影响**：
   - 哪些测试会受到影响？
   - 是否需要更新测试？
   - 是否需要添加新测试？

#### 测试覆盖检查

重构前必须确保有足够的测试覆盖：

1. **检查测试覆盖率**：
   - 使用 `cargo tarpaulin` 检查测试覆盖率
   - 目标覆盖率：> 80%
   - 关键业务逻辑覆盖率：> 90%

2. **补充测试**：
   - 如果测试覆盖不足，先补充测试再进行重构
   - 确保关键路径有完整的测试覆盖
   - 确保边界情况有测试覆盖

3. **验证现有测试**：
   - 确保所有现有测试通过
   - 确保测试能够验证重构后的功能一致性

---

### 重构时的测试要求

#### 重构前的测试要求

1. **完整的测试覆盖**：
   - 重构前必须有完整的测试覆盖（目标：> 80%，关键逻辑：> 90%）
   - 使用 `cargo tarpaulin` 验证测试覆盖率

2. **所有测试通过**：
   - 确保所有现有测试通过
   - 运行完整的测试套件（`cargo test`）

3. **测试质量**：
   - 确保测试能够验证代码的正确性
   - 确保测试能够检测到功能回归

#### 重构过程中的测试要求

1. **分步骤进行**：
   - 重构应分步骤进行，每个步骤应该是独立的、可测试的
   - 避免一次性进行大范围重构

2. **每个步骤后运行测试**：
   - 每个重构步骤后必须运行测试确保通过
   - 使用 `cargo test` 运行测试套件
   - 使用 `cargo clippy` 检查代码质量

3. **使用 Git 提交点**：
   - 使用 Git 提交点标记每个重构步骤
   - 每个提交应该是一个完整的、可测试的重构步骤
   - 提交信息应清晰说明重构内容（使用 `refactor` 类型）

4. **便于回滚**：
   - 每个重构步骤应该可以独立回滚
   - 如果某个步骤导致测试失败，应该立即回滚

#### 重构后的测试要求

1. **所有测试通过**：
   - 重构后必须确保所有测试通过
   - 运行完整的测试套件（单元测试、集成测试）

2. **功能一致性验证**：
   - 验证重构后的代码功能与重构前完全一致
   - 确保没有功能回归

3. **测试覆盖率检查**：
   - 检查测试覆盖率是否下降
   - 如果覆盖率下降，需要补充测试

4. **性能测试**（如适用）：
   - 如果重构涉及性能关键路径，需要进行性能测试
   - 确保性能没有下降

---

### 重构影响范围评估

#### 公共 API 影响评估

1. **评估 API 影响**：
   - 重构是否影响公共 API？
   - 哪些公共函数、结构体、枚举会受到影响？

2. **保持向后兼容**：
   - 公共 API 重构必须保持向后兼容（参考 [API 设计规范](#-api-设计规范)）
   - 如需破坏性变更，必须提供迁移路径和迁移文档

3. **使用废弃标记**：
   - 使用 `#[deprecated]` 标记即将废弃的 API
   - 在文档中说明废弃原因和替代方案

4. **提供迁移路径**：
   - 如果必须进行破坏性变更，必须提供清晰的迁移路径
   - 创建迁移文档（参考 `docs/migration/` 目录）

#### 性能影响评估

1. **评估性能影响**：
   - 重构是否会影响性能？
   - 关键路径的性能是否会下降？

2. **性能测试**：
   - 关键路径重构后应进行性能测试
   - 使用基准测试工具（如 `criterion`）进行性能测试

3. **性能优化**：
   - 如果性能下降，需要优化或回滚
   - 确保重构后的性能不低于重构前

#### 文档影响评估

1. **评估文档影响**：
   - 哪些文档需要更新？
   - 架构文档是否需要更新？
   - API 文档是否需要更新？

2. **同步更新文档**：
   - 重构模块时必须同步更新架构文档（参考 [文档同步要求](#-文档同步要求)）
   - API 变更时必须更新文档注释
   - 更新 CHANGELOG.md 记录重构信息

3. **文档验证**：
   - 使用 [架构文档审查指南](./development/references/review-architecture-consistency.md) 验证文档与代码的一致性

---

### 重构步骤和流程

#### 1. 准备阶段

1. **明确重构目标**：
   - 明确重构的目的和预期效果
   - 识别需要重构的代码模块

2. **进行影响分析**：
   - 识别受影响的模块和依赖关系
   - 评估对公共 API、性能、文档、测试的影响

3. **检查测试覆盖**：
   - 使用 `cargo tarpaulin` 检查测试覆盖率
   - 如果测试覆盖不足，先补充测试

4. **创建重构计划**：
   - 制定详细的重构计划
   - 将重构分解为多个小步骤
   - 确定每个步骤的验证方法

#### 2. 执行阶段

1. **分步骤进行重构**：
   - 按照重构计划，分步骤进行重构
   - 每个步骤应该是独立的、可测试的

2. **每个步骤后运行测试**：
   - 每个重构步骤后运行测试确保通过
   - 使用 `cargo test` 运行测试套件
   - 使用 `cargo clippy` 检查代码质量

3. **提交代码**：
   - 使用 Git 提交点标记每个重构步骤
   - 提交信息应清晰说明重构内容（使用 `refactor` 类型）
   - 提交格式：`refactor(<scope>): <subject>`

#### 3. 验证阶段

1. **运行完整测试套件**：
   - 运行所有单元测试和集成测试
   - 确保所有测试通过

2. **验证功能一致性**：
   - 验证重构后的代码功能与重构前完全一致
   - 确保没有功能回归

3. **检查性能影响**：
   - 如果涉及性能关键路径，进行性能测试
   - 确保性能没有下降

4. **更新文档**：
   - 更新架构文档（如适用）
   - 更新 API 文档（如适用）
   - 更新 CHANGELOG.md

#### 4. 审查阶段

1. **提交 PR**：
   - 创建 Pull Request 进行代码审查
   - PR 描述应清晰说明重构内容和影响

2. **代码审查**：
   - 确保重构符合项目规范
   - 确保文档已同步更新
   - 确保测试覆盖充分

3. **合并代码**：
   - 审查通过后合并代码
   - 确保 CI/CD 通过

---

### 重构最佳实践

#### 小步重构

1. **优先小范围重构**：
   - 优先进行小范围、可验证的重构
   - 每个重构步骤应该是独立的、可测试的

2. **避免大范围变更**：
   - 避免一次性进行大范围重构
   - 大范围重构应分解为多个小步骤

3. **逐步改进**：
   - 通过多个小步骤逐步改进代码
   - 每个步骤都应该能够独立验证

#### 保持功能一致性

1. **不改变外部行为**：
   - 重构不应改变代码的外部行为
   - 重构后的代码功能应与重构前完全一致

2. **使用测试验证**：
   - 使用测试验证功能一致性
   - 确保所有测试通过

3. **避免功能增强**：
   - 重构时不应添加新功能
   - 功能增强应该在单独的 PR 中进行

#### 保持代码质量

1. **遵循项目规范**：
   - 重构时应遵循项目代码规范
   - 使用 `cargo fmt` 格式化代码
   - 使用 `cargo clippy` 检查代码质量

2. **提高代码清晰度**：
   - 重构后的代码应更清晰、更易维护
   - 消除重复代码，提高代码复用性

3. **参考代码审查指南**：
   - 参考 [代码审查指南](./development/references/review-code.md) 识别重复代码和优化机会
   - 使用已封装的工具函数替换重复实现

#### 向后兼容性

1. **保持公共 API 兼容**：
   - 公共 API 重构必须保持向后兼容
   - 如需破坏性变更，必须提供迁移路径

2. **使用兼容性技术**：
   - 使用类型别名保持向后兼容（参考 [API 设计规范](#-api-设计规范)）
   - 使用包装器函数保持向后兼容
   - 使用 `#[deprecated]` 标记即将废弃的 API

3. **提供迁移文档**：
   - 如果必须进行破坏性变更，必须提供迁移文档
   - 参考 `docs/migration/` 目录中的迁移文档模板

---

### 重构检查清单

在进行重构时，使用以下检查清单确保重构质量：

#### 重构前检查

- [ ] 明确重构目标和范围
- [ ] 进行影响分析（API、性能、文档、测试）
- [ ] 检查测试覆盖率（目标：> 80%，关键逻辑：> 90%）
- [ ] 如果测试覆盖不足，先补充测试
- [ ] 创建详细的重构计划

#### 重构过程中检查

- [ ] 分步骤进行重构，每个步骤后运行测试
- [ ] 每个步骤后使用 `cargo test` 验证测试通过
- [ ] 每个步骤后使用 `cargo clippy` 检查代码质量
- [ ] 使用 Git 提交点标记每个重构步骤
- [ ] 提交信息清晰说明重构内容（使用 `refactor` 类型）

#### 重构后检查

- [ ] 运行完整测试套件，确保所有测试通过
- [ ] 验证功能一致性，确保没有功能回归
- [ ] 检查测试覆盖率是否下降
- [ ] 如果涉及性能关键路径，进行性能测试
- [ ] 更新架构文档（如适用）
- [ ] 更新 API 文档（如适用）
- [ ] 更新 CHANGELOG.md

#### 代码审查检查

- [ ] PR 描述清晰说明重构内容和影响
- [ ] 确保重构符合项目规范
- [ ] 确保文档已同步更新
- [ ] 确保测试覆盖充分
- [ ] 确保 CI/CD 通过

---

### 相关文档

- [API 设计规范](#-api-设计规范) - 公共 API 设计原则和向后兼容性要求
- [文档同步要求](#-文档同步要求) - 重构模块时的文档更新要求
- [代码审查指南](./development/references/review-code.md) - 代码重复检查和优化机会识别
- [测试规范](#-测试规范) - 测试覆盖要求和测试组织规范
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/) - Rust API 设计最佳实践

---

## 🏷️ 命名规范

### 文件命名

- **模块文件**：`snake-_case.rs`（如 `jira-_client.rs`、`pr-_helpers.rs`）
- **测试文件**：与源文件同名，放在 `tests/` 目录或使用 `#[cfg(test)]` 模块
- **文档文件**：`kebab-case.md`（如 `development.md`、`pr.md`）
  - **架构文档**：`{module}.md`（如 `pr.md`、`git.md`，包含 Lib 层和 Commands 层两部分）
  - **指南文档**：`{topic}.md`（如 `development.md`、`document.md`）
  - **需求文档**：`{topic}.md`（如 `jira.md`、`integration.md`，存放到 `docs/requirements/`）
  - **迁移文档**：`{version}-to-{version}.md`（如 `1.5.6-to-1.5.7.md`）

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

### 常量管理规范

项目使用统一的常量管理模块 `lib/base/constants/` 来管理跨模块使用的常量，提升代码一致性、维护性和可读性。

#### 常量模块组织

常量模块按功能分类组织：

```
src/lib/base/constants/
├── mod.rs          # 模块声明和重新导出
├── errors.rs       # 通用错误消息常量
├── git.rs          # Git 相关常量
├── messages.rs     # 用户交互消息和日志消息常量
├── network.rs      # 网络相关常量
└── validation.rs   # 验证相关常量
```

#### 何时使用常量模块

**应该使用 `lib/base/constants/` 的情况**：

1. **跨模块使用的字符串常量**
   - 错误消息、用户提示消息、日志消息
   - API URL、域名、端点路径
   - 事件类型、状态值

2. **需要统一管理的常量**
   - 多个模块共享的常量
   - 需要集中维护的常量
   - 可能被修改的配置常量

**示例**：
```rust
// ✅ 跨模块使用的错误消息
use workflow::base::constants::file_operations::READ_FILE_FAILED;

// ✅ 跨模块使用的用户消息
use workflow::base::constants::messages::user::OPERATION_CANCELLED;

// ✅ 跨模块使用的验证消息
use workflow::base::constants::validation::branch::EMPTY_NAME;
```

**不应该使用 `lib/base/constants/` 的情况**：

1. **模块内部使用的常量**
   - 仅在单个模块内使用的常量
   - 应该放在模块内部，而不是 `constants/` 模块

2. **配置值**
   - 运行时配置值应使用 `Settings` 模块
   - 常量模块只用于编译时确定的常量

**示例**：
```rust
// ✅ 模块内部常量（放在模块内部）
// src/lib/jira/logs/mod.rs
const DEFAULT_LOG_DIR: &str = "logs";

// ❌ 不应该放在 constants/ 模块
// 因为只在 jira/logs 模块内使用
```

#### 常量模块结构

常量模块使用嵌套模块组织，按功能域分类：

```rust
// src/lib/base/constants/errors.rs
//! 通用错误消息常量

/// 文件操作错误消息
pub mod file_operations {
    /// 创建目录失败
    pub const CREATE_DIR_FAILED: &str = "Failed to create directory";

    /// 读取文件失败
    pub const READ_FILE_FAILED: &str = "Failed to read file";
}

/// HTTP 客户端错误消息
pub mod http_client {
    /// 创建 HTTP 客户端失败
    pub const CREATE_CLIENT_FAILED: &str = "Failed to create HTTP client";
}
```

#### 常量命名规范

1. **使用描述性名称**
   - 常量名应清晰描述其用途
   - 使用完整的单词，避免缩写（除非是通用缩写）

2. **按功能分组**
   - 相关常量放在同一个嵌套模块中
   - 使用有意义的模块名（如 `file_operations`、`http_client`）

3. **添加文档注释**
   - 所有公共常量都应添加 `///` 文档注释
   - 说明常量的用途和使用场景

**示例**：
```rust
/// 文件操作错误消息
pub mod file_operations {
    /// 创建目录失败
    ///
    /// 用于文件操作相关的错误消息
    pub const CREATE_DIR_FAILED: &str = "Failed to create directory";
}
```

#### 常量使用最佳实践

1. **优先使用常量模块**
   - 跨模块使用的字符串常量应优先使用 `constants/` 模块
   - 避免在代码中硬编码字符串

2. **统一错误消息格式**
   - 使用 `MessageFormatter` 格式化错误消息
   - 错误消息常量作为格式化函数的参数

**示例**：
```rust
use workflow::base::constants::file_operations::READ_FILE_FAILED;
use workflow::base::format::MessageFormatter;

// ✅ 使用常量 + MessageFormatter
let error_msg = MessageFormatter::error("read", "config.toml", READ_FILE_FAILED);

// ❌ 避免硬编码
let error_msg = "Failed to read config.toml: Failed to read file";
```

3. **保持一致性**
   - 相同功能的常量应使用相同的命名模式
   - 相关常量应放在同一个模块中

4. **及时更新**
   - 添加新功能时，如果涉及跨模块常量，应及时添加到 `constants/` 模块
   - 修改常量值时，确保所有使用处都已更新

#### 常量模块维护

1. **模块职责**
   - `errors.rs`：通用错误消息常量
   - `git.rs`：Git 相关常量
   - `messages.rs`：用户交互消息和日志消息常量
   - `network.rs`：网络相关常量
   - `validation.rs`：验证相关常量

2. **添加新常量**
   - 确定常量的功能域
   - 选择合适的模块文件
   - 如果功能域不存在，创建新的嵌套模块
   - 添加文档注释

3. **重构常量**
   - 将分散的字符串常量提取到 `constants/` 模块
   - 统一管理，提升可维护性

#### 相关文档

- [格式化工具模块](../architecture/tools.md#8-格式化工具模块-format) - MessageFormatter 使用说明
- [错误处理规范](./development/error-handling.md) - 错误消息格式规范

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

对于在多个命令中重复使用的参数，应该提取为共用参数组（见 [CLI 检查指南](./development/references/review-cli.md)）：

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
- [CLI 检查指南](./development/references/review-cli.md) - 参数复用检查和参数提取指南
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

### 平台特定代码组织

项目支持跨平台开发（macOS、Linux、Windows），需要正确处理平台特定代码。

#### 使用条件编译组织平台特定代码

使用 Rust 的条件编译属性 `#[cfg(...)]` 来组织平台特定代码：

```rust
// 平台特定的函数实现
#[cfg(target_os = "macos")]
fn get_system_path() -> PathBuf {
    PathBuf::from("/usr/local/bin")
}

#[cfg(target_os = "linux")]
fn get_system_path() -> PathBuf {
    PathBuf::from("/usr/bin")
}

#[cfg(target_os = "windows")]
fn get_system_path() -> PathBuf {
    PathBuf::from("C:\\Program Files\\Workflow")
}

// 平台特定的模块
#[cfg(target_os = "macos")]
mod macos_specific;

#[cfg(target_os = "linux")]
mod linux_specific;

#[cfg(target_os = "windows")]
mod windows_specific;
```

#### 平台特定代码的模块化组织

对于复杂的平台特定功能，建议使用独立的模块文件：

```
src/lib/base/util/
├── mod.rs
├── platform.rs          # 平台检测（跨平台）
├── clipboard.rs         # 剪贴板功能（平台特定）
├── clipboard_macos.rs   # macOS 实现
├── clipboard_linux.rs   # Linux 实现
└── clipboard_windows.rs # Windows 实现
```

在 `mod.rs` 中组织：

```rust
// mod.rs
pub mod platform;

#[cfg(target_os = "macos")]
mod clipboard_macos;
#[cfg(target_os = "linux")]
mod clipboard_linux;
#[cfg(target_os = "windows")]
mod clipboard_windows;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub mod clipboard;
```

#### 平台特定依赖管理

在 `Cargo.toml` 中使用条件依赖：

```toml
# 仅在特定平台启用依赖
[target.'cfg(all(not(target_env = "musl"), not(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))))'.dependencies]
clipboard = "0.5"
```

**规则**：
- 使用 `target.'cfg(...)'.dependencies` 来指定平台特定依赖
- 明确说明为什么某些平台禁用某些依赖（如 Linux ARM64 和 musl 静态链接版本不支持剪贴板功能）
- 在文档中说明平台限制

#### 平台检测工具

使用 `Platform` 结构体进行运行时平台检测：

```rust
use crate::base::util::Platform;

let platform = Platform::detect();
if platform.is_macos() {
    // macOS 特定逻辑
} else if platform.is_linux() {
    // Linux 特定逻辑
} else if platform.is_windows() {
    // Windows 特定逻辑
}
```

**使用场景**：
- **编译时检测**：使用 `#[cfg(...)]` 条件编译，适用于编译期已知的平台差异
- **运行时检测**：使用 `Platform::detect()`，适用于需要在运行时判断平台的场景

#### 平台特定功能的测试要求

**测试覆盖要求**：
- 平台特定功能必须添加单元测试
- 使用条件编译确保测试只在对应平台运行：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_specific_feature() {
        // macOS 特定测试
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_specific_feature() {
        // Linux 特定测试
    }
}
```

**跨平台测试策略**：
- 在 CI/CD 中为所有支持的平台运行测试
- 使用 GitHub Actions 的矩阵构建策略测试多个平台
- 确保所有平台的测试都能通过

**测试注意事项**：
- 某些功能可能在某些平台上不可用（如 Linux ARM64 和 musl 静态链接版本不支持剪贴板功能）
- 在测试中正确处理平台限制，避免在不支持的平台上运行相关测试

#### 平台特定代码的最佳实践

1. **优先使用条件编译**：对于编译期已知的平台差异，使用 `#[cfg(...)]` 条件编译
2. **运行时检测作为补充**：对于需要在运行时判断的场景，使用 `Platform::detect()`
3. **模块化组织**：将平台特定代码组织到独立的模块文件中，保持代码清晰
4. **文档说明**：在代码和文档中明确说明平台限制和平台特定行为
5. **测试覆盖**：确保所有平台的代码都有相应的测试覆盖

#### 相关文档

- `src/lib/base/util/platform.rs` - 平台检测工具实现
- `.github/workflows/release.yml` - 跨平台构建和测试配置
- `Cargo.toml` - 平台特定依赖配置示例

---

## ⚙️ 配置管理

### 配置验证规则

所有配置加载时必须验证配置的有效性，确保配置的正确性和安全性。

#### 配置验证时机

配置验证应在以下时机进行：

1. **配置加载时**：使用 `workflow config validate` 命令验证配置
2. **配置更新时**：更新配置后自动验证
3. **程序启动时**：可选，在关键配置缺失时提示用户

#### 配置验证内容

配置验证应检查以下内容：

- **格式验证**：配置文件格式是否正确（TOML、JSON、YAML）
- **必需字段**：检查必需字段是否存在
- **字段类型**：验证字段类型是否正确
- **值有效性**：验证字段值的有效性
  - URL 格式（必须以 `http://` 或 `https://` 开头）
  - 邮箱格式（必须包含 `@`）
  - 路径格式（路径是否存在、是否可访问）
  - 枚举值（是否在允许的枚举值范围内）

#### 配置验证实现

使用 `ConfigValidateCommand` 进行配置验证：

```rust
use crate::commands::config::validate::ConfigValidateCommand;

// 验证配置
ConfigValidateCommand::validate(None, false, false)?;

// 自动修复配置错误
ConfigValidateCommand::validate(None, true, false)?;

// 严格模式（警告视为错误）
ConfigValidateCommand::validate(None, false, true)?;
```

#### 配置错误消息

配置验证失败时，应提供清晰的错误消息：

```rust
// ✅ 好的错误消息
ValidationError {
    field: "jira.email".to_string(),
    message: format!("Invalid email format: '{}'", email),
    fixable: false,
    fix_suggestion: None,
}

// ✅ 提供修复建议
ValidationError {
    field: "jira.service_address".to_string(),
    message: format!(
        "Invalid URL format: '{}' (must start with http:// or https://)",
        service_address
    ),
    fixable: true,
    fix_suggestion: Some(format!(
        "Updated 'jira.service_address' from '{}' to 'https://{}'",
        service_address,
        service_address.trim_start_matches("http://").trim_start_matches("https://")
    )),
}
```

#### 配置验证失败处理

配置验证失败时的处理流程：

1. **显示错误信息**：列出所有验证错误和警告
2. **提供修复建议**：对于可修复的错误，提供修复建议
3. **自动修复**：使用 `--fix` 选项自动修复可修复的错误
4. **退出码**：验证失败时返回非零退出码（用于 CI/CD）

**参考**：
- [配置验证命令架构文档](../architecture/config.md#4-配置验证命令-validaters) - 详细的配置验证实现说明
- `src/commands/config/validate.rs` - 配置验证实现

### 配置迁移规则

当配置格式发生变化时，必须提供配置迁移机制，确保用户配置能够平滑升级。

#### 迁移版本管理

**重要**：迁移版本号**独立于**软件版本号！

- **软件版本**（如 `1.4.8`）：表示软件本身的版本，在 `Cargo.toml` 中定义
- **迁移版本**（如 `v1.0.0`）：表示**配置格式的版本**，只有当配置格式发生变化时才需要迁移

**迁移版本命名规范**：
- 使用语义化版本（Semantic Versioning）
- **Major**（v1.0.0 → v2.0.0）：重大配置格式变化，不向后兼容
- **Minor**（v1.0.0 → v1.1.0）：新增配置项或格式变化，向后兼容
- **Patch**（v1.0.0 → v1.0.1）：通常不使用，因为配置格式变化通常需要 minor 或 major

#### 添加新迁移版本的步骤

当需要添加新的迁移版本时，按以下步骤操作：

1. **创建迁移实现文件**：
   ```rust
   // src/commands/migrate/v1_1_0.rs
   //! v1.1.0 迁移实现

   pub fn migrate_v1_1_0(dry_run: bool, cleanup: bool) -> Result<()> {
       // 1. 检测需要迁移的内容
       // 2. 执行迁移逻辑
       // 3. 可选：清理旧文件
       Ok(())
   }
   ```

2. **在 mod.rs 中导出新模块**：
   ```rust
   // src/commands/migrate/mod.rs
   pub mod v1_1_0;
   ```

3. **在 migrations.rs 中注册新版本**：
   ```rust
   // src/commands/migrate/migrations.rs
   pub const MIGRATIONS: &[&str] = &[
       "v1.0.0",
       "v1.1.0",  // 添加新版本
   ];
   ```

4. **创建迁移文档**：
   - 使用迁移文档模板：`docs/migration/templates/migration.template`
   - 创建迁移文档：`docs/migration/{旧版本}-to-{新版本}.md`
   - 更新迁移文档索引：`docs/migration/README.md`

5. **创建迁移脚本**（如需要）：
   - Shell 脚本：`scripts/migrate/{旧版本}-to-{新版本}.sh`
   - PowerShell 脚本：`scripts/migrate/{旧版本}-to-{新版本}.ps1`

#### 迁移实现要求

迁移实现应遵循以下要求：

- **幂等性**：迁移可以多次执行而不产生副作用
- **可回滚**：迁移前备份原始配置，支持回滚
- **预览模式**：支持 `--dry-run` 预览迁移结果
- **清理选项**：支持 `--cleanup` 清理旧配置文件
- **错误处理**：迁移失败时提供清晰的错误信息
- **日志记录**：记录迁移历史，避免重复迁移

#### 迁移历史管理

使用 `MigrationHistory` 管理迁移历史：

```rust
use crate::commands::migrate::history::MigrationHistory;

// 检查是否已迁移
if MigrationHistory::has_migrated("v1.1.0")? {
    return Ok(());  // 已迁移，跳过
}

// 执行迁移
migrate_v1_1_0(dry_run, cleanup)?;

// 记录迁移历史
MigrationHistory::record("v1.1.0")?;
```

**参考**：
- [迁移系统架构文档](../architecture/migrate.md) - 详细的迁移系统说明
- [迁移文档索引](../migration/README.md) - 迁移文档列表和编写规范
- `src/commands/migrate/` - 迁移实现代码

### 配置默认值管理规则

所有配置项都应提供合理的默认值，确保程序在配置文件缺失或字段缺失时仍能正常运行。

#### 默认值定义方式

使用 `Default` trait 和 `#[serde(default)]` 属性定义默认值：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Jira 配置
    #[serde(default, skip_serializing_if = "JiraSettings::is_empty")]
    pub jira: JiraSettings,

    /// 日志配置
    #[serde(default, skip_serializing_if = "LogSettings::is_empty")]
    pub log: LogSettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogSettings {
    /// 日志级别（默认：info）
    #[serde(default = "default_log_level")]
    pub level: Option<String>,
}

fn default_log_level() -> Option<String> {
    Some("info".to_string())
}
```

#### 默认值管理原则

1. **集中管理**：所有默认值应在代码中明确定义，避免硬编码
2. **合理且安全**：默认值应合理且安全，不会导致安全漏洞
3. **用户友好**：默认值应提供良好的用户体验，减少用户配置负担
4. **文档化**：默认值应在文档中明确说明

#### 默认值实现位置

- **配置结构体默认值**：在配置结构体的 `Default` trait 实现中定义
- **字段默认值函数**：使用 `default = "function_name"` 指定默认值函数
- **Provider 特定默认值**：对于有多个选项的配置（如 LLM Provider），根据 Provider 提供不同默认值

```rust
// Provider 特定默认值示例
pub fn default_llm_model(provider: &str) -> String {
    match provider {
        "openai" => "gpt-4".to_string(),
        "anthropic" => "claude-3-opus".to_string(),
        _ => "gpt-4".to_string(),
    }
}
```

#### 默认值变更的影响评估

修改默认值时，必须评估以下影响：

1. **向后兼容性**：默认值变更是否会影响现有用户
2. **用户体验**：新默认值是否提供更好的用户体验
3. **安全性**：新默认值是否引入安全风险
4. **性能影响**：新默认值是否影响程序性能
5. **文档更新**：必须更新相关文档说明默认值变更

**默认值变更流程**：

1. **评估影响**：评估默认值变更的影响范围
2. **更新代码**：更新默认值实现
3. **更新文档**：更新配置文档和 CHANGELOG.md
4. **测试验证**：验证新默认值的行为
5. **发布说明**：在发布说明中说明默认值变更

#### 配置加载时的默认值处理

配置加载时，如果配置文件不存在或字段缺失，应使用默认值：

```rust
impl Settings {
    /// 从配置文件加载设置
    /// 如果配置文件不存在或字段缺失，使用默认值
    pub fn load() -> Self {
        match Paths::workflow_config() {
            Ok(config_path) => {
                if !config_path.exists() {
                    Self::default()  // 文件不存在，返回默认值
                } else {
                    match FileReader::new(&config_path).to_string() {
                        Ok(content) => {
                            // 解析失败时使用默认值
                            toml::from_str::<Self>(&content).unwrap_or_default()
                        }
                        Err(_) => Self::default(),
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }
}
```

**参考**：
- [Settings 模块架构文档](../architecture/settings.md#3-defaults默认值模块) - 默认值模块说明
- `src/lib/base/settings/settings.rs` - Settings 实现
- `src/lib/base/settings/defaults.rs` - 默认值函数（如存在）

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

**注意**：Workflow CLI 支持通过模板系统自定义分支命名格式。详细配置方法请参考 [模板配置指南](./template.md#分支命名模板-templatebranch)。

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

**注意**：Workflow CLI 支持通过模板系统自定义提交消息格式，包括是否使用 Conventional Commits 格式。详细配置方法请参考 [模板配置指南](./template.md#提交消息模板-templatecommit)。

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

## 🔒 安全性规则

### 概述

安全性是 CLI 工具开发的重要考虑因素。本项目处理大量敏感信息（GitHub API Token、Jira API Token、LLM API Key 等），必须确保这些信息的安全处理。

**核心原则**：
- 禁止在代码中硬编码敏感信息
- 所有用户输入必须验证和清理
- 日志中自动过滤敏感信息
- 定期检查依赖安全漏洞

---

### API Token 和敏感信息处理

#### 禁止硬编码敏感信息

**规则**：禁止在代码中硬编码 API Token、密码等敏感信息。

```rust
// ❌ 禁止：硬编码 API Token
const GITHUB_TOKEN: &str = "ghp_xxxxxxxxxxxxxxxxxxxx";

// ✅ 正确：从配置文件或环境变量读取
let token = Settings::get().github.token
    .ok_or_else(|| anyhow!("GitHub token not configured"))?;
```

#### 使用配置文件或环境变量

**规则**：敏感信息应存储在配置文件或环境变量中。

1. **配置文件存储**：
   - 配置文件位置：`~/.workflow/config/workflow.toml`（macOS/Linux）或 `%APPDATA%\workflow\config\workflow.toml`（Windows）
   - 配置文件不应提交到代码仓库
   - 配置文件应设置适当的文件权限（仅所有者可读）

2. **环境变量存储**：
   - 敏感信息可以通过环境变量传递
   - 环境变量名称应清晰明确（如 `GITHUB_TOKEN`、`JIRA_API_TOKEN`）
   - 在文档中说明环境变量的使用方法

#### 日志中自动过滤敏感信息

**规则**：所有日志输出前必须过滤敏感信息。

**使用 `mask_sensitive_value` 函数**：

```rust
use crate::base::util::string::mask_sensitive_value;

// ❌ 不安全：直接输出敏感信息
trace_info!("API token: {}", token);

// ✅ 安全：使用 mask_sensitive_value 过滤
trace_info!("API token: {}", mask_sensitive_value(token));
```

**使用 `Sensitive` trait**：

```rust
use crate::base::util::string::Sensitive;

// ❌ 不安全：直接输出敏感信息
log_info!("API token: {}", token);

// ✅ 安全：使用 mask() 方法过滤
log_info!("API token: {}", token.mask());
```

**敏感信息类型**：
- API Token（GitHub、Jira、LLM 等）
- 密码和密钥
- 用户输入中的敏感信息
- 配置文件路径中的敏感信息（如包含用户名的路径）

**参考**：详细的敏感信息过滤规则见 [日志和调试](#-日志和调试) 章节中的"敏感信息过滤规则"部分。

#### 配置导出时过滤敏感信息

**规则**：配置导出时必须过滤敏感信息。

**实现方式**：
- 使用 `filter_secrets` 函数过滤敏感字段
- 敏感字段应显示为 `***` 或 `[REDACTED]`
- 导出配置前必须验证敏感信息已过滤

**参考**：`src/commands/config/export.rs` 中的 `filter_secrets` 实现。

---

### 输入验证和清理

#### 所有用户输入必须验证

**规则**：所有用户输入必须验证和清理，防止注入攻击。

**使用 `InputDialog` 的验证器**：

```rust
use crate::base::dialog::InputDialog;

// ✅ 正确：使用验证器验证输入
let input = InputDialog::new("Enter JIRA ID")
    .validator(|input: &str| {
        if input.is_empty() {
            Err("JIRA ID cannot be empty".into())
        } else if !input.contains('-') {
            Err("Invalid JIRA ID format (expected: PROJ-123)".into())
        } else {
            Ok(())
        }
    })
    .interact()?;
```

**验证规则**：
- 检查输入是否为空
- 检查输入格式是否正确（如 JIRA ID 格式、URL 格式）
- 检查输入长度是否在合理范围内
- 检查输入是否包含非法字符

#### 防止注入攻击

**规则**：防止命令注入、路径遍历等攻击。

1. **命令注入防护**：
   - 避免直接拼接用户输入到命令中
   - 使用参数化命令执行（如 `duct::cmd`）
   - 验证命令参数的安全性

```rust
// ❌ 不安全：直接拼接用户输入
let command = format!("git checkout {}", user_input);
std::process::Command::new("sh")
    .arg("-c")
    .arg(&command)
    .output()?;

// ✅ 安全：使用参数化命令
duct::cmd("git", &["checkout", &user_input])
    .run()?;
```

2. **路径遍历防护**：
   - 文件路径必须规范化
   - 检查路径是否在允许的目录范围内
   - 使用 `Path::canonicalize()` 规范化路径

```rust
use std::path::Path;

// ✅ 安全：规范化路径并检查
let path = Path::new(&user_input)
    .canonicalize()
    .context("Invalid path")?;

// 检查路径是否在允许的目录内
let allowed_dir = Path::new("/allowed/directory")
    .canonicalize()?;

if !path.starts_with(&allowed_dir) {
    bail!("Path outside allowed directory");
}
```

3. **URL 验证**：
   - 验证 URL 格式是否正确
   - 检查 URL 协议（只允许 `http://` 或 `https://`）
   - 验证 URL 主机名

```rust
// ✅ 安全：验证 URL 格式
fn validate_url(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        bail!("URL must start with http:// or https://");
    }

    // 进一步验证 URL 格式
    url::Url::parse(url)
        .context("Invalid URL format")?;

    Ok(())
}
```

---

### 依赖安全检查

#### 使用 cargo audit 检查安全漏洞

**规则**：定期使用 `cargo audit` 检查依赖安全漏洞。

**检查频率**：
- 每次添加新依赖前
- 每月定期检查
- 发布前必须检查

**检查命令**：
```bash
# 安装 cargo-audit
cargo install cargo-audit

# 检查安全漏洞
cargo audit
```

**参考**：详细的依赖安全检查流程见 [依赖管理](#-依赖管理) 章节中的"依赖安全漏洞处理流程"部分。

#### 定期更新依赖

**规则**：定期更新依赖以修复安全漏洞。

**更新优先级**：
- **安全补丁**：最高优先级，应立即更新
- **补丁版本更新**：可以自动更新
- **次版本更新**：需要测试验证
- **主版本更新**：需要全面评估和测试

**参考**：详细的依赖更新规则见 [依赖管理](#-依赖管理) 章节中的"依赖更新频率要求"部分。

#### 新增依赖前检查

**规则**：新增依赖前检查是否有已知安全漏洞。

**检查步骤**：
1. 使用 `cargo audit` 检查依赖是否有已知漏洞
2. 检查依赖的维护状态（是否活跃维护）
3. 检查依赖的许可证是否兼容
4. 评估依赖的安全记录

---

### 安全性检查清单

在进行代码开发时，使用以下检查清单确保安全性：

#### API Token 和敏感信息检查

- [ ] 没有硬编码 API Token、密码等敏感信息
- [ ] 敏感信息从配置文件或环境变量读取
- [ ] 日志输出前已过滤敏感信息（使用 `mask_sensitive_value` 或 `Sensitive` trait）
- [ ] 配置导出时已过滤敏感信息
- [ ] 配置文件设置了适当的文件权限

#### 输入验证检查

- [ ] 所有用户输入都经过验证和清理
- [ ] 使用 `InputDialog` 的验证器验证输入
- [ ] 防止命令注入（使用参数化命令执行）
- [ ] 防止路径遍历（规范化路径并检查范围）
- [ ] URL 格式验证（只允许 `http://` 或 `https://`）

#### 依赖安全检查

- [ ] 新增依赖前已检查安全漏洞（`cargo audit`）
- [ ] 定期更新依赖以修复安全漏洞
- [ ] 安全补丁已及时更新
- [ ] 依赖的维护状态良好

---

### 相关文档

- [日志和调试](#-日志和调试) - 敏感信息过滤规则
- [依赖管理](#-依赖管理) - 依赖安全漏洞处理流程和依赖更新频率要求
- `src/lib/base/util/string.rs` - 敏感信息处理工具（`mask_sensitive_value`、`Sensitive` trait）
- `src/commands/config/export.rs` - 配置导出时的敏感信息过滤（`filter_secrets`）
- `src/lib/base/dialog/input.rs` - 输入验证机制（`InputDialog` 验证器）

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
  - [ ] 使用 [架构文档审查指南](./development/references/review-architecture-consistency.md) 进行全面检查
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
# docs/guidelines/development/references/review-architecture-consistency.md

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
- **检查工具**：使用 [架构文档审查指南](./development/references/review-architecture-consistency.md) 进行全面检查
- **检查记录**：建议记录检查结果到 `docs/architecture/check-log.md`（如已创建）

### 相关文档

- [架构文档审查指南](./development/references/review-architecture-consistency.md) - 详细的架构文档检查方法和流程
- [文档审查指南](./development/references/review-document-completeness.md) - 完整的文档检查指南
- [深入检查指南](./development/workflows/review.md) - 综合深入检查流程

---

## 🔄 定期检查机制

### 检查计划

为确保架构文档与代码实现始终保持一致，建立以下定期检查机制：

#### 检查频率

1. **每次发布前**：全面检查所有架构文档
   - 使用 [架构文档审查指南](./development/references/review-architecture-consistency.md) 进行全面检查
   - 检查范围：`docs/architecture/` 目录下的所有文档（约 30+ 个文档）
   - 检查内容：模块结构、API 接口、功能描述、配置项、依赖关系、错误处理
   - 检查结果：记录到 `docs/architecture/check-log.md`

2. **每月**：抽查部分模块的文档准确性
   - 随机选择 5-10 个模块进行抽查
   - 重点关注最近有代码变更的模块
   - 检查内容：模块结构、API 接口、功能描述
   - 检查结果：记录到 `docs/architecture/check-log.md`

3. **每季度**：全面审查所有架构文档
   - 使用 [架构文档审查指南](./development/references/review-architecture-consistency.md) 进行全面检查
   - 检查范围：所有架构文档
   - 检查内容：所有检查项（模块结构、统计、API、功能、依赖、错误处理）
   - 检查结果：记录到 `docs/architecture/check-log.md`

#### 检查责任人

- **代码审查者**：在代码审查时检查相关模块的文档是否已更新
- **文档维护者**：负责定期检查计划的执行和检查记录的维护
- **发布负责人**：负责发布前的全面文档检查

#### 检查方法

1. **使用检查指南**：
   - 参考 [架构文档审查指南](./development/references/review-architecture-consistency.md) 进行系统化检查
   - 使用快速检查清单进行快速验证

2. **记录检查结果**：
   - 在 `docs/architecture/check-log.md` 中记录每次检查的结果
   - 记录发现的问题和修复状态
   - 跟踪问题的修复进度

3. **问题修复**：
   - 发现问题后，及时创建 issue 或任务
   - 优先修复高优先级问题（公共 API 不一致、核心模块结构变化等）
   - 定期回顾检查记录，确保问题得到及时修复

### 检查记录

所有检查结果应记录到 `docs/architecture/check-log.md` 文件中。

**记录格式**：
- 检查日期
- 检查范围（全部/部分模块）
- 检查人员
- 发现的问题列表
- 修复状态（待修复/已修复）
- 检查结果（通过/需要更新）

**记录模板**：参考 `docs/architecture/check-log.md` 文件中的记录格式。

### 相关文档

- [架构文档审查指南](./development/references/review-architecture-consistency.md) - 详细的架构文档检查方法和流程
- [检查记录文件](../architecture/check-log.md) - 架构文档检查记录

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

### 依赖更新频率要求

#### 定期检查依赖更新

1. **检查频率**：
   - 每月检查一次依赖更新
   - 使用 `cargo outdated` 检查过时的依赖
   - 关注主要版本更新和安全补丁

2. **更新优先级**：
   - **安全补丁**：最高优先级，应立即更新
   - **补丁版本更新**：可以自动更新（如 `1.2.3` → `1.2.4`）
   - **次版本更新**：需要测试验证（如 `1.2.3` → `1.3.0`）
   - **主版本更新**：需要全面评估和测试（如 `1.2.3` → `2.0.0`）

3. **更新策略**：
   - 补丁版本更新：可以自动更新，风险较低
   - 次版本更新：需要运行测试套件验证
   - 主版本更新：需要全面评估、测试和可能的代码修改

#### 安全漏洞依赖的紧急更新流程

1. **检测漏洞**：
   - 使用 `cargo audit` 检查安全漏洞
   - 关注 RustSec 安全公告
   - 关注依赖库的安全公告

2. **评估影响**：
   - 评估漏洞的严重程度（Critical、High、Medium、Low）
   - 评估漏洞对项目的影响范围
   - 评估漏洞是否被实际利用

3. **紧急更新**：
   - 发现安全漏洞时，应立即更新到修复版本
   - 如果无法立即更新，评估风险并制定缓解措施
   - 安全漏洞更新应优先于其他更新

4. **验证和发布**：
   - 运行完整测试套件
   - 进行安全测试（如适用）
   - 更新 CHANGELOG.md 记录安全更新

### 依赖版本锁定规则

#### 何时使用精确版本

以下情况应使用精确版本：

1. **关键依赖**：
   - 核心功能依赖：使用精确版本确保稳定性（如 `serde = "1.0"`）
   - 安全关键依赖：使用精确版本避免意外更新（如加密库）

2. **已知兼容性问题**：
   - 如果依赖有已知的兼容性问题，使用精确版本锁定
   - 如果依赖的更新会导致破坏性变更，使用精确版本

3. **生产环境稳定性**：
   - 生产环境使用的依赖应使用精确版本
   - 避免在生产环境中使用版本范围

#### 何时允许版本范围

以下情况可以使用版本范围：

1. **非关键依赖**：
   - 非核心功能依赖可以使用版本范围（如 `serde = "1.0"` 允许 `1.0.x` 的补丁更新）
   - 工具类依赖可以使用更宽松的版本范围

2. **开发依赖**：
   - 开发依赖可以使用更宽松的版本范围
   - 测试依赖可以使用版本范围

3. **功能标志依赖**：
   - 可选功能的依赖可以使用版本范围
   - 不影响核心功能的依赖可以使用版本范围

#### 版本范围格式

Rust/Cargo 支持的版本范围格式：

- `"1.0"` - 允许 `1.0.x` 的补丁更新（推荐用于大多数依赖）
- `"^1.0"` - 允许 `1.x.x` 的次版本更新（语义化版本，等同于 `"1.0"`）
- `"~1.0.0"` - 允许 `1.0.x` 的补丁更新（精确到次版本）
- `"1.0.0"` - 精确版本，不允许更新（用于关键依赖）
- `"*"` - 允许任何版本（不推荐，仅用于特殊情况）

**推荐做法**：
- 大多数依赖使用 `"1.0"` 格式（允许补丁更新）
- 关键依赖使用精确版本 `"1.0.0"`
- 避免使用 `"*"` 通配符

### 依赖安全漏洞处理流程

#### 发现安全漏洞时的处理流程

1. **检测漏洞**：
   - 使用 `cargo audit` 检查安全漏洞
   - 关注 [RustSec Advisory Database](https://rustsec.org/) 安全公告
   - 关注依赖库的官方安全公告
   - 在 CI/CD 中添加自动安全检查

2. **评估影响**：
   - 评估漏洞的严重程度（Critical、High、Medium、Low）
   - 评估漏洞对项目的影响范围
   - 评估漏洞是否被实际利用
   - 评估漏洞是否影响生产环境

3. **制定更新计划**：
   - 查找修复版本（检查依赖的更新日志）
   - 评估更新对代码的影响
   - 评估更新是否会导致破坏性变更
   - 制定测试计划

4. **执行更新**：
   - 更新依赖到修复版本
   - 运行测试确保功能正常
   - 如果更新导致破坏性变更，需要修改代码
   - 提交更新并添加清晰的提交信息（使用 `chore` 或 `security` 类型）

5. **验证和发布**：
   - 运行完整测试套件（`cargo test`）
   - 进行安全测试（如适用）
   - 更新 CHANGELOG.md 记录安全更新
   - 如果漏洞严重，考虑发布安全补丁版本

#### 依赖更新的测试要求

1. **补丁版本更新**：
   - 运行完整测试套件
   - 确保所有测试通过

2. **次版本更新**：
   - 运行完整测试套件
   - 进行集成测试
   - 检查是否有破坏性变更

3. **主版本更新**：
   - 运行完整测试套件
   - 进行全面测试（单元测试、集成测试）
   - 检查 API 变更和破坏性变更
   - 更新代码以适配新版本（如需要）

4. **安全漏洞更新**：
   - 运行完整测试套件
   - 进行回归测试
   - 验证安全漏洞已修复
   - 进行安全测试（如适用）

#### 无法更新的情况

如果遇到以下情况，需要特殊处理：

1. **依赖没有修复版本**：
   - 评估风险并制定缓解措施
   - 考虑使用替代依赖
   - 如果风险可接受，等待修复版本

2. **更新会导致破坏性变更**：
   - 评估是否值得更新
   - 如果值得更新，制定迁移计划
   - 如果更新成本过高，考虑使用替代依赖

3. **更新会影响关键功能**：
   - 制定迁移计划
   - 分步骤进行更新
   - 确保有回滚方案

#### 工具支持

1. **安全检查工具**：
   ```bash
   # 安装 cargo-audit
   cargo install cargo-audit

   # 检查安全漏洞
   cargo audit
   ```

2. **依赖更新检查工具**：
   ```bash
   # 安装 cargo-outdated
   cargo install cargo-outdated

   # 检查过时的依赖
   cargo outdated
   ```

3. **CI/CD 集成**：
   - 在 CI/CD 流程中添加 `cargo audit` 检查
   - 定期运行依赖更新检查
   - 设置安全漏洞警报

### 依赖审查

添加新依赖前，考虑：

- 是否真的需要这个依赖？
- 是否有更轻量的替代方案？
- 依赖的维护状态如何？
- 依赖的许可证是否兼容？

---

## ⚡ 性能优化

### 概述

性能优化是确保应用高效运行的重要手段。本规则定义了性能测试要求、内存使用优化规则和异步操作使用规则，帮助开发者编写高性能代码。

**核心原则**：
- 关键路径必须进行性能测试
- 避免不必要的内存分配
- 网络请求应使用异步操作
- 大文件处理应使用流式处理

---

### 性能测试要求

#### 关键路径性能测试

1. **识别关键路径**：
   - 识别应用中的性能关键路径（如频繁调用的函数、主循环、数据处理流程）
   - 识别用户感知明显的操作（如命令执行、文件处理、网络请求）

2. **性能测试工具**：
   - 使用 `criterion` 进行基准测试（推荐）
   - 使用 `cargo bench` 运行基准测试
   - 使用 `cargo test --bench` 运行基准测试

3. **性能测试实现**：

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // 测试代码
            my_function(black_box(input))
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

4. **性能测试要求**：
   - 关键路径必须进行性能测试
   - 性能测试应作为 CI/CD 的一部分
   - 性能测试结果应记录和跟踪

#### 性能回归测试要求

1. **回归测试时机**：
   - 性能关键代码变更后必须运行性能测试
   - 重构性能关键代码后必须运行性能测试
   - 添加新依赖后评估性能影响

2. **性能阈值**：
   - 建立性能基准线
   - 设置性能回归阈值（如性能下降不超过 5%）
   - 如果性能下降超过阈值，需要优化或回滚

3. **性能监控**：
   - 使用 `criterion` 的统计功能跟踪性能趋势
   - 记录性能测试历史数据
   - 识别性能回归趋势

#### 性能基准测试要求

1. **建立基准线**：
   - 为关键路径建立性能基准线
   - 记录基准测试结果（平均值、中位数、P95、P99）
   - 在文档中记录性能基准

2. **定期运行基准测试**：
   - 定期运行基准测试（如每次发布前）
   - 记录性能趋势
   - 识别性能退化

3. **基准测试工具**：

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench my_benchmark

# 显示详细输出
cargo bench -- --nocapture
```

4. **基准测试最佳实践**：
   - 使用 `black_box` 防止编译器过度优化
   - 多次运行取平均值
   - 在稳定的环境中运行（避免 CPU 频率变化影响）

---

### 内存使用优化规则

#### 避免不必要的内存分配

1. **优先使用栈分配**：
   - 优先使用栈分配，避免堆分配
   - 小数据结构应使用栈分配
   - 大数据结构才使用堆分配

2. **使用引用而非拥有所有权**：
   - 使用 `&str` 而不是 `String`（如果不需要拥有所有权）
   - 使用 `&[T]` 而不是 `Vec<T>`（如果不需要拥有所有权）
   - 使用 `Cow<'_, str>` 避免不必要的克隆

```rust
// ✅ 好的做法：使用引用
fn process_data(data: &str) {
    // 不需要拥有所有权，使用引用
}

// ❌ 不好的做法：不必要的所有权转移
fn process_data(data: String) {
    // 如果不需要拥有所有权，使用 &str
}
```

3. **使用智能指针减少复制**：
   - 使用 `Box<T>` 减少大结构体的复制
   - 使用 `Rc<T>` 或 `Arc<T>` 共享数据
   - 使用 `Cow<'_, T>` 延迟克隆

```rust
// ✅ 好的做法：使用 Box 避免大结构体复制
struct LargeStruct {
    data: [u8; 1024],
}

fn process_large(large: Box<LargeStruct>) {
    // Box 只复制指针，不复制数据
}

// ✅ 好的做法：使用 Cow 延迟克隆
use std::borrow::Cow;

fn process_string(s: Cow<'_, str>) {
    // 如果不需要修改，不克隆；如果需要修改，才克隆
}
```

#### 预分配内存

1. **预分配 Vec**：
   - 使用 `Vec::with_capacity` 预分配内存
   - 如果知道大小，预分配可以避免多次重新分配

```rust
// ✅ 好的做法：预分配内存
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}

// ❌ 不好的做法：多次重新分配
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);  // 可能多次重新分配
}
```

2. **预分配 String**：
   - 使用 `String::with_capacity` 预分配字符串
   - 如果知道字符串长度，预分配可以提高性能

```rust
// ✅ 好的做法：预分配字符串
let mut s = String::with_capacity(100);
for i in 0..100 {
    s.push_str(&i.to_string());
}
```

3. **预分配 HashMap**：
   - 使用 `HashMap::with_capacity` 预分配哈希表
   - 如果知道元素数量，预分配可以避免多次重新哈希

```rust
use std::collections::HashMap;

// ✅ 好的做法：预分配哈希表
let mut map = HashMap::with_capacity(100);
for i in 0..100 {
    map.insert(i, i * 2);
}
```

#### 大文件处理

1. **使用流式处理**：
   - 大文件处理时使用 `BufReader`、`BufWriter`
   - 避免一次性将整个文件加载到内存
   - 使用迭代器处理大数据集

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

// ✅ 好的做法：流式处理大文件
fn process_large_file(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        // 逐行处理，不加载整个文件
        process_line(&line)?;
    }
    Ok(())
}

// ❌ 不好的做法：一次性加载整个文件
fn process_large_file_bad(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;  // 可能内存不足
    // 处理整个文件内容
    Ok(())
}
```

2. **使用迭代器处理大数据集**：
   - 使用迭代器链式处理数据
   - 避免创建中间集合
   - 使用 `Iterator` trait 的方法（`map`、`filter`、`fold` 等）

```rust
// ✅ 好的做法：使用迭代器处理大数据集
let sum: u64 = (0..1_000_000)
    .filter(|&x| x % 2 == 0)
    .map(|x| x * 2)
    .sum();

// ❌ 不好的做法：创建中间集合
let evens: Vec<_> = (0..1_000_000)
    .filter(|&x| x % 2 == 0)
    .collect();  // 创建中间集合
let doubled: Vec<_> = evens.iter().map(|&x| x * 2).collect();
let sum: u64 = doubled.iter().sum();
```

---

### 异步操作使用规则

#### 网络请求

1. **使用异步操作**：
   - 网络请求应使用异步操作（`async/await`）
   - 使用 `tokio` 或 `async-std` 异步运行时
   - 避免阻塞主线程的网络请求

```rust
// ✅ 好的做法：使用异步网络请求
use tokio;

async fn fetch_data(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

// ❌ 不好的做法：阻塞主线程
fn fetch_data_bad(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)?;  // 阻塞主线程
    let body = response.text()?;
    Ok(body)
}
```

2. **并发网络请求**：
   - 使用 `tokio::join!` 并发执行多个网络请求
   - 使用 `futures::future::join_all` 并发执行多个 Future

```rust
use tokio;

// ✅ 好的做法：并发执行多个网络请求
async fn fetch_multiple(urls: Vec<&str>) -> Result<Vec<String>> {
    let futures: Vec<_> = urls.iter()
        .map(|url| fetch_data(url))
        .collect();

    let results = futures::future::join_all(futures).await;
    results.into_iter().collect()
}
```

#### 文件 I/O 操作

1. **大文件 I/O 使用异步**：
   - 大文件 I/O 操作考虑异步处理
   - 使用 `tokio::fs` 进行异步文件操作
   - 小文件操作可以使用同步 I/O

```rust
use tokio::fs;

// ✅ 好的做法：异步文件操作（大文件）
async fn read_large_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).await?;
    Ok(content)
}

// ✅ 也可以：同步文件操作（小文件）
fn read_small_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}
```

2. **并发文件操作**：
   - 使用 `tokio::spawn` 并发执行多个文件操作
   - 使用 `tokio::join!` 并发执行多个异步任务

```rust
use tokio::fs;

// ✅ 好的做法：并发文件操作
async fn process_multiple_files(paths: Vec<PathBuf>) -> Result<Vec<String>> {
    let handles: Vec<_> = paths.iter()
        .map(|path| {
            tokio::spawn(async move {
                fs::read_to_string(path).await
            })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await??);
    }
    Ok(results)
}
```

#### 并发处理

1. **使用 tokio::spawn**：
   - 使用 `tokio::spawn` 并发执行任务
   - 使用 `tokio::join!` 处理多个异步任务
   - 使用 `tokio::select!` 处理多个异步任务

```rust
use tokio;

// ✅ 好的做法：并发执行任务
async fn process_concurrent() -> Result<()> {
    let task1 = tokio::spawn(async {
        // 任务 1
    });

    let task2 = tokio::spawn(async {
        // 任务 2
    });

    tokio::join!(task1, task2)?;
    Ok(())
}
```

2. **使用 tokio::select!**：
   - 使用 `tokio::select!` 处理多个异步任务
   - 等待第一个完成的任务

```rust
use tokio;

// ✅ 好的做法：使用 select! 处理多个异步任务
async fn process_select() -> Result<()> {
    tokio::select! {
        result = task1() => {
            // 处理 task1 的结果
        }
        result = task2() => {
            // 处理 task2 的结果
        }
    }
    Ok(())
}
```

3. **控制并发数量**：
   - 避免过度并发，控制并发数量
   - 使用 `tokio::sync::Semaphore` 限制并发数
   - 使用 `futures::stream::StreamExt` 的 `buffer_unordered` 控制并发

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

// ✅ 好的做法：限制并发数量
async fn process_with_limit(tasks: Vec<Task>) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(10));  // 最多 10 个并发

    let handles: Vec<_> = tasks.iter()
        .map(|task| {
            let semaphore = semaphore.clone();
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                // 执行任务
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }
    Ok(())
}
```

---

### 性能优化检查清单

在进行性能优化时，使用以下检查清单：

#### 性能测试检查

- [ ] 关键路径已进行性能测试
- [ ] 性能测试已添加到 CI/CD
- [ ] 建立了性能基准线
- [ ] 性能回归测试已设置阈值

#### 内存优化检查

- [ ] 避免不必要的内存分配
- [ ] 使用引用而非拥有所有权（如 `&str` 而非 `String`）
- [ ] 预分配内存（`Vec::with_capacity`、`String::with_capacity` 等）
- [ ] 大文件处理使用流式处理（`BufReader`、`BufWriter`）
- [ ] 使用迭代器处理大数据集，避免中间集合

#### 异步操作检查

- [ ] 网络请求使用异步操作（`async/await`）
- [ ] 大文件 I/O 使用异步操作（`tokio::fs`）
- [ ] 并发任务使用 `tokio::spawn` 或 `tokio::join!`
- [ ] 控制并发数量，避免过度并发

---

### 相关文档

- [二进制大小分析指南](./cargo-bloat.md) - 使用 `cargo-bloat` 分析二进制文件大小
- [代码重构规则](#-代码重构规则) - 重构时的性能影响评估
- [测试规范](#-测试规范) - 测试性能要求
- [Rust 性能优化指南](https://doc.rust-lang.org/book/ch13-00-functional-features.html) - Rust 官方性能优化指南

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
- [模板配置指南](./template.md) - 模板系统配置和使用方法
- [主架构文档](../architecture/architecture.md) - 项目总体架构
- [Rust 官方文档](https://doc.rust-lang.org/) - Rust 语言文档
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/) - Rust API 设计指南

---

---

**最后更新**: 2025-12-23
