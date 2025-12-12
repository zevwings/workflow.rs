# 提交前检查指南

> 本文档定义了代码开发完成后、提交代码前需要完成的检查项，确保代码质量和项目规范。

---

## 📋 目录

- [检查步骤](#-检查步骤)
- [文档检查](#-文档检查)
- [CLI 和 Completion 检查](#-cli-和-completion-检查)
- [代码优化检查](#-代码优化检查)
- [测试用例检查](#-测试用例检查)
- [代码质量检查](#-代码质量检查)
- [其他检查项](#-其他检查项)
- [生成检查报告](#-生成检查报告)
- [快速检查清单](#-快速检查清单)
- [常见问题](#-常见问题)

---

## 🚀 检查步骤

按照以下步骤依次完成检查：

### 第一步：CLI 和 Completion 检查

检查 CLI 命令结构和补全脚本：
- CLI 命令注册和参数定义
- 补全脚本完整性测试
- 补全脚本优化

**对应章节**：[CLI 和 Completion 检查](#-cli-和-completion-检查)

### 第二步：代码优化检查

检查代码是否已优化：
- 共用代码提取（参数、工具函数、错误处理）
- 代码重复检查
- 工具函数复用
- 配置管理

**对应章节**：[代码优化检查](#-代码优化检查)

### 第三步：测试用例检查

检查测试覆盖：
- 单元测试
- 集成测试
- 文档测试
- 测试数据和边界情况
- 测试冗余检查（重复测试、过时测试、无效测试）

**对应章节**：[测试用例检查](#-测试用例检查)

### 第四步：代码质量检查

检查代码质量：
- 代码格式化
- Clippy 警告
- 编译检查
- 导入语句和顺序
- 自动修复

**对应章节**：[代码质量检查](#-代码质量检查)

### 第五步：文档检查

检查所有文档是否已更新，包括：
- README.md 命令清单和版本号
- 架构文档更新
- 文档索引更新
- 迁移文档（如有需要）

**对应章节**：[文档检查](#-文档检查)

### 第六步：其他检查项

检查其他重要项：
- 版本管理
- 依赖管理
- 平台兼容性
- 性能检查
- 安全性检查
- 用户体验

**对应章节**：[其他检查项](#-其他检查项)

### 最后：生成检查报告

完成所有检查后，生成检查报告文档。

**对应章节**：[生成检查报告](#-生成检查报告)

---

## 📚 文档检查

### 1.1 README.md 更新

**检查项**：
- [ ] 命令清单部分（第 367-541 行）是否已更新新增/修改的命令
- [ ] 架构总览部分（第 666-742 行）是否反映了架构变更
- [ ] 快速开始部分是否准确
- [ ] 版本号是否与 `Cargo.toml` 一致

**位置**：`README.md`

### 1.2 架构文档更新

**检查项**：
- [ ] 相关命令架构文档是否已更新（`docs/architecture/commands/*_COMMAND_ARCHITECTURE.md`）
- [ ] 相关 lib 层架构文档是否已更新（`docs/architecture/lib/*_ARCHITECTURE.md`）
- [ ] `CLI_ARCHITECTURE.md` 是否反映了命令结构变更
- [ ] `COMPLETION_ARCHITECTURE.md` 是否反映了补全相关变更

**位置**：`docs/architecture/`

### 1.3 文档索引更新

**检查项**：
- [ ] 新增文档是否已添加到 `docs/README.md` 索引

**位置**：`docs/README.md`

### 1.4 迁移文档

**检查项**：
- [ ] 如有破坏性变更，是否已创建迁移指南（`docs/migration/`）
- [ ] 版本号是否正确

---

## 🔧 CLI 和 Completion 检查

### 2.1 CLI 命令结构

**检查项**：
- [ ] 新增命令是否已在 `Commands` 枚举中注册（`src/lib/cli/commands.rs`）
- [ ] 子命令是否已在对应枚举中定义
- [ ] 命令文档注释（`///`）是否完整
- [ ] 参数命名是否一致（如 `jira_id` vs `JIRA_ID`）
- [ ] 共用参数是否已提取（参考 [代码优化检查](#-代码优化检查)）

**位置**：`src/lib/cli/`

**示例**：
```rust
// src/lib/cli/commands.rs
#[derive(Subcommand)]
pub enum Commands {
    /// 新增命令的描述
    NewCommand {
        #[command(subcommand)]
        subcommand: NewSubcommand,
    },
}
```

### 2.2 Completion 完整性

**检查项**：
- [ ] 运行补全完整性测试：`cargo test --test completeness`
- [ ] 新增命令是否包含在补全脚本中
- [ ] 所有 shell 类型（zsh, bash, fish, powershell, elvish）是否正常生成
- [ ] 补全脚本文件命名是否正确

**测试命令**：
```bash
# 运行补全完整性测试
cargo test --test completeness

# 手动生成补全脚本验证
cargo run -- completion generate
```

**位置**：`tests/completion/completeness.rs`

### 2.3 CLI 参数优化

**检查项**：
- [ ] 重复参数是否已提取为共用结构体（如 `OutputFormatArgs`、`DryRunArgs`）
- [ ] 是否使用 `#[command(flatten)]` 复用参数组
- [ ] 参数验证逻辑是否统一

**参考**：见 [代码优化检查 - 提取共用参数](#提取共用参数)

---

## 🔍 代码优化检查

### 3.1 如何提取共用代码

#### 提取共用参数

**场景**：多个命令使用相同的参数（如输出格式、dry-run 等）

**步骤**：

1. **创建共用参数结构体**

在 `src/lib/cli/common.rs` 中定义：

```rust
//! 共用 CLI 参数定义
//!
//! 提供多个命令共享的参数组，减少代码重复。

use clap::Args;

/// 输出格式选项
///
/// 支持多种输出格式：table（默认）、json、yaml、markdown。
/// 优先级：json > yaml > markdown > table
#[derive(Args, Debug, Clone)]
pub struct OutputFormatArgs {
    /// Output in table format (default)
    #[arg(long)]
    pub table: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Output in YAML format
    #[arg(long)]
    pub yaml: bool,

    /// Output in Markdown format
    #[arg(long)]
    pub markdown: bool,
}

/// Dry-run 选项
///
/// 预览操作而不实际执行。
#[derive(Args, Debug, Clone)]
pub struct DryRunArgs {
    /// Dry run mode (preview changes without actually doing it)
    #[arg(long, short = 'n')]
    pub dry_run: bool,
}
```

2. **在 CLI 模块中导出**

在 `src/lib/cli/mod.rs` 中：

```rust
mod common;
pub use common::{OutputFormatArgs, DryRunArgs};
```

3. **在命令中使用**

在命令枚举中使用 `#[command(flatten)]`：

```rust
// src/lib/cli/jira.rs
use super::common::OutputFormatArgs;

#[derive(Subcommand)]
pub enum JiraSubcommand {
    Info {
        #[arg(value_name = "JIRA_ID")]
        jira_id: Option<String>,

        #[command(flatten)]
        output_format: OutputFormatArgs,
    },
    // ... 其他命令
}
```

4. **在命令实现中使用**

```rust
// src/commands/jira/info.rs
use crate::cli::OutputFormatArgs;
use crate::commands::jira::helpers::OutputFormat;

pub fn show(
    jira_id: Option<String>,
    output_format: &OutputFormatArgs,
) -> Result<()> {
    // 转换为内部格式
    let format = OutputFormat::from(output_format);

    // 使用 format 进行输出
    match format {
        OutputFormat::Json => output_json(&issue)?,
        OutputFormat::Yaml => output_yaml(&issue)?,
        // ...
    }

    Ok(())
}
```

**优势**：
- ✅ 减少代码重复（每个命令不再需要重复定义 4 个布尔参数）
- ✅ 类型安全（使用结构体而非多个独立参数）
- ✅ 自动生成补全脚本（clap 自动处理）
- ✅ 易于维护（修改一处即可影响所有使用该参数的命令）

#### 提取共用工具函数

**场景**：多个命令使用相同的逻辑（如获取 JIRA ID、格式化日期等）

**步骤**：

1. **创建 helpers 模块**

在命令目录下创建 `helpers.rs`：

```rust
// src/commands/jira/helpers.rs
//! Jira 命令公共帮助函数
//!
//! 提供 Jira 命令之间共享的公共功能，避免代码重复。

use crate::base::dialog::InputDialog;
use anyhow::{Context, Result};

/// 获取 JIRA ID（从参数或交互式输入）
pub fn get_jira_id(
    jira_id: Option<String>,
    prompt_message: Option<&str>,
) -> Result<String> {
    if let Some(id) = jira_id {
        Ok(id)
    } else {
        let message = prompt_message
            .unwrap_or("Enter Jira ticket ID (e.g., PROJ-123)");
        InputDialog::new(message)
            .prompt()
            .context("Failed to read Jira ticket ID")
    }
}
```

2. **在命令中使用**

```rust
// src/commands/jira/info.rs
use super::helpers::get_jira_id;

pub fn show(jira_id: Option<String>, ...) -> Result<()> {
    let jira_id = get_jira_id(jira_id, None)?;
    // ... 使用 jira_id
}
```

**优势**：
- ✅ 统一错误处理
- ✅ 减少代码重复
- ✅ 易于测试和维护

#### 提取共用错误处理

**场景**：多个命令使用相同的错误处理模式

**步骤**：

1. **使用 anyhow 的 Context**

```rust
use anyhow::{Context, Result};

pub fn download_file(url: &str) -> Result<Vec<u8>> {
    reqwest::blocking::get(url)
        .context("Failed to download file")?
        .bytes()
        .context("Failed to read response body")?
        .to_vec()
        .context("Failed to convert to bytes")
}
```

2. **创建自定义错误类型**（如果需要）

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JiraError {
    #[error("Jira ticket not found: {0}")]
    NotFound(String),

    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),
}
```

**优势**：
- ✅ 统一的错误消息格式
- ✅ 更好的错误上下文信息
- ✅ 易于调试和排查问题

### 3.2 代码重复检查

**检查项**：
- [ ] 输出格式处理逻辑是否已提取
- [ ] 参数解析逻辑是否可复用
- [ ] 文件操作是否使用共用工具（`lib/base/util`）
- [ ] HTTP 请求是否统一使用 `HttpClient`
- [ ] Git 操作是否统一使用 `lib/git` 模块

**工具函数位置**：
- `lib/base/util/` - 通用工具函数
- `lib/base/dialog/` - 交互式对话框
- `lib/base/indicator/` - 进度指示器
- `commands/*/helpers.rs` - 命令特定帮助函数

### 3.3 工具函数复用

**检查项**：
- [ ] 是否使用 `lib/base/util` 中的工具函数
  - `format_size()` - 文件大小格式化
  - `mask_sensitive_value()` - 敏感值隐藏
  - `Browser`、`Clipboard` - 浏览器和剪贴板操作
  - `Checksum`、`Unzip` - 文件操作
- [ ] 是否使用 `lib/base/dialog` 进行用户交互
- [ ] 是否使用 `lib/base/indicator` 显示进度

**示例**：
```rust
use crate::base::util::{format_size, Browser, Clipboard};
use crate::base::dialog::InputDialog;
use crate::base::indicator::Spinner;

// 使用工具函数
let size = format_size(1024 * 1024); // "1.0 MB"
Browser::open("https://example.com")?;
Clipboard::copy("text")?;

// 使用对话框
let input = InputDialog::new("Enter value:").prompt()?;

// 使用进度指示器
let result = Spinner::with("Processing...", || {
    // 执行耗时操作
})?;
```

### 3.4 配置管理

**检查项**：
- [ ] 是否使用 `Settings` 统一管理配置
- [ ] 是否使用 `Paths` 统一管理路径
- [ ] 配置验证是否统一

**示例**：
```rust
use crate::base::settings::{Settings, Paths};

// 使用 Settings
let settings = Settings::load()?;
let jira_config = &settings.jira;

// 使用 Paths
let config_dir = Paths::config_dir()?;
let completion_dir = Paths::completion_dir()?;
```

---

## 🧪 测试用例检查

### 4.1 单元测试

**检查项**：
- [ ] 新增功能是否有对应的单元测试
- [ ] 边界情况是否已覆盖
- [ ] 错误处理是否已测试

**位置**：与源代码在同一文件中（`#[cfg(test)]` 模块）

**示例**：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_jira_id_with_param() {
        let result = get_jira_id(Some("PROJ-123".to_string()), None);
        assert_eq!(result.unwrap(), "PROJ-123");
    }

    #[test]
    fn test_get_jira_id_without_param() {
        // 需要 mock InputDialog
    }
}
```

### 4.2 集成测试

**检查项**：
- [ ] CLI 命令是否在 `tests/cli/` 中有测试
- [ ] 补全完整性测试是否通过（`tests/completion/completeness.rs`）
- [ ] 集成测试是否覆盖主要流程（`tests/integration/`）

**位置**：`tests/` 目录

**运行测试**：
```bash
# 运行所有测试（包括单元测试、集成测试和文档测试）
make test
# 或
cargo test --verbose

# 运行特定测试
cargo test --test completeness  # 补全完整性测试
cargo test --lib --tests      # 仅单元测试和集成测试（不包括文档测试）
cargo test --doc              # 仅文档测试
```

### 4.3 测试覆盖率

**检查项**：
- [ ] 运行 `make test` 或 `cargo test --verbose` 确保所有测试通过（包括 doctest）
- [ ] 新增代码路径是否被测试覆盖
- [ ] 测试用例命名是否清晰（`test_*`）
- [ ] 文档中的代码示例（doctest）是否能够正常编译和运行

### 4.4 测试数据

**检查项**：
- [ ] 测试 fixtures（`tests/fixtures/`）是否需要更新
- [ ] Mock 数据是否准确

### 4.5 测试冗余检查

**检查项**：
- [ ] 是否存在重复的测试用例（测试相同功能或相同代码路径）
- [ ] 是否有过时或不再需要的测试（功能已移除或重构，但测试未更新）
- [ ] 测试是否真正有效（是否测试了正确的功能，断言是否正确）
- [ ] 是否有测试用例测试了已不存在的代码路径（代码已删除但测试仍存在）
- [ ] 是否有测试用例测试了已废弃的 API 或方法

**检查方法**：
1. **查找重复测试**：检查是否有多个测试用例测试相同的功能
   ```bash
   # 搜索测试文件，查找相似的测试用例
   grep -r "test_" tests/ src/ | sort | uniq -d
   ```

2. **检查过时测试**：
   - 查看测试是否引用了已删除的函数或模块
   - 检查测试是否使用了已废弃的 API
   - 运行 `cargo test` 查看是否有编译错误或警告

3. **验证测试有效性**：
   - 检查测试的断言是否正确（是否真正验证了功能）
   - 检查测试是否可能因为实现变更而失效
   - 检查测试是否测试了边界情况和错误情况

**示例**：
```rust
// ❌ 错误：重复测试相同功能
#[test]
fn test_parse_jira_id_1() {
    assert_eq!(parse_jira_id("PROJ-123"), Some("PROJ-123"));
}

#[test]
fn test_parse_jira_id_2() {
    assert_eq!(parse_jira_id("PROJ-123"), Some("PROJ-123")); // 重复
}

// ✅ 正确：测试不同的场景
#[test]
fn test_parse_jira_id_valid() {
    assert_eq!(parse_jira_id("PROJ-123"), Some("PROJ-123"));
}

#[test]
fn test_parse_jira_id_invalid() {
    assert_eq!(parse_jira_id("invalid"), None);
}

// ❌ 错误：测试已删除的函数
#[test]
fn test_old_function() {
    old_function(); // 函数已删除，测试无效
}

// ❌ 错误：测试断言不正确
#[test]
fn test_calculate_total() {
    let result = calculate_total(&[1, 2, 3]);
    assert!(result > 0); // 断言太弱，没有验证具体值
}

// ✅ 正确：测试断言明确
#[test]
fn test_calculate_total() {
    let result = calculate_total(&[1, 2, 3]);
    assert_eq!(result, 6); // 断言明确，验证具体值
}
```

**优势**：
- ✅ 减少测试维护成本（删除无效测试）
- ✅ 提高测试质量（确保测试真正有效）
- ✅ 避免测试混乱（减少重复测试）
- ✅ 提高测试运行效率（减少不必要的测试执行时间）

---

## ✅ 代码质量检查

### 5.1 格式化检查

**命令**：
```bash
make lint
# 或
cargo fmt --check
```

**检查项**：
- [ ] 代码格式是否符合 `rustfmt.toml` 配置
- [ ] 缩进是否统一（4 个空格）
- [ ] 行宽是否在 100 字符以内

**自动修复**：
```bash
cargo fmt
```

### 5.2 Clippy 检查

**命令**：
```bash
cargo clippy -- -D warnings
```

**检查项**：
- [ ] 所有 Clippy 警告是否已修复
- [ ] 复杂度阈值是否符合 `.clippy.toml` 配置
- [ ] 是否使用了不必要的 `unwrap()`（应使用 `expect()` 或错误处理）

**自动修复**：
```bash
cargo clippy --fix --allow-dirty --allow-staged
```

### 5.3 编译检查

**命令**：
```bash
cargo check
```

**检查项**：
- [ ] 代码是否能够编译通过
- [ ] 是否有未使用的导入或变量
- [ ] 类型是否匹配

### 5.4 导入检查

**检查项**：
- [ ] 所有导入是否统一从文件顶部导入
- [ ] 是否避免在代码中间使用 `crate::` 路径直接调用函数
- [ ] 导入语句是否按标准库、第三方库、本地模块的顺序组织

**正确示例**：
```rust
// 文件顶部统一导入
use crate::repo::config::RepoConfig;
use anyhow::{Context, Result};

pub fn get_branch_prefix() -> Option<String> {
    // 使用顶部导入的函数
    RepoConfig::get_branch_prefix()
}
```

**错误示例**：
```rust
// ❌ 错误：在代码中间使用 crate:: 路径直接调用
pub fn get_branch_prefix() -> Option<String> {
    crate::repo::config::RepoConfig::get_branch_prefix()
}
```

**导入顺序规范**：
1. 标准库导入（`std::*`）
2. 第三方库导入（按字母顺序）
3. 本地模块导入（`crate::*`）

**示例**：
```rust
// 标准库
use std::path::PathBuf;
use std::fs;

// 第三方库
use anyhow::{Context, Result};
use clap::Args;

// 本地模块
use crate::repo::config::RepoConfig;
use crate::base::util::format_size;
```

### 5.5 自动修复

**命令**：
```bash
make fix
# 或手动执行：
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged
cargo fix --allow-dirty --allow-staged
```

**检查项**：
- [ ] 运行自动修复后是否还有问题
- [ ] 自动修复后的代码是否仍需手动调整

---

## 🔎 其他检查项

### 6.1 版本管理

**检查项**：
- [ ] `Cargo.toml` 版本号是否已更新
- [ ] `Cargo.lock` 是否已更新
- [ ] 变更日志是否需要更新

### 6.2 依赖管理

**检查项**：
- [ ] 新增依赖是否必要
- [ ] 依赖版本是否合理
- [ ] 是否有安全漏洞（可运行 `cargo audit`）

### 6.3 平台兼容性

**检查项**：
- [ ] 是否考虑了跨平台兼容性（macOS/Linux/Windows）
- [ ] 平台特定代码是否使用了条件编译（`#[cfg(...)]`）
- [ ] 是否测试了不同平台

**示例**：
```rust
#[cfg(target_os = "macos")]
fn get_system_proxy() -> Result<String> {
    // macOS 特定实现
}

#[cfg(target_os = "linux")]
fn get_system_proxy() -> Result<String> {
    // Linux 特定实现
}
```

### 6.4 性能检查

**检查项**：
- [ ] 是否有明显的性能问题（如不必要的克隆、重复计算）
- [ ] 大文件处理是否使用了流式处理
- [ ] 网络请求是否使用了重试机制

### 6.5 安全性检查

**检查项**：
- [ ] 敏感信息（API keys、tokens）是否硬编码
- [ ] 文件操作是否检查了路径安全性
- [ ] 用户输入是否进行了验证

### 6.6 用户体验

**检查项**：
- [ ] 错误消息是否对用户友好
- [ ] 进度提示是否清晰
- [ ] 帮助信息是否完整（`--help`）

---

## 📄 生成检查报告

完成所有检查步骤后，需要生成检查报告文档。

### 报告位置

**文件路径**：`docs/report/CHECK_REPORT_{timestamp}.md`

其中 `{timestamp}` 为当前日期和时间，格式为 `YYYY-MM-DD_HH-MM-SS`（如：`2024-12-19_14-30-00`）。

**生成带时间戳的文件名**：

在 Rust 代码中（自动获取当前时间）：
```rust
use workflow::base::util::date::format_filename_timestamp;

// 函数会自动获取当前时间，无需提前获取
let timestamp = format_filename_timestamp();
let report_path = format!("docs/report/CHECK_REPORT_{}.md", timestamp);
// 输出：docs/report/CHECK_REPORT_2024-12-19_14-30-00.md
```

> **注意**：`format_filename_timestamp()` 函数会在调用时自动获取当前系统时间，无需提前获取。每次调用都会返回最新的时间戳。

在命令行中（手动获取当前时间）：
```bash
# Unix/macOS/Linux
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
echo "docs/report/CHECK_REPORT_${TIMESTAMP}.md"

# Windows PowerShell
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
Write-Host "docs/report/CHECK_REPORT_${timestamp}.md"
```

**示例文件名**：`docs/report/CHECK_REPORT_2024-12-19_14-30-00.md`

### 报告内容

检查报告应包含以下内容：

#### 1. 检查概览

- 检查日期和时间
- 检查人员
- 检查范围（本次提交涉及的功能/模块）

#### 2. 各步骤检查结果

按照检查步骤，记录每个步骤的检查结果：

**第一步：CLI 和 Completion 检查**
- [ ] CLI 命令结构检查结果
- [ ] 补全脚本完整性测试结果
- [ ] 补全脚本优化情况
- 问题记录和修复情况

**第二步：代码优化检查**
- [ ] 共用代码提取情况
- [ ] 代码重复检查结果
- [ ] 工具函数复用情况
- [ ] 配置管理情况
- 问题记录和修复情况

**第三步：测试用例检查**
- [ ] 单元测试覆盖情况
- [ ] 集成测试覆盖情况
- [ ] 文档测试结果
- [ ] 测试数据和边界情况
- [ ] 测试冗余检查结果（重复测试、过时测试、无效测试）
- 问题记录和修复情况

**第四步：代码质量检查**
- [ ] 代码格式化结果
- [ ] Clippy 警告检查结果
- [ ] 编译检查结果
- [ ] 导入语句检查结果
- [ ] 自动修复情况
- 问题记录和修复情况

**第五步：文档检查**
- [ ] README.md 更新情况
- [ ] 架构文档更新情况
- [ ] 文档索引更新情况
- [ ] 迁移文档情况
- 问题记录和修复情况

**第六步：其他检查项**
- [ ] 版本管理检查结果
- [ ] 依赖管理检查结果
- [ ] 平台兼容性检查结果
- [ ] 性能检查结果
- [ ] 安全性检查结果
- [ ] 用户体验检查结果
- 问题记录和修复情况

#### 3. 问题汇总

- **P0（必须修复）**：列出所有必须修复的问题
- **P1（建议修复）**：列出所有建议修复的问题
- **P2（可选修复）**：列出所有可选修复的问题

每个问题应包含：
- 问题描述
- 问题位置（文件路径和行号）
- 修复状态（已修复/待修复/已记录）
- 修复方案（如适用）

#### 4. 测试结果汇总

- 单元测试：通过/失败数量
- 集成测试：通过/失败数量
- 文档测试：通过/失败数量
- 补全完整性测试：通过/失败数量
- 总体测试覆盖率（如适用）

#### 5. 代码质量指标

- 代码格式化：通过/失败
- Clippy 警告：数量（0 为通过）
- 编译状态：通过/失败
- 代码重复率（如适用）

#### 6. 总结

- 检查完成度（百分比）
- 是否准备好提交（是/否）
- 剩余待办事项
- 建议和备注

### 报告模板

```markdown
# 提交前检查报告

**检查日期**：YYYY-MM-DD HH:MM:SS
**检查人员**：[姓名]
**检查范围**：[本次提交涉及的功能/模块]

---

## 检查概览

[简要描述本次检查的范围和目的]

---

## 各步骤检查结果

### 第一步：CLI 和 Completion 检查

**状态**：✅ 通过 / ⚠️ 部分通过 / ❌ 未通过

**检查项**：
- [x] CLI 命令结构检查结果：[描述]
- [x] 补全脚本完整性测试结果：[描述]
- [x] 补全脚本优化情况：[描述]

**问题记录**：
- [问题描述] - [修复状态] - [修复方案]

---

### 第二步：代码优化检查

**状态**：✅ 通过 / ⚠️ 部分通过 / ❌ 未通过

**检查项**：
- [x] 共用代码提取情况：[描述]
- [x] 代码重复检查结果：[描述]
- [x] 工具函数复用情况：[描述]
- [x] 配置管理情况：[描述]

**问题记录**：
- [问题描述] - [修复状态] - [修复方案]

---

### 第三步：测试用例检查

**状态**：✅ 通过 / ⚠️ 部分通过 / ❌ 未通过

**检查项**：
- [x] 单元测试覆盖情况：[描述]
- [x] 集成测试覆盖情况：[描述]
- [x] 文档测试结果：[描述]
- [x] 测试数据和边界情况：[描述]
- [x] 测试冗余检查结果：[描述]（重复测试、过时测试、无效测试）

**问题记录**：
- [问题描述] - [修复状态] - [修复方案]

---

### 第四步：代码质量检查

**状态**：✅ 通过 / ⚠️ 部分通过 / ❌ 未通过

**检查项**：
- [x] 代码格式化结果：[描述]
- [x] Clippy 警告检查结果：[描述]
- [x] 编译检查结果：[描述]
- [x] 导入语句检查结果：[描述]
- [x] 自动修复情况：[描述]

**问题记录**：
- [问题描述] - [修复状态] - [修复方案]

---

### 第五步：文档检查

**状态**：✅ 通过 / ⚠️ 部分通过 / ❌ 未通过

**检查项**：
- [x] README.md 更新情况：[描述]
- [x] 架构文档更新情况：[描述]
- [x] 文档索引更新情况：[描述]
- [x] 迁移文档情况：[描述]

**问题记录**：
- [问题描述] - [修复状态] - [修复方案]

---

### 第六步：其他检查项

**状态**：✅ 通过 / ⚠️ 部分通过 / ❌ 未通过

**检查项**：
- [x] 版本管理检查结果：[描述]
- [x] 依赖管理检查结果：[描述]
- [x] 平台兼容性检查结果：[描述]
- [x] 性能检查结果：[描述]
- [x] 安全性检查结果：[描述]
- [x] 用户体验检查结果：[描述]

**问题记录**：
- [问题描述] - [修复状态] - [修复方案]

---

## 问题汇总

### P0（必须修复）

- [ ] [问题描述] - [位置] - [修复状态]

### P1（建议修复）

- [ ] [问题描述] - [位置] - [修复状态]

### P2（可选修复）

- [ ] [问题描述] - [位置] - [修复状态]

---

## 测试结果汇总

- **单元测试**：X 通过，Y 失败
- **集成测试**：X 通过，Y 失败
- **文档测试**：X 通过，Y 失败
- **补全完整性测试**：X 通过，Y 失败

---

## 代码质量指标

- **代码格式化**：✅ 通过 / ❌ 失败
- **Clippy 警告**：X 个警告
- **编译状态**：✅ 通过 / ❌ 失败

---

## 总结

**检查完成度**：X%

**是否准备好提交**：✅ 是 / ❌ 否

**剩余待办事项**：
- [待办事项 1]
- [待办事项 2]

**建议和备注**：
[其他建议和备注]
```

### 生成报告命令

完成所有检查后，创建或更新报告文件：

```bash
# 创建或更新检查报告
vim docs/CHECK_REPORT.md
# 或使用其他编辑器
```

### 报告使用

- 报告应保存在 `docs/CHECK_REPORT.md`
- 报告应在提交代码前完成
- 报告可用于代码审查和问题追踪
- 报告应包含所有检查步骤的详细结果

---

## ⚡ 快速检查清单

在提交代码前，运行以下命令：

```bash
# 1. 代码质量检查
make lint

# 2. 运行所有测试（包括单元测试、集成测试和文档测试）
make test
# 或
cargo test --verbose  # 与 make test 行为一致，但会显示详细输出

# 3. 补全脚本完整性测试
cargo test --test completeness

# 4. 构建验证
cargo build --release

# 5. 使用内置检查命令（如果可用）
workflow check
```

### 检查优先级

#### 必须完成（P0）
1. ✅ 代码质量检查（`make lint`）
2. ✅ 测试通过（`make test`）
3. ✅ 补全脚本完整性（`cargo test --test completeness`）
4. ✅ 编译通过（`cargo check`）

#### 应该完成（P1）
1. ✅ 文档更新（README.md、架构文档）
2. ✅ CLI 命令注册和补全
3. ✅ 基本测试用例

#### 建议完成（P2）
1. ✅ 代码优化（提取共用代码）
2. ✅ 测试覆盖率提升
3. ✅ 性能优化

---

## ❓ 常见问题

### Q: 如何快速检查所有项？

**A**: 运行 `make lint && make test`，然后手动检查文档和测试覆盖率。

### Q: 补全脚本测试失败怎么办？

**A**: 检查 `src/lib/cli/` 中的命令结构，确保所有命令都已正确注册。运行 `cargo run -- completion generate` 手动生成补全脚本验证。

### Q: 如何提取共用代码？

**A**: 参考 [代码优化检查](#-代码优化检查) 部分的具体示例：
- 使用 clap 的 `Args` trait 和 `#[command(flatten)]` 提取共用参数
- 创建 `helpers.rs` 模块提取共用工具函数
- 使用 `anyhow::Context` 统一错误处理

### Q: 文档更新优先级？

**A**: 优先更新 README.md 和相关的命令架构文档，其他文档可以后续补充。

### Q: 如何验证补全脚本？

**A**:
1. 运行 `cargo test --test completeness` 验证完整性
2. 运行 `cargo run -- completion generate` 生成补全脚本
3. 手动测试补全功能（在 shell 中按 Tab 键）

### Q: 代码优化是必须的吗？

**A**: 不是必须的，但强烈建议。代码优化可以提高代码质量、可维护性和可读性。如果时间紧迫，可以先完成 P0 和 P1 项，P2 项可以后续优化。

---

## 📝 提交前最终检查

在提交代码前，确认以下所有项：

- [ ] 所有 P0 检查项已通过
- [ ] 代码已格式化（`cargo fmt`）
- [ ] 测试全部通过（`cargo test`）
- [ ] 文档已更新
- [ ] 提交信息清晰且符合规范
- [ ] 已推送到远程分支
- [ ] PR 描述完整（如适用）

---

**完成以上检查后，代码即可提交！** 🎉
