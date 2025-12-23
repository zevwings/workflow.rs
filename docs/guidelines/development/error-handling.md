# 错误处理规范

> 本文档定义了 Workflow CLI 项目的错误处理规范和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [概述](#-概述)
- [color-eyre 配置要求](#-color-eyre-配置要求)
- [错误类型](#-错误类型)
- [错误信息](#-错误信息)
- [错误消息格式规范](#-错误消息格式规范)
- [错误消息内容要求](#-错误消息内容要求)
- [错误消息管理](#-错误消息管理)
- [错误处理模式](#-错误处理模式)
- [分层错误处理](#-分层错误处理)
- [错误消息结构化](#-错误消息结构化)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档定义了错误处理规范，包括错误类型、错误信息格式、错误处理模式和分层错误处理。

### 核心原则

- **统一性**：统一使用 `color_eyre::Result<T>` 作为函数返回类型
- **上下文**：为错误消息添加上下文信息
- **用户友好**：错误消息应清晰、可操作

### 使用场景

- 编写新代码时参考
- 错误处理代码审查时检查
- 调试和错误排查时使用

### 快速参考

| 操作 | 方法 | 说明 |
|------|------|------|
| **添加上下文** | `wrap_err_with()` | 为错误添加上下文 |
| **快速返回错误** | `bail!()` | 快速返回错误 |
| **断言** | `ensure!()` | 进行断言 |

---

## color-eyre 配置要求

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

---

## 错误类型

统一使用 `color_eyre::Result<T>` 作为函数返回类型：

```rust
use color_eyre::Result;

pub fn download_logs(ticket_id: &str) -> Result<Vec<u8>> {
    // 实现
}
```

---

## 错误信息

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

---

## 错误消息格式规范

### 用户友好的错误消息格式

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

### 使用统一的错误消息格式

使用 `MessageFormatter::error()` 格式化常见错误消息：

```rust
use crate::base::format::MessageFormatter;

let error_msg = MessageFormatter::error("read", "config.toml", "Permission denied");
// 输出: "Failed to read config.toml: Permission denied"
```

---

## 错误消息内容要求

### 避免技术术语

错误消息应使用用户可理解的语言：

```rust
// ✅ 好的做法：用户友好的语言
color_eyre::eyre::bail!(
    "Configuration file not found. Please run 'workflow setup' to create it."
);

// ❌ 不好的做法：技术术语
color_eyre::eyre::bail!("FileNotFoundError: Config file missing");
```

### 提供解决方案

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

### 区分用户错误和系统错误

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

---

## 错误消息管理

### 使用错误消息常量

使用错误消息常量统一管理，避免硬编码：

```rust
use crate::base::constants::errors::file_operations::READ_CONFIG_FAILED;

// ✅ 好的做法：使用常量
color_eyre::eyre::bail!("{}: {}", READ_CONFIG_FAILED, path.display());

// ❌ 不好的做法：硬编码字符串
color_eyre::eyre::bail!("Failed to read config file: {}", path.display());
```

### 错误消息模板

错误消息模板应包含格式说明：

```rust
use crate::base::constants::errors::validation_errors::JIRA_ID_FORMAT_HELP;

color_eyre::eyre::bail!(
    "Invalid JIRA ID format.\n{}\n\nError details: {}",
    JIRA_ID_FORMAT_HELP,
    input
);
```

---

## 错误处理模式

### 1. 使用 `WrapErr` 添加上下文

```rust
use color_eyre::{eyre::WrapErr, Result};

let result = operation()
    .wrap_err_with(|| format!("Failed to perform operation with id: {}", id))?;
```

### 2. 使用 `ContextCompat` 添加上下文

```rust
use color_eyre::{eyre::ContextCompat, Result};

let result = operation()
    .context("Failed to perform operation")?;
```

### 3. 使用 `eyre!` 创建错误

```rust
use color_eyre::eyre::eyre;

if condition {
    return Err(eyre!("Error message with context: {}", value));
}
```

### 4. 使用 `bail!` 快速返回错误

```rust
use color_eyre::eyre::bail;

if value < 0 {
    bail!("Value must be non-negative, got: {}", value);
}
```

### 5. 使用 `ensure!` 进行断言

```rust
use color_eyre::eyre::ensure;

ensure!(
    status_code < 400,
    "HTTP request failed with status: {}",
    status_code
);
```

---

## 分层错误处理

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

---

## 错误消息结构化

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

## 🔍 故障排除

### 问题 1：错误消息不清晰

**症状**：错误消息缺少上下文信息

**解决方案**：

1. 使用 `wrap_err_with()` 添加上下文
2. 使用 `MessageFormatter::error()` 格式化错误消息
3. 确保错误消息包含操作上下文和目标信息

### 问题 2：错误堆栈跟踪不完整

**症状**：错误堆栈跟踪信息不足

**解决方案**：

1. 确保在 `main()` 函数中最早调用 `color_eyre::install()?`
2. 使用 `wrap_err_with()` 在关键点添加上下文
3. 避免过早使用 `?` 操作符，先添加上下文

---

## 📚 相关文档

### 开发规范

- [代码风格规范](./code-style.md) - 代码风格规范
- [日志和调试规范](./references/logging.md) - 日志和调试规范

### 检查工作流

- [提交前检查](./workflows/pre-commit.md) - 代码质量检查流程

---

## ✅ 检查清单

使用本规范时，请确保：

- [ ] 在 `main()` 函数中最早调用 `color_eyre::install()?`
- [ ] 统一使用 `color_eyre::Result<T>` 作为函数返回类型
- [ ] 为错误消息添加上下文信息
- [ ] 错误消息使用用户友好的语言
- [ ] 区分用户错误和系统错误
- [ ] 使用错误消息常量统一管理

---

**最后更新**: 2025-12-23

