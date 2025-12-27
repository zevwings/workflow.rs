# Git 模块架构文档

## 📋 概述

Git 模块是 Workflow CLI 的核心功能之一，提供完整的 Git 仓库操作功能，包括提交管理、分支管理、仓库检测、暂存管理、Pre-commit hooks 支持和配置管理。该模块采用模块化设计，每个功能领域有独立的结构体，通过统一的辅助函数减少代码重复。

**技术实现：**
- **底层库**：使用 `git2` (libgit2 Rust 绑定) 作为 Git 操作的核心实现
- **认证机制**：提供统一的 `GitAuth` 认证回调，支持 SSH 和 HTTPS 两种认证方式
- **性能优化**：直接使用 git2 API，消除了进程启动开销，性能提升 10-100 倍

**模块统计：**
- 总代码行数：约 2500+ 行
- 文件数量：13 个（包括 client 子模块）
- 主要结构体：10 个（GitBranch, GitCommit, GitRepo, GitStash, GitConfig, GitPreCommit, GitCherryPick, GitAuth, GitRepository, GitRemote）
- 核心封装层：client 模块（GitRepository, GitRemote）

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/git/
├── mod.rs          # Git 模块声明和导出 (62行)
├── auth.rs         # 认证回调机制 (400+行)
├── branch.rs       # 分支管理操作 (1800+行)
├── commit.rs       # 提交相关操作 (1100+行)
├── repo.rs         # 仓库检测和类型识别 (200+行)
├── stash.rs        # 暂存管理 (550+行)
├── config.rs       # Git 配置管理 (75行)
├── pre_commit.rs   # Pre-commit hooks 支持 (100+行)
├── cherry_pick.rs  # Cherry-pick 操作 (200+行)
├── client/         # Git2 封装层（核心抽象）
│   ├── mod.rs      # 模块声明
│   ├── repository.rs  # GitRepository 封装 (400+行)
│   └── remote.rs      # GitRemote 封装 (200+行)
├── table.rs        # 表格格式化
├── tag.rs          # Tag 管理
└── types.rs        # 类型定义 (15行)
```

### 依赖模块

- **`git2`**：Git 操作核心库（libgit2 Rust 绑定）
  - 版本：`0.18`
  - 用途：所有 Git 操作（分支、提交、仓库、tag、stash 等）
  - 优势：类型安全、高性能、无需系统 Git 依赖
- **`lib/base/util/`**：工具函数（日志输出等）

### 模块集成

- **PR 模块集成** (`lib/pr/`)：
  - `GitBranch::checkout-_branch()` - 创建或切换分支
  - `GitCommit::commit()` - 提交更改
  - `GitBranch::push()` - 推送到远程
  - `GitRepo::detect-_repo-_type()` - 检测仓库类型（用于工厂函数）
  - `GitBranch::merge-_branch()` - 合并分支
  - `GitStash::stash-_push()` / `stash-_pop()` - 保存/恢复工作区更改

- **配置管理集成**：
  - `GitConfig::set-_global-_user()` - 设置 Git 全局配置
  - 用于初始化设置和 GitHub 账号切换

- **环境检查集成** (`commands/check/`)：
  - `GitRepo::is-_git-_repo()` - 检查是否在 Git 仓库中
  - `GitCommit::status()` - 检查 Git 状态

- **分支管理集成** (`commands/branch/`)：
  - `GitBranch::get-_all-_branches()` - 获取所有分支
  - `GitBranch::is-_merged()` - 检查分支是否已合并
  - `GitRepo::extract-_repo-_name()` - 提取仓库名（用于配置分组）
  - `GitRepo::prune-_remote()` - 清理远程分支引用

---

## 🔄 集成关系

Git 模块是 Workflow CLI 的核心功能模块，为所有需要 Git 操作的模块提供统一的 Git 操作接口。该模块通过以下方式与其他模块集成：

1. **PR 模块**：提供分支管理、提交管理、仓库检测等功能，支持 PR 的创建、合并、同步等操作
2. **分支管理命令**：提供分支列表、合并检查、仓库名提取等功能
3. **环境检查**：提供仓库检测和状态检查功能
4. **配置管理**：提供 Git 全局配置设置功能，用于初始化设置和账号切换

### 主要集成场景

- **PR 创建**：使用 `GitBranch::checkout-_branch()` 创建分支，`GitCommit::commit()` 提交更改
- **PR 合并**：使用 `GitBranch::merge-_branch()` 合并分支，`GitStash` 管理工作区状态
- **仓库检测**：使用 `GitRepo::detect-_repo-_type()` 检测仓库类型，用于平台选择
- **分支清理**：使用 `GitBranch::is-_merged()` 和 `GitRepo::prune-_remote()` 清理已合并的分支

---

## 🏗️ 架构设计

### 设计原则

1. **模块化设计**：每个功能领域有独立的结构体，职责清晰
2. **零大小结构体**：使用 unit struct 组织相关函数，符合 Rust 最佳实践
3. **统一封装层**：通过 `client` 模块提供统一的 `GitRepository` 和 `GitRemote` 封装，统一管理 git2 API
4. **错误处理统一**：使用 `color_eyre::Result` 和 `wrap_err` 提供清晰的错误信息
5. **类型安全**：使用枚举类型（`RepoType`, `MergeStrategy`）提高类型安全性

### 核心组件

#### 1. 分支管理 (`branch.rs`)

**职责**：提供分支相关的所有操作

- **`GitBranch`**：分支管理结构体（零大小结构体）

**核心方法**：
- `current_branch()` - 获取当前分支名
- `checkout_branch()` - 创建或切换到分支
- `push()` - 推送到远程仓库
- `merge_branch()` - 合并分支（支持多种策略）
- `delete()` / `delete_remote()` - 删除本地/远程分支

**关键特性**：
- 支持 `git switch` 和 `git checkout` 的自动回退
- 多种合并策略（Merge, Squash, FastForwardOnly）
- 智能的默认分支检测


#### 2. 提交管理 (`commit.rs`)

**职责**：提供提交相关的操作

- **`GitCommit`**：提交管理结构体（零大小结构体）

**核心方法**：
- `status()` - 检查 Git 状态
- `commit()` - 提交更改（支持 pre-commit hooks）
- `get_diff()` - 获取 Git 修改内容

**关键特性**：
- 自动暂存所有更改
- 集成 pre-commit hooks 支持
- 提供 diff 内容用于 LLM 生成

#### 3. 仓库检测 (`repo.rs`)

**职责**：提供仓库检测和类型识别

- **`GitRepo`**：仓库检测结构体（零大小结构体）

**核心方法**：
- `is_git_repo()` - 检查是否在 Git 仓库中
- `detect_repo_type()` - 检测远程仓库类型（GitHub、Codeup 等）
- `fetch()` - 从远程获取更新

**关键特性**：
- 支持 GitHub 和 Codeup 仓库类型识别
- 支持 SSH Host 别名识别

#### 4. 暂存管理 (`stash.rs`)

**职责**：提供 stash 相关操作

- **`GitStash`**：暂存管理结构体（零大小结构体）

**核心方法**：
- `stash_push()` - 保存未提交的修改到 stash
- `stash_pop()` - 恢复 stash 中的修改

**关键特性**：
- 自动检测合并冲突
- 提供详细的冲突解决提示

#### 5. 配置管理 (`config.rs`)

**职责**：提供 Git 配置管理

- **`GitConfig`**：配置管理结构体（零大小结构体）

**主要方法**：
- `set-_global-_user()` - 设置 Git 全局配置（email 和 name）
- `get-_global-_user()` - 读取 Git 全局配置

**使用场景**：
- 初始化设置时配置 Git 用户信息
- GitHub 账号切换时更新配置

#### 6. Pre-commit Hooks (`pre-_commit.rs`)

**职责**：提供 pre-commit hooks 支持

- **`GitPreCommit`**：Pre-commit hooks 结构体（零大小结构体）

**主要方法**：
- `has-_pre-_commit()` - 检查是否存在 pre-commit hooks
- `run-_pre-_commit()` - 执行 pre-commit hooks

**关键特性**：
- 支持 Git hooks 和 pre-commit 工具
- 自动检测多种 pre-commit 配置方式

**使用场景**：
- 提交前自动执行 hooks
- 支持代码质量检查

#### 7. Cherry-pick 操作 (`cherry_pick.rs`)

**职责**：提供 Git cherry-pick 相关的完整功能

- **`GitCherryPick`**：Cherry-pick 管理结构体（零大小结构体）

**核心方法**：
- `cherry_pick(commit)` - Cherry-pick 提交到当前分支
- `cherry_pick_no_commit(commit)` - Cherry-pick 但不提交
- `cherry_pick_continue()` / `cherry_pick_abort()` - 继续/中止操作

**关键特性**：
- 支持普通 cherry-pick 和 no-commit 模式
- 自动检测 cherry-pick 状态

#### 8. 认证管理 (`auth.rs`)

**职责**：提供统一的 Git 远程操作认证回调机制

- **`GitAuth`**：认证管理结构体（零大小结构体）

**主要方法**：
- `get_remote_callbacks()` - 创建远程操作认证回调

**关键特性**：
- **SSH 认证**：
  - 优先级 1：SSH Agent（自动检测）
  - 优先级 2：SSH 密钥文件（从 SSH config 或默认位置查找）
  - 智能匹配：根据远程 URL 匹配 SSH config 中的 Host 配置
- **HTTPS 认证**：
  - 支持 `GITHUB_TOKEN` 和 `GIT_TOKEN` 环境变量
  - 支持 `GIT_USERNAME` 环境变量
- **认证缓存**：使用 `OnceLock` 缓存认证信息，避免重复查找
- **错误提示**：认证失败时提供详细的配置指导

**使用场景**：
- 推送到远程仓库（`push`）
- 从远程获取更新（`fetch`）
- 删除远程分支/tag（`delete_remote`）
- 所有需要认证的远程操作

**使用示例**：
```rust
use workflow::git::GitAuth;
use git2::PushOptions;

let mut callbacks = GitAuth::get_remote_callbacks();
let mut push_options = PushOptions::new();
push_options.remote_callbacks(callbacks);
```

#### 9. Git2 封装层 (`client/`)

**职责**：提供统一的 git2 API 封装，简化 git2 的使用

**设计目标**：
- 统一错误处理格式
- 简化仓库和远程操作
- 提供清晰的错误信息
- 封装 git2 的复杂生命周期管理
- 支持高级操作（通过 `as_inner()` 访问底层 API）

**核心结构体**：

##### GitRepository 封装

**结构体定义**：
```rust
pub struct GitRepository {
    inner: Repository,
}
```

**主要方法**：

**打开仓库**：
- `open() -> Result<Self>` - 打开当前目录的 Git 仓库
  - 自动检测当前目录是否为 Git 仓库
  - 提供清晰的错误信息（"Failed to open Git repository. Make sure you're in a Git repository."）

- `open_at(path: impl AsRef<Path>) -> Result<Self>` - 打开指定路径的 Git 仓库
  - 支持任意路径的仓库打开
  - 错误信息包含路径信息，便于调试

**基本信息**：
- `signature() -> Result<Signature>` - 获取仓库签名（作者信息）
  - 自动从 Git 配置中读取用户信息
  - 统一错误处理（"Failed to get repository signature"）

- `head() -> Result<Reference>` - 获取 HEAD 引用
  - 统一错误处理（"Failed to get HEAD reference"）

- `current_branch_name() -> Result<String>` - 获取当前分支名
  - 自动从 HEAD 引用中提取分支名
  - 去除 `refs/heads/` 前缀
  - 如果 HEAD 不在分支上，返回清晰的错误信息

**远程操作**：
- `find_origin_remote() -> Result<GitRemote>` - 查找 origin 远程仓库
  - 常用操作的便捷方法
  - 统一错误处理（"Failed to find remote 'origin'"）

- `find_remote(name: &str) -> Result<GitRemote>` - 查找指定名称的远程仓库
  - 支持查找任意名称的远程仓库
  - 错误信息包含远程名称，便于调试

**引用和索引操作**：
- `find_reference(name: &str) -> Result<Reference>` - 查找引用
  - 支持查找任意引用（分支、tag 等）
  - 错误信息包含引用名称

- `index() -> Result<Index>` - 获取索引
  - 用于暂存区操作
  - 统一错误处理（"Failed to get repository index"）

**配置选项**：
- `get_fetch_options() -> FetchOptions` - 获取配置好的 FetchOptions（包含认证）
  - 静态方法，无需仓库实例
  - 自动配置认证回调（`GitAuth::get_remote_callbacks()`）
  - 简化远程获取操作

- `get_push_options() -> PushOptions` - 获取配置好的 PushOptions（包含认证）
  - 静态方法，无需仓库实例
  - 自动配置认证回调（`GitAuth::get_remote_callbacks()`）
  - 简化远程推送操作

**逃生舱机制**：
- `as_inner() -> &Repository` - 直接访问底层 Repository（不可变）
  - 用于需要 git2 高级功能的场景
  - 保持封装层的灵活性
  - 允许访问所有 git2 API

- `as_inner_mut() -> &mut Repository` - 直接访问底层 Repository（可变）
  - 用于需要修改仓库状态的场景
  - 保持封装层的灵活性

##### GitRemote 封装

**结构体定义**：
```rust
pub struct GitRemote {
    inner: Remote,
}
```

**主要方法**：

**基本信息**：
- `url() -> Option<&str>` - 获取远程 URL
  - 返回 `Option`，因为远程可能没有配置 URL
  - 直接返回底层 `Remote::url()` 的结果

**操作**：
- `push(refspecs: &[&str], options: Option<&mut PushOptions>) -> Result<()>` - 推送到远程
  - 封装 git2 的 push 操作
  - 统一错误处理（"Failed to push to remote"）
  - 支持自定义 PushOptions（通常使用 `GitRepository::get_push_options()`）

- `fetch(refspecs: &[&str], options: Option<&mut FetchOptions>, reflog_message: Option<&str>) -> Result<()>` - 从远程获取
  - 封装 git2 的 fetch 操作
  - 统一错误处理（"Failed to fetch from remote"）
  - 支持自定义 FetchOptions（通常使用 `GitRepository::get_fetch_options()`）
  - 支持自定义 reflog 消息

**逃生舱机制**：
- `as_inner() -> &Remote` - 直接访问底层 Remote（不可变）
  - 用于需要 git2 高级功能的场景
  - 保持封装层的灵活性

- `as_inner_mut() -> &mut Remote` - 直接访问底层 Remote（可变）
  - 用于需要修改远程配置的场景
  - 保持封装层的灵活性

**设计优势**：

1. **统一错误处理**：
   - 所有方法都使用 `wrap_err` 或 `wrap_err_with` 提供清晰的错误信息
   - 错误信息包含上下文，便于调试

2. **简化常用操作**：
   - `find_origin_remote()` 提供便捷方法，避免重复查找 "origin"
   - `get_fetch_options()` 和 `get_push_options()` 自动配置认证

3. **类型安全**：
   - 封装 git2 的类型，提供更清晰的 API
   - 编译时类型检查，减少运行时错误

4. **灵活性**：
   - 通过逃生舱机制（`as_inner()`）支持高级操作
   - 不限制 git2 的功能，只是简化常用操作

5. **生命周期管理**：
   - 封装 git2 的复杂生命周期管理
   - 自动处理资源释放

**使用示例**：
```rust
use workflow::git::client::GitRepository;

// 打开仓库
let repo = GitRepository::open()?;

// 获取当前分支
let branch_name = repo.current_branch_name()?;

// 查找远程仓库
let mut remote = repo.find_origin_remote()?;

// 使用配置好的 FetchOptions 获取更新
let mut fetch_options = GitRepository::get_fetch_options();
remote.fetch(&[], Some(&mut fetch_options), None)?;

// 需要高级功能时，使用逃生舱
let inner_repo = repo.as_inner();
let config = inner_repo.config()?;
```

#### 10. 类型定义 (`types.rs`)

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
    pub fn current-_branch() -> Result<String> { ... }
    // ...
}
```

**优势**：
- 职责清晰，符合单一职责原则
- 命名空间明确（`GitBranch::current-_branch()`）
- 易于维护和扩展

#### 2. git2 API 模式

直接使用 git2 API 进行 Git 操作：

```rust
// 打开仓库
let repo = git2::Repository::open(".")?;

// 获取当前分支
let head = repo.head()?;
let branch_name = head.shorthand();

// 推送到远程
let mut remote = repo.find_remote("origin")?;
let mut callbacks = GitAuth::get_remote_callbacks();
let mut push_options = PushOptions::new();
push_options.remote_callbacks(callbacks);
remote.push(&[refspec], Some(&mut push_options))?;
```

**优势**：
- 类型安全：编译时类型检查
- 高性能：消除进程启动开销（10-100 倍性能提升）
- 无需系统 Git：纯 Rust 实现，减少外部依赖
- 更好的错误处理：清晰的错误信息和上下文

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

`switch-_or-_checkout()` 函数实现自动回退：

```rust
// 优先使用 git switch，失败时回退到 git checkout
switch-_or-_checkout(
    &["switch", branch-_name],
    &["checkout", branch-_name],
    error-_msg,
)?;
```

**优势**：
- 支持新旧 Git 版本
- 自动适配不同环境

### 错误处理

#### 分层错误处理

1. **辅助函数层**：统一错误上下文
   ```rust
   cmd-_read(&["branch", "--show-current"])
       .context("Failed to get current branch")
   ```

2. **业务逻辑层**：添加业务上下文
   ```rust
   GitBranch::checkout-_branch(branch-_name)
       .with-_context(|| format!("Failed to checkout branch: {}", branch-_name))
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
  ├── GitConfig::xxx()      # 配置管理
  ├── GitPreCommit::xxx()   # Pre-commit hooks
  ├── GitCherryPick::xxx()  # Cherry-pick 操作
  └── GitAuth::xxx()        # 认证回调
  ↓
client/ (Git2 封装层)
  ├── GitRepository::open() / open_at()  # 打开仓库
  ├── GitRepository::find_remote()        # 查找远程
  └── GitRemote::fetch() / push()         # 远程操作
  ↓
git2 API (底层实现)
  ├── Repository            # 仓库操作
  ├── Remote                # 远程操作
  ├── Index                 # 索引操作
  ├── Commit                # 提交操作
  ├── Branch                # 分支操作
  └── RemoteCallbacks       # 认证回调
```

### 典型调用示例

#### 1. 分支操作（使用 git2）

```
GitBranch::checkout_branch(branch_name)
  ↓
GitRepository::open()  # 打开仓库
  ↓
repo.find_reference()  # 查找分支引用（通过 as_inner()）
  ↓
repo.set_head() + repo.checkout_head()  # 切换分支（通过 as_inner_mut()）
```

#### 2. 提交操作（使用 git2）

```
GitCommit::commit(commit_title, true)
  ↓
GitPreCommit::run_pre_commit()  # 如果存在 pre-commit hooks
  ↓
repo.index() + index.add_all() + index.write()  # 暂存所有文件
  ↓
index.write_tree() + repo.commit()  # 创建提交
```

#### 3. 推送操作（使用 git2 + 认证）

```
GitBranch::push(branch_name, force)
  ↓
GitRepository::open()  # 打开仓库
  ↓
repo.find_origin_remote()  # 查找远程（返回 GitRemote）
  ↓
GitAuth::get_remote_callbacks()  # 获取认证回调
  ↓
remote.push() + PushOptions  # 推送到远程（通过 GitRemote）
```

#### 4. 合并操作（使用 git2）

```
GitBranch::merge_branch(source_branch, strategy)
  ↓
repo.merge_analysis()  # 分析合并类型
  ↓
repo.merge_commits()  # 执行合并
  ↓
index.has_conflicts()  # 检查冲突
  ↓
repo.commit()  # 创建合并提交
```

### 数据流

#### 分支操作数据流

```
用户输入（分支名）
  ↓
GitBranch::checkout_branch()
  ↓
GitRepository::open()  # 打开 GitRepository 封装
  ↓
repo.find_reference()  # 检查分支存在性
  ↓
repo.set_head()  # 设置 HEAD
  ↓
repo.checkout_head()  # 检出工作区
  ↓
返回结果
```

#### 提交操作数据流

```
用户输入（提交消息）
  ↓
GitCommit::commit()
  ↓
repo.statuses()  # 检查是否有更改
  ↓
repo.index() + index.add_all() + index.write()  # 暂存所有文件
  ↓
GitPreCommit::run_pre_commit()  # 如果存在 hooks
  ↓
index.write_tree() + repo.commit()  # 创建提交
  ↓
返回结果
```

#### 远程操作数据流（带认证）

```
用户操作（push/fetch）
  ↓
GitBranch::push() / GitRepo::fetch()
  ↓
GitRepository::open()  # 打开 GitRepository 封装
  ↓
repo.find_origin_remote()  # 查找远程（返回 GitRemote）
  ↓
GitAuth::get_remote_callbacks()  # 获取认证回调
  ↓
根据 URL 类型选择认证方式：
  - SSH: SSH Agent 或 SSH 密钥文件
  - HTTPS: GITHUB_TOKEN 或 GIT_TOKEN
  ↓
remote.push() / remote.fetch()  # 执行远程操作
  ↓
返回结果
```

---

## 🔐 认证机制

### 概述

Git 模块使用 `GitAuth` 提供统一的认证回调机制，支持 SSH 和 HTTPS 两种认证方式。所有需要认证的远程操作（push、fetch、delete_remote 等）都使用此机制。

### 认证流程

#### SSH 认证

**优先级顺序：**
1. **SSH Agent**：优先使用 SSH Agent 中的密钥（在认证回调中实时尝试，最方便，适合开发环境）
2. **SSH 密钥文件**：如果 SSH Agent 不可用，使用缓存的密钥文件（在初始化时查找并缓存）：
   - **优先级 1**：SSH config 匹配（根据远程 URL 匹配 `~/.ssh/config` 中的 Host 配置）
   - **优先级 2**：默认密钥顺序：`~/.ssh/id_ed25519` → `~/.ssh/id_rsa` → `~/.ssh/id_ecdsa`

**SSH config 匹配逻辑：**
- 从远程 URL 提取 host（如 `github.com`）
- 解析 `~/.ssh/config` 文件
- 匹配 `Host` 或 `HostName` 配置
- 返回对应的 `IdentityFile` 路径

**示例配置：**
```ssh-config
# ~/.ssh/config
Host github-personal
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_personal

Host github-work
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_work
```

#### HTTPS 认证

**环境变量优先级：**
- `GITHUB_TOKEN`：GitHub Personal Access Token（优先级 1，先尝试此变量）
- `GIT_TOKEN`：通用 Git Token（优先级 2，如果 `GITHUB_TOKEN` 不存在则使用）
- `GIT_USERNAME`：HTTPS 用户名（可选，用于 HTTPS 认证）

**使用示例：**
```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxx
export GIT_USERNAME=your-username  # 可选
```

### 认证缓存机制

使用 `OnceLock` 实现单例模式，在程序运行期间只初始化一次：

```rust
static AUTH_INFO: OnceLock<CachedAuthInfo> = OnceLock::new();
```

**缓存内容：**
- SSH 密钥文件路径（如果找到）
- HTTPS token（从环境变量读取）
- HTTPS 用户名（从环境变量读取）

**优势：**
- 避免重复查找 SSH 密钥和环境变量
- 提高性能，减少 I/O 操作
- 统一管理认证信息

### 错误处理

认证失败时，提供详细的错误信息和配置指导：

**SSH 认证失败：**
```
SSH authentication failed: ...

Troubleshooting:
1. Add SSH key to agent: ssh-add ~/.ssh/id_ed25519
2. Check key permissions: chmod 600 ~/.ssh/id_ed25519
3. Test SSH connection: ssh -T git@github.com
4. Or use HTTPS URL with GITHUB_TOKEN environment variable
```

**HTTPS 认证失败：**
```
No HTTPS credentials found. Please set one of:
1. GITHUB_TOKEN environment variable
2. GIT_TOKEN environment variable
```

### 使用示例

#### 推送操作（自动认证）

```rust
use workflow::git::{GitBranch, GitAuth};
use git2::PushOptions;

// 推送会自动使用 GitAuth 进行认证
GitBranch::push("feature/new", false)?;
```

#### 手动配置认证回调

```rust
use workflow::git::GitAuth;
use git2::{PushOptions, Repository};

let repo = Repository::open(".")?;
let mut remote = repo.find_remote("origin")?;

// 获取认证回调
let mut callbacks = GitAuth::get_remote_callbacks();

// 配置推送选项
let mut push_options = PushOptions::new();
push_options.remote_callbacks(callbacks);

// 推送
remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))?;
```

### 支持的远程操作

以下操作自动使用 `GitAuth` 进行认证：

- ✅ `GitBranch::push()` - 推送分支
- ✅ `GitBranch::push_force_with_lease()` - 强制推送
- ✅ `GitBranch::delete_remote()` - 删除远程分支
- ✅ `GitRepo::fetch()` - 获取远程更新
- ✅ `GitTag::push()` - 推送 tag
- ✅ `GitTag::delete_remote()` - 删除远程 tag
- ✅ `GitBranch::pull()` - 拉取分支（内部使用 fetch）

---

## 📋 使用示例

### 🚀 快速开始

#### 导入 Git 模块

```rust
// Git2 封装层（推荐）
use workflow::git::client::GitRepository;
use workflow::git::client::GitRemote;

// 高级封装（业务逻辑层）
use workflow::git::{GitBranch, GitCommit, GitRepo, GitStash, GitCherryPick};
```

#### Git2 封装层基本用法

```rust
use workflow::git::client::GitRepository;

// 打开仓库
let repo = GitRepository::open()?;                    // 当前目录
let repo = GitRepository::open_at("/path/to/repo")?;   // 指定路径

// 获取签名
let signature = repo.signature()?;

// 查找远程
let mut remote = repo.find_origin_remote()?;

// 获取当前分支
let branch = repo.current_branch_name()?;
```

### 核心示例

#### 示例 1: 基本使用（高级封装）

```rust
use workflow::git::{GitBranch, GitCommit, GitRepo, GitStash};

// 获取当前分支
let branch = GitBranch::current_branch()?;

// 创建或切换分支
GitBranch::checkout_branch("feature/new")?;

// 提交更改
GitCommit::commit("Fix bug", false)?;

// 推送到远程
GitBranch::push("feature/new", true)?;
```

#### 示例 2: 合并分支

```rust
use workflow::git::{GitBranch, MergeStrategy};

// 普通合并
GitBranch::merge_branch("feature/new", MergeStrategy::Merge)?;

// Squash 合并
GitBranch::merge_branch("feature/new", MergeStrategy::Squash)?;
```

#### 示例 3: 推送操作（使用 Git2 封装层）

```rust
use workflow::git::client::GitRepository;

let mut repo = GitRepository::open()?;
let mut remote = repo.find_origin_remote()?;

let mut push_options = GitRepository::get_push_options();
remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))?;
```

---

## 📝 扩展性

### 添加新的 Git 操作

1. 在对应的模块文件中添加方法
2. 使用 `GitRepository::open()` 或 `GitRepository::open_at()` 打开仓库
3. 使用 `GitRepository` 和 `GitRemote` 封装的方法进行操作
4. 对于高级操作，使用 `as_inner()` 或 `as_inner_mut()` 访问底层 git2 API
5. 如果是远程操作，使用 `GitAuth::get_remote_callbacks()` 进行认证
6. 添加文档注释
7. 在 `mod.rs` 中导出（如需要）

**示例**：
```rust
// branch.rs
use crate::git::GitRepository;

impl GitBranch {
    pub fn rename_branch(old_name: &str, new_name: &str) -> Result<()> {
        let repo = GitRepository::open()?;

        // 查找旧分支引用
        let old_ref = repo.find_reference(&format!("refs/heads/{}", old_name))?;
        let target = old_ref.target().ok_or_else(|| eyre!("Invalid reference"))?;

        // 创建新分支引用（使用 as_inner_mut() 访问底层 API）
        repo.as_inner_mut().reference(&format!("refs/heads/{}", new_name), target, true, "Rename branch")?;

        // 如果是当前分支，更新 HEAD
        if repo.head()?.shorthand() == Some(old_name) {
            repo.set_head(&format!("refs/heads/{}", new_name))?;
        }

        // 删除旧引用
        old_ref.delete()?;

        Ok(())
    }
}
```

### 添加新的仓库类型

1. 在 `types.rs` 中添加新的 `RepoType` 变体
2. 在 `repo.rs` 的 `parse-_repo-_type-_from-_url()` 中添加识别逻辑

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
2. 在 `branch.rs` 的 `merge-_branch()` 方法中添加对应的处理逻辑

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [PR 模块架构文档](./pr.md) - PR 模块如何使用 Git 操作
- [Settings 模块架构文档](./settings.md) - 配置管理如何使用 Git 配置

---

## ✅ 总结

Git 模块采用清晰的模块化设计，基于 git2 库实现：

1. **模块化结构**：每个功能领域有独立的结构体，职责清晰
2. **git2 实现**：使用 git2 (libgit2 Rust 绑定) 作为底层实现，类型安全、高性能
3. **统一认证机制**：通过 `GitAuth` 提供统一的认证回调，支持 SSH 和 HTTPS
4. **类型安全**：使用枚举类型和 git2 强类型 API 提高类型安全性
5. **错误处理统一**：使用 `color-eyre::Result` 和 `context` 提供清晰的错误信息
6. **易于扩展**：模块化设计便于添加新功能
7. **完整功能**：支持分支、提交、仓库检测、暂存、配置、pre-commit hooks 和 cherry-pick 操作

**设计优势**：
- ✅ **职责清晰**：每个结构体负责单一功能领域
- ✅ **高性能**：直接使用 git2 API，消除进程启动开销（10-100 倍性能提升）
- ✅ **类型安全**：编译时类型检查，减少运行时错误
- ✅ **易于维护**：模块化设计，低耦合
- ✅ **无需系统 Git**：纯 Rust 实现，减少外部依赖
- ✅ **统一认证**：智能的认证机制，支持多种认证方式

**技术改进**：
- ✅ **性能提升**：消除了所有核心操作的进程启动开销（~50-200ms per call）
- ✅ **类型安全**：使用强类型 API，编译时检查
- ✅ **部署简化**：不再需要系统 Git，减少外部依赖
- ✅ **跨平台一致性**：纯 Rust 实现，避免平台差异

通过模块化设计和 git2 API，实现了高性能、类型安全、易于维护和扩展的目标。

---

**最后更新**: 2025-01-27
