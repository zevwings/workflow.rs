# PR 命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 PR 命令模块架构，包括：
- PR 创建、合并、关闭、查询等操作
- 分支集成功能
- 与 Jira 的集成（状态更新、工作历史管理）
- 与 Git 操作的集成（分支管理、提交等）

PR 命令模块是 Workflow CLI 的核心功能之一，提供完整的 Pull Request 生命周期管理，支持 GitHub 和 Codeup 两种代码托管平台。

**模块统计：**
- 命令数量：9 个（create, merge, close, status, list, update, integrate, summarize）
- 总代码行数：约 1900+ 行
- 支持平台：GitHub、Codeup
- 主要依赖：`lib/pr/`（平台抽象层）、`lib/git/`、`lib/jira/`、`lib/base/llm/`

---

## 📁 相关文件

### CLI 入口层

PR 命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Pr` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow pr` 子命令分发到对应的命令处理函数
- **命令枚举**：`PRCommands` 定义了所有 PR 相关的子命令（create, merge, close, status, list, update, integrate, summarize）

### 命令封装层

```
src/commands/pr/
├── mod.rs          # PR 命令模块声明（9 行）
├── helpers.rs      # PR 命令辅助函数（176 行）
├── create.rs       # 创建 PR 命令（717 行）
├── integrate.rs    # 集成分支命令（343 行）
├── merge.rs        # 合并 PR 命令（142 行）
├── close.rs        # 关闭 PR 命令（142 行）
├── status.rs       # PR 状态查询命令（50 行）
├── list.rs         # 列出 PR 命令（21 行）
├── update.rs       # 更新 PR 命令（59 行）
└── summarize.rs    # PR 总结命令（425 行）
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
  - `ProxyManager::ensure_proxy_enabled()` - 自动启用代理（如果系统代理已启用）

---

## 🔄 调用流程

### 整体架构流程

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

### 命令分发流程

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
  PRCommands::Integrate => integrate::PullRequestIntegrateCommand::integrate()
  PRCommands::Summarize => summarize::SummarizeCommand::summarize()
}
```

### 数据流

#### Create 命令数据流

```
用户输入 (Jira ticket, title, description)
  ↓
Jira 集成 (获取 ticket 信息、配置状态)
  ↓
LLM 生成 (分支名、PR 标题、描述)
  ↓
Git 操作 (创建分支、提交、推送)
  ↓
PR API (创建 PR)
  ↓
Jira 更新 (更新状态、添加评论、写入历史)
  ↓
浏览器 (打开 PR URL)
```

#### Merge 命令数据流

```
用户输入 (PR ID 或自动检测)
  ↓
PR API (检查状态、合并 PR)
  ↓
Git 操作 (切换到默认分支、删除当前分支)
  ↓
Jira 更新 (更新 ticket 状态、删除工作历史)
```

---

## 1. 创建 PR 命令 (`create.rs`)

### 相关文件

```
src/commands/pr/create.rs (717 行)
```

### 调用流程

```
src/main.rs::PRCommands::Create
  ↓
commands/pr/create.rs::PullRequestCreateCommand::create()
  ↓
  0. 自动启用代理（ProxyManager::ensure_proxy_enabled()）
  1. 运行检查（check::CheckCommand::run_all()）
  2. 获取或输入 Jira ticket（resolve_jira_ticket()）
  3. 配置 Jira 状态（ensure_jira_status()）
  4. 获取或生成 PR 标题（resolve_title()）
  5. 生成 commit_title、分支名和描述（generate_commit_title_and_branch_name()）
     ├─ PullRequestLLM::generate() (尝试使用 LLM 生成)
     └─ 回退到默认方法（generate_branch_name()）
  6. 获取描述（resolve_description()）
  7. 选择变更类型（select_change_types()）
  8. 生成 PR body（generate_pull_request_body()）
  9. 创建或更新分支（create_or_update_branch()）
     ├─ 检查是否有未提交的修改
     ├─ 判断当前分支状态
     └─ 根据情况执行：create_branch_from_default / commit_and_push / create_with_stash
  10. 创建或获取 PR（create_or_get_pull_request()）
      ├─ get_current_branch_pr_id() (检查是否已有 PR)
      └─ provider.create_pull_request() (创建新 PR)
  11. 更新 Jira ticket（update_jira_ticket()）
      ├─ Jira::assign_ticket()
      ├─ Jira::move_ticket()
      ├─ Jira::add_comment()
      └─ JiraWorkHistory::write_work_history()
  12. 复制 PR URL 并打开浏览器（copy_and_open_pull_request()）
```

### 功能说明

创建 PR 命令是 PR 模块中最复杂的命令，提供完整的 PR 创建流程：

0. **代理自动启用**：如果系统代理（VPN）已启用，自动在当前进程中设置代理环境变量，确保网络请求通过代理。
1. **前置检查**：运行所有检查（git status、network 等），支持 dry-run 模式。
2. **Jira 集成**：支持可选的 Jira ticket 输入，自动验证，自动配置状态，创建后自动更新 ticket。
3. **PR 标题生成**：优先使用输入标题，或从 Jira 获取，或提示输入。
4. **分支名和 commit 标题生成**：使用 LLM 生成，失败则回退到默认方法。
5. **分支管理**：智能处理各种分支状态（未提交修改、未推送分支等）。
6. **PR body 生成**：支持选择变更类型，自动生成格式化的 PR body。
7. **Jira 更新**：分配任务，更新状态，添加评论，写入历史。

### 关键步骤说明

1. **分支创建策略**：
   - 在默认分支上：如果有未提交修改，直接创建新分支。
   - 在非默认分支上：询问用户是否在当前分支创建 PR，或创建新分支（使用 stash）。

2. **LLM 集成**：
   - 使用 `PullRequestLLM::generate()` 生成分支名和 PR 标题。
   - 如果生成失败，回退到 `generate_branch_name()`。

---

## 2. 集成分支命令 (`integrate.rs`)

### 相关文件

```
src/commands/pr/integrate.rs (343 行)
```

### 调用流程

```
src/main.rs::PRCommands::Integrate
  ↓
commands/pr/integrate.rs::PullRequestIntegrateCommand::integrate()
  ↓
  1. 运行检查（check::CheckCommand::run_all()）
  2. 获取当前分支，检查工作区状态并 stash
  3. 验证并准备源分支（prepare_source_branch()）
  4. 确定合并策略（--ff-only, --squash, default）
  5. 执行合并（GitBranch::merge_branch()）
  6. 根据分支类型处理合并后的操作（更新 PR 或恢复 stash）
  7. 检查并关闭被合并分支的 PR
  8. 删除被合并的源分支
```

### 功能说明

集成分支命令用于将指定分支合并到当前分支（本地 Git 操作）：
1. **工作区管理**：自动检查并提示 stash。
2. **源分支验证**：不允许合并默认分支，自动检测分支存在性。
3. **合并策略**：支持 fast-forward, squash 和普通合并。
4. **PR 更新**：合并后自动推送更新当前 PR，关闭源分支 PR。
5. **分支清理**：合并成功后自动删除源分支。

---

## 3. 合并 PR 命令 (`merge.rs`)

### 相关文件

```
src/commands/pr/merge.rs (142 行)
```

### 调用流程

```
src/main.rs::PRCommands::Merge
  ↓
commands/pr/merge.rs::PullRequestMergeCommand::merge()
  ↓
  0. 自动启用代理（ProxyManager::ensure_proxy_enabled()）
  1. 运行检查，获取 PR ID
  2. 合并 PR（merge_pull_request()）
     └─ provider.merge_pull_request()
  3. 合并后清理（cleanup_after_merge()）
     └─ helpers::cleanup_branch() (切换到默认分支并删除当前分支)
  4. 更新 Jira 状态（update_jira_status()）
```

### 功能说明

合并 PR 命令通过 API 合并 PR：
0. **代理自动启用**：如果系统代理（VPN）已启用，自动在当前进程中设置代理环境变量。
1. **PR ID 解析**：支持参数提供或自动检测。
2. **合并操作**：通过平台 API 执行合并，处理竞态条件。
3. **合并后清理**：切换到默认分支，删除当前分支（本地和远程）。
4. **Jira 更新**：更新 ticket 状态为合并状态，删除工作历史。

---

## 4. 关闭 PR 命令 (`close.rs`)

### 相关文件

```
src/commands/pr/close.rs (142 行)
```

### 调用流程

```
src/main.rs::PRCommands::Close
  ↓
commands/pr/close.rs::PullRequestCloseCommand::close()
  ↓
  1. 获取 PR ID，检查是否为默认分支
  2. 检查 PR 状态（check_if_already_closed()）
  3. 关闭 PR（close_pull_request()）
  4. 删除远程分支（delete_remote_branch()）
  5. 清理本地（cleanup_after_close()）
```

### 功能说明

关闭 PR 命令用于关闭 PR 并清理相关分支：
1. **安全检查**：不允许在默认分支上操作。
2. **关闭操作**：通过平台 API 关闭 PR。
3. **分支清理**：删除远程分支，切换到默认分支，删除本地分支。

---

## 5. PR 状态查询命令 (`status.rs`)

### 相关文件

```
src/commands/pr/status.rs (50 行)
```

### 功能说明
PR 状态查询命令用于显示 PR 的详细信息（状态、作者、评论等）。支持 PR ID 或分支名查询。

---

## 6. 列出 PR 命令 (`list.rs`)

### 相关文件

```
src/commands/pr/list.rs (21 行)
```

### 功能说明
列出仓库中的所有 PR。支持按状态过滤（--state）和限制数量（--limit）。

---

## 7. 更新 PR 命令 (`update.rs`)

### 相关文件

```
src/commands/pr/update.rs (59 行)
```

### 调用流程

```
src/main.rs::PRCommands::Update
  ↓
commands/pr/update.rs::PullRequestUpdateCommand::update()
  ↓
  0. 自动启用代理（ProxyManager::ensure_proxy_enabled()）
  1. 获取当前分支的 PR 标题
  2. 提交更改（GitCommit::commit()）
  3. 推送到远程（GitBranch::push()）
```

### 功能说明
快速更新 PR 代码。自动使用 PR 标题作为提交消息，暂存所有更改，提交并推送。

---

## 8. PR 总结命令 (`summarize.rs`)

### 相关文件

```
src/commands/pr/summarize.rs (425 行)
```

### 调用流程

```
src/main.rs::PRCommands::Summarize
  ↓
commands/pr/summarize.rs::SummarizeCommand::summarize()
  ↓
  1. 创建平台提供者（create_provider()）
  2. 获取 PR ID（参数或自动检测当前分支）
  3. 获取 PR 标题（provider.get_pull_request_title()）
  4. 获取 PR diff（provider.get_pull_request_diff()）
  5. 使用 LLM 生成总结（PullRequestLLM::summarize_pr()）
  6. 解析 diff 提取文件修改（parse_diff_to_file_changes()）
  7. 格式化代码修改为 markdown（format_file_changes_as_markdown()）
  8. 合并总结和代码修改部分
  9. 保存到文件（~/Documents/Workflow/SUMMARIZE_FOR_PR_{PR_ID}/{filename}.md）
```

### 功能说明

PR 总结命令使用 LLM 生成 PR 的详细总结文档：

1. **PR 信息获取**：
   - 自动获取 PR 标题和完整的 diff 内容
   - 支持通过参数指定 PR ID，或自动检测当前分支对应的 PR
   - 如果当前分支没有对应的 PR，会提示用户手动指定 PR ID

2. **LLM 总结生成**：
   - 使用配置的 LLM 提供商（OpenAI、DeepSeek、Proxy）生成 PR 总结
   - 支持多语言（en, zh, zh-CN, zh-TW 等）
   - 语言可通过 `--language` 参数指定，或从配置文件读取，默认使用 "en"
   - LLM 会自动生成总结内容和文件名

3. **代码变更提取**：
   - 解析标准的 unified diff 格式
   - 提取每个文件的修改内容（包括 hunk 信息）
   - 自动跳过二进制文件
   - 处理空文件和新增/删除的文件

4. **智能格式化**：
   - 根据文件扩展名自动识别代码块语言（支持 rust, javascript, typescript, python, go, java, cpp, c, markdown, json, yaml, toml, bash, sql, html, css, xml 等）
   - 从文件路径推断文件用途（Purpose），如 "Command implementation"、"LLM service integration" 等
   - 生成格式化的 markdown 文档，包含：
     - PR 标题作为一级标题
     - LLM 生成的总结内容
     - Code Changes 部分，包含所有修改文件的详细内容

5. **文件保存**：
   - 保存路径格式：`{document_base_dir}/summarize/{repo-name}-{PR_ID}-{filename}.md`
   - 默认格式：`~/Documents/Workflow/summarize/{repo-name}-{PR_ID}-{filename}.md`
   - 示例：`~/Documents/Workflow/summarize/workflow.rs-123-add-pr-summarize-feature.md`
   - 文件名由 LLM 根据 PR 内容自动生成
   - 如果目录不存在，会自动创建
   - 基础目录可通过配置文件 `log.download_base_dir` 自定义，默认为 `~/Documents/Workflow`
   - 仓库名称从 Git remote URL 自动提取（owner/repo 格式，提取 repo 部分）

### 关键步骤说明

1. **Diff 解析**（`parse_diff_to_file_changes()`）：
   - 支持标准的 unified diff 格式（`diff --git a/path/to/file b/path/to/file`）
   - 自动跳过二进制文件（检测 "Binary files" 或 "GIT binary patch" 标记）
   - 提取每个文件的修改内容（包括 hunk 信息，即 `@@` 行之后的内容）
   - 处理空文件和新增/删除的文件（没有 hunk 的情况）
   - 从 diff 行中提取文件路径（使用 "b/" 后面的路径，即新文件路径）

2. **语言检测**（`detect_language_from_path()`）：
   - 支持多种编程语言的语法高亮
   - 根据文件扩展名自动识别语言类型
   - 支持的语言：rust, javascript, typescript, python, go, java, cpp, c, markdown, json, yaml, toml, bash, sql, html, css, xml 等
   - 如果无法识别，使用空字符串（markdown 渲染器会尝试自动检测）

3. **文件用途推断**（`infer_file_purpose()`）：
   - 根据文件路径关键词推断用途
   - 支持的关键词匹配：
     - `command`/`cmd` → "Command implementation" 或 "Implements the PR summarization command functionality"
     - `prompt` → "System prompt definition for LLM interactions"
     - `platform` → "Platform-specific API implementation"
     - `llm` → "LLM service integration and response parsing"
     - `mod.rs` → "Module declaration and exports"
     - `main.rs` → "Main entry point and command routing"
     - `test` → "Test implementation"
     - `config`/`settings` → "Configuration and settings management"
     - `helper`/`util` → "Utility functions and helpers"
   - 如果无法推断，返回空字符串（不显示 Purpose 部分）

4. **文档结构**：
   - 文档开头：PR 标题作为一级标题（`# {PR_TITLE}`）
   - 总结部分：LLM 生成的总结内容（如果 LLM 已经包含标题，则直接使用）
   - Code Changes 部分：
     - 二级标题：`## Code Changes`
     - 介绍文字：说明每个文件的详细修改内容
     - 每个文件：
       - 三级标题：`### {file_path}`
       - Purpose（可选）：`**Purpose**: {purpose}`
       - 代码块：```{language}\n{content}\n```

5. **输出路径构建**（`build_output_path()`）：
   - 从配置文件读取基础目录（`log.download_base_dir`），默认为 `~/Documents/Workflow`
   - 路径格式：`{base_dir}/summarize/{repo-name}-{PR_ID}-{filename}.md`
   - 仓库名称从 Git remote URL 自动提取（owner/repo 格式，提取 repo 部分）
   - 仓库名称会自动清理，移除文件名中不允许的字符（`/`、`\`、`:`、`*`、`?`、`"`、`<`、`>`、`|` 等）
   - 文件命名：`{repo-name}-{PR_ID}-{filename}.md`（文件名由 LLM 生成）
   - 自动创建目录（如果不存在）

### 使用示例

```bash
workflow pr summarize                    # 总结当前分支的 PR
workflow pr summarize 123                # 总结指定 PR ID
workflow pr summarize --language zh      # 使用中文生成总结
```

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式
每个命令都是一个独立的结构体，实现统一的方法接口。

#### 2. 工厂模式
使用工厂函数 `create_provider()` 创建平台提供者，命令层无需关心平台差异。

#### 3. 策略模式
不同的合并策略（FastForwardOnly, Squash, Merge）在 integrate 命令中实现。

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

### Close 命令
```bash
workflow pr close                            # 关闭当前 PR
```

### Integrate 命令
```bash
workflow pr integrate feature-branch         # 集成分支
workflow pr integrate feature-branch --squash # Squash 合并
```

---

## 📝 扩展性（可选）

### 添加新命令
1. 在 `commands/pr/` 下创建新的命令文件
2. 实现命令结构体和处理方法
3. 在 `commands/pr/mod.rs` 中导出
4. 在 `src/main.rs` 中的 `PRCommands` 枚举中添加新命令
5. 在 `src/main.rs` 的命令分发逻辑中添加处理代码

### 添加新的平台支持
1. 在 `lib/pr/` 下创建新的平台模块
2. 实现 `PlatformProvider` trait
3. 在 `lib/pr/platform.rs` 的 `create_provider()` 中添加平台检测逻辑

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [PR 模块架构文档](../lib/PR_ARCHITECTURE.md) - PR 平台抽象层
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md)
- [Jira 模块架构文档](../lib/JIRA_ARCHITECTURE.md)

---

## ✅ 总结

PR 命令模块采用清晰的分层架构设计：

1. **特性1**：职责分离 - CLI 层负责分发，命令层负责交互，Lib 层负责业务。
2. **特性2**：高度集成 - 深度集成 Jira、Git 和 LLM 功能，提供自动化的工作流体验。

**设计优势**：
- ✅ 健壮性：完善的错误处理和竞态条件处理
- ✅ 易用性：提供智能默认值和交互式操作
- ✅ 可维护性：清晰的模块划分和代码复用
