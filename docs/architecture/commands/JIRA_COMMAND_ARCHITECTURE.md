# Jira 命令模块架构文档

## 📋 概述

Jira 命令层是 Workflow CLI 的命令接口，提供 Jira ticket 信息查看和附件下载等功能。该层采用命令模式设计，通过调用 `lib/jira/` 模块提供的 API 实现业务功能。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/jira/` 模块提供。

**命令结构**：
- `workflow jira` - Jira 操作命令（info, related, changelog, comment, comments, attachments, clean）

---

## 📁 相关文件

### CLI 入口层

Jira 命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Jira` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow jira` 子命令分发到对应的命令处理函数

### 命令封装层 (`commands/jira/`)

```
src/commands/jira/
├── mod.rs          # Jira 命令模块声明
├── info.rs         # 显示 ticket 信息命令（~354 行）
├── related.rs      # 显示关联信息命令（PR 和分支）
├── changelog.rs    # 显示变更历史命令（~200 行）
├── comment.rs      # 添加评论命令（~191 行）
├── comments.rs     # 显示评论命令（~313 行）
├── attachments.rs  # 下载附件命令（~30 行）
└── clean.rs        # 清理本地数据命令（~58 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/jira/`) 的 API

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/jira/`**：Jira 集成
  - `Jira::get_ticket_info()` - 获取 ticket 信息
- **`lib/jira/api/`**：Jira API 接口
  - `JiraIssueApi::get_issue_changelog()` - 获取变更历史
- **`lib/jira/history/`**：Jira 工作历史模块（`JiraWorkHistory`）
  - `JiraWorkHistory::find_prs_by_jira_ticket()` - 查找关联的 PR
- **`lib/git/`**：Git 操作模块
  - `GitBranch::find_branches_by_jira_ticket()` - 查找关联的分支
- **`lib/jira/logs/`**：Jira 日志处理模块（`JiraLogs`）
  - `JiraLogs::new()` - 创建日志管理器
  - `JiraLogs::download_from_jira()` - 下载附件
  - `JiraLogs::clean_dir()` - 清理目录
- **`lib/base/settings/`**：配置管理
  - `Settings::get()` - 获取配置（`log_output_folder_name`、`log_download_base_dir` 等）

详细架构文档：参见 [Jira 模块架构文档](../lib/JIRA_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/jira/*.rs (命令封装层，处理交互)
  ↓
lib/jira/ (通过 Jira API 调用，具体实现见相关模块文档)
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand
  ├─ Info → InfoCommand::show()
  ├─ Changelog → ChangelogCommand::show()
  ├─ Comment → CommentCommand::add()
  ├─ Comments → CommentsCommand::show()
  ├─ Attachments → AttachmentsCommand::download()
  └─ Clean → CleanCommand::clean()
```

---

## 1. 显示 Ticket 信息命令 (`info`)

### 相关文件

```
src/commands/jira/info.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::JiraSubcommand::Info
  ↓
commands/jira/info.rs::InfoCommand::show(jira_id)
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 调用 Jira::get_ticket_info(jira_id) 获取 ticket 信息
  3. 显示基本信息（Key, ID, Summary, Status）
  4. 显示描述（如果有）
  5. 显示附件列表（如果有）
  6. 显示评论数量（如果有）
  7. 显示 Jira URL
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（可选，不提供时会交互式输入）
   - `output_format` - 输出格式选项（使用共用参数组 `OutputFormatArgs`）
     - `--table` - 表格格式输出（默认）
     - `--json` - JSON 格式输出
     - `--yaml` - YAML 格式输出
     - `--markdown` - Markdown 格式输出

   **注意**：输出格式参数使用 `OutputFormatArgs` 共用参数组，通过 `#[command(flatten)]` 特性展开。这样可以减少代码重复，提高可维护性。详见 [CLI 架构文档](../lib/CLI_ARCHITECTURE.md)。

2. **用户交互**：
   - 如果未提供 `jira_id`，使用 `dialoguer::Input` 交互式输入（提示："Enter Jira ticket ID (e.g., PROJ-123)"）
   - 格式化显示 ticket 信息
   - 使用分隔线和图标美化输出

3. **核心功能**：
   - 通过 `Jira::get_ticket_info()` API 获取 ticket 信息
   - 格式化显示所有相关信息

### 关键步骤说明

1. **信息获取**：
   - 调用 `Jira::get_ticket_info()` 获取 ticket 信息

2. **信息展示**：
   - 显示基本信息（Key, ID, Summary, Status）
   - 显示描述（如果有）
   - 显示附件列表（格式化文件大小）
   - 显示评论数量
   - 显示 Jira URL

### Jira API 调用

- **`Jira::get_ticket_info(jira_id)`** - 获取 ticket 信息
  - 参数：`jira_id` - Jira ticket ID
  - 返回：Issue 结构体（包含所有 ticket 信息）

---

## 2. 显示关联信息命令 (`related`)

### 相关文件

```
src/commands/jira/related.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::JiraSubcommand::Related
  ↓
commands/jira/related.rs::RelatedCommand::show()
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 确定输出格式（table、json、yaml、markdown）
  3. 调用 JiraWorkHistory::find_prs_by_jira_ticket() 查找关联的 PR
  4. 调用 GitBranch::find_branches_by_jira_ticket() 查找关联的分支
  5. 根据输出格式格式化显示关联信息
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（可选，不提供时会交互式输入）
   - `output_format` - 输出格式选项（使用共用参数组 `OutputFormatArgs`）
     - `--table` - 表格格式输出（默认）
     - `--json` - JSON 格式输出
     - `--yaml` - YAML 格式输出
     - `--markdown` - Markdown 格式输出

2. **用户交互**：
   - 如果未提供 `jira_id`，使用 `InputDialog` 交互式输入
   - 支持多种输出格式，便于脚本处理和文档生成

3. **核心功能**：
   - 通过 `JiraWorkHistory::find_prs_by_jira_ticket()` 查找关联的 PR
   - 通过 `GitBranch::find_branches_by_jira_ticket()` 查找关联的分支
   - 显示 PR 信息（URL、分支、创建时间、合并时间等）
   - 显示分支信息（分支名、最后提交时间等）

### 关键步骤说明

1. **PR 查找**：
   - 调用 `JiraWorkHistory::find_prs_by_jira_ticket()` 从工作历史中查找关联的 PR
   - 返回包含 PR URL、分支、创建时间、合并时间等信息的列表

2. **分支查找**：
   - 调用 `GitBranch::find_branches_by_jira_ticket()` 从 Git 仓库中查找关联的分支
   - 返回包含分支名、最后提交时间等信息的列表

3. **格式化输出**：
   - **表格格式**：人类可读的表格格式，显示 PR 和分支详情
   - **JSON 格式**：结构化 JSON 输出，便于脚本处理
   - **YAML 格式**：结构化 YAML 输出（当前使用 JSON）
   - **Markdown 格式**：Markdown 格式输出，便于文档生成

### API 调用

- **`JiraWorkHistory::find_prs_by_jira_ticket(jira_id)`** - 查找关联的 PR
  - 参数：`jira_id` - Jira ticket ID
  - 返回：PR 条目列表（包含 PR URL、分支、时间等信息）

- **`GitBranch::find_branches_by_jira_ticket(jira_id)`** - 查找关联的分支
  - 参数：`jira_id` - Jira ticket ID
  - 返回：分支列表（包含分支名、最后提交时间等信息）

### 使用示例

```bash
# 显示关联的 PR 和分支
workflow jira related PROJ-123

# JSON 格式输出
workflow jira related PROJ-123 --json

# Markdown 格式输出
workflow jira related PROJ-123 --markdown

# 交互式输入 JIRA ID
workflow jira related
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
```

---

## 3. 显示变更历史命令 (`changelog`)

### 相关文件

```
src/commands/jira/changelog.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::JiraSubcommand::Changelog
  ↓
commands/jira/changelog.rs::ChangelogCommand::show()
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 确定输出格式（table、json、yaml、markdown）
  3. 调用 JiraIssueApi::get_issue_changelog(jira_id) 获取变更历史
  4. 根据输出格式格式化显示变更历史
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（可选，不提供时会交互式输入）
   - `output_format` - 输出格式选项（使用共用参数组 `OutputFormatArgs`）
     - `--table` - 表格格式输出（默认）
     - `--json` - JSON 格式输出
     - `--yaml` - YAML 格式输出
     - `--markdown` - Markdown 格式输出

2. **用户交互**：
   - 如果未提供 `jira_id`，使用 `InputDialog` 交互式输入
   - 支持多种输出格式，便于脚本处理和文档生成

3. **核心功能**：
   - 通过 `JiraIssueApi::get_issue_changelog()` API 获取变更历史
   - 显示所有字段的变更记录
   - 显示变更时间、作者、字段变更详情

### 关键步骤说明

1. **变更历史获取**：
   - 调用 `JiraIssueApi::get_issue_changelog()` 获取完整的变更历史
   - 返回包含所有历史记录的 changelog 数据

2. **格式化输出**：
   - **表格格式**：人类可读的表格格式，显示变更详情
   - **JSON 格式**：结构化 JSON 输出，便于脚本处理
   - **YAML 格式**：结构化 YAML 输出（当前使用 JSON）
   - **Markdown 格式**：Markdown 格式输出，便于文档生成

### Jira API 调用

- **`JiraIssueApi::get_issue_changelog(jira_id)`** - 获取变更历史
  - 参数：`jira_id` - Jira ticket ID
  - 返回：Changelog 结构体（包含所有变更历史记录）

### 使用示例

```bash
# 显示所有变更历史
workflow jira changelog PROJ-123

# JSON 格式输出
workflow jira changelog PROJ-123 --json

# Markdown 格式输出
workflow jira changelog PROJ-123 --markdown
```

---

## 4. 显示评论命令 (`comments`)

### 相关文件

```
src/commands/jira/comments.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::JiraSubcommand::Comments
  ↓
commands/jira/comments.rs::CommentsCommand::show()
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 调用 Jira::get_ticket_info(jira_id) 获取 ticket 信息
  3. 提取评论数据（issue.fields.comment）
  4. 应用过滤条件（--author、--since）
  5. 应用分页（--limit、--offset）
  6. 排序（默认降序）
  7. 根据输出格式格式化显示评论
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（可选，不提供时会交互式输入）
   - `--limit <LIMIT>` - 限制显示的评论数量
   - `--offset <OFFSET>` - 分页偏移量
   - `--author <EMAIL>` - 只显示指定作者的评论
   - `--since <DATE>` - 只显示指定日期之后的评论（RFC3339 格式）
   - `output_format` - 输出格式选项（使用共用参数组 `OutputFormatArgs`）
     - `--table` - 表格格式输出（默认）
     - `--json` - JSON 格式输出
     - `--yaml` - YAML 格式输出
     - `--markdown` - Markdown 格式输出

2. **用户交互**：
   - 如果未提供 `jira_id`，使用 `InputDialog` 交互式输入
   - 支持多种输出格式，便于脚本处理和文档生成

3. **核心功能**：
   - 通过 `Jira::get_ticket_info()` API 获取 ticket 信息（包含评论）
   - 支持按作者、时间过滤评论
   - 支持分页显示评论
   - 默认按时间降序排序（最新的在前）

### 关键步骤说明

1. **评论获取**：
   - 调用 `Jira::get_ticket_info()` 获取 ticket 信息
   - 从 `issue.fields.comment` 提取评论数据

2. **过滤和排序**：
   - **按作者过滤**：使用 `--author` 选项，匹配邮箱地址
   - **按时间过滤**：使用 `--since` 选项，只显示指定日期之后的评论
   - **排序**：默认按创建时间降序排序（最新的在前）

3. **分页**：
   - 使用 `--limit` 限制显示的评论数量
   - 使用 `--offset` 指定分页偏移量

4. **格式化输出**：
   - **表格格式**：人类可读的表格格式，显示评论详情
   - **JSON 格式**：结构化 JSON 输出，便于脚本处理
   - **YAML 格式**：结构化 YAML 输出（当前使用 JSON）
   - **Markdown 格式**：Markdown 格式输出，便于文档生成

### Jira API 调用

- **`Jira::get_ticket_info(jira_id)`** - 获取 ticket 信息（包含评论）
  - 参数：`jira_id` - Jira ticket ID
  - 返回：Issue 结构体（包含评论数据）

### 使用示例

```bash
# 显示所有评论
workflow jira comments PROJ-123

# 只显示最近 10 条评论
workflow jira comments PROJ-123 --limit 10

# 只显示指定作者的评论
workflow jira comments PROJ-123 --author user@example.com

# 只显示指定日期之后的评论
workflow jira comments PROJ-123 --since 2025-01-01T00:00:00Z

# JSON 格式输出
workflow jira comments PROJ-123 --json

# Markdown 格式输出
workflow jira comments PROJ-123 --markdown
```

---

## 5. 下载附件命令 (`attachments`)

### 相关文件

```
src/commands/jira/attachments.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::JiraSubcommand::Attachments
  ↓
commands/jira/attachments.rs::AttachmentsCommand::download(jira_id)
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 显示下载提示信息
  3. 创建 JiraLogs 实例：JiraLogs::new()
  4. 调用 JiraLogs::download_from_jira(jira_id, None, true)
     └─ 内部处理：下载所有附件、合并分片、解压文件
  5. 输出成功信息和文件路径
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（可选，不提供时会交互式输入）

2. **用户交互**：
   - 如果未提供 `jira_id`，使用 `dialoguer::Input` 交互式输入（提示："Enter Jira ticket ID (e.g., PROJ-123)"）
   - 显示下载进度和结果

3. **核心功能**：
   - 通过 `JiraLogs::download_from_jira()` API 实现下载功能
   - 下载所有附件（不仅仅是日志附件）
   - 自动处理附件下载、分片合并、文件解压等操作

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例（自动加载配置和初始化 HTTP 客户端）

2. **下载执行**：
   - 调用 `JiraLogs::download_from_jira()` 执行下载
   - 下载所有附件（`download_all_attachments = true`）
   - 自动处理分片 ZIP 文件的合并和解压

3. **结果输出**：
   - 显示下载成功信息
   - 显示文件保存路径

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::download_from_jira(jira_id, output_folder, download_all_attachments)`** - 下载附件
  - 参数：
    - `jira_id` - Jira ticket ID
    - `output_folder` - 输出文件夹名称（可选，None 时使用配置的默认值）
    - `download_all_attachments` - 是否下载所有附件（true）
  - 返回：基础目录路径

---

## 6. 清理本地数据命令 (`clean`)

### 相关文件

```
src/commands/jira/clean.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::JiraSubcommand::Clean
  ↓
commands/jira/clean.rs::CleanCommand::clean(jira_id, dry_run, list_only)
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 根据参数显示不同的提示信息
  3. 创建 JiraLogs 实例：JiraLogs::new()
  4. 调用 JiraLogs::clean_dir(jira_id, dry_run, list_only)
     └─ 内部处理：计算目录信息、列出内容、预览或删除
  5. 输出操作结果
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（可选，不提供时会交互式输入；如果为空字符串，会报错）
   - `all` - 如果为 true，清理整个基础目录（忽略 jira_id）
   - `dry_run` - Dry run 模式选项（使用共用参数组 `DryRunArgs`）
     - `--dry-run` / `-n` - 预览模式，不实际删除
   - `list_only` - 只列出目录内容

   **注意**：`dry_run` 参数使用 `DryRunArgs` 共用参数组，通过 `#[command(flatten)]` 特性展开。这样可以减少代码重复，提高可维护性。详见 [CLI 架构文档](../lib/CLI_ARCHITECTURE.md)。

2. **用户交互**：
   - 如果未提供 `jira_id` 且未指定 `--all`，使用 `dialoguer::Input` 交互式输入
     - 提示："Enter Jira ticket ID (e.g., PROJ-123, or leave empty to clean all)"
     - 如果用户直接按 Enter（留空），清理整个基础目录
   - 如果提供了空字符串作为 `jira_id` 参数，会报错（应使用 `--all` 或省略参数）
   - 如果指定了 `--all`，直接清理整个基础目录（忽略 jira_id）
   - 根据参数显示不同的提示信息
   - 在删除前会显示目录信息和确认对话框
   - 显示操作结果

3. **核心功能**：
   - 通过 `JiraLogs::clean_dir()` API 清理本地数据目录
   - 支持预览模式和列表模式

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例

2. **操作执行**：
   - 调用 `JiraLogs::clean_dir()` 执行清理操作
   - 根据参数决定操作模式（预览、列表、删除）

3. **结果输出**：
   - 显示操作结果
   - 如果删除成功，显示成功信息

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::clean_dir(jira_id, dry_run, list_only)`** - 清理本地数据目录
  - 参数：
    - `jira_id` - Jira ticket ID（空字符串表示清理整个基础目录）
    - `dry_run` - 预览模式
    - `list_only` - 只列出目录内容
  - 返回：是否成功删除（bool）

### 数据流

#### Info 命令数据流

```
命令行参数 (JIRA_ID, --json, --yaml, --markdown)
  ↓
InfoCommand::show()
  ↓
Jira::get_ticket_info()
  ↓
Jira API (获取 ticket 信息)
  ↓
格式化显示 ticket 信息
```

#### Related 命令数据流

```
命令行参数 (JIRA_ID, --json, --yaml, --markdown)
  ↓
RelatedCommand::show()
  ↓
JiraWorkHistory::find_prs_by_jira_ticket()
  ↓
GitBranch::find_branches_by_jira_ticket()
  ↓
格式化显示关联的 PR 和分支信息
```

#### Attachments 命令数据流

```
命令行参数 (JIRA_ID)
  ↓
AttachmentsCommand::download()
  ↓
JiraLogs::download_from_jira()
  ↓
Jira API (获取附件列表)
  ↓
下载所有附件到本地
  ↓
合并分片、解压文件
  ↓
输出文件路径
```

#### Changelog 命令数据流

```
命令行参数 (JIRA_ID, --json, --yaml, --markdown)
  ↓
ChangelogCommand::show()
  ↓
JiraIssueApi::get_issue_changelog()
  ↓
Jira API (获取变更历史)
  ↓
格式化显示变更历史
```

#### Comments 命令数据流

```
命令行参数 (JIRA_ID, --limit, --offset, --author, --since, --json, --yaml, --markdown)
  ↓
CommentsCommand::show()
  ↓
Jira::get_ticket_info()
  ↓
Jira API (获取 ticket 信息，包含评论)
  ↓
过滤评论（按作者、时间）
  ↓
排序和分页
  ↓
格式化显示评论
```

#### Clean 命令数据流

```
命令行参数 (JIRA_ID, --dry-run, --list)
  ↓
CleanCommand::clean()
  ↓
JiraLogs::clean_dir()
  ↓
计算目录信息、列出内容
  ↓
预览或删除目录
  ↓
输出操作结果
```

---

## 🏗️ 架构设计

### 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `InfoCommand::show()` - 显示 ticket 信息
- `RelatedCommand::show()` - 显示关联的 PR 和分支信息
- `ChangelogCommand::show()` - 显示变更历史
- `CommentsCommand::show()` - 显示评论
- `AttachmentsCommand::download()` - 下载所有附件
- `CleanCommand::clean()` - 清理本地数据

### 2. 分层调用模式

**命令层（CLI → Commands）**：
所有命令通过 `src/main.rs` 直接调用对应的命令结构体：
```
src/main.rs::main()
  ↓
match cli.subcommand
  ├─ Info → InfoCommand::show()
  ├─ Changelog → ChangelogCommand::show()
  ├─ Comment → CommentCommand::add()
  ├─ Comments → CommentsCommand::show()
  ├─ Attachments → AttachmentsCommand::download()
  └─ Clean → CleanCommand::clean()
```

**库层调用（Commands → Jira/JiraLogs）**：
命令层通过 `Jira` 和 `JiraLogs` API 调用核心业务逻辑：
```
InfoCommand::show()
  ↓
Jira::get_ticket_info()

RelatedCommand::show()
  ↓
JiraWorkHistory::find_prs_by_jira_ticket()
  ↓
GitBranch::find_branches_by_jira_ticket()

ChangelogCommand::show()
  ↓
JiraIssueApi::get_issue_changelog()

CommentsCommand::show()
  ↓
Jira::get_ticket_info()

AttachmentsCommand::download()
  ↓
JiraLogs::new()
  ↓
JiraLogs::download_from_jira()

CleanCommand::clean()
  ↓
JiraLogs::new()
  ↓
JiraLogs::clean_dir()
```

### 3. 依赖注入模式

- 命令层不直接创建依赖，而是通过 `Jira::get_ticket_info()` 和 `JiraLogs::new()` 创建实例
- `JiraLogs` 内部自动加载配置和初始化依赖

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
   - `clap` 自动处理参数验证和错误提示

2. **命令层**：用户交互错误、业务逻辑错误
   - 参数验证错误

3. **库层**：文件操作错误、API 调用错误
   - 通过 `Jira` 和 `JiraLogs` API 返回的错误信息
   - 文件不存在、API 调用失败等

#### 容错机制

- **API 调用错误**：
  - Info 命令：Jira API 调用失败会返回错误信息
  - Attachments 命令：Jira API 调用失败会通过 `JiraLogs` API 返回错误信息

- **文件操作错误**：
  - Clean 命令：目录不存在或删除失败会通过 `JiraLogs` API 返回错误信息

---

## 📝 扩展性

### 添加新命令

1. 在 `commands/jira/` 下创建新的命令文件（如 `new_command.rs`）
2. 实现命令结构体和处理方法（如 `NewCommand::execute()`）
3. 在 `commands/jira/mod.rs` 中导出命令结构体
4. 在 `src/main.rs` 中添加命令枚举（`JiraSubcommand`）
5. 在 `src/main.rs` 的 `main()` 函数中添加命令分发逻辑

### 添加新的用户交互

1. 使用 `dialoguer` 库添加交互式输入
2. 在命令方法中处理用户输入
3. 调用 `Jira` 或 `JiraLogs` API 执行操作

### 添加新的输出格式

1. 在命令方法中格式化输出
2. 使用 `log_*!` 宏输出信息
3. 使用 `log_break!` 宏添加分隔线

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Jira 模块架构文档](../lib/JIRA_ARCHITECTURE.md)
- [日志命令模块架构文档](./LOG_COMMAND_ARCHITECTURE.md)
- [PR 命令模块架构文档](./PR_COMMAND_ARCHITECTURE.md)

---

## 📋 使用示例

### Info 命令

```bash
# 提供 JIRA ID
workflow jira info PROJ-123

# JSON 格式输出
workflow jira info PROJ-123 --json

# Markdown 格式输出
workflow jira info PROJ-123 --markdown

# 交互式输入 JIRA ID
workflow jira info
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
```

### Related 命令

```bash
# 提供 JIRA ID
workflow jira related PROJ-123

# JSON 格式输出
workflow jira related PROJ-123 --json

# Markdown 格式输出
workflow jira related PROJ-123 --markdown

# 交互式输入 JIRA ID
workflow jira related
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
```

### Changelog 命令

```bash
# 提供 JIRA ID
workflow jira changelog PROJ-123

# JSON 格式输出
workflow jira changelog PROJ-123 --json

# Markdown 格式输出
workflow jira changelog PROJ-123 --markdown

# 交互式输入 JIRA ID
workflow jira changelog
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
```

### Comments 命令

```bash
# 提供 JIRA ID
workflow jira comments PROJ-123

# 只显示最近 10 条评论
workflow jira comments PROJ-123 --limit 10

# 只显示指定作者的评论
workflow jira comments PROJ-123 --author user@example.com

# 只显示指定日期之后的评论
workflow jira comments PROJ-123 --since 2025-01-01T00:00:00Z

# JSON 格式输出
workflow jira comments PROJ-123 --json

# Markdown 格式输出
workflow jira comments PROJ-123 --markdown

# 交互式输入 JIRA ID
workflow jira comments
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
```

### Attachments 命令

```bash
# 提供 JIRA ID
workflow jira attachments PROJ-123

# 交互式输入 JIRA ID
workflow jira attachments
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
```

### Clean 命令

```bash
# 交互式输入 JIRA ID（直接按 Enter 则清理全部）
workflow jira clean
# 提示: Enter Jira ticket ID (e.g., PROJ-123, or leave empty to clean all)
# 直接按 Enter 即可清理全部

# 清理指定 JIRA ID 的本地数据目录
workflow jira clean PROJ-123

# 清理整个本地数据基础目录（使用 --all 标志）
workflow jira clean --all
# 或使用短选项
workflow jira clean -a

# 注意：空字符串作为参数会报错
# workflow jira clean ""  # ❌ 会报错，应使用 --all 或省略参数

# 预览清理操作（dry-run）
workflow jira clean --dry-run PROJ-123
# 或使用短选项
workflow jira clean -n PROJ-123

# 列出目录内容
workflow jira clean --list PROJ-123
# 或使用短选项
workflow jira clean -l PROJ-123
```

---

## ✅ 总结

Jira 命令层采用清晰的命令模式设计：

1. **CLI 层**：参数解析和命令分发
2. **命令层**：用户交互和格式化输出
3. **库层调用**：通过 `Jira` 和 `JiraLogs` API 调用核心业务逻辑

**设计优势**：
- ✅ **职责分离**：命令层专注于用户交互和输出格式化
- ✅ **易于扩展**：添加新命令只需实现命令结构体和处理方法
- ✅ **交互友好**：支持交互式输入和参数传递两种方式
- ✅ **错误处理**：完整的错误处理和容错机制

---

**最后更新**: 2025-12-16
