# PR 命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 PR 命令模块架构，包括：
- PR 创建、合并、关闭、查询等操作
- 分支集成功能
- 与 Jira 的集成（状态更新、工作历史管理）
- 与 Git 操作的集成（分支管理、提交等）

PR 命令模块是 Workflow CLI 的核心功能之一，提供完整的 Pull Request 生命周期管理，支持 GitHub 和 Codeup 两种代码托管平台。

**模块统计：**
- 命令数量：8 个（create, merge, close, status, list, update, integrate）
- 总代码行数：约 1500+ 行
- 支持平台：GitHub、Codeup
- 主要依赖：`lib/pr/`（平台抽象层）、`lib/git/`、`lib/jira/`

---

## 📁 相关文件

### CLI 入口层

```
src/bin/pr.rs (170 行)
```
- **职责**：独立的 PR 命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将请求分发到对应的命令处理函数
- **命令枚举**：`PRCommands` 定义了所有 PR 相关的子命令

### 命令封装层 (`commands/pr/`)

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
└── update.rs       # 更新 PR 命令（59 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/pr/`) 的功能
- 协调 Git、Jira 等模块的操作

### 依赖模块（简要说明）

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/pr/`**：PR 平台抽象层（`create_provider()`、`helpers::*`、`llm::PullRequestLLM`）
- **`lib/git/`**：Git 操作（`GitBranch`、`GitCommit`、`GitRepo`、`GitStash`）
- **`lib/jira/`**：Jira 集成（`Jira`、`JiraStatus`、`JiraWorkHistory`）
- **`lib/base/util/`**：工具函数（`Browser`、`Clipboard`、`confirm()`）
- **`lib/commands/config/check/`**：检查命令（`CheckCommand::run_all()`）

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
bin/pr.rs (CLI 入口，参数解析)
  ↓
commands/pr/*.rs (命令封装层，处理交互)
  ↓
lib/pr/* (通过 API 调用，具体实现见相关模块文档)
  ↓
GitHub API 或 Codeup API
```

### 命令分发流程

```
bin/pr.rs::main()
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
}
```

---

## 1. 创建 PR 命令 (`create`)

### 相关文件

```
src/commands/pr/create.rs (717 行)
```

### 调用流程

```
bin/pr.rs::PRCommands::Create
  ↓
commands/pr/create.rs::PullRequestCreateCommand::create()
  ↓
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
     └─ 根据情况执行：
        - create_branch_from_default() (在默认分支上创建新分支)
        - commit_and_push_current_branch() (在当前分支上提交并推送)
        - create_new_branch_with_stash() (使用 stash 创建新分支)
        - handle_existing_remote_branch() (处理已推送的分支)
        - handle_unpushed_branch() (处理未推送的分支)
  10. 创建或获取 PR（create_or_get_pull_request()）
      ├─ get_current_branch_pr_id() (检查是否已有 PR)
      └─ provider.create_pull_request() (创建新 PR)
  11. 更新 Jira ticket（update_jira_ticket()）
      ├─ Jira::assign_ticket() (分配任务)
      ├─ Jira::move_ticket() (更新状态)
      ├─ Jira::add_comment() (添加评论)
      └─ JiraWorkHistory::write_work_history() (写入历史记录)
  12. 复制 PR URL 并打开浏览器（copy_and_open_pull_request()）
```

### 功能说明

创建 PR 命令是 PR 模块中最复杂的命令，提供完整的 PR 创建流程：

1. **前置检查**：
   - 运行所有检查（git status、network 等）
   - 支持 dry-run 模式（不实际创建 PR）

2. **Jira 集成**：
   - 支持可选的 Jira ticket 输入
   - 自动验证 ticket 格式
   - 自动配置和读取 Jira 状态
   - 创建 PR 后自动更新 Jira ticket

3. **PR 标题生成**：
   - 优先使用用户输入的标题
   - 如果有 Jira ticket，尝试从 Jira 获取标题
   - 否则提示用户输入

4. **分支名和 commit 标题生成**：
   - 尝试使用 LLM 根据标题生成分支名和 PR 标题
   - 如果 LLM 生成失败，回退到默认方法
   - 自动应用分支名前缀（Jira ticket、github_branch_prefix）

5. **分支管理**：
   - 智能处理各种分支状态：
     - 在默认分支上有未提交修改 → 创建新分支
     - 在非默认分支上有未提交修改 → 询问用户是否在当前分支创建 PR
     - 在非默认分支上无未提交修改 → 检查分支是否已推送，处理相应情况

6. **PR body 生成**：
   - 支持选择变更类型（feat, fix, docs, style, refactor, test, chore）
   - 自动生成格式化的 PR body

7. **Jira 更新**：
   - 分配任务给自己
   - 更新 ticket 状态
   - 添加 PR URL 评论
   - 写入工作历史记录

### 关键步骤说明

1. **分支创建策略**：
   - 在默认分支上：如果有未提交修改，直接创建新分支（Git 会自动带过去）
   - 在非默认分支上：询问用户是否在当前分支创建 PR，或创建新分支（使用 stash）

2. **LLM 集成**：
   - 使用 `PullRequestLLM::generate()` 生成分支名和 PR 标题
   - 如果生成失败，回退到 `generate_branch_name()` 和 `generate_commit_title()`

3. **PR 创建**：
   - 先检查分支是否已有 PR（`get_current_branch_pr_id()`）
   - 如果有，直接返回 PR URL
   - 如果没有，创建新 PR

---

## 2. 集成分支命令 (`integrate`)

### 相关文件

```
src/commands/pr/integrate.rs (343 行)
```

### 调用流程

```
bin/pr.rs::PRCommands::Integrate
  ↓
commands/pr/integrate.rs::PullRequestIntegrateCommand::integrate()
  ↓
  1. 运行检查（check::CheckCommand::run_all()，可选）
  2. 获取当前分支（GitBranch::current_branch()）
  3. 检查工作区状态并 stash（check_working_directory()）
  4. 验证并准备源分支（prepare_source_branch()）
     ├─ 检查是否为默认分支（不允许）
     ├─ 检查分支是否存在（本地或远程）
     └─ 如果只在远程，先 fetch
  5. 确定合并策略（determine_merge_strategy()）
     ├─ FastForwardOnly (--ff-only)
     ├─ Squash (--squash)
     └─ Merge (默认)
  6. 执行合并（GitBranch::merge_branch()）
  7. 根据分支类型处理合并后的操作：
     ├─ 远程分支：检查并更新当前分支的 PR（check_and_update_current_branch_pr()）
     └─ 本地分支：恢复 stash，可选推送（--no-push）
  8. 检查并关闭被合并分支的 PR（check_and_close_source_branch_pr()）
  9. 删除被合并的源分支（delete_merged_branch()）
```

### 功能说明

集成分支命令用于将指定分支合并到当前分支，这是一个本地 Git 操作，与 `merge` 命令（通过 API 合并 PR）不同：

1. **工作区管理**：
   - 自动检查是否有未提交的更改
   - 如果有，提示用户 stash 或取消操作

2. **源分支验证**：
   - 不允许合并默认分支
   - 自动检测分支是否存在（本地或远程）
   - 如果只在远程，先 fetch 确保有最新引用

3. **合并策略**：
   - `--ff-only`：只允许 fast-forward 合并
   - `--squash`：使用 squash 合并（压缩为一个提交）
   - 默认：普通合并

4. **PR 更新**：
   - 如果当前分支已有 PR，合并后自动推送更新 PR
   - 如果被合并分支有 PR，自动关闭该 PR

5. **分支清理**：
   - 合并成功后，自动删除被合并的源分支（本地和远程）

### 关键步骤说明

1. **合并冲突处理**：
   - 如果合并失败，检查是否有冲突
   - 如果有冲突，提供详细的解决指导
   - 自动恢复 stash（如果有）

2. **远程分支处理**：
   - 如果源分支只在远程，使用 `origin/branch-name` 作为合并引用
   - 合并后自动推送更新当前分支的 PR

3. **分支删除**：
   - 先尝试普通删除，失败则尝试强制删除
   - 删除远程分支时，如果失败只记录警告，不中断流程

---

## 3. 合并 PR 命令 (`merge`)

### 相关文件

```
src/commands/pr/merge.rs (142 行)
```

### 调用流程

```
bin/pr.rs::PRCommands::Merge
  ↓
commands/pr/merge.rs::PullRequestMergeCommand::merge()
  ↓
  1. 运行检查（check::CheckCommand::run_all()）
  2. 获取 PR ID（resolve_pull_request_id()）
  3. 获取当前分支名和默认分支
  4. 合并 PR（merge_pull_request()）
     ├─ provider.get_pull_request_status() (检查 PR 状态)
     ├─ 如果已合并，跳过合并步骤
     └─ provider.merge_pull_request() (执行合并)
  5. 合并后清理（cleanup_after_merge()）
     └─ helpers::cleanup_branch() (切换到默认分支并删除当前分支)
  6. 更新 Jira 状态（update_jira_status()）
     ├─ JiraWorkHistory::read_work_history() (读取工作历史)
     ├─ extract_jira_ticket_from_pr_title() (从 PR 标题提取 ticket)
     ├─ JiraStatus::read_pull_request_merged_status() (读取合并状态)
     ├─ Jira::move_ticket() (更新 Jira ticket 状态)
     └─ JiraWorkHistory::delete_work_history_entry() (删除工作历史)
```

### 功能说明

合并 PR 命令通过 API 合并 PR，与 `integrate` 命令（本地 Git 合并）不同：

1. **前置检查**：
   - 运行所有检查（git status、network 等）

2. **PR ID 解析**：
   - 支持从命令行参数提供 PR ID
   - 如果不提供，自动从当前分支检测 PR ID

3. **合并操作**：
   - 先检查 PR 状态，如果已合并则跳过
   - 执行合并操作（通过平台 API）
   - 处理竞态条件（PR 在检查后、合并前被其他进程合并）

4. **合并后清理**：
   - 切换到默认分支
   - 删除当前分支（本地和远程）
   - 清理远程分支引用

5. **Jira 更新**：
   - 从工作历史或 PR 标题提取 Jira ticket
   - 更新 ticket 状态为合并状态
   - 删除工作历史记录

### 关键步骤说明

1. **状态检查**：
   - 合并前先检查 PR 状态，避免重复合并
   - 如果已合并，跳过合并步骤但继续执行后续清理和 Jira 更新

2. **错误处理**：
   - 使用 `helpers::is_pr_already_merged_error()` 检查是否是"已合并"错误
   - 处理竞态条件（PR 在检查后、合并前被其他进程合并）

3. **Jira ticket 提取**：
   - 优先从工作历史读取
   - 如果历史中没有，从 PR 标题提取（使用 `extract_jira_ticket_id()`）

---

## 4. 关闭 PR 命令 (`close`)

### 相关文件

```
src/commands/pr/close.rs (142 行)
```

### 调用流程

```
bin/pr.rs::PRCommands::Close
  ↓
commands/pr/close.rs::PullRequestCloseCommand::close()
  ↓
  1. 获取 PR ID（resolve_pull_request_id()）
  2. 获取当前分支名和默认分支
  3. 检查是否为默认分支（不允许关闭）
  4. 检查 PR 状态（check_if_already_closed()）
     └─ provider.get_pull_request_status() (如果已关闭，跳过关闭步骤)
  5. 关闭 PR（close_pull_request()）
     └─ provider.close_pull_request() (处理竞态条件)
  6. 删除远程分支（delete_remote_branch()）
     └─ GitBranch::delete_remote() (如果分支已被删除，忽略错误)
  7. 清理本地（cleanup_after_close()）
     └─ helpers::cleanup_branch() (切换到默认分支并删除当前分支)
```

### 功能说明

关闭 PR 命令用于关闭 PR 并清理相关分支：

1. **PR ID 解析**：
   - 支持从命令行参数提供 PR ID
   - 如果不提供，自动从当前分支检测 PR ID

2. **安全检查**：
   - 不允许在默认分支上关闭 PR

3. **状态检查**：
   - 关闭前先检查 PR 状态
   - 如果已关闭，跳过关闭步骤但继续执行后续清理

4. **关闭操作**：
   - 通过平台 API 关闭 PR
   - 处理竞态条件（PR 在检查后、关闭前被其他进程关闭）

5. **分支清理**：
   - 删除远程分支（如果存在）
   - 切换到默认分支
   - 删除当前分支（本地和远程）
   - 清理远程分支引用

### 关键步骤说明

1. **错误处理**：
   - 使用 `helpers::is_pr_already_closed_error()` 检查是否是"已关闭"错误
   - 处理竞态条件（PR 在检查后、关闭前被其他进程关闭）

2. **远程分支删除**：
   - 如果远程分支已被 API 删除，忽略错误
   - 如果删除失败，记录警告但不中断流程

---

## 5. PR 状态查询命令 (`status`)

### 相关文件

```
src/commands/pr/status.rs (50 行)
```

### 调用流程

```
bin/pr.rs::PRCommands::Status
  ↓
commands/pr/status.rs::PullRequestStatusCommand::show()
  ↓
  1. 获取 PR 标识符（get_pr_identifier()）
     ├─ 如果提供了参数：
     │   ├─ 尝试解析为数字（GitHub PR ID）
     │   └─ 如果不是数字，可能是分支名（Codeup 支持）
     └─ 如果没有提供，从当前分支获取（resolve_pull_request_id()）
  2. 显示 PR 信息（show_pr_info()）
     └─ provider.get_pull_request_info() (获取并格式化显示 PR 信息)
```

### 功能说明

PR 状态查询命令用于显示 PR 的详细信息：

1. **PR 标识符解析**：
   - 支持提供 PR ID（GitHub 只支持数字）
   - 支持提供分支名（Codeup 支持）
   - 如果不提供，自动从当前分支检测

2. **信息显示**：
   - 获取 PR 详细信息（状态、作者、评论等）
   - 格式化显示到终端

### 关键步骤说明

1. **平台差异**：
   - GitHub 只支持数字 PR ID
   - Codeup 支持 PR ID 或分支名

---

## 6. 列出 PR 命令 (`list`)

### 相关文件

```
src/commands/pr/list.rs (21 行)
```

### 调用流程

```
bin/pr.rs::PRCommands::List
  ↓
commands/pr/list.rs::PullRequestListCommand::list()
  ↓
  1. 创建平台提供者（create_provider()）
  2. 获取 PR 列表（provider.get_pull_requests()）
     ├─ 按状态过滤（state: open, closed, merged）
     └─ 限制结果数量（limit）
  3. 格式化输出到终端
```

### 功能说明

列出 PR 命令用于列出仓库中的所有 PR：

1. **过滤选项**：
   - `--state`：按状态过滤（open, closed, merged）
   - `--limit`：限制结果数量

2. **输出格式**：
   - 格式化显示 PR 列表（ID、标题、状态等）

---

## 7. 更新 PR 命令 (`update`)

### 相关文件

```
src/commands/pr/update.rs (59 行)
```

### 调用流程

```
bin/pr.rs::PRCommands::Update
  ↓
commands/pr/update.rs::PullRequestUpdateCommand::update()
  ↓
  1. 获取当前分支的 PR 标题（get_pull_request_title()）
     ├─ get_current_branch_pr_id() (获取当前分支的 PR ID)
     └─ provider.get_pull_request_title() (获取 PR 标题)
  2. 确定提交消息（使用 PR 标题或默认消息）
  3. 提交更改（GitCommit::commit()）
  4. 推送到远程（GitBranch::push()）
```

### 功能说明

更新 PR 命令用于快速更新 PR 代码：

1. **提交消息**：
   - 自动使用 PR 标题作为提交消息
   - 如果没有 PR 或获取标题失败，使用默认消息 "update"

2. **操作流程**：
   - 自动暂存所有文件
   - 提交更改
   - 推送到远程（更新 PR）

### 关键步骤说明

1. **PR 检测**：
   - 自动检测当前分支是否有 PR
   - 如果没有 PR，给出警告但继续执行

---

## 8. PR 命令辅助函数 (`helpers.rs`)

### 相关文件

```
src/commands/pr/helpers.rs (176 行)
```

### 功能说明

PR 命令辅助函数提供命令之间共享的通用功能：

1. **错误检查函数**：
   - `is_pr_already_merged_error()` - 检查错误是否表示 PR 已合并
   - `is_pr_already_closed_error()` - 检查错误是否表示 PR 已关闭

2. **分支清理函数**：
   - `cleanup_branch()` - 通用的分支清理逻辑
     - 切换到默认分支
     - 删除当前分支（本地和远程）
     - 清理远程分支引用
     - 处理 stash

### 关键步骤说明

1. **错误检查**：
   - 用于处理竞态条件（PR 在检查后、操作前被其他进程修改）
   - 检查错误消息和 HTTP 状态码

2. **分支清理**：
   - 统一的清理逻辑，被 `merge` 和 `close` 命令使用
   - 自动处理 stash、分支删除、远程引用清理

---

## 📊 数据流

### Create 命令数据流

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

### Merge 命令数据流

```
用户输入 (PR ID 或自动检测)
  ↓
PR API (检查状态、合并 PR)
  ↓
Git 操作 (切换到默认分支、删除当前分支)
  ↓
Jira 更新 (更新 ticket 状态、删除工作历史)
```

### Integrate 命令数据流

```
用户输入 (源分支、合并策略)
  ↓
Git 操作 (stash、合并分支、推送)
  ↓
PR API (更新当前分支 PR、关闭源分支 PR)
  ↓
Git 操作 (删除源分支)
```

---

## 🔗 与其他模块的集成

命令层通过调用 `lib/` 模块提供的 API 实现功能，主要使用的接口包括：

- **`lib/pr/`**：`create_provider()`、`helpers::resolve_pull_request_id()`、`helpers::get_current_branch_pr_id()`、`helpers::generate_pull_request_body()`、`helpers::generate_branch_name()`、`helpers::generate_commit_title()`、`PullRequestLLM::generate()`
- **`lib/git/`**：`GitBranch`、`GitCommit`、`GitRepo`、`GitStash` 的各种方法
- **`lib/jira/`**：`Jira::get_ticket_info()`、`Jira::assign_ticket()`、`Jira::move_ticket()`、`Jira::add_comment()`、`JiraStatus::*`、`JiraWorkHistory::*`
- **`lib/base/util/`**：`Browser::open()`、`Clipboard::copy()`、`confirm()`
- **`lib/commands/config/check/`**：`CheckCommand::run_all()`

详细实现请参考相关模块架构文档。

---

## 🎯 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `PullRequestCreateCommand::create()`
- `PullRequestMergeCommand::merge()`
- `PullRequestCloseCommand::close()`
- `PullRequestStatusCommand::show()`
- `PullRequestListCommand::list()`
- `PullRequestUpdateCommand::update()`
- `PullRequestIntegrateCommand::integrate()`

### 2. 工厂模式

使用工厂函数 `create_provider()` 创建平台提供者：
- 自动检测仓库类型（GitHub 或 Codeup）
- 返回对应的平台实现
- 命令层无需关心平台差异

### 3. 策略模式

不同的合并策略：
- `FastForwardOnly` - 只允许 fast-forward 合并
- `Squash` - Squash 合并
- `Merge` - 普通合并

### 4. 辅助函数模式

将通用逻辑提取到辅助函数：
- `helpers::cleanup_branch()` - 分支清理逻辑
- `helpers::is_pr_already_merged_error()` - 错误检查
- `helpers::is_pr_already_closed_error()` - 错误检查

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **平台层**：API 调用错误、网络错误

### 容错机制

- **PR 状态检查失败**：
  - 使用错误检查函数（`is_pr_already_merged_error()`、`is_pr_already_closed_error()`）处理竞态条件
  - 如果 PR 已合并/关闭，跳过操作但继续执行后续清理

- **分支操作失败**：
  - 提供清晰的错误提示
  - 自动尝试多种策略（如分支删除：先普通删除，失败则强制删除）

- **Jira 集成失败**：
  - 如果 Jira 操作失败，记录警告但不中断 PR 操作流程

- **LLM 生成失败**：
  - 如果 LLM 生成失败，自动回退到默认方法
  - 不影响 PR 创建流程

---

## 📝 扩展性

### 添加新命令

1. 在 `commands/pr/` 下创建新的命令文件
2. 实现命令结构体和处理方法
3. 在 `commands/pr/mod.rs` 中导出
4. 在 `bin/pr.rs` 中添加命令枚举和处理逻辑

### 添加新的平台支持

1. 在 `lib/pr/` 下创建新的平台模块（参考相关模块文档）
2. 实现 `PlatformProvider` trait
3. 在 `lib/pr/platform.rs` 的 `create_provider()` 中添加平台检测逻辑

### 添加新的合并策略

1. 在 `lib/git/` 中添加新的合并策略枚举（参考相关模块文档）
2. 在 `GitBranch::merge_branch()` 中实现新策略
3. 在 `integrate` 命令中添加对应的命令行参数

---

## 📚 相关文档

- [PR 模块架构文档](../lib/PR_ARCHITECTURE.md) - PR 平台抽象层相关
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md) - Git 操作相关
- [Jira 模块架构文档](../lib/JIRA_ARCHITECTURE.md) - Jira 集成相关
- [主架构文档](../ARCHITECTURE.md)

---

## 使用示例

### Create 命令

```bash
# 创建 PR（交互式输入）
pr create

# 创建 PR（提供 Jira ticket）
pr create PROJ-123

# 创建 PR（提供标题和描述）
pr create PROJ-123 --title "Fix bug" --description "Fixed a critical bug"

# 干运行模式（不实际创建 PR）
pr create --dry-run
```

### Merge 命令

```bash
# 合并当前分支的 PR
pr merge

# 合并指定 PR
pr merge 123

# 强制合并（跳过检查）
pr merge --force
```

### Close 命令

```bash
# 关闭当前分支的 PR
pr close

# 关闭指定 PR
pr close 123
```

### Status 命令

```bash
# 显示当前分支的 PR 状态
pr status

# 显示指定 PR 的状态
pr status 123

# 显示指定分支的 PR 状态（Codeup 支持）
pr status feature-branch
```

### List 命令

```bash
# 列出所有 PR
pr list

# 列出打开的 PR
pr list --state open

# 列出前 10 个 PR
pr list --limit 10
```

### Update 命令

```bash
# 更新当前分支的 PR（使用 PR 标题作为提交消息）
pr update
```

### Integrate 命令

```bash
# 合并分支到当前分支
pr integrate feature-branch

# 只允许 fast-forward 合并
pr integrate feature-branch --ff-only

# 使用 squash 合并
pr integrate feature-branch --squash

# 合并但不推送
pr integrate feature-branch --no-push
```

---

## 总结

PR 命令模块采用清晰的分层架构设计：
- **CLI 层**：参数解析和命令分发
- **命令层**：用户交互和业务逻辑协调
- **平台层**：统一的平台抽象接口
- **依赖模块**：Git、Jira、工具函数等

每个命令职责清晰，通过辅助函数减少代码重复，通过工厂模式实现平台无关的命令实现。整个模块提供了完整的 PR 生命周期管理功能，并与 Jira 深度集成，实现了高效的开发工作流。

