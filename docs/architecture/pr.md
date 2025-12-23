# PR 模块架构文档

## 📋 概述

PR 模块是 Workflow CLI 的核心模块，提供完整的 Pull Request 生命周期管理功能。该模块采用分层架构设计，包括：

- **Lib 层**（`lib/pr/`）：提供 Pull Request 的平台抽象层，支持 GitHub 和 Codeup 平台，通过 `PlatformProvider` trait 实现统一的平台接口
- **Commands 层**（`commands/pr/`）：提供 CLI 命令封装，处理用户交互，协调 Git、Jira 等模块的操作

PR 模块支持 GitHub 和 Codeup 两种代码托管平台，提供 13 个命令（create, merge, close, status, list, update, sync, rebase, pick, summarize, approve, comment, reword），总代码行数约 7000+ 行。

**模块统计：**
- Lib 层代码行数：约 3000+ 行
- Commands 层代码行数：约 4000+ 行
- 命令数量：13 个
- 支持平台：GitHub、Codeup
- 主要结构体：`PlatformProvider` trait、`GitHub`、`CreateGenerator`、`RewordGenerator`、`SummaryGenerator`、`SourcePrInfo`、`ExtractedPrInfo`

---

## 📁 Lib 层架构（核心业务逻辑）

PR 模块（`lib/pr/`）是 Workflow CLI 的核心库模块，提供 Pull Request 的平台抽象层。目前支持 GitHub 平台，通过 `PlatformProvider` trait 实现统一的平台接口，使用工厂函数实现多态分发。该模块专注于平台 API 的抽象和调用，不涉及命令层的业务逻辑。

### 模块结构

```
src/lib/pr/
├── mod.rs              # PR 模块声明和导出
├── platform.rs         # PlatformProvider trait 定义
├── factory.rs          # 平台工厂函数（create_provider）
├── body_parser.rs      # PR Body 解析器（提取 Jira ticket、描述、变更类型等）
├── table.rs            # PR 表格显示结构体
│
├── github/             # GitHub 平台实现
│   ├── mod.rs          # GitHub 模块导出
│   ├── platform.rs     # GitHub 平台实现
│   ├── requests.rs     # GitHub API 请求结构体
│   ├── responses.rs    # GitHub API 响应结构体
│   └── errors.rs       # GitHub 错误处理
│
├── llm/                # LLM 内容生成
│   ├── mod.rs          # LLM 模块导出
│   ├── create.rs       # PR 创建内容生成
│   ├── reword.rs       # PR 标题和描述重写
│   ├── summary.rs      # PR 总结生成
│   ├── file_summary.rs # 单文件总结生成
│   └── helpers.rs      # LLM 辅助函数
│
└── helpers/            # PR 辅助函数（已拆分）
    ├── mod.rs
    ├── pr_id.rs        # PR ID 相关
    ├── repo.rs         # 仓库信息相关
    └── content.rs      # 内容生成相关
```

### 依赖模块

- **`lib/git/`**：Git 操作（检测仓库类型，用于工厂函数自动选择平台）
- **`lib/base/llm/`**：AI 功能（PR 标题生成，通过 `llm.rs` 模块封装）
- **`lib/base/http/`**：HTTP 客户端（API 请求）
- **`lib/base/settings/`**：配置管理（环境变量读取，如 `GITHUB_TOKEN` 等）

**注意**：PR 模块不直接依赖 Jira、Git 分支操作、工具函数等模块，这些集成由命令层（`commands/pr/`）负责协调。

---

## 🏗️ Lib 层架构设计

### 设计原则

1. **平台抽象**：通过 `PlatformProvider` trait 实现统一的平台接口
2. **多态分发**：使用工厂函数 `create_provider()` 实现动态分发
3. **模块化设计**：按平台拆分模块，职责清晰
4. **统一错误处理**：平台特定错误处理统一封装
5. **代码复用**：请求/响应结构体分离，便于维护

### 核心组件

#### 1. 平台抽象层 (`platform.rs`)

**职责**：定义统一的 PR 平台接口和工厂函数

- **`PlatformProvider` trait**：定义所有平台必须实现的 12 个方法
  - `create_pull_request()` - 创建 PR
  - `merge_pull_request()` - 合并 PR
  - `get_pull_request_info()` - 获取 PR 信息
  - `get_pull_request_url()` - 获取 PR URL
  - `get_pull_request_title()` - 获取 PR 标题
  - `get_current_branch_pull_request()` - 获取当前分支的 PR ID
  - `get_pull_requests()` - 列出 PR（可选）
  - `get_pull_request_status()` - 获取 PR 状态
  - `close_pull_request()` - 关闭 PR
  - `add_comment()` - 添加 PR 评论
  - `approve_pull_request()` - 批准 PR
  - `update_pr_base()` - 更新 PR 的 base 分支

- **`create_provider()` 工厂函数**（位于 `factory.rs`）：
  - 自动检测仓库类型（GitHub）
  - 返回 `Box<dyn PlatformProvider>` trait 对象
  - 实现真正的多态分发

- **`PullRequestStatus` 结构体**：PR 状态信息（state, merged, merged_at）

- **`TYPES_OF_CHANGES` 常量**：PR 变更类型定义

#### 2. GitHub 平台实现 (`github/`)

**职责**：GitHub REST API v3 的完整实现

- **`platform.rs`**：实现 `PlatformProvider` trait
- **`requests.rs`**：GitHub API 请求结构体
- **`responses.rs`**：GitHub API 响应结构体
- **`errors.rs`**：GitHub 特定错误处理

**关键特性**：
- 使用 GitHub REST API v3
- 需要 `GITHUB_TOKEN` 环境变量
- 支持所有 trait 方法

#### 3. 工厂函数层 (`factory.rs`)

**职责**：提供平台工厂函数，实现平台实例的创建

- **`create_provider()`**：根据仓库类型创建对应的平台提供者
- 自动检测仓库类型（通过 `GitRepo::detect_repo_type()`）
- 目前仅支持 GitHub 平台

#### 4. LLM 功能层 (`llm/`)

**职责**：提供使用 LLM 生成 PR 内容的功能

- **`CreateGenerator`**：PR 创建内容生成（分支名、标题、描述）
- **`RewordGenerator`**：PR 标题和描述重写（基于 PR diff）
- **`SummaryGenerator`**：PR 总结生成（详细的总结文档）
- **`FileSummaryGenerator`**：单文件修改总结生成

**关键特性**：
- 统一的 Generator 模式（struct + impl）
- 支持 diff 长度限制，避免超过 LLM token 限制
- 使用 `lib/base/llm/` 模块进行 LLM 调用

#### 5. 辅助函数层 (`helpers/`)

**职责**：提供 PR 相关的通用辅助函数

**主要函数**：
- `pr_id.rs`：PR ID 相关函数
  - `extract_pull_request_id_from_url()` - 从 URL 提取 PR ID
- `repo.rs`：仓库信息相关函数
  - `extract_github_repo_from_url()` - 从 URL 提取 GitHub 仓库信息
- `content.rs`：内容生成相关函数
  - `generate_commit_title()` - 生成 commit 标题
  - `generate_pull_request_body()` - 生成 PR body
- `mod.rs`：公共函数
  - `get_current_branch_pr_id()` - 获取当前分支的 PR ID
  - `detect_repo_type()` - 检测仓库类型（向后兼容）

#### 6. PR Body 解析器 (`body_parser.rs`)

**职责**：从 PR body 中提取信息的纯函数，无用户交互

**主要函数**：
- `extract_info_from_source_pr()` - 从源 PR 提取所有信息（Jira ticket、描述、变更类型）
- `extract_jira_ticket_from_body()` - 从 PR body 提取 Jira ticket ID
- `extract_description_from_body()` - 从 PR body 提取描述
- `parse_change_types_from_body()` - 从 PR body 解析变更类型

**数据结构**：
- `SourcePrInfo` - 源 PR 信息（标题、URL、body）
- `ExtractedPrInfo` - 提取的信息（Jira ticket、描述、变更类型）

**使用场景**：
- `pr pick` 命令：从源 PR 提取信息用于创建新 PR
- 可被其他命令复用（如 sync、rebase 等）

#### 7. PR 表格显示 (`table.rs`)

**职责**：提供统一的 PR 列表表格行结构，用于表格格式显示

**核心组件**：

#### PullRequestRow 结构体

```rust
#[derive(Tabled)]
pub struct PullRequestRow {
    #[tabled(rename = "#")]
    pub number: String,
    #[tabled(rename = "State")]
    pub state: String,
    #[tabled(rename = "Branch")]
    pub branch: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Author")]
    pub author: String,
    #[tabled(rename = "URL")]
    pub url: String,
}
```

**特性**：
- 使用 `tabled` crate 的 `Tabled` trait
- 自动格式化表格列
- 支持自定义列名（通过 `#[tabled(rename = "...")]`）

**使用场景**：
- `pr list` 命令：使用 `TableBuilder` 和 `PullRequestRow` 显示 PR 列表
- 统一的表格格式，提供一致的用户体验

---

## 📁 Commands 层架构（命令封装）

PR 命令模块是 Workflow CLI 的核心功能之一，提供完整的 Pull Request 生命周期管理，支持 GitHub 和 Codeup 两种代码托管平台。

### 相关文件

#### CLI 入口层

PR 命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Pr` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow pr` 子命令分发到对应的命令处理函数
- **命令枚举**：`PRCommands` 定义了所有 PR 相关的子命令（create, merge, close, status, list, update, sync, rebase, pick, summarize, approve, comment, reword）

#### 命令封装层

```
src/commands/pr/
├── mod.rs          # PR 命令模块声明（12 行）
├── helpers.rs      # PR 命令辅助函数（230 行）
├── create.rs       # 创建 PR 命令（697 行）
├── sync.rs         # 同步分支命令（488 行，合并了原 integrate 功能）
├── merge.rs        # 合并 PR 命令（142 行）
├── close.rs        # 关闭 PR 命令（142 行）
├── status.rs       # PR 状态查询命令（50 行）
├── list.rs         # 列出 PR 命令（21 行）
├── update.rs       # 更新 PR 命令（59 行）
├── summarize.rs    # PR 总结命令（425 行）
├── rebase.rs       # Rebase 分支并更新 PR base 命令（507 行）
├── pick.rs         # Pick 提交并创建新 PR 命令（978 行）
├── approve.rs      # 批准 PR 命令
├── comment.rs      # 添加 PR 评论命令
└── reword.rs       # Reword PR 标题和描述命令（214 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/pr/`) 的功能
- 协调 Git、Jira 等模块的操作

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：

- **`lib/pr/`**：PR 平台抽象层（`create_provider()`、`helpers::*`、`llm::PullRequestLLM`）
  - `create_provider()` - 创建平台提供者
  - `helpers::resolve_pull_request_id()` - 解析 PR ID
  - `helpers::get_current_branch_pr_id()` - 获取当前分支 PR
  - `PullRequestLLM::generate()` - 生成 PR 标题

- **`lib/git/`**：Git 操作（`GitBranch`、`GitCommit`、`GitRepo`、`GitStash`）
  - `GitBranch` - 分支创建、合并、删除
  - `GitCommit` - 提交管理

- **`lib/jira/`**：Jira 集成（`Jira`、`JiraStatus`、`JiraWorkHistory`）
  - `Jira::get_ticket_info()` - 获取 ticket 信息
  - `Jira::move_ticket()` - 更新状态
  - `Jira::add_comment()` - 添加评论

- **`lib/base/util/`**：工具函数（`Browser`、`Clipboard`、`confirm()`）
  - `Browser::open()` - 打开浏览器
  - `confirm()` - 用户确认

- **`commands/check/`**：检查命令（`CheckCommand::run_all()`）
  - 运行环境检查（Git 状态、网络）

- **`lib/proxy/`**：代理管理（`ProxyManager`）
  - `ProxyManager::ensure_proxy_enabled()` - 确保代理已启用（已移除自动调用，需手动启用）

---

## 🔄 集成关系

### Lib 层和 Commands 层的协作

PR 模块采用清晰的分层架构，Lib 层和 Commands 层通过以下方式协作：

1. **平台抽象**：Commands 层通过 `create_provider()` 工厂函数获取平台提供者，无需关心具体平台实现
2. **业务逻辑分离**：Lib 层专注于平台 API 调用，Commands 层负责用户交互和业务流程协调
3. **模块协调**：Commands 层协调 Git、Jira、LLM 等模块，实现完整的 PR 生命周期管理

### 数据流向

#### 创建 PR 数据流

```
用户输入 (Jira ticket, title, description)
  ↓
Commands 层 (处理交互、参数解析)
  ↓
Lib 层 (create_provider() → GitHub 实现)
  ↓
GitHub API
```

#### 合并 PR 数据流

```
用户输入 (PR ID 或自动检测)
  ↓
Commands 层 (解析 PR ID、检查状态)
  ↓
Lib 层 (merge_pull_request())
  ↓
GitHub API
  ↓
Commands 层 (清理分支、更新 Jira)
```

### 调用流程

#### 整体架构流程

```
用户输入
  ↓
src/main.rs (CLI 入口，参数解析)
  ↓
commands/pr/*.rs (命令封装层，处理交互)
  ↓
lib/pr/* (通过 API 调用，具体实现见相关模块文档)
  ↓
GitHub API 或 Codeup API
```

#### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (clap 解析参数)
  ↓
match cli.subcommand {
  PRCommands::Create => create::PullRequestCreateCommand::create()
  PRCommands::Merge => merge::PullRequestMergeCommand::merge()
  PRCommands::Close => close::PullRequestCloseCommand::close()
  PRCommands::Status => status::PullRequestStatusCommand::show()
  PRCommands::List => list::PullRequestListCommand::list()
  PRCommands::Update => update::PullRequestUpdateCommand::update()
  PRCommands::Sync => sync::PullRequestSyncCommand::sync()
  PRCommands::Rebase => rebase::PullRequestRebaseCommand::rebase()
  PRCommands::Pick => pick::PullRequestPickCommand::pick()
  PRCommands::Summarize => summarize::SummarizeCommand::summarize()
  PRCommands::Approve => approve::PullRequestApproveCommand::approve()
  PRCommands::Comment => comment::PullRequestCommentCommand::comment()
  PRCommands::Reword => reword::PullRequestRewordCommand::reword()
}
```

---

## 📋 Commands 层命令详情

### 1. 创建 PR 命令 (`create.rs`)

创建 PR 命令是 PR 模块中最复杂的命令，提供完整的 PR 创建流程：

1. **前置检查**：运行所有检查（git status、network 等），支持 dry-run 模式。
2. **Jira 集成**：支持可选的 Jira ticket 输入，自动验证，自动配置状态，创建后自动更新 ticket。
3. **PR 标题生成**：优先使用输入标题，或从 Jira 获取，或提示输入。
4. **分支名和 commit 标题生成**：使用 LLM 生成，失败则回退到默认方法。
5. **分支管理**：智能处理各种分支状态（未提交修改、未推送分支等）。
6. **PR body 生成**：支持选择变更类型，自动生成格式化的 PR body。
7. **Jira 更新**：分配任务，更新状态，添加评论，写入历史。

### 2. 同步分支命令 (`sync.rs`)

同步分支命令用于将指定分支同步到当前分支（本地 Git 操作），支持 merge、rebase 或 squash 三种方式：

1. **工作区管理**：自动检查并提示 stash。
2. **源分支验证**：自动检测分支存在性（本地或远程）。
3. **同步策略**：支持 merge（默认）、rebase、squash 和 fast-forward only。
4. **PR 管理**：自动更新当前分支的 PR（如果存在），并根据源分支是否有 PR 进行交互式处理。
5. **分支清理**：交互式确认是否删除源分支（如果有 PR，会询问是否关闭 PR 并删除分支）。

### 3. Rebase 分支命令 (`rebase.rs`)

Rebase 分支命令用于将当前分支 rebase 到目标分支，并可选地更新 PR 的 base 分支：

1. **预检查**：运行环境检查（Git 状态、网络等），失败时提示用户是否继续。
2. **工作区管理**：自动检测未提交更改，提示用户 stash。
3. **目标分支验证**：检查目标分支是否存在（本地或远程）。
4. **Rebase 操作**：执行 `git rebase` 操作，重写当前分支的提交历史。
5. **冲突处理**：如果发生冲突，暂停操作并提示用户手动解决。
6. **PR 管理**：自动检测当前分支的 PR，如果找到则提示用户确认是否更新 PR base。
7. **安全推送**：默认推送到远程，使用 `--force-with-lease` 确保安全。

### 4. Pick 提交命令 (`pick.rs`)

Pick 提交命令用于跨分支移植代码，从源分支 cherry-pick 提交到目标分支并创建新 PR：

1. **跨分支操作**：支持从任意分支 cherry-pick 到任意分支，类似于 backport/forwardport，但支持任意方向。
2. **智能提交检测**：自动检测源分支相对于目标分支的新提交。
3. **源 PR 信息提取**：如果源分支有 PR，自动提取标题、描述、Jira ticket、变更类型等信息。
4. **Cherry-pick 处理**：使用 `--no-commit` 模式，允许在创建分支前统一提交。
5. **冲突处理**：检测冲突并提供详细的解决指引，支持放弃或继续。
6. **交互式 PR 创建**：复用 `create` 命令的交互式流程，支持 LLM 生成分支名和标题。
7. **状态恢复**：无论成功或失败，都会恢复原分支和 stash，确保工作区状态一致。

### 5. 合并 PR 命令 (`merge.rs`)

合并 PR 命令通过 API 合并 PR：

1. **PR ID 解析**：支持参数提供或自动检测。
2. **合并操作**：通过平台 API 执行合并，处理竞态条件。
3. **合并后清理**：切换到默认分支，删除当前分支（本地和远程）。
4. **Jira 更新**：更新 ticket 状态为合并状态，删除工作历史。

### 6. 关闭 PR 命令 (`close.rs`)

关闭 PR 命令用于关闭 PR 并清理相关分支：

1. **安全检查**：不允许在默认分支上操作。
2. **关闭操作**：通过平台 API 关闭 PR。
3. **分支清理**：删除远程分支，切换到默认分支，删除本地分支。

### 7. PR 状态查询命令 (`status.rs`)

PR 状态查询命令用于显示 PR 的详细信息（状态、作者、评论等）。支持 PR ID 或分支名查询。

### 8. 列出 PR 命令 (`list.rs`)

列出仓库中的所有 PR。支持按状态过滤（--state）和限制数量（--limit）。

### 9. 更新 PR 命令 (`update.rs`)

快速更新 PR 代码。自动使用 PR 标题作为提交消息，暂存所有更改，提交并推送。

### 10. PR 总结命令 (`summarize.rs`)

PR 总结命令使用 LLM 生成 PR 的详细总结文档：

1. **PR 信息获取**：自动获取 PR 标题和完整的 diff 内容
2. **LLM 总结生成**：使用配置的 LLM 提供商生成 PR 总结
3. **代码变更提取**：解析标准的 unified diff 格式
4. **智能格式化**：根据文件扩展名自动识别代码块语言
5. **文件保存**：保存到 `~/Documents/Workflow/summarize/` 目录

### 11. 批准 PR 命令 (`approve.rs`)

批准 PR 命令用于批准指定的 Pull Request：

1. **PR ID 解析**：支持通过参数指定 PR ID，或自动检测当前分支对应的 PR
2. **错误处理**：如果尝试批准自己的 PR，会返回明确的错误信息

### 12. 添加 PR 评论命令 (`comment.rs`)

添加 PR 评论命令用于向指定的 Pull Request 添加评论：

1. **评论内容**：支持多个单词作为评论内容（使用 `trailing_var_arg` 参数）
2. **PR ID 解析**：支持通过参数指定 PR ID，或自动检测当前分支对应的 PR

### 13. Reword PR 命令 (`reword.rs`)

Reword PR 命令用于基于 PR diff 自动生成并更新 PR 标题和描述：

1. **PR ID 解析**：支持通过参数指定 PR ID，或自动检测当前分支对应的 PR
2. **PR 信息获取**：自动获取当前 PR 的标题和描述，获取完整的 PR diff 内容
3. **LLM 生成**：使用 `RewordGenerator::reword_from_diff()` 基于 PR diff 生成新的标题和描述
4. **更新选项**：支持 `--title`、`--description` 和 `--dry-run` 参数
5. **用户确认**：显示当前内容和新内容的对比，用户确认后执行更新

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式
每个命令都是一个独立的结构体，实现统一的方法接口。

#### 2. 工厂模式
使用工厂函数 `create_provider()` 创建平台提供者，命令层无需关心平台差异。

#### 3. 策略模式
不同的合并策略（FastForwardOnly, Squash, Merge, Rebase）在 sync 命令中实现。

#### 4. 辅助函数模式
将通用逻辑（如 `cleanup_branch`）提取到 `helpers.rs` 中共享。

### 错误处理

#### 分层错误处理
1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **平台层**：API 调用错误、网络错误

#### 容错机制
- **PR 状态检查**：处理竞态条件，避免重复操作。
- **分支操作**：尝试多种策略（如强制删除），失败仅警告不中断。
- **Jira 集成**：操作失败仅记录警告，不中断 PR 流程。
- **LLM 生成**：生成失败自动回退到默认方法。

---

## 📝 扩展性

### 添加新平台

1. 在 `lib/pr/` 下创建新的平台目录（如 `gitlab/`）
2. 创建以下文件：
   - `mod.rs` - 模块导出
   - `platform.rs` - 实现 `PlatformProvider` trait
   - `requests.rs` - API 请求结构体
   - `responses.rs` - API 响应结构体
   - `errors.rs` - 错误处理
3. 在 `lib/pr/platform.rs` 的 `create_provider()` 函数中添加新平台的分支
4. 在 `lib/git/repo.rs` 中添加仓库类型检测逻辑
5. 在 `lib/pr/mod.rs` 中导出新平台

### 添加新命令

1. 在 `commands/pr/` 下创建新的命令文件
2. 实现命令结构体和处理方法
3. 在 `commands/pr/mod.rs` 中导出
4. 在 `src/main.rs` 中的 `PRCommands` 枚举中添加新命令
5. 在 `src/main.rs` 的命令分发逻辑中添加处理代码

---

## 📋 使用示例

### Create 命令
```bash
workflow pr create                           # 交互式
workflow pr create PROJ-123                  # 指定 ticket
workflow pr create --dry-run                 # 干运行
```

### Merge 命令
```bash
workflow pr merge                            # 合并当前 PR
workflow pr merge 123                        # 合并指定 PR
```

### Sync 命令
```bash
workflow pr sync feature-branch              # 同步分支（merge，交互式确认删除）
workflow pr sync feature-branch --squash      # Squash 合并
workflow pr sync master --rebase              # 使用 rebase 同步基础分支
workflow pr sync master --no-push             # 只同步到本地，不推送
```

### Pick 命令
```bash
# 将 develop 的修改应用到 master
workflow pr pick develop master

# 预览模式
workflow pr pick develop master --dry-run
```

### Rebase 命令
```bash
workflow pr rebase master                     # Rebase 当前分支到 master（默认推送）
workflow pr rebase master --no-push           # 只 rebase 到本地，不推送
workflow pr rebase master --dry-run           # 预览模式
```

### Summarize 命令
```bash
workflow pr summarize                    # 总结当前分支的 PR
workflow pr summarize 123                # 总结指定 PR ID
workflow pr summarize --language zh      # 使用中文生成总结
```

### Approve 命令
```bash
workflow pr approve                            # 批准当前分支的 PR
workflow pr approve 123                         # 批准指定 PR ID
```

### Comment 命令
```bash
workflow pr comment "Great work!"             # 向当前分支的 PR 添加评论
workflow pr comment 123 "Looks good to me"   # 向指定 PR ID 添加评论
```

### Reword 命令
```bash
# 基本用法：更新当前分支 PR 的标题和描述
workflow pr reword

# 仅更新标题
workflow pr reword --title

# 仅更新描述
workflow pr reword --description

# 预览模式（不实际更新）
workflow pr reword --dry-run

# 指定 PR ID
workflow pr reword 123
```

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Jira 模块架构文档](./jira.md) - Jira 集成详情
- [Git 模块架构文档](./git.md) - Git 操作详情
- [LLM 模块架构文档](./llm.md) - AI 功能详情

---

## ✅ 总结

PR 模块采用清晰的分层架构设计：

1. **平台抽象层**：`PlatformProvider` trait 定义统一的平台接口
2. **工厂函数层**：`create_provider()` 实现多态分发，自动检测仓库类型
3. **平台实现层**：GitHub 实现 trait，模块化组织
4. **LLM 功能层**：提供 PR 内容的 AI 生成功能（创建、重写、总结）
5. **辅助函数层**：提供通用的 PR 相关辅助函数（已按功能拆分）
6. **命令封装层**：提供 CLI 命令封装，处理用户交互，协调 Git、Jira 等模块

**设计优势**：
- ✅ **多态支持**：通过 trait 对象实现真正的多态
- ✅ **代码复用**：消除调用层的重复代码
- ✅ **易于扩展**：添加新平台只需实现 trait
- ✅ **模块化**：按平台拆分，职责清晰
- ✅ **类型安全**：使用 trait 和类型系统保证类型安全
- ✅ **平台无关**：调用者无需关心具体平台实现
- ✅ **健壮性**：完善的错误处理和竞态条件处理
- ✅ **易用性**：提供智能默认值和交互式操作
- ✅ **可维护性**：清晰的模块划分和代码复用

通过平台抽象和工厂模式，实现了代码复用、易于维护和扩展的目标。命令层（`commands/pr/`）使用本模块提供的接口，实现了完整的 PR 生命周期管理功能。

---

**最后更新**: 2025-12-16

