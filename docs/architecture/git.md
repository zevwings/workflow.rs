# Git 模块架构文档

## 📋 概述

Git 模块是 Workflow CLI 的核心功能之一，提供完整的 Git 仓库操作功能，包括提交管理、分支管理、仓库检测、暂存管理、Pre-commit hooks 支持和配置管理。该模块采用模块化设计，每个功能领域有独立的结构体，通过统一的辅助函数减少代码重复。

**模块统计：**
- 总代码行数：约 1448 行
- 文件数量：9 个
- 主要结构体：7 个（GitBranch, GitCommit, GitRepo, GitStash, GitConfig, GitPreCommit, GitCherryPick）
- 辅助模块：1 个（helpers.rs）

---

## 📁 模块结构

### 核心模块文件

```
src/lib/git/
├── mod.rs          # Git 模块声明和导出 (40行)
├── branch.rs       # 分支管理操作 (608行)
├── commit.rs       # 提交相关操作 (172行)
├── repo.rs         # 仓库检测和类型识别 (203行)
├── stash.rs        # 暂存管理 (102行)
├── config.rs       # Git 配置管理 (67行)
├── pre_commit.rs   # Pre-commit hooks 支持 (107行)
├── cherry_pick.rs  # Cherry-pick 操作 (98行)
├── helpers.rs      # Git 操作辅助函数 (115行)
└── types.rs        # 类型定义 (15行)
```

### 依赖模块

- **`duct`**：命令执行库（执行 Git 命令）
- **`lib/base/util/`**：工具函数（日志输出等）

### 模块集成

- **PR 模块集成** (`lib/pr/`)：
  - `GitBranch::checkout_branch()` - 创建或切换分支
  - `GitCommit::commit()` - 提交更改
  - `GitBranch::push()` - 推送到远程
  - `GitRepo::detect_repo_type()` - 检测仓库类型（用于工厂函数）
  - `GitBranch::merge_branch()` - 合并分支
  - `GitStash::stash_push()` / `stash_pop()` - 保存/恢复工作区更改

- **配置管理集成**：
  - `GitConfig::set_global_user()` - 设置 Git 全局配置
  - 用于初始化设置和 GitHub 账号切换

- **环境检查集成** (`commands/check/`)：
  - `GitRepo::is_git_repo()` - 检查是否在 Git 仓库中
  - `GitCommit::status()` - 检查 Git 状态

- **分支管理集成** (`commands/branch/`)：
  - `GitBranch::get_all_branches()` - 获取所有分支
  - `GitBranch::is_merged()` - 检查分支是否已合并
  - `GitRepo::extract_repo_name()` - 提取仓库名（用于配置分组）
  - `GitRepo::prune_remote()` - 清理远程分支引用

---

## 🏗️ 架构设计

### 设计原则

1. **模块化设计**：每个功能领域有独立的结构体，职责清晰
2. **零大小结构体**：使用 unit struct 组织相关函数，符合 Rust 最佳实践
3. **统一辅助函数**：通过 `helpers.rs` 提供统一的 Git 命令执行接口
4. **错误处理统一**：使用 `anyhow::Result` 和 `context` 提供清晰的错误信息
5. **类型安全**：使用枚举类型（`RepoType`, `MergeStrategy`）提高类型安全性

### 核心组件

#### 1. 分支管理 (`branch.rs`)

**职责**：提供分支相关的所有操作

- **`GitBranch`**：分支管理结构体（零大小结构体）

**主要方法**：
- `current_branch()` - 获取当前分支名
- `is_branch_exists()` - 检查分支是否存在（本地或远程）
- `has_local_branch()` - 检查本地分支是否存在
- `has_remote_branch()` - 检查远程分支是否存在
- `checkout_branch()` - 创建或切换到分支
- `get_default_branch()` - 获取默认分支
- `get_all_branches()` - 获取所有分支（本地和远程）
- `extract_base_branch_names()` - 提取分支基础名称（去掉前缀）
- `is_branch_ahead()` - 检查分支是否领先于指定分支
- `pull()` - 从远程拉取分支
- `push()` - 推送到远程仓库
- `delete()` - 删除本地分支
- `delete_remote()` - 删除远程分支
- `merge_branch()` - 合并分支
- `has_merge_conflicts()` - 检查是否有合并冲突

**关键特性**：
- 支持 `git switch` 和 `git checkout` 的自动回退
- 多种合并策略（Merge, Squash, FastForwardOnly）
- 智能的默认分支检测（支持多种方法）

**使用场景**：
- PR 创建时创建和切换分支
- PR 合并时合并分支和清理
- 分支列表查询
- 分支清理操作

#### 2. 提交管理 (`commit.rs`)

**职责**：提供提交相关的操作

- **`GitCommit`**：提交管理结构体（零大小结构体）

**主要方法**：
- `status()` - 检查 Git 状态
- `has_commit()` - 检查是否有未提交的更改
- `has_staged()` - 检查是否有暂存的文件
- `add_all()` - 添加所有文件到暂存区
- `commit()` - 提交更改（支持 pre-commit hooks）
- `get_diff()` - 获取 Git 修改内容（工作区和暂存区）

**关键特性**：
- 自动暂存所有更改
- 集成 pre-commit hooks 支持
- 提供 diff 内容用于 LLM 生成

**使用场景**：
- PR 创建时提交更改
- PR 更新时提交更改
- 环境检查时检查状态

#### 3. 仓库检测 (`repo.rs`)

**职责**：提供仓库检测和类型识别

- **`GitRepo`**：仓库检测结构体（零大小结构体）

**主要方法**：
- `is_git_repo()` - 检查是否在 Git 仓库中
- `detect_repo_type()` - 检测远程仓库类型（GitHub、Codeup 等）
- `get_remote_url()` - 获取远程仓库 URL
- `get_git_dir()` - 获取 Git 目录路径
- `fetch()` - 从远程获取更新
- `prune_remote()` - 清理远程分支引用

**关键特性**：
- 支持 GitHub 和 Codeup 仓库类型识别
- 支持 SSH Host 别名识别

**使用场景**：
- PR 操作前检测仓库类型
- 环境检查时验证 Git 仓库
- 自动识别平台类型

#### 4. 暂存管理 (`stash.rs`)

**职责**：提供 stash 相关操作

- **`GitStash`**：暂存管理结构体（零大小结构体）

**主要方法**：
- `stash_push()` - 保存未提交的修改到 stash
- `stash_pop()` - 恢复 stash 中的修改
- `has_unmerged()` - 检查是否有未合并的文件（冲突）

**关键特性**：
- 自动检测合并冲突
- 提供详细的冲突解决提示

**使用场景**：
- PR 集成分支时保存工作区更改
- 切换分支前保存更改

#### 5. 配置管理 (`config.rs`)

**职责**：提供 Git 配置管理

- **`GitConfig`**：配置管理结构体（零大小结构体）

**主要方法**：
- `set_global_user()` - 设置 Git 全局配置（email 和 name）
- `get_global_user()` - 读取 Git 全局配置

**使用场景**：
- 初始化设置时配置 Git 用户信息
- GitHub 账号切换时更新配置

#### 6. Pre-commit Hooks (`pre_commit.rs`)

**职责**：提供 pre-commit hooks 支持

- **`GitPreCommit`**：Pre-commit hooks 结构体（零大小结构体）

**主要方法**：
- `has_pre_commit()` - 检查是否存在 pre-commit hooks
- `run_pre_commit()` - 执行 pre-commit hooks

**关键特性**：
- 支持 Git hooks 和 pre-commit 工具
- 自动检测多种 pre-commit 配置方式

**使用场景**：
- 提交前自动执行 hooks
- 支持代码质量检查

#### 7. Cherry-pick 操作 (`cherry_pick.rs`)

**职责**：提供 Git cherry-pick 相关的完整功能

- **`GitCherryPick`**：Cherry-pick 管理结构体（零大小结构体）

**主要方法**：
- `cherry_pick(commit)` - Cherry-pick 提交到当前分支
- `cherry_pick_no_commit(commit)` - Cherry-pick 但不提交（保留在工作区）
- `cherry_pick_continue()` - 继续 cherry-pick 操作
- `cherry_pick_abort()` - 中止 cherry-pick 操作
- `is_cherry_pick_in_progress()` - 检查是否正在进行 cherry-pick 操作

**关键特性**：
- 支持普通 cherry-pick 和 no-commit 模式
- 支持继续和中止操作
- 自动检测 cherry-pick 状态

**使用场景**：
- PR pick 命令：从源 PR 提取提交并应用到新分支
- 提交迁移：将提交从一个分支应用到另一个分支
- 冲突处理：检测和处理 cherry-pick 冲突

**注意**：
- 如果遇到冲突，cherry-pick 会暂停，需要用户手动解决冲突后继续
- `cherry_pick_no_commit()` 会将修改保留在工作区，需要手动提交

#### 8. 辅助函数 (`helpers.rs`)

**职责**：提供通用的 Git 命令执行辅助函数

**主要函数**：
- `cmd_read()` - 执行 Git 命令并读取输出
- `cmd_run()` - 执行 Git 命令（不读取输出）
- `check_success()` - 静默执行并检查是否成功
- `check_ref_exists()` - 检查 Git 引用是否存在
- `switch_or_checkout()` - 尝试 git switch，失败时回退到 git checkout
- `remove_branch_prefix()` - 移除分支名称的前缀

**常量**：
- `COMMON_DEFAULT_BRANCHES` - 常见默认分支名常量

**设计优势**：
- 统一错误处理格式
- 减少代码重复（约 120-150 行）
- 提高代码可维护性

#### 9. 类型定义 (`types.rs`)

**职责**：定义 Git 相关类型

**类型**：
- `RepoType` - 仓库类型枚举（GitHub, Codeup, Unknown）
- `MergeStrategy` - 合并策略枚举（Merge, Squash, FastForwardOnly）

### 设计模式

#### 1. 模块化设计模式

每个功能领域有独立的结构体，使用零大小结构体（unit struct）组织相关函数：

```rust
pub struct GitBranch;  // 零大小结构体
impl GitBranch {
    pub fn current_branch() -> Result<String> { ... }
    // ...
}
```

**优势**：
- 职责清晰，符合单一职责原则
- 命名空间明确（`GitBranch::current_branch()`）
- 易于维护和扩展

#### 2. 辅助函数模式

通过 `helpers.rs` 提供统一的 Git 命令执行接口：

```rust
// 统一接口
cmd_read(&["branch", "--show-current"])
cmd_run(&["add", "--all"])
check_success(&["diff", "--quiet"])
```

**优势**：
- 减少代码重复（约 120-150 行）
- 统一错误处理格式
- 提高代码可维护性

#### 3. 策略模式

通过枚举类型实现不同的策略：

```rust
pub enum MergeStrategy {
    Merge,           // 普通合并
    Squash,          // Squash 合并
    FastForwardOnly, // 只允许 fast-forward
}
```

**优势**：
- 类型安全
- 易于扩展新策略

#### 4. 回退模式

`switch_or_checkout()` 函数实现自动回退：

```rust
// 优先使用 git switch，失败时回退到 git checkout
switch_or_checkout(
    &["switch", branch_name],
    &["checkout", branch_name],
    error_msg,
)?;
```

**优势**：
- 支持新旧 Git 版本
- 自动适配不同环境

### 错误处理

#### 分层错误处理

1. **辅助函数层**：统一错误上下文
   ```rust
   cmd_read(&["branch", "--show-current"])
       .context("Failed to get current branch")
   ```

2. **业务逻辑层**：添加业务上下文
   ```rust
   GitBranch::checkout_branch(branch_name)
       .with_context(|| format!("Failed to checkout branch: {}", branch_name))
   ```

3. **命令层**：用户友好的错误提示

#### 容错机制

- **Git 命令失败**：提供清晰的错误信息和解决建议
- **合并冲突**：检测冲突并提供详细的解决步骤
- **Pre-commit hooks 失败**：提供明确的错误信息
- **仓库类型未知**：返回 `RepoType::Unknown`，不中断流程

---

## 🔄 调用流程与数据流

### 整体架构流程

```
调用者（命令层或其他模块）
  ↓
lib/git/*.rs (核心业务逻辑层)
  ├── GitBranch::xxx()      # 分支操作
  ├── GitCommit::xxx()      # 提交操作
  ├── GitRepo::xxx()        # 仓库检测
  ├── GitStash::xxx()       # 暂存操作
  ├── GitConfig::xxx()     # 配置管理
  ├── GitPreCommit::xxx()   # Pre-commit hooks
  └── GitCherryPick::xxx()  # Cherry-pick 操作
  ↓
helpers.rs (辅助函数层)
  ├── cmd_read()
  ├── cmd_run()
  └── check_success()
  ↓
duct::cmd (命令执行层)
  └── git 命令
```

### 典型调用示例

#### 1. 分支操作

```
GitBranch::checkout_branch(branch_name)
  ↓
helpers::switch_or_checkout()  # 尝试 git switch，失败时回退
  ↓
helpers::cmd_run()  # 执行 git 命令
```

#### 2. 提交操作

```
GitCommit::commit(commit_title, true)
  ↓
GitPreCommit::run_pre_commit()  # 如果存在 pre-commit hooks
  ↓
GitCommit::add_all()  # 暂存所有文件
  ↓
helpers::cmd_run()  # 执行 git commit
```

#### 3. 合并操作

```
GitBranch::merge_branch(source_branch, strategy)
  ↓
GitBranch::has_merge_conflicts()  # 检查冲突
  ↓
GitBranch::checkout_branch(default_branch)  # 切换到默认分支
  ↓
GitBranch::delete(branch_name, false)  # 删除本地分支
```

### 数据流

#### 分支操作数据流

```
用户输入（分支名）
  ↓
GitBranch::checkout_branch()
  ↓
检查分支存在性（本地/远程）
  ↓
switch_or_checkout()  # 尝试 git switch，失败时回退
  ↓
helpers::cmd_run()  # 执行 git 命令
  ↓
返回结果
```

#### 提交操作数据流

```
用户输入（提交消息）
  ↓
GitCommit::commit()
  ↓
检查是否有更改
  ↓
GitCommit::add_all()  # 暂存所有文件
  ↓
GitPreCommit::run_pre_commit()  # 如果存在 hooks
  ↓
helpers::cmd_run()  # 执行 git commit
  ↓
返回结果
```

---

## 📋 使用示例

### 基本使用

```rust
use workflow::git::{GitBranch, GitCommit, GitRepo, GitStash};

// 获取当前分支
let branch = GitBranch::current_branch()?;

// 检查分支是否存在
let (local, remote) = GitBranch::is_branch_exists("feature/new")?;

// 创建或切换分支
GitBranch::checkout_branch("feature/new")?;

// 提交更改
GitCommit::commit("Fix bug", false)?;

// 推送到远程
GitBranch::push("feature/new", true)?;

// 检测仓库类型
let repo_type = GitRepo::detect_repo_type()?;

// 保存工作区更改
GitStash::stash_push(Some("WIP: working on feature"))?;

// Cherry-pick 提交
GitCherryPick::cherry_pick("abc123")?;

// Cherry-pick 但不提交
GitCherryPick::cherry_pick_no_commit("abc123")?;

// 检查是否正在进行 cherry-pick
if GitCherryPick::is_cherry_pick_in_progress() {
    // 解决冲突后继续
    GitCherryPick::cherry_pick_continue()?;
    // 或中止操作
    // GitCherryPick::cherry_pick_abort()?;
}
```

### 合并分支

```rust
use workflow::git::{GitBranch, MergeStrategy};

// 普通合并
GitBranch::merge_branch("feature/new", MergeStrategy::Merge)?;

// Squash 合并
GitBranch::merge_branch("feature/new", MergeStrategy::Squash)?;

// 只允许 fast-forward
GitBranch::merge_branch("feature/new", MergeStrategy::FastForwardOnly)?;
```

### 检查冲突

```rust
use workflow::git::GitBranch;

// 检查是否有合并冲突
if GitBranch::has_merge_conflicts()? {
    // 处理冲突
}
```

---

## 📝 扩展性

### 添加新的 Git 操作

1. 在对应的模块文件中添加方法
2. 使用 `helpers.rs` 中的辅助函数
3. 添加文档注释
4. 在 `mod.rs` 中导出（如需要）

**示例**：
```rust
// branch.rs
impl GitBranch {
    pub fn rename_branch(old_name: &str, new_name: &str) -> Result<()> {
        helpers::cmd_run(&["branch", "-m", old_name, new_name])
            .context("Failed to rename branch")
    }
}
```

### 添加新的仓库类型

1. 在 `types.rs` 中添加新的 `RepoType` 变体
2. 在 `repo.rs` 的 `parse_repo_type_from_url()` 中添加识别逻辑

**示例**：
```rust
// types.rs
pub enum RepoType {
    GitHub,
    Codeup,
    GitLab,  // 新增
    Unknown,
}
```

### 添加新的合并策略

1. 在 `types.rs` 中添加新的 `MergeStrategy` 变体
2. 在 `branch.rs` 的 `merge_branch()` 方法中添加对应的处理逻辑

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [PR 模块架构文档](./PR_ARCHITECTURE.md) - PR 模块如何使用 Git 操作
- [Settings 模块架构文档](./SETTINGS_ARCHITECTURE.md) - 配置管理如何使用 Git 配置

---

## ✅ 总结

Git 模块采用清晰的模块化设计：

1. **模块化结构**：每个功能领域有独立的结构体，职责清晰
2. **统一辅助函数**：通过 `helpers.rs` 提供统一的命令执行接口
3. **类型安全**：使用枚举类型提高类型安全性
4. **错误处理统一**：使用 `anyhow::Result` 和 `context` 提供清晰的错误信息
5. **易于扩展**：模块化设计便于添加新功能
6. **完整功能**：支持分支、提交、仓库检测、暂存、配置、pre-commit hooks 和 cherry-pick 操作

**设计优势**：
- ✅ **职责清晰**：每个结构体负责单一功能领域
- ✅ **代码复用**：统一的辅助函数减少重复代码
- ✅ **易于维护**：模块化设计，低耦合
- ✅ **类型安全**：枚举类型保证类型安全
- ✅ **兼容性好**：自动回退机制支持不同 Git 版本

通过模块化设计和统一辅助函数，实现了代码复用、易于维护和扩展的目标。

---

**最后更新**: 2025-12-16
