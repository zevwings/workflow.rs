# Commit 模块架构文档

## 📋 概述

Commit 模块（`lib/commit/`）是 Workflow CLI 的核心库模块，提供 Commit 相关的业务逻辑，包括 amend 和 reword 操作的预览信息生成、格式化显示、完成提示生成以及历史 commit reword 的执行逻辑。

**注意**：本文档仅描述 `lib/commit/` 模块的架构。关于 Commit 命令层的详细内容，请参考 [Commit 命令模块架构文档](../commands/COMMIT_COMMAND_ARCHITECTURE.md)。

**模块统计：**
- 总代码行数：约 645 行
- 文件数量：3 个
- 主要组件：`CommitAmend`、`CommitReword`、`AmendPreview`、`RewordPreview`、`RewordHistoryOptions`、`RewordHistoryResult`
- 支持功能：预览信息生成、格式化显示、完成提示生成、历史 commit reword（rebase 交互式编辑）

---

## 📁 模块结构

### 核心模块文件

```
src/lib/commit/
├── mod.rs          # Commit 模块声明和导出 (13行)
├── amend.rs        # Commit Amend 业务逻辑 (189行)
└── reword.rs       # Commit Reword 业务逻辑 (443行)
```

### 依赖模块

- **`lib/git/`**：Git 操作（获取分支信息、commit 信息、工作区状态等）
  - `GitBranch::current_branch()` - 获取当前分支
  - `GitBranch::is_commit_in_remote()` - 检查 commit 是否在远程
  - `GitCommit::get_worktree_status()` - 获取工作区状态
  - `GitCommit::format_worktree_status()` - 格式化工作区状态
  - `GitCommit::get_parent_commit()` - 获取父 commit
  - `GitCommit::get_commits_from_to_head()` - 获取从指定 commit 到 HEAD 的所有 commits
- **`lib/git/stash.rs`**：Git stash 操作（历史 commit reword 时自动 stash）
  - `GitStash::stash_push()` - 暂存更改
  - `GitStash::stash_pop()` - 恢复暂存

### 模块集成

- **`commands/commit/`**：Commit 命令层
  - `amend.rs` - 使用 `CommitAmend` 进行预览、格式化、完成提示
  - `reword.rs` - 使用 `CommitReword` 进行预览、格式化、完成提示、历史 commit reword

---

## 🏗️ 架构设计

### 设计原则

1. **职责单一**：每个组件专注于特定功能（预览、格式化、执行等）
2. **无状态设计**：所有方法都是静态方法，不维护状态
3. **错误传播**：使用 `Result` 类型传播错误，由调用层处理
4. **可测试性**：业务逻辑与 Git 操作分离，便于单元测试

### 核心组件

#### 1. Commit Amend 业务逻辑 (`amend.rs`)

**职责**：提供 amend 操作相关的业务逻辑

**主要方法**：
- `create_preview()` - 创建 amend 预览信息
- `format_preview()` - 格式化预览信息为字符串
- `format_commit_info_detailed()` - 格式化详细 commit 信息（包含工作区状态）
- `should_show_force_push_warning()` - 检查是否需要显示 force push 警告
- `format_completion_message()` - 生成完成提示信息

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
- `format_commit_info()` - 格式化 commit 信息为字符串
- `create_preview()` - 创建 reword 预览信息
- `format_preview()` - 格式化预览信息为字符串
- `should_show_force_push_warning()` - 检查是否需要显示 force push 警告
- `format_completion_message()` - 生成完成提示信息
- `reword_history_commit()` - 执行历史 commit reword（核心业务逻辑）

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
- `original_sha` - 原始 commit SHA
- `new_message` - 新提交消息（可选）
- `original_message` - 原始提交消息
- `files_to_add` - 要添加的文件列表
- `operation_type` - 操作类型
- `is_pushed` - 是否已推送到远程

**`RewordPreview`**：
- `original_sha` - 原始 commit SHA
- `original_message` - 原始提交消息
- `new_message` - 新提交消息
- `is_head` - 是否是 HEAD
- `is_pushed` - 是否已推送到远程

**`RewordHistoryOptions`**：
- `commit_sha` - 要修改的 commit SHA
- `new_message` - 新的提交消息
- `auto_stash` - 是否自动 stash

**`RewordHistoryResult`**：
- `success` - 是否成功
- `has_conflicts` - 是否有冲突
- `was_stashed` - 是否进行了 stash

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

## 🔄 调用流程与数据流

### 整体架构流程

```
命令层 (commands/commit/)
  ↓
业务逻辑层 (lib/commit/)
  ↓
Git 操作层 (lib/git/)
  ↓
Git 仓库操作
```

### 典型调用示例

#### 1. Amend 预览信息生成

```
CommitAmendCommand::execute()
  ↓
CommitAmend::create_preview()
  ↓
GitBranch::is_commit_in_remote() (检查是否已推送)
  ↓
返回 AmendPreview
  ↓
CommitAmend::format_preview() (格式化输出)
```

#### 2. Reword 历史 Commit

```
CommitRewordCommand::execute()
  ↓
CommitReword::reword_history_commit()
  ↓
检查工作区状态
  ↓
GitStash::stash_push() (如果需要)
  ↓
GitCommit::get_parent_commit() (获取父 commit)
  ↓
GitCommit::get_commits_from_to_head() (获取 commits 列表)
  ↓
创建 rebase todo 文件
  ↓
创建编辑器脚本
  ↓
执行 rebase (git rebase -i)
  ↓
GitStash::stash_pop() (恢复 stash)
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

### 基本使用

```rust
use workflow::commit::{CommitAmend, AmendPreview};

// 创建 amend 预览信息
let preview = CommitAmend::create_preview(
    &commit_info,
    &Some("New message".to_string()),
    &vec!["file1.rs".to_string()],
    "Modify message and add files",
    "feature/branch",
)?;

// 格式化预览信息
let formatted = CommitAmend::format_preview(&preview);
println!("{}", formatted);
```

### Reword 历史 Commit

```rust
use workflow::commit::{CommitReword, RewordHistoryOptions};

// 执行历史 commit reword
let options = RewordHistoryOptions {
    commit_sha: "abc1234".to_string(),
    new_message: "New commit message".to_string(),
    auto_stash: true,
};

let result = CommitReword::reword_history_commit(options)?;

if result.has_conflicts {
    // 处理冲突
} else {
    // 成功
}
```

---

## 📝 扩展性（可选）

### 添加新功能

1. 在相应的模块文件中添加新的方法
2. 如果需要新的数据结构，定义新的结构体
3. 在命令层调用新方法

**示例**：
```rust
// src/lib/commit/amend.rs
impl CommitAmend {
    pub fn new_feature() -> Result<()> {
        // 实现
    }
}
```

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Commit 命令模块架构文档](../commands/COMMIT_COMMAND_ARCHITECTURE.md) - 命令层模块
- [Git 模块架构文档](./GIT_ARCHITECTURE.md) - Git 操作模块

---

## ✅ 总结

Commit 模块采用清晰的无状态设计：

1. **职责单一**：每个组件专注于特定功能
2. **无状态设计**：所有方法都是静态方法，不维护状态
3. **策略模式**：根据场景选择不同的执行策略（amend vs rebase）
4. **预览机制**：在执行操作前生成预览信息
5. **自动化处理**：自动处理 stash、临时文件、冲突等场景

**设计优势**：
- ✅ 职责清晰，易于维护
- ✅ 无状态设计，线程安全
- ✅ 可测试性强，业务逻辑与 Git 操作分离
- ✅ 自动化处理，减少用户操作
- ✅ 错误处理完善，提供清晰的错误信息

**当前实现状态**：
- ✅ Amend 业务逻辑完整实现
- ✅ Reword 业务逻辑完整实现（支持 HEAD 和历史 commit）
- ✅ 预览信息生成完整
- ✅ 格式化显示完整
- ✅ 历史 commit reword（rebase）完整实现
- ✅ 临时文件管理完整
- ✅ 冲突检测和处理完整

---

**最后更新**: 2025-12-16
