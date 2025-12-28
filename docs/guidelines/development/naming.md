# 命名规范

> 本文档定义了 Workflow CLI 项目的命名规范和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [概述](#-概述)
- [文件命名](#-文件命名)
- [函数命名](#-函数命名)
- [结构体命名](#-结构体命名)
- [常量命名](#-常量命名)
- [CLI 参数命名规范](#-cli-参数命名规范)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档定义了命名规范，包括文件、函数、结构体、常量和 CLI 参数的命名规范。

### 核心原则

- **一致性**：相同语义的命名必须保持一致
- **可读性**：命名应清晰表达意图
- **遵循约定**：遵循 Rust 官方命名约定

### 使用场景

- 编写新代码时参考
- 代码审查时检查
- 重构代码时使用

---

## 文件命名

- **模块文件**：`snake_case.rs`（如 `jira_client.rs`、`pr_helpers.rs`）
- **测试文件**：与源文件同名，放在 `tests/` 目录或使用 `#[cfg(test)]` 模块
- **文档文件**：`kebab-case.md`（如 `testing/README.md`、`pr.md`）
  - **架构文档**：`{module}.md`（如 `pr.md`、`git.md`，包含 Lib 层和 Commands 层两部分）
  - **指南文档**：`{topic}.md`（如 `testing/README.md`、`document.md`）
  - **需求文档**：`{topic}.md`（如 `jira.md`、`integration.md`，存放到 `docs/requirements/`）
  - **迁移文档**：`{version}-to-{version}.md`（如 `1.5.6-to-1.5.7.md`）

---

## 函数命名

- **动作函数**：使用动词（如 `download`、`create`、`merge`）
- **查询函数**：使用 `get_` 前缀（如 `get_status`、`get_info`）
- **检查函数**：使用 `is_` 或 `has_` 前缀（如 `is_valid`、`has_permission`）
- **转换函数**：使用 `to_` 或 `into_` 前缀（如 `to_string`、`into_json`）

---

## 结构体命名

- 使用名词或名词短语（如 `HttpClient`、`JiraTicket`）
- 避免使用 `Data`、`Info`、`Manager` 等泛化名称，使用具体名称

---

## 常量命名

- 使用 `SCREAMING_SNAKE_CASE`
- 放在模块顶层或专门的常量模块中

```rust
// src/lib/jira/logs/constants.rs
pub const MAX_DOWNLOAD_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

---

## CLI 参数命名规范

CLI 参数命名需要遵循以下规范，确保一致性和可维护性。

### 结构体字段名

- 使用 `snake_case`（如 `jira_id`、`dry_run`、`output_format`）

```rust
#[derive(Args, Debug, Clone)]
pub struct JiraIdArg {
    pub jira_id: Option<String>,  // ✅ snake_case
}
```

### value_name 规范

- 使用 `SCREAMING_SNAKE_CASE`（如 `JIRA_ID`、`DRY_RUN`、`PR_ID`）
- 用于在帮助信息中显示参数值的占位符

```rust
/// Jira ticket ID (optional, will prompt interactively if not provided)
#[arg(value_name = "JIRA_ID")]  // ✅ SCREAMING_SNAKE_CASE
pub jira_id: Option<String>,
```

### 参数长名规范

- 使用 `kebab-case`（clap 自动从字段名转换，如 `--jira-id`、`--dry-run`）
- 字段名使用 `snake_case`，clap 会自动转换为 `kebab-case`

```rust
#[arg(long)]  // 自动生成 --jira-id
pub jira_id: Option<String>,
```

### 参数短名规范

- 使用单个字符（如 `-n`、`-f`、`-v`）
- 优先使用常见的短名（如 `-n` 用于 dry-run，`-f` 用于 force）

```rust
#[arg(long, short = 'n', action = clap::ArgAction::SetTrue)]
pub dry_run: bool,  // --dry-run 或 -n
```

### 参数类型规范

- **可选参数**：使用 `Option<T>`
- **必需参数**：直接使用类型（如 `String`、`usize`）
- **布尔标志**：使用 `bool` + `action = clap::ArgAction::SetTrue`

```rust
// ✅ 可选参数
#[arg(value_name = "JIRA_ID")]
pub jira_id: Option<String>,

// ✅ 必需参数
#[arg(value_name = "BRANCH_NAME")]
pub branch_name: String,

// ✅ 布尔标志
#[arg(long, short = 'f', action = clap::ArgAction::SetTrue)]
pub force: bool,
```

### 文档注释规范

所有参数必须有文档注释，说明参数的用途、格式和默认行为：

```rust
/// Jira ticket ID (optional, will prompt interactively if not provided)
///
/// Examples:
///   workflow jira info PROJ-123
///   workflow jira info  # Will prompt for JIRA ID
#[arg(value_name = "JIRA_ID")]
pub jira_id: Option<String>,
```

### 命名一致性规范

- **相同语义的参数必须使用相同的命名**：
  - ✅ 统一使用 `jira_id`（而不是 `jira_ticket`、`jira-id` 等）
  - ✅ 统一使用 `dry_run`（而不是 `dry-run`、`dryrun` 等）
  - ✅ 统一使用 `output_format`（而不是 `format`、`output` 等）

- **value_name 必须与字段名语义一致**：
  - 字段名：`jira_id` → value_name：`JIRA_ID`
  - 字段名：`dry_run` → value_name：`DRY_RUN`（但通常布尔标志不需要 value_name）

### 共用参数规范

对于在多个命令中重复使用的参数，应该提取为共用参数组（见 [CLI 检查指南](./references/review-cli.md)）：

```rust
// src/lib/cli/args.rs
/// 可选 JIRA ID 参数
#[derive(Args, Debug, Clone)]
pub struct JiraIdArg {
    /// Jira ticket ID (optional, will prompt interactively if not provided)
    #[arg(value_name = "JIRA_ID")]
    pub jira_id: Option<String>,
}

// 在命令中使用
use super::args::JiraIdArg;

#[derive(Subcommand)]
pub enum MySubcommand {
    Info {
        #[command(flatten)]
        jira_id: JiraIdArg,  // ✅ 使用共用参数
    },
}
```

### 示例对比

```rust
// ❌ 不好的做法
Create {
    #[arg(value_name = "jira_ticket")]  // value_name 应该大写
    jira_ticket: Option<String>,  // 命名不一致（应该用 jira_id）
}

// ✅ 好的做法
Create {
    #[command(flatten)]
    jira_id: JiraIdArg,  // 使用共用参数，命名一致
}
```

**参考**：
- [CLI 检查指南](./references/review-cli.md) - 参数复用检查和参数提取指南
- [clap 文档](https://docs.rs/clap/) - clap 参数定义规范

---

## 🔍 故障排除

### 问题 1：命名不一致

**症状**：相同语义的命名在不同地方使用了不同的形式

**解决方案**：

1. 统一使用项目命名规范
2. 使用共用参数组避免重复定义
3. 在代码审查时检查命名一致性

### 问题 2：CLI 参数命名混乱

**症状**：CLI 参数命名不规范，导致帮助信息不清晰

**解决方案**：

1. 遵循 CLI 参数命名规范
2. 使用 `value_name` 提供清晰的占位符
3. 添加文档注释说明参数用途

---

## 📚 相关文档

### 开发规范

- [代码风格规范](./code-style.md) - 代码风格规范
- [模块组织规范](./module-organization.md) - 模块组织规范

### 检查工作流

- [CLI 检查指南](./references/review-cli.md) - CLI 参数检查流程

---

## ✅ 检查清单

使用本规范时，请确保：

- [ ] 文件命名遵循规范
- [ ] 函数命名清晰表达意图
- [ ] 结构体命名使用具体名称
- [ ] 常量命名使用 `SCREAMING_SNAKE_CASE`
- [ ] CLI 参数命名一致
- [ ] 使用共用参数组避免重复

---

**最后更新**: 2025-12-23

