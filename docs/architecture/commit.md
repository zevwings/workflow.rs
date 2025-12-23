# Commit 模块架构文档

## 📋 概述

Commit 模块是 Workflow CLI 的核心模块，提供完整的提交修改和管理功能。该模块采用分层架构设计，包括：

- **Lib 层**（`lib/commit/`）：提供 Commit 相关的业务逻辑，包括 amend 和 reword 操作的预览信息生成、格式化显示、完成提示生成以及历史 commit reword 的执行逻辑
- **Commands 层**（`commands/commit/`）：提供 CLI 命令封装，处理用户交互，包括 commit amend、reword 和 squash 功能

Commit 模块支持修改最后一次提交（amend）、修改历史提交（reword）和压缩多个提交（squash）等功能。

**模块统计：**
- Lib 层代码行数：约 645 行
- Commands 层代码行数：约 776 行
- 命令数量：3 个（amend、reword、squash）
- 文件数量：Lib 层 3 个，Commands 层 5 个
- 主要组件：`CommitAmend`、`CommitReword`、`CommitSquash`、`AmendPreview`、`RewordPreview`、`RewordHistoryOptions`、`RewordHistoryResult`
- 支持功能：预览信息生成、格式化显示、完成提示生成、历史 commit reword（rebase 交互式编辑）

---

## 📁 Lib 层架构（核心业务逻辑）

Commit 模块（`lib/commit/`）是 Workflow CLI 的核心库模块，提供 Commit 相关的业务逻辑，包括 amend 和 reword 操作的预览信息生成、格式化显示、完成提示生成以及历史 commit reword 的执行逻辑。

### 模块结构

```
src/lib/commit/
├── mod.rs          # Commit 模块声明和导出 (13行)
├── amend.rs        # Commit Amend 业务逻辑 (189行)
└── reword.rs       # Commit Reword 业务逻辑 (443行)
```

### 依赖模块

- **`lib/git/`**：Git 操作（获取分支信息、commit 信息、工作区状态等）
  - `GitBranch::current-_branch()` - 获取当前分支
  - `GitBranch::is-_commit-_in-_remote()` - 检查 commit 是否在远程
  - `GitCommit::get-_worktree-_status()` - 获取工作区状态
  - `GitCommit::format-_worktree-_status()` - 格式化工作区状态
  - `GitCommit::get-_parent-_commit()` - 获取父 commit
  - `GitCommit::get-_commits-_from-_to-_head()` - 获取从指定 commit 到 HEAD 的所有 commits
- **`lib/git/stash.rs`**：Git stash 操作（历史 commit reword 时自动 stash）
  - `GitStash::stash-_push()` - 暂存更改
  - `GitStash::stash-_pop()` - 恢复暂存

### 模块集成

- **`commands/commit/`**：Commit 命令层
  - `amend.rs` - 使用 `CommitAmend` 进行预览、格式化、完成提示
  - `reword.rs` - 使用 `CommitReword` 进行预览、格式化、完成提示、历史 commit reword
  - `squash.rs` - 使用 `CommitSquash` 进行预览、格式化、完成提示、执行 squash

---

## 🏗️ Lib 层架构设计

### 设计原则

1. **职责单一**：每个组件专注于特定功能（预览、格式化、执行等）
2. **无状态设计**：所有方法都是静态方法，不维护状态
3. **错误传播**：使用 `Result` 类型传播错误，由调用层处理
4. **可测试性**：业务逻辑与 Git 操作分离，便于单元测试

### 核心组件

#### 1. Commit Amend 业务逻辑 (`amend.rs`)

**职责**：提供 amend 操作相关的业务逻辑

**主要方法**：
- `create-_preview()` - 创建 amend 预览信息
- `format-_preview()` - 格式化预览信息为字符串
- `format-_commit-_info-_detailed()` - 格式化详细 commit 信息（包含工作区状态）
- `should-_show-_force-_push-_warning()` - 检查是否需要显示 force push 警告
- `format-_completion-_message()` - 生成完成提示信息

**关键特性**：
- 预览信息包含原始 SHA、新消息、文件列表、操作类型等
- 自动检测 commit 是否已推送到远程
- 如果已推送，在预览和完成提示中显示 force push 警告
- 格式化输出使用统一的样式（分隔线、对齐等）

**使用场景**：
- `commit amend` 命令：生成预览信息、格式化显示、生成完成提示

#### 2. Commit Reword 业务逻辑 (`reword.rs`)

**职责**：提供 reword 操作相关的业务逻辑

**主要方法**：
- `format-_commit-_info()` - 格式化 commit 信息为字符串
- `create-_preview()` - 创建 reword 预览信息
- `format-_preview()` - 格式化预览信息为字符串
- `should-_show-_force-_push-_warning()` - 检查是否需要显示 force push 警告
- `format-_completion-_message()` - 生成完成提示信息
- `reword-_history-_commit()` - 执行历史 commit reword（核心业务逻辑）

**关键特性**：
- 预览信息包含原始 SHA、新消息、操作类型（HEAD amend 或历史 rebase）等
- 自动检测 commit 是否已推送到远程
- 历史 commit reword 使用 rebase 交互式编辑
- 自动处理 stash、rebase todo 文件、编辑器脚本等
- 支持冲突检测和处理

**使用场景**：
- `commit reword` 命令：生成预览信息、格式化显示、生成完成提示、执行历史 commit reword

#### 3. 数据结构

**`AmendPreview`**：
- `original-_sha` - 原始 commit SHA
- `new-_message` - 新提交消息（可选）
- `original-_message` - 原始提交消息
- `files-_to-_add` - 要添加的文件列表
- `operation-_type` - 操作类型
- `is-_pushed` - 是否已推送到远程

**`RewordPreview`**：
- `original-_sha` - 原始 commit SHA
- `original-_message` - 原始提交消息
- `new-_message` - 新提交消息
- `is-_head` - 是否是 HEAD
- `is-_pushed` - 是否已推送到远程

**`RewordHistoryOptions`**：
- `commit-_sha` - 要修改的 commit SHA
- `new-_message` - 新的提交消息
- `auto-_stash` - 是否自动 stash

**`RewordHistoryResult`**：
- `success` - 是否成功
- `has-_conflicts` - 是否有冲突
- `was-_stashed` - 是否进行了 stash

### 设计模式

#### 1. 策略模式

历史 commit reword 使用不同的策略：
- HEAD commit：使用 `git commit --amend`
- 历史 commit：使用 `git rebase -i`

**优势**：
- 根据场景选择最优策略
- 代码清晰，易于维护

#### 2. 模板方法模式

预览信息生成遵循固定流程：
1. 收集信息（commit 信息、分支信息等）
2. 检查是否已推送
3. 生成预览数据结构
4. 格式化输出

**优势**：
- 流程统一，易于扩展
- 便于添加新的预览信息字段

#### 3. 临时文件管理

历史 commit reword 使用临时文件：
- Rebase todo 文件
- Commit 消息文件
- 编辑器脚本文件

**优势**：
- 自动化 rebase 流程
- 避免手动交互
- 清理机制确保不留下临时文件

### 错误处理

#### 分层错误处理

1. **业务逻辑层**：参数验证、状态检查等
2. **Git 操作层**：Git 命令执行失败、文件系统错误等

#### 容错机制

- **Rebase 冲突检测**：检测 rebase 过程中的冲突，提供解决指导
- **Stash 恢复**：如果 rebase 失败，自动恢复 stash
- **临时文件清理**：无论成功或失败，都清理临时文件
- **根 commit 检查**：检查是否是根 commit（无父 commit），无法 rebase

---

## 📁 Commands 层架构（命令封装）

Commit 命令模块提供交互式的提交修改功能，支持修改最后一次提交（amend）、修改历史提交（reword）和压缩多个提交（squash）。

> **架构说明**：本模块遵循项目的三层架构设计，详见 [architecture.md](./architecture.md#三层架构设计)

### 相关文件

#### CLI 入口层

```
src/bin/workflow.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow commit` 子命令分发到对应的命令处理函数

#### 命令封装层

```
src/commands/commit/
├── mod.rs          # Commit 命令模块声明（11 行）
├── amend.rs        # Commit amend 命令（240 行）
├── reword.rs       # Commit reword 命令（229 行）
└── squash.rs       # Commit squash 命令（199 行）
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
  - `CommitAmend::create-_preview()` - 创建 amend 预览信息
  - `CommitAmend::format-_preview()` - 格式化预览信息
  - `CommitAmend::format-_commit-_info-_detailed()` - 格式化详细 commit 信息
  - `CommitAmend::format-_completion-_message()` - 生成完成提示信息
  - `CommitReword::create-_preview()` - 创建 reword 预览信息
  - `CommitReword::format-_preview()` - 格式化预览信息
  - `CommitReword::format-_commit-_info()` - 格式化 commit 信息
  - `CommitReword::reword-_history-_commit()` - 执行历史 commit reword
  - `CommitSquash::get-_branch-_commits()` - 获取当前分支创建之后的提交
  - `CommitSquash::create-_preview()` - 创建 squash 预览信息
  - `CommitSquash::format-_preview()` - 格式化预览信息
  - `CommitSquash::execute-_squash()` - 执行 squash 操作
  - `CommitSquash::should-_show-_force-_push-_warning()` - 检查是否需要显示 force push 警告
  - `CommitSquash::format-_completion-_message()` - 生成完成提示信息
- **`lib/git/`**：Git 操作
  - `GitBranch::current-_branch()` - 获取当前分支
  - `GitBranch::get-_default-_branch()` - 获取默认分支
  - `GitBranch::is-_commit-_in-_remote()` - 检查 commit 是否在远程
  - `GitCommit::has-_last-_commit()` - 检查是否有最后一次 commit
  - `GitCommit::get-_last-_commit-_info()` - 获取最后一次 commit 信息
  - `GitCommit::get-_commit-_info()` - 获取指定 commit 信息
  - `GitCommit::get-_worktree-_status()` - 获取工作区状态
  - `GitCommit::get-_modified-_files()` - 获取修改的文件
  - `GitCommit::get-_untracked-_files()` - 获取未跟踪的文件
  - `GitCommit::get-_branch-_commits()` - 获取分支 commits
  - `GitCommit::parse-_commit-_ref()` - 解析 commit 引用
  - `GitCommit::is-_commit-_in-_current-_branch()` - 检查 commit 是否在当前分支
  - `GitCommit::is-_head-_commit()` - 检查是否是 HEAD commit
  - `GitCommit::has-_commit()` - 检查是否有未提交的更改
  - `GitCommit::add-_all()` - 暂存所有文件
  - `GitCommit::add-_files()` - 暂存指定文件
  - `GitCommit::amend()` - 执行 amend 操作
- **`commands/check/`**：环境检查
  - `CheckCommand::run-_all()` - 运行所有检查
- **`lib/base/dialog/`**：用户交互对话框
  - `ConfirmDialog` - 确认对话框
  - `InputDialog` - 输入对话框
  - `SelectDialog` - 选择对话框
  - `MultiSelectDialog` - 多选对话框

---

## 🔄 集成关系

### Lib 层和 Commands 层的协作

Commit 模块采用清晰的分层架构，Lib 层和 Commands 层通过以下方式协作：

1. **预览信息生成**：Commands 层调用 Lib 层的 `create-_preview()` 方法生成预览信息
2. **格式化显示**：Commands 层调用 Lib 层的 `format-_preview()` 方法格式化输出
3. **执行操作**：Commands 层调用 Lib 层的执行方法（如 `reword-_history-_commit()`）执行实际操作
4. **完成提示**：Commands 层调用 Lib 层的 `format-_completion-_message()` 生成完成提示

### 调用流程

#### 整体架构流程

```
命令层 (commands/commit/)
  ↓
业务逻辑层 (lib/commit/)
  ↓
Git 操作层 (lib/git/)
  ↓
Git 仓库操作
```

#### 命令分发流程

```
src/bin/workflow.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.command {
  Commands::Commit { subcommand } => match subcommand {
    CommitSubcommand::Amend { message, no-_edit, no-_verify } =>
      CommitAmendCommand::execute(message, no-_edit, no-_verify)
    CommitSubcommand::Reword { commit-_id } =>
      CommitRewordCommand::execute(commit-_id)
    CommitSubcommand::Squash =>
      CommitSquashCommand::execute()
  }
}
```

---

## 📋 Commands 层命令详情

### 1. Commit Amend 命令 (`amend.rs`)

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

6. **自动推送确认**：
   - 如果 commit 已推送到远程，amend 完成后会询问是否要 force push
   - 使用 `--force-with-lease` 安全地强制推送
   - 用户可以选择跳过推送，稍后手动执行

### 2. Commit Reword 命令 (`reword.rs`)

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

6. **自动推送确认**：
   - 如果 commit 已推送到远程，reword 完成后会询问是否要 force push
   - 使用 `--force-with-lease` 安全地强制推送
   - 用户可以选择跳过推送，稍后手动执行

### 3. Commit Squash 命令 (`squash.rs`)

Commit squash 命令用于将多个提交压缩为一个提交，支持以下功能：

1. **获取分支提交**：
   - 自动检测当前分支基于哪个分支创建
   - 获取当前分支创建之后的所有提交
   - 只显示可以压缩的提交（排除已合并到基础分支的提交）

2. **交互式多选**：
   - 显示所有可用的 commits（包含 SHA 和消息）
   - 使用 `MultiSelectDialog` 进行多选
   - 标记最旧的提交（`[OLDEST]`）
   - 支持选择多个连续的或不连续的提交

3. **新消息输入**：
   - 交互式输入新的提交消息
   - 验证消息不能为空
   - 支持重新输入（如果用户不满意）

4. **预览和确认**：
   - 显示详细的预览信息（要压缩的 commits、新消息、基础 commit 等）
   - 如果 commits 已推送到远程，显示 force push 警告
   - 最终确认机制

5. **执行 squash**：
   - 使用 `git rebase -i` 进行交互式 rebase
   - 自动创建 rebase todo 文件（将选中的 commits 标记为 `squash`）
   - 自动创建编辑器脚本（自动编辑 todo 文件和消息文件）
   - 自动处理 stash（如果有未提交的更改）
   - 支持冲突检测和处理

6. **自动推送确认**：
   - 如果 commits 已推送到远程，squash 完成后会询问是否要 force push
   - 使用 `--force-with-lease` 安全地强制推送
   - 用户可以选择跳过推送，稍后手动执行

---

## 🔄 调用流程与数据流

### 典型调用示例

#### 1. Amend 预览信息生成

```
CommitAmendCommand::execute()
  ↓
CommitAmend::create-_preview()
  ↓
GitBranch::is-_commit-_in-_remote() (检查是否已推送)
  ↓
返回 AmendPreview
  ↓
CommitAmend::format-_preview() (格式化输出)
```

#### 2. Reword 历史 Commit

```
CommitRewordCommand::execute()
  ↓
CommitReword::reword-_history-_commit()
  ↓
检查工作区状态
  ↓
GitStash::stash-_push() (如果需要)
  ↓
GitCommit::get-_parent-_commit() (获取父 commit)
  ↓
GitCommit::get-_commits-_from-_to-_head() (获取 commits 列表)
  ↓
创建 rebase todo 文件
  ↓
创建编辑器脚本
  ↓
执行 rebase (git rebase -i)
  ↓
GitStash::stash-_pop() (恢复 stash)
  ↓
返回 RewordHistoryResult
```

### 数据流

```
用户输入（消息、文件等）
  ↓
命令层收集参数
  ↓
业务逻辑层生成预览数据结构
  ↓
格式化输出（字符串）
  ↓
用户确认
  ↓
执行 Git 操作
  ↓
生成完成提示
```

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

### Commit Squash 命令

```bash
# 交互式 squash（选择要压缩的 commits）
workflow commit squash
```

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

## 📝 扩展性

### 添加新功能

1. 在相应的模块文件中添加新的方法
2. 如果需要新的数据结构，定义新的结构体
3. 在命令层调用新方法

### 添加新命令

1. 在 `src/lib/cli/commit.rs` 中添加新的 `CommitSubcommand` 变体
2. 在 `src/commands/commit/mod.rs` 中声明新命令模块
3. 创建新命令文件（如 `src/commands/commit/new-_command.rs`）
4. 在 `src/bin/workflow.rs` 中添加命令分发逻辑
5. 在 `lib/commit/` 中添加相应的业务逻辑（如果需要）

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Git 模块架构文档](./git.md) - Git 操作模块

---

## ✅ 总结

Commit 模块采用清晰的分层架构和无状态设计：

1. **职责单一**：每个组件专注于特定功能
2. **无状态设计**：所有方法都是静态方法，不维护状态
3. **策略模式**：根据场景选择不同的执行策略（amend vs rebase）
4. **预览机制**：在执行操作前生成预览信息
5. **自动化处理**：自动处理 stash、临时文件、冲突等场景
6. **交互式工作流**：提供完整的交互式流程，包括选择、输入、确认等
7. **分支保护**：检查默认分支，防止误操作

**设计优势**：
- ✅ 职责清晰，易于维护
- ✅ 无状态设计，线程安全
- ✅ 可测试性强，业务逻辑与 Git 操作分离
- ✅ 自动化处理，减少用户操作
- ✅ 错误处理完善，提供清晰的错误信息
- ✅ 用户体验友好，减少误操作
- ✅ 提供预览和确认机制，提高安全性

**当前实现状态**：
- ✅ Amend 业务逻辑完整实现
- ✅ Reword 业务逻辑完整实现（支持 HEAD 和历史 commit）
- ✅ Squash 业务逻辑完整实现（支持多选、预览、确认）
- ✅ 预览信息生成完整
- ✅ 格式化显示完整
- ✅ 历史 commit reword（rebase）完整实现
- ✅ 临时文件管理完整
- ✅ 冲突检测和处理完整

---

**最后更新**: 2025-12-16

