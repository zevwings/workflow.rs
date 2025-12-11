# Commit 命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Commit 命令模块架构，包括：
- Commit amend 功能（修改最后一次提交的消息和文件）
- Commit reword 功能（修改指定提交的消息，不改变内容）

Commit 命令模块提供交互式的提交修改功能，支持修改最后一次提交（amend）和修改历史提交（reword）。amend 命令支持修改提交消息、添加文件或两者同时进行，并提供完整的预览和确认机制。reword 命令支持修改 HEAD 或历史提交的消息，对于历史提交使用 rebase 交互式编辑，对于 HEAD 使用 amend。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/commit/` 模块提供。

**模块统计：**
- 命令数量：2 个（amend、reword）
- 总代码行数：约 476 行
- 文件数量：3 个
- 主要依赖：`lib/commit/`、`lib/git/`、`commands/check/`、`lib/base/dialog/`

---

## 📁 相关文件

### CLI 入口层

```
src/bin/workflow.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow commit` 子命令分发到对应的命令处理函数

### 命令封装层

```
src/commands/commit/
├── mod.rs          # Commit 命令模块声明（7 行）
├── amend.rs        # Commit amend 命令（240 行）
└── reword.rs       # Commit reword 命令（229 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择、确认等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/commit/`) 的功能
- 执行 Git 操作（通过 `lib/git/` 模块）

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/commit/`**：Commit 业务逻辑
  - `CommitAmend::create_preview()` - 创建 amend 预览信息
  - `CommitAmend::format_preview()` - 格式化预览信息
  - `CommitAmend::format_commit_info_detailed()` - 格式化详细 commit 信息
  - `CommitAmend::format_completion_message()` - 生成完成提示信息
  - `CommitReword::create_preview()` - 创建 reword 预览信息
  - `CommitReword::format_preview()` - 格式化预览信息
  - `CommitReword::format_commit_info()` - 格式化 commit 信息
  - `CommitReword::reword_history_commit()` - 执行历史 commit reword
- **`lib/git/`**：Git 操作
  - `GitBranch::current_branch()` - 获取当前分支
  - `GitBranch::get_default_branch()` - 获取默认分支
  - `GitBranch::is_commit_in_remote()` - 检查 commit 是否在远程
  - `GitCommit::has_last_commit()` - 检查是否有最后一次 commit
  - `GitCommit::get_last_commit_info()` - 获取最后一次 commit 信息
  - `GitCommit::get_commit_info()` - 获取指定 commit 信息
  - `GitCommit::get_worktree_status()` - 获取工作区状态
  - `GitCommit::get_modified_files()` - 获取修改的文件
  - `GitCommit::get_untracked_files()` - 获取未跟踪的文件
  - `GitCommit::get_branch_commits()` - 获取分支 commits
  - `GitCommit::parse_commit_ref()` - 解析 commit 引用
  - `GitCommit::is_commit_in_current_branch()` - 检查 commit 是否在当前分支
  - `GitCommit::is_head_commit()` - 检查是否是 HEAD commit
  - `GitCommit::has_commit()` - 检查是否有未提交的更改
  - `GitCommit::add_all()` - 暂存所有文件
  - `GitCommit::add_files()` - 暂存指定文件
  - `GitCommit::amend()` - 执行 amend 操作
- **`commands/check/`**：环境检查
  - `CheckCommand::run_all()` - 运行所有检查
- **`lib/base/dialog/`**：用户交互对话框
  - `ConfirmDialog` - 确认对话框
  - `InputDialog` - 输入对话框
  - `SelectDialog` - 选择对话框
  - `MultiSelectDialog` - 多选对话框

详细架构文档：参见 [Commit 模块架构文档](../lib/COMMIT_ARCHITECTURE.md)、[Git 模块架构文档](../lib/GIT_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/bin/workflow.rs (workflow 主命令，参数解析)
  ↓
commands/commit/*.rs (命令封装层，处理交互)
  ↓
lib/commit/* (业务逻辑层，预览、格式化等)
  ↓
lib/git/* (Git API 调用，具体实现见相关模块文档)
  ↓
Git 仓库操作
```

### 命令分发流程

```
src/bin/workflow.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.command {
  Commands::Commit { subcommand } => match subcommand {
    CommitSubcommand::Amend { message, no_edit, no_verify } =>
      CommitAmendCommand::execute(message, no_edit, no_verify)
    CommitSubcommand::Reword { commit_id } =>
      CommitRewordCommand::execute(commit_id)
  }
}
```

### 数据流

```
用户输入参数
  ↓
命令参数解析（clap）
  ↓
环境检查（CheckCommand::run_all()）
  ↓
分支保护检查（检查是否是默认分支）
  ↓
Git 状态检查（检查是否有 commit、工作区状态等）
  ↓
用户交互（选择操作类型、输入消息、选择文件等）
  ↓
预览信息生成（lib/commit/）
  ↓
用户确认
  ↓
执行 Git 操作（lib/git/）
  ↓
完成提示
```

---

## 1. Commit Amend 命令 (`amend.rs`)

### 相关文件

```
src/commands/commit/amend.rs (240 行)
```

### 调用流程

```
CommitAmendCommand::execute(message, no_edit, no_verify)
  ↓
CheckCommand::run_all() (环境检查)
  ↓
GitBranch::current_branch() (获取当前分支)
  ↓
GitBranch::get_default_branch() (检查是否是默认分支)
  ↓
GitCommit::has_last_commit() (检查是否有最后一次 commit)
  ↓
GitCommit::get_last_commit_info() (获取最后一次 commit 信息)
  ↓
GitCommit::get_worktree_status() (获取工作区状态)
  ↓
CommitAmend::format_commit_info_detailed() (格式化显示当前 commit 信息)
  ↓
选择操作类型（如果未提供参数）：
  - CommitAmendCommand::select_operation() (交互式选择)
  - 或根据参数确定操作类型
  ↓
根据操作类型执行：
  - MessageOnly: CommitAmendCommand::input_new_message() (输入新消息)
  - FilesOnly: CommitAmendCommand::select_files_to_add() (选择文件)
  - MessageAndFiles: 输入消息 + 选择文件
  - NoEdit: 检查并暂存未提交的更改
  ↓
CommitAmend::create_preview() (创建预览信息)
  ↓
CommitAmend::format_preview() (格式化预览信息)
  ↓
ConfirmDialog::prompt() (最终确认)
  ↓
GitCommit::add_files() (暂存文件，如果需要)
  ↓
GitCommit::amend() (执行 amend 操作)
  ↓
CommitAmend::format_completion_message() (生成完成提示)
```

### 功能说明

Commit amend 命令用于修改最后一次提交，支持以下功能：

1. **修改提交消息**：
   - 交互式输入新消息
   - 支持通过 `--message` 参数直接指定
   - 使用 `--no-edit` 跳过消息编辑

2. **添加文件到提交**：
   - 交互式选择要添加的文件（支持多选）
   - 从修改的文件和未跟踪的文件中选择
   - 自动暂存选中的文件

3. **修改消息并添加文件**：
   - 同时修改消息和添加文件

4. **仅重新提交**：
   - 不修改消息，不添加文件
   - 仅重新执行提交（可用于触发 pre-commit hooks）

5. **预览和确认**：
   - 显示详细的预览信息（原始 SHA、新消息、要添加的文件等）
   - 如果 commit 已推送到远程，显示 force push 警告
   - 最终确认机制

### 关键步骤说明

1. **分支保护检查**：
   - 检查当前分支是否是默认分支（main/master）
   - 默认分支不允许直接修改提交历史
   - 如果是在默认分支，提示用户切换到功能分支

2. **操作类型选择**：
   - 如果提供了 `--no-edit`，使用 `NoEdit` 操作
   - 如果提供了 `--message`，默认使用 `MessageOnly` 操作
   - 否则交互式选择操作类型

3. **文件选择**：
   - 获取所有修改的文件和未跟踪的文件
   - 使用 `MultiSelectDialog` 进行多选
   - 如果未选择文件，询问是否仅修改消息

4. **预览信息生成**：
   - 检查 commit 是否已推送到远程
   - 生成包含原始 SHA、新消息、文件列表等的预览信息
   - 如果已推送，添加 force push 警告

5. **执行 amend**：
   - 暂存选中的文件（如果需要）
   - 调用 `GitCommit::amend()` 执行 amend 操作
   - 处理 pre-commit hooks（通过 `no_verify` 参数控制）

6. **完成提示**：
   - 如果 commit 已推送，显示 force push 提示

---

## 2. Commit Reword 命令 (`reword.rs`)

### 相关文件

```
src/commands/commit/reword.rs (229 行)
```

### 调用流程

```
CommitRewordCommand::execute(commit_id)
  ↓
CheckCommand::run_all() (环境检查)
  ↓
GitBranch::current_branch() (获取当前分支)
  ↓
GitBranch::get_default_branch() (检查是否是默认分支)
  ↓
GitCommit::has_last_commit() (检查是否有最后一次 commit)
  ↓
解析 commit 引用（如果未提供，默认 HEAD）：
  - GitCommit::parse_commit_ref(commit_ref)
  ↓
GitCommit::is_commit_in_current_branch() (验证 commit 是否在当前分支)
  ↓
GitCommit::get_commit_info() (获取 commit 信息)
  ↓
CommitReword::format_commit_info() (格式化显示 commit 信息)
  ↓
ConfirmDialog::prompt() (确认使用此 commit，或重新选择)
  ↓
如果用户选择重新选择：
  - CommitRewordCommand::select_commit_interactively() (交互式选择 commit)
  ↓
CommitRewordCommand::input_new_message_with_confirm() (输入新消息并确认)
  ↓
GitCommit::is_head_commit() (检查是否是 HEAD)
  ↓
CommitReword::create_preview() (创建预览信息)
  ↓
CommitReword::format_preview() (格式化预览信息)
  ↓
ConfirmDialog::prompt() (最终确认)
  ↓
根据是否是 HEAD 执行：
  - HEAD: GitCommit::amend() (使用 amend)
  - 历史 commit: CommitReword::reword_history_commit() (使用 rebase -i)
  ↓
CommitReword::format_completion_message() (生成完成提示)
```

### 功能说明

Commit reword 命令用于修改指定提交的消息，不改变提交内容，支持以下功能：

1. **修改 HEAD commit**：
   - 如果目标是 HEAD，使用 `git commit --amend` 修改
   - 简单快速，不需要 rebase

2. **修改历史 commit**：
   - 如果目标是历史 commit，使用 `git rebase -i` 交互式编辑
   - 自动处理 rebase 流程（创建 todo 文件、编辑器脚本等）
   - 支持自动 stash 未提交的更改

3. **交互式选择 commit**：
   - 支持通过 commit SHA、HEAD~n 等引用指定
   - 如果用户选择重新选择，提供交互式选择界面
   - 显示最近 20 个 commits，支持 fuzzy-matcher 搜索

4. **消息输入和确认**：
   - 交互式输入新消息
   - 支持重新输入（如果用户不满意）
   - 多重确认机制

5. **预览和确认**：
   - 显示详细的预览信息（原始 SHA、新消息、操作类型等）
   - 如果 commit 已推送到远程，显示 force push 警告
   - 最终确认机制

### 关键步骤说明

1. **分支保护检查**：
   - 检查当前分支是否是默认分支
   - 默认分支不允许直接修改提交历史

2. **Commit 引用解析**：
   - 支持 HEAD、HEAD~n、SHA 等引用格式
   - 如果未提供，默认使用 HEAD
   - 验证 commit 是否在当前分支历史中

3. **交互式选择 commit**：
   - 获取最近 20 个 commits
   - 格式化显示（包含 SHA、消息、HEAD 标记）
   - 使用 `SelectDialog` 进行选择（支持 fuzzy-matcher）
   - 从选中的选项解析出 commit SHA

4. **消息输入和确认**：
   - 使用 `InputDialog` 输入新消息（默认值为当前消息）
   - 输入后再次确认
   - 如果用户不满意，可以重新输入

5. **HEAD vs 历史 commit**：
   - 检查目标 commit 是否是 HEAD
   - HEAD：直接使用 `GitCommit::amend()`
   - 历史 commit：使用 `CommitReword::reword_history_commit()`

6. **历史 commit reword（rebase）**：
   - 自动 stash 未提交的更改（如果启用）
   - 找到目标 commit 的父 commit（rebase 起点）
   - 获取从父 commit 到 HEAD 的所有 commits
   - 创建 rebase todo 文件（将目标 commit 标记为 `reword`）
   - 创建编辑器脚本（自动编辑 todo 文件和消息文件）
   - 执行 rebase
   - 处理冲突（如果发生）
   - 恢复 stash（如果之前 stash 了）

7. **预览信息生成**：
   - 检查 commit 是否已推送到远程
   - 生成包含原始 SHA、新消息、操作类型等的预览信息
   - 如果已推送，添加 force push 警告

8. **完成提示**：
   - 如果 commit 已推送，显示 force push 提示

---

## 🏗️ 架构设计

### 设计模式

#### 1. 分层架构

命令层专注于用户交互和参数解析，业务逻辑层处理核心功能，Git 层提供底层操作。

**优势**：
- 职责清晰，易于维护
- 业务逻辑可复用
- 便于测试

#### 2. 交互式工作流

提供完整的交互式流程，包括选择、输入、确认等步骤。

**优势**：
- 用户体验友好
- 减少误操作
- 提供预览和确认机制

#### 3. 预览机制

在执行操作前生成预览信息，让用户了解将要执行的操作。

**优势**：
- 提高操作安全性
- 减少误操作
- 提供清晰的操作反馈

### 错误处理

#### 分层错误处理

1. **CLI 层**：参数解析错误、命令不存在等
2. **命令层**：用户取消操作、输入验证失败等
3. **库层**：Git 操作失败、文件系统错误等

#### 容错机制

- **分支保护**：检查是否是默认分支，防止误操作
- **Commit 验证**：验证 commit 是否存在、是否在当前分支等
- **工作区状态检查**：检查是否有未提交的更改，自动处理 stash
- **Rebase 冲突处理**：检测 rebase 冲突，提供解决指导

---

## 📋 使用示例

### Commit Amend 命令

```bash
# 交互式 amend（选择操作类型）
workflow commit amend

# 仅修改提交消息
workflow commit amend --message "New commit message"

# 不编辑消息，仅重新提交（可用于触发 hooks）
workflow commit amend --no-edit

# 跳过 pre-commit hooks
workflow commit amend --no-verify
```

### Commit Reword 命令

```bash
# 修改 HEAD commit（默认）
workflow commit reword

# 修改 HEAD commit（显式指定）
workflow commit reword HEAD

# 修改倒数第二个 commit
workflow commit reword HEAD~2

# 修改指定 SHA 的 commit
workflow commit reword abc1234
```

---

## 📝 扩展性（可选）

### 添加新命令

1. 在 `src/lib/cli/commit.rs` 中添加新的 `CommitSubcommand` 变体
2. 在 `src/commands/commit/mod.rs` 中声明新命令模块
3. 创建新命令文件（如 `src/commands/commit/new_command.rs`）
4. 在 `src/bin/workflow.rs` 中添加命令分发逻辑
5. 在 `lib/commit/` 中添加相应的业务逻辑（如果需要）

**示例**：
```rust
// src/commands/commit/new_command.rs
pub struct CommitNewCommand;

impl CommitNewCommand {
    pub fn execute() -> Result<()> {
        // 实现
    }
}
```

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Commit 模块架构文档](../lib/COMMIT_ARCHITECTURE.md) - Lib 层模块
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md) - Git 操作模块

---

## ✅ 总结

Commit 命令模块采用清晰的分层架构设计：

1. **职责分离**：命令层处理交互，业务逻辑层处理核心功能
2. **交互式工作流**：提供完整的交互式流程，包括选择、输入、确认等
3. **预览机制**：在执行操作前生成预览信息，提高操作安全性
4. **分支保护**：检查默认分支，防止误操作
5. **智能处理**：自动处理 stash、冲突等场景

**设计优势**：
- ✅ 职责清晰，易于维护
- ✅ 用户体验友好，减少误操作
- ✅ 提供预览和确认机制，提高安全性
- ✅ 支持多种操作场景（amend、reword、HEAD、历史 commit）

**当前实现状态**：
- ✅ Commit amend 功能完整实现
- ✅ Commit reword 功能完整实现（支持 HEAD 和历史 commit）
- ✅ 交互式工作流完整
- ✅ 预览和确认机制完整
- ✅ 分支保护机制完整
