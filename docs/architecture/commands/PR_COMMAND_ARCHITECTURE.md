# PR 命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 PR 命令模块架构，包括：
- PR 创建、合并、关闭、查询等操作
- 分支集成功能
- 与 Jira 的集成（状态更新、工作历史管理）
- 与 Git 操作的集成（分支管理、提交等）

PR 命令模块是 Workflow CLI 的核心功能之一，提供完整的 Pull Request 生命周期管理，支持 GitHub 和 Codeup 两种代码托管平台。

**模块统计：**
- 命令数量：12 个（create, merge, close, status, list, update, sync, rebase, pick, summarize, approve, comment）
- 总代码行数：约 4000+ 行
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
- **命令枚举**：`PRCommands` 定义了所有 PR 相关的子命令（create, merge, close, status, list, update, sync, rebase, pick, summarize, approve, comment）

### 命令封装层

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
└── comment.rs      # 添加 PR 评论命令
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
  PRCommands::Sync => sync::PullRequestSyncCommand::sync()
  PRCommands::Rebase => rebase::PullRequestRebaseCommand::rebase()
  PRCommands::Pick => pick::PullRequestPickCommand::pick()
  PRCommands::Summarize => summarize::SummarizeCommand::summarize()
  PRCommands::Approve => approve::PullRequestApproveCommand::approve()
  PRCommands::Comment => comment::PullRequestCommentCommand::comment()
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

## 2. 同步分支命令 (`sync.rs`)

### 相关文件

```
src/commands/pr/sync.rs (488 行)
```

**注意**：`pr sync` 命令已合并了原 `pr integrate` 命令的所有功能，现在是一个统一的命令，通过参数控制所有行为。

### 调用流程

```
src/main.rs::PRCommands::Sync
  ↓
commands/pr/sync.rs::PullRequestSyncCommand::sync()
  ↓
  1. 运行检查（check::CheckCommand::run_all()）
  2. 获取当前分支，检查工作区状态并 stash
  3. 验证并准备源分支（prepare_source_branch()）
  4. 确定同步策略（--rebase, --ff-only, --squash, default merge）
  5. 执行同步（GitBranch::merge_branch() 或 GitBranch::rebase_onto()）
  6. 根据分支类型处理同步后的操作（更新当前分支 PR 或恢复 stash）
  7. 推送到远程（如果未指定 --no-push）
  8. 处理源分支的 PR 和分支清理（交互式）
     - 查找源分支的 PR ID
     - 如果有 PR：询问是否关闭 PR 并删除分支
     - 如果没有 PR：询问是否删除分支
```

### 功能说明

同步分支命令用于将指定分支同步到当前分支（本地 Git 操作），支持 merge、rebase 或 squash 三种方式：
1. **工作区管理**：自动检查并提示 stash。
2. **源分支验证**：自动检测分支存在性（本地或远程）。
3. **同步策略**：支持 merge（默认）、rebase、squash 和 fast-forward only。
4. **PR 管理**：自动更新当前分支的 PR（如果存在），并根据源分支是否有 PR 进行交互式处理。
5. **分支清理**：交互式确认是否删除源分支（如果有 PR，会询问是否关闭 PR 并删除分支）。

### 参数设计

**必需参数**:
- `SOURCE_BRANCH` - 要同步的源分支名称

**可选参数**:
- `--rebase` - 使用 rebase 而不是 merge（默认使用 merge）
- `--squash` - 使用 squash 合并（将所有提交压缩为一个）
- `--ff-only` - 只允许 fast-forward 合并（如果无法 fast-forward 则失败）
- `--no-push` - 不同步后推送（默认会推送）

**注意**：
- `--rebase`、`--squash` 和 `--ff-only` 是互斥的，只能使用其中一个
- **分支删除**：同步完成后会询问用户是否删除源分支（交互式确认），默认选择是
- **PR 管理**：自动更新当前分支的 PR，并根据源分支是否有 PR 进行交互式处理

### 详细工作流程

#### Merge 模式（默认）

```
1. 获取当前分支
2. 检查工作区状态
   - 如果有未提交更改，自动 stash
3. 验证并准备源分支
   - 检查分支是否存在（本地或远程）
   - 如果只在远程，先 fetch
4. 合并 SOURCE_BRANCH 到当前分支
   - 如果有冲突，暂停并提示用户
   - 合并成功会自动创建 merge commit
5. 恢复 stash（如果有）
6. 管理 PR（自动）
   - 更新当前分支的 PR（如果存在）
7. 推送到远程（如果未指定 --no-push）
   - 如果指定 --no-push，只同步到本地，不推送
8. 处理源分支的 PR 和分支清理（交互式）
   - 查找源分支的 PR ID
   - 如果有 PR：询问是否关闭 PR 并删除分支
   - 如果没有 PR：询问是否删除分支
   - 自动跳过删除当前分支和默认分支
```

#### Rebase 模式（--rebase）

```
1. 获取当前分支
2. 检查工作区状态
   - 如果有未提交更改，自动 stash
3. 验证并准备源分支
   - 检查分支是否存在（本地或远程）
   - 如果只在远程，先 fetch
4. Rebase 当前分支到 SOURCE_BRANCH
   - 如果有冲突，暂停并提示用户
   - Rebase 会重写当前分支的提交历史
5. 恢复 stash（如果有）
6. 管理 PR（自动）
   - 更新当前分支的 PR（如果存在）
7. 强制推送到远程（如果未指定 --no-push，使用 --force-with-lease）
   - 如果指定 --no-push，只同步到本地，不推送
8. 处理源分支的 PR 和分支清理（交互式）
   - 查找源分支的 PR ID
   - 如果有 PR：询问是否关闭 PR 并删除分支
   - 如果没有 PR：询问是否删除分支
   - 自动跳过删除当前分支和默认分支
```

#### Squash 模式（--squash）

```
1. 获取当前分支
2. 检查工作区状态
   - 如果有未提交更改，自动 stash
3. 验证并准备源分支
   - 检查分支是否存在（本地或远程）
   - 如果只在远程，先 fetch
4. Squash 合并 SOURCE_BRANCH 到当前分支
   - 将所有提交压缩为一个提交
   - 如果有冲突，暂停并提示用户
5. 恢复 stash（如果有）
6. 管理 PR（自动）
   - 更新当前分支的 PR（如果存在）
7. 推送到远程（如果未指定 --no-push）
   - 如果指定 --no-push，只同步到本地，不推送
8. 处理源分支的 PR 和分支清理（交互式）
   - 查找源分支的 PR ID
   - 如果有 PR：询问是否关闭 PR 并删除分支
   - 如果没有 PR：询问是否删除分支
   - 自动跳过删除当前分支和默认分支
```

### 使用场景

#### 场景 1：同步 master 到当前分支（merge）

```bash
# 在 feature-branch 上
git checkout feature-branch
workflow pr sync master

# 结果：
# - 将 master 的最新更改合并到 feature-branch
# - 自动推送
```

#### 场景 2：同步 master 到当前分支（rebase）

```bash
# 在 feature-branch 上
git checkout feature-branch
workflow pr sync master --rebase

# 结果：
# - 将 feature-branch rebase 到 master 的最新提交
# - 强制推送（使用 --force-with-lease）
```

#### 场景 3：只同步到本地并提交，不推送

```bash
workflow pr sync master --no-push

# 结果：
# - 同步 source_branch 到当前分支
# - 自动提交（merge 会创建 merge commit，rebase 会重写提交历史）
# - 不推送到远程（用户可以手动推送）
#
# 使用场景：
# - 需要先本地测试同步后的代码
# - 需要批量同步多个分支后再统一推送
# - 需要审查同步后的更改再推送
```

#### 场景 4：使用 squash 合并功能分支

```bash
workflow pr sync feature-sub --squash

# 结果：
# - 将 feature-sub 的所有提交压缩为一个提交
# - 合并到当前分支
# - 自动删除分支和管理 PR
```

#### 场景 5：Fast-forward only

```bash
workflow pr sync master --ff-only

# 结果：
# - 只允许 fast-forward 合并
# - 如果无法 fast-forward，失败并提示
```

### 边界情况处理

#### 情况 1：当前分支不存在

**处理方式**:
- 检查是否在 Git 仓库中
- 检查当前分支是否存在
- 如果不存在，提示错误并退出

#### 情况 2：源分支不存在

**处理方式**:
- 检查 SOURCE_BRANCH 是否存在（本地和远程）
- 如果不存在，提示错误并退出
- 提供建议：检查分支名拼写

#### 情况 3：合并/变基冲突

**处理方式**:
- 暂停操作
- 显示冲突文件列表
- 提示用户解决冲突：
  ```
  Merge/Rebase conflict detected!
  Please resolve conflicts in:
    - file1.rs
    - file2.rs

  After resolving:
    1. git add <resolved-files>
    2. git commit  # for merge
    2. git rebase --continue  # for rebase
    3. workflow pr sync --continue  # 继续执行
  ```
- 提供 `--continue` 选项继续执行
- 提供 `--abort` 选项中止操作

#### 情况 4：工作区有未提交更改

**处理方式**:
- 自动检测未提交更改
- 自动 stash（保存更改）
- 操作完成后恢复 stash
- 如果 stash 恢复有冲突，提示用户手动解决

#### 情况 5：Fast-forward 失败（使用 --ff-only）

**处理方式**:
- 检测到无法 fast-forward
- 提示错误：
  ```
  Cannot fast-forward merge. Use 'workflow pr sync SOURCE_BRANCH'
  (without --ff-only) to perform a regular merge.
  ```
- 退出，不执行合并

#### 情况 6：Rebase 后推送失败

**处理方式**:
- 使用 `--force-with-lease` 安全地强制推送
- 如果推送失败（例如远程有新的提交），提示用户：
  ```
  Push failed. Remote branch has new commits.
  Please pull and rebase again, or use 'git push --force' manually.
  ```

#### 情况 7：源分支有 PR，用户拒绝关闭 PR 和删除分支

**处理方式**:
- 询问用户是否关闭 PR 并删除分支
- 如果用户选择否，跳过关闭 PR 和删除分支操作
- 提示：`Skipping PR closure and branch deletion as requested by user`
- PR 和分支保留

#### 情况 8：源分支没有 PR，用户拒绝删除分支

**处理方式**:
- 询问用户是否删除源分支
- 如果用户选择否，跳过删除操作
- 提示：`Skipping branch deletion as requested by user`
- 分支保留在本地和远程

#### 情况 9：源分支是默认分支

**处理方式**:
- 自动检测源分支是否为默认分支
- 如果是默认分支，自动跳过删除（安全保护）
- 提示：`Source branch 'master' is the default branch, skipping deletion for safety`

### 与其他命令的关系

#### 与原 `pr integrate` 的关系

**`pr sync` 已合并 `pr integrate` 的所有功能**：

| 特性 | 原 `pr integrate` | `pr sync`（合并后） |
|------|-------------------|-------------------|
| **操作类型** | 合并（merge） | 同步（merge、rebase 或 squash） |
| **默认行为** | Merge + 删除分支 + 管理 PR | Merge + 交互式确认删除分支 + 自动管理 PR |
| **合并策略** | Merge/Squash | Merge/Rebase/Squash |
| **分支清理** | ✅ 自动删除 | ✅ 交互式确认删除（默认选择是） |
| **PR 管理** | ✅ 自动管理 | ✅ 自动管理（总是执行） |
| **参数** | `--squash`, `--ff-only`, `--no-push` | `--rebase`, `--squash`, `--ff-only`, `--no-push` |

**迁移指南**：

原 `pr integrate` 用法：
```bash
workflow pr integrate feature-sub
workflow pr integrate feature-sub --squash
workflow pr integrate feature-sub --no-push
```

新的 `pr sync` 用法（保持相同行为）：
```bash
workflow pr sync feature-sub  # 默认行为相同（会询问是否删除分支）
workflow pr sync feature-sub --squash
workflow pr sync feature-sub --no-push
```

### 错误处理和用户体验

#### 错误消息

- **当前分支不存在**: `Not on a valid branch. Please checkout a branch first.`
- **源分支不存在**: `Source branch '{SOURCE_BRANCH}' does not exist. Please check the branch name.`
- **合并冲突**: `Merge conflict detected. Please resolve conflicts and use '--continue' to proceed.`
- **Rebase 冲突**: `Rebase conflict detected. Please resolve conflicts and use '--continue' to proceed.`
- **Fast-forward 失败**: `Cannot fast-forward merge. Remove '--ff-only' to perform a regular merge.`

#### 成功消息

```
✓ Synced master to feature-branch
✓ Merged 5 commits
✓ Pushed to remote
```

#### 交互式提示

**冲突解决提示**：

```
⚠️  Conflict detected during merge/rebase

Conflicted files:
  - src/file1.rs
  - src/file2.rs

To resolve:
  1. Edit conflicted files
  2. git add <resolved-files>
  3. workflow pr sync --continue

To abort:
  workflow pr sync --abort
```

---

## 3. Rebase 分支命令 (`rebase.rs`)

### 相关文件

```
src/commands/pr/rebase.rs (507 行)
```

### 调用流程

```
src/main.rs::PRCommands::Rebase
  ↓
commands/pr/rebase.rs::PullRequestRebaseCommand::rebase()
  ↓
  1. 运行预检查（check::CheckCommand::run_all()）
  2. 获取当前分支
  3. 验证目标分支存在（本地或远程）
  4. 检查工作区状态并 stash（如果有未提交更改）
  5. 拉取目标分支最新代码（GitRepo::fetch()）
  6. 执行 rebase（GitBranch::rebase_onto()）
  7. 处理 rebase 冲突（如果发生）
  8. 更新 PR base（如果找到 PR，提示用户确认）
     └─ provider.update_pr_base()
  9. 恢复 stash（如果有）
  10. 推送到远程（默认推送，使用 --force-with-lease）
```

### 功能说明

Rebase 分支命令用于将当前分支 rebase 到目标分支，并可选地更新 PR 的 base 分支：

1. **预检查**：运行环境检查（Git 状态、网络等），失败时提示用户是否继续。
2. **工作区管理**：自动检测未提交更改，提示用户 stash。
3. **目标分支验证**：检查目标分支是否存在（本地或远程）。
4. **Rebase 操作**：执行 `git rebase` 操作，重写当前分支的提交历史。
5. **冲突处理**：如果发生冲突，暂停操作并提示用户手动解决。
6. **PR 管理**：自动检测当前分支的 PR，如果找到则提示用户确认是否更新 PR base。
7. **安全推送**：默认推送到远程，使用 `--force-with-lease` 确保安全。

### 参数设计

**必需参数**:
- `TARGET_BRANCH` - 目标分支名称（要 rebase 到的分支）

**可选参数**:
- `--no-push` - 不推送到远程（默认会推送）
- `--dry-run` - 预览模式（显示将要执行的操作，不实际执行）

**注意**：
- PR ID 会自动从当前分支检测，无需手动指定
- 如果找到 PR，会提示用户确认是否更新 PR base
- 默认推送到远程（除非使用 `--no-push`）
- 使用 `--force-with-lease` 安全地强制推送

### 详细工作流程

```
1. 运行预检查
   - 检查 Git 状态、网络连接等
   - 如果检查失败，提示用户是否继续

2. 获取当前分支
   - 验证当前在有效的分支上

3. 验证目标分支
   - 检查目标分支是否存在（本地或远程）
   - 如果不存在，提示错误并退出

4. 检查工作区状态
   - 如果有未提交更改，提示用户是否 stash
   - 如果用户确认，自动 stash 更改

5. 拉取最新代码
   - 执行 git fetch 获取远程最新代码

6. 执行 rebase
   - 执行 git rebase TARGET_BRANCH
   - 如果发生冲突，暂停并提示用户解决

7. 处理冲突（如果发生）
   - 显示冲突文件列表
   - 提示用户解决冲突的步骤
   - 等待用户手动解决后继续

8. 更新 PR base（如果找到 PR）
   - 自动检测当前分支的 PR ID
   - 如果找到 PR，提示用户确认是否更新 PR base
   - 如果用户确认，调用 provider.update_pr_base()

9. 恢复 stash（如果有）
   - 如果之前 stash 了更改，自动恢复

10. 推送到远程（默认）
    - 使用 git push --force-with-lease 安全地强制推送
    - 如果使用 --no-push，跳过推送
```

### 使用场景

#### 场景 1：Rebase 当前分支到 master

```bash
# 在 feature-branch 上
git checkout feature-branch
workflow pr rebase master

# 结果：
# - 将 feature-branch rebase 到 master 的最新提交
# - 如果找到 PR，提示是否更新 PR base 到 master
# - 默认推送到远程（使用 --force-with-lease）
```

#### 场景 2：只 rebase 到本地，不推送

```bash
workflow pr rebase master --no-push

# 结果：
# - 只 rebase 到本地，不推送到远程
# - 用户可以手动推送
```

#### 场景 3：预览模式

```bash
workflow pr rebase master --dry-run

# 结果：
# - 显示将要执行的操作
# - 不实际执行 rebase
```

### 边界情况处理

#### 情况 1：目标分支不存在

**处理方式**:
- 检查目标分支是否存在（本地和远程）
- 如果不存在，提示错误并退出

#### 情况 2：Rebase 冲突

**处理方式**:
- 暂停操作
- 显示冲突文件列表
- 提示用户解决冲突：
  ```
  Rebase conflicts detected!
  Please resolve the conflicts manually:
    1. Review conflicted files
    2. Resolve conflicts
    3. Stage resolved files: git add <files>
    4. Continue rebase: git rebase --continue
    5. Push when ready: git push --force-with-lease
  ```

#### 情况 3：工作区有未提交更改

**处理方式**:
- 自动检测未提交更改
- 提示用户是否 stash
- 如果用户确认，自动 stash
- 操作完成后恢复 stash

#### 情况 4：未找到 PR

**处理方式**:
- 自动检测当前分支的 PR
- 如果未找到，跳过 PR base 更新（不报错）
- 记录警告信息

#### 情况 5：推送失败

**处理方式**:
- 使用 `--force-with-lease` 安全地强制推送
- 如果推送失败（例如远程有新的提交），提示用户重新 rebase

### 与其他命令的关系

#### 与 `pr sync` 的关系

| 特性 | `pr sync` | `pr rebase` |
|------|-----------|-------------|
| **操作类型** | 同步分支（merge/rebase/squash） | Rebase 当前分支 |
| **目标** | 将源分支同步到当前分支 | 将当前分支 rebase 到目标分支 |
| **PR 管理** | 自动更新当前分支的 PR | 提示用户确认更新 PR base |
| **分支清理** | 交互式确认删除源分支 | 不删除分支 |
| **推送** | 默认推送（merge 正常推送，rebase 使用 force-with-lease） | 默认推送（使用 force-with-lease） |

**使用建议**：
- 使用 `pr sync` 将其他分支的更改同步到当前分支
- 使用 `pr rebase` 将当前分支 rebase 到目标分支并更新 PR base

---

## 4. Pick 提交命令 (`pick.rs`)

### 相关文件

```
src/commands/pr/pick.rs (978 行)
```

### 调用流程

```
src/main.rs::PRCommands::Pick
  ↓
commands/pr/pick.rs::PullRequestPickCommand::pick()
  ↓
  1. 运行预检查（check::CheckCommand::run_all()）
  2. 验证分支存在（validate_branches()）
  3. 拉取最新代码（GitRepo::fetch()）
  4. 检测新提交（get_new_commits()）
     └─ GitBranch::get_commits_between()
  5. 保存当前分支和工作区状态
  6. 检查工作区状态（check_working_directory()）
     └─ 如果有未提交修改，自动 stash
  7. 切换到 TO_BRANCH（GitBranch::checkout_branch()）
  8. Cherry-pick 所有提交（cherry_pick_commits_no_commit()）
     └─ GitCherryPick::cherry_pick_no_commit() (--no-commit)
  9. 处理 cherry-pick 结果
     ├─ 成功：继续执行
     └─ 冲突：handle_cherry_pick_conflict()（暂停并提示用户）
  10. 获取源 PR 信息（get_source_pr_info()）
      └─ 临时切换到源分支获取 PR 信息
  11. 询问是否创建 PR（confirm()）
  12. 交互式 PR 创建流程（create_pr_interactively()）
      ├─ 从源 PR 提取信息（extract_info_from_source_pr()）
      ├─ 确定 LLM 输入（determine_llm_input()）
      ├─ 生成分支名和 PR 标题（generate_commit_title_and_branch_name_for_pick()）
      ├─ 确定 Jira ticket（resolve_jira_ticket()）
      ├─ 配置 Jira 状态（ensure_jira_status()）
      ├─ 确定 PR 标题（resolve_title()）
      ├─ 获取描述（resolve_description()）
      ├─ 选择变更类型（select_change_types()）
      ├─ 生成 PR body（generate_pull_request_body()）
      ├─ 创建或更新分支（create_or_update_branch()）
      ├─ 创建或获取 PR（create_or_get_pull_request()）
      ├─ 更新 Jira ticket（update_jira_ticket()）
      └─ 复制 PR URL 并打开浏览器（copy_and_open_pull_request()）
  13. 恢复原分支和 stash（无论 PR 创建是否成功）
```

### 功能说明

Pick 提交命令用于跨分支移植代码，从源分支 cherry-pick 提交到目标分支并创建新 PR：

1. **跨分支操作**：支持从任意分支 cherry-pick 到任意分支，类似于 backport/forwardport，但支持任意方向。
2. **智能提交检测**：自动检测源分支相对于目标分支的新提交。
3. **源 PR 信息提取**：如果源分支有 PR，自动提取标题、描述、Jira ticket、变更类型等信息。
4. **Cherry-pick 处理**：使用 `--no-commit` 模式，允许在创建分支前统一提交。
5. **冲突处理**：检测冲突并提供详细的解决指引，支持放弃或继续。
6. **交互式 PR 创建**：复用 `create` 命令的交互式流程，支持 LLM 生成分支名和标题。
7. **状态恢复**：无论成功或失败，都会恢复原分支和 stash，确保工作区状态一致。

### 参数设计

**必需参数**:
- `FROM_BRANCH` - 源分支名称（要 cherry-pick 的分支）
- `TO_BRANCH` - 目标分支名称（新 PR 的 base 分支）

**可选参数**:
- `--dry-run` - 预览模式，显示将要执行的操作但不实际执行

**注意**：
- 当前实现中，分支名和 PR 标题通过交互式流程生成（使用 LLM 或用户输入）
- 未来可能支持 `--branch-name`、`--pr-title`、`--pr-description`、`--no-create-pr`、`--force` 等参数

### 详细工作流程

```
1. 预检查
   - 运行环境检查（Git 状态、网络连接）

2. 验证分支存在
   - 检查 FROM_BRANCH 是否存在（本地或远程）
   - 检查 TO_BRANCH 是否存在（本地或远程）

3. 拉取最新代码
   - GitRepo::fetch() 更新远程分支信息

4. 检测新提交
   - 使用 GitBranch::get_commits_between(TO_BRANCH, FROM_BRANCH)
   - 如果没有新提交，提示并退出

5. 保存当前状态
   - 保存当前分支名
   - 检查工作区状态，如果有未提交修改，自动 stash

6. 切换到 TO_BRANCH
   - GitBranch::checkout_branch(TO_BRANCH)

7. Cherry-pick 所有提交
   - 按顺序 cherry-pick 每个提交（使用 --no-commit）
   - 如果遇到冲突，暂停并提示用户

8. 获取源 PR 信息
   - 临时切换到源分支
   - 获取 PR ID、标题、URL、body
   - 恢复原分支（TO_BRANCH）

9. 询问是否创建 PR
   - 如果用户选择不创建，询问是否保留修改

10. 交互式 PR 创建流程
    - 从源 PR body 提取信息（Jira ticket、描述、变更类型）
    - 使用 LLM 生成分支名和 PR 标题
    - 交互式确认或输入各项信息
    - 创建新分支并提交
    - 创建 PR
    - 更新 Jira（如果有 ticket）
    - 复制 URL 并打开浏览器

11. 恢复原分支和 stash
    - 无论 PR 创建是否成功，都会恢复
```

### 使用场景

#### 场景 1：将 develop 的修改应用到 master

```bash
workflow pr pick develop master

# 结果：
# - 创建新分支（LLM 生成名称）
# - Cherry-pick develop 的所有新提交
# - 创建基于 master 的新 PR
# - 从 develop 的 PR 复制信息（如果存在）
```

#### 场景 2：将 feature 分支应用到 release 分支

```bash
workflow pr pick feature-branch release/v1.0

# 结果：
# - 创建新分支
# - Cherry-pick feature-branch 的所有提交
# - 创建基于 release/v1.0 的新 PR
```

#### 场景 3：预览模式

```bash
workflow pr pick develop master --dry-run

# 结果：
# - 显示将要执行的操作
# - 不实际执行
```

### 边界情况处理

#### 情况 1：源分支没有新提交

**处理方式**:
- 检测到 FROM_BRANCH 相对于 TO_BRANCH 没有新提交
- 提示用户：`No new commits to cherry-pick from {FROM_BRANCH} to {TO_BRANCH}`
- 退出，不创建分支和 PR

#### 情况 2：Cherry-pick 冲突

**处理方式**:
- 检测冲突（检查 `.git/CHERRY_PICK_HEAD` 文件或错误消息）
- 暂停 cherry-pick 操作
- 显示详细的解决指引
- 询问用户是否放弃并恢复原分支
- 如果用户选择保留冲突状态，保持在 TO_BRANCH 上让用户解决

#### 情况 3：源分支不存在

**处理方式**:
- 检查 FROM_BRANCH 是否存在（本地和远程）
- 如果不存在，提示错误并退出

#### 情况 4：目标分支不存在

**处理方式**:
- 检查 TO_BRANCH 是否存在（本地和远程）
- 如果不存在，提示错误并退出

#### 情况 5：工作区有未提交修改

**处理方式**:
- 自动检测未提交修改
- 询问用户是否 stash
- 如果用户同意，自动 stash 并在完成后恢复

#### 情况 6：PR 创建失败

**处理方式**:
- 无论 PR 创建是否成功，都会恢复原分支和 stash
- 如果失败，返回错误信息
- 确保工作区状态一致

### 与其他命令的关系

#### 与 `pr create` 的关系

| 特性 | `pr pick` | `pr create` |
|------|-----------|-------------|
| **操作方式** | Cherry-pick + 创建 PR | 直接创建 PR |
| **提交来源** | 从源分支 cherry-pick | 当前工作区的修改 |
| **分支创建** | 基于 TO_BRANCH 创建新分支 | 基于当前分支或默认分支 |
| **源 PR 信息** | 自动从源 PR 提取 | 手动输入或 LLM 生成 |
| **使用场景** | 跨分支移植代码 | 创建新功能 PR |

#### 与 `pr sync` 的关系

| 特性 | `pr pick` | `pr sync` |
|------|-----------|-----------|
| **操作对象** | 从分支 A 到分支 B | 从分支 A 到当前分支 |
| **操作方式** | Cherry-pick | Merge、Rebase 或 Squash |
| **创建新分支** | ✅ 是 | ❌ 否（在当前分支操作） |
| **创建新 PR** | ✅ 是（可选） | ❌ 否 |
| **使用场景** | 跨分支移植代码 | 同步基础分支更新 |

#### 与 `pr rebase` 的关系

| 特性 | `pr pick` | `pr rebase` |
|------|-----------|-------------|
| **操作方式** | Cherry-pick | Rebase |
| **创建新分支** | ✅ 是 | ❌ 否（在当前分支操作） |
| **创建新 PR** | ✅ 是 | ❌ 否（更新现有 PR） |
| **使用场景** | 跨分支移植代码 | 修正当前分支的基础分支 |
| **结果** | 新分支 + 新 PR | 修改现有分支 + 更新 PR base |

**使用建议**：
- 使用 `pr pick` 跨分支移植代码并创建新 PR
- 使用 `pr sync` 同步基础分支到当前分支
- 使用 `pr rebase` 修正当前分支的基础分支

---

## 5. 合并 PR 命令 (`merge.rs`)

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
  1. 运行检查，获取 PR ID
  2. 合并 PR（merge_pull_request()）
     └─ provider.merge_pull_request()
  3. 合并后清理（cleanup_after_merge()）
     └─ helpers::cleanup_branch() (切换到默认分支并删除当前分支)
  4. 更新 Jira 状态（update_jira_status()）
```

### 功能说明

合并 PR 命令通过 API 合并 PR：
1. **PR ID 解析**：支持参数提供或自动检测。
2. **合并操作**：通过平台 API 执行合并，处理竞态条件。
3. **合并后清理**：切换到默认分支，删除当前分支（本地和远程）。
4. **Jira 更新**：更新 ticket 状态为合并状态，删除工作历史。

---

## 6. 关闭 PR 命令 (`close.rs`)

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

## 7. PR 状态查询命令 (`status.rs`)

### 相关文件

```
src/commands/pr/status.rs (50 行)
```

### 功能说明
PR 状态查询命令用于显示 PR 的详细信息（状态、作者、评论等）。支持 PR ID 或分支名查询。

---

## 8. 列出 PR 命令 (`list.rs`)

### 相关文件

```
src/commands/pr/list.rs (21 行)
```

### 功能说明
列出仓库中的所有 PR。支持按状态过滤（--state）和限制数量（--limit）。

---

## 9. 更新 PR 命令 (`update.rs`)

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
  1. 获取当前分支的 PR 标题
  2. 提交更改（GitCommit::commit()）
  3. 推送到远程（GitBranch::push()）
```

### 功能说明
快速更新 PR 代码。自动使用 PR 标题作为提交消息，暂存所有更改，提交并推送。

---

## 10. PR 总结命令 (`summarize.rs`)

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

## 11. 批准 PR 命令 (`approve.rs`)

### 相关文件

```
src/commands/pr/approve.rs (44 行)
```

### 调用流程

```
src/main.rs::PRCommands::Approve
  ↓
commands/pr/approve.rs::PullRequestApproveCommand::approve()
  ↓
  1. 获取 PR ID（参数或自动检测当前分支）
  2. 创建平台提供者（create_provider()）
  3. 批准 PR（provider.approve_pull_request()）
```

### 功能说明

批准 PR 命令用于批准指定的 Pull Request：

1. **PR ID 解析**：
   - 支持通过参数指定 PR ID
   - 如果不提供参数，自动检测当前分支对应的 PR
   - 如果当前分支没有对应的 PR，会提示用户手动指定 PR ID

2. **错误处理**：
   - 如果尝试批准自己的 PR，会返回明确的错误信息
   - 其他错误会添加上下文信息以便调试

### 使用示例

```bash
workflow pr approve                    # 批准当前分支的 PR
workflow pr approve 123                 # 批准指定 PR ID
```

---

## 12. 添加 PR 评论命令 (`comment.rs`)

### 相关文件

```
src/commands/pr/comment.rs (39 行)
```

### 调用流程

```
src/main.rs::PRCommands::Comment
  ↓
commands/pr/comment.rs::PullRequestCommentCommand::comment()
  ↓
  1. 获取评论内容（将多个单词组合成一个字符串）
  2. 获取 PR ID（参数或自动检测当前分支）
  3. 创建平台提供者（create_provider()）
  4. 添加评论（provider.add_comment()）
```

### 功能说明

添加 PR 评论命令用于向指定的 Pull Request 添加评论：

1. **评论内容**：
   - 支持多个单词作为评论内容（使用 `trailing_var_arg` 参数）
   - 多个单词会自动组合成一个字符串（用空格分隔）
   - 评论内容为必需参数，如果为空会提示错误

2. **PR ID 解析**：
   - 支持通过参数指定 PR ID
   - 如果不提供参数，自动检测当前分支对应的 PR
   - 如果当前分支没有对应的 PR，会提示用户手动指定 PR ID

### 使用示例

```bash
workflow pr comment "Great work!"                    # 向当前分支的 PR 添加评论
workflow pr comment 123 "Looks good to me"          # 向指定 PR ID 添加评论
workflow pr comment "This needs more tests"        # 多个单词自动组合
```

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

### Approve 命令
```bash
workflow pr approve                            # 批准当前分支的 PR
workflow pr approve 123                       # 批准指定 PR ID
```

### Comment 命令
```bash
workflow pr comment "Great work!"             # 向当前分支的 PR 添加评论
workflow pr comment 123 "Looks good to me"   # 向指定 PR ID 添加评论
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

### PR Pick 命令相关文档

**注意**：PR Pick 命令的设计文档和流程图已归档，功能已完成。详细使用说明请参考本文档的 [Pick 提交命令章节](#4-pick-提交命令-pickrs)。

### 其他相关文档

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
