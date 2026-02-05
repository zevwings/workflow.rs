# Git Hooks 实现文档

## 📋 文档信息

- **状态**: ⏳ 待实施
- **实现度**: 0%
- **分类**: Git 功能增强
- **优先级**: P0（高优先级）
- **创建日期**: 2025-02-05

---

## 🎯 概述

### 背景

Workflow CLI 使用 `git2` 库进行 Git 操作，但 `git2` 不支持 Git hooks 机制。为了保持与标准 Git 命令的兼容性，并支持项目中已有的 `.git/hooks/` 脚本（如 `pre-commit`），需要实现一套自定义的 hooks 执行流程。

### 目标

1. **兼容性**：完全兼容标准 Git hooks（`.git/hooks/` 目录或 `core.hooksPath` 指向的目录）
2. **工具兼容性**：兼容 pre-commit/prek 工具（通过 `.pre-commit-config.yaml` 配置）
3. **透明性**：用户无需感知 hooks 的执行，体验与标准 Git 命令一致
4. **可控性**：支持跳过 hooks（`--no-verify`）和配置管理
5. **可扩展性**：支持多种脚本类型（bash、Python、Rust 二进制等）

### 范围

- ✅ **包含**：客户端 hooks（pre-commit、pre-push、post-commit 等）
- ❌ **不包含**：服务器端 hooks（pre-receive、update、post-receive 等）

---

## 📊 需求分析

### 1. 功能需求

#### 1.1 支持的 Hooks 类型

| Hook 名称 | 执行时机 | 优先级 | 说明 |
|----------|---------|--------|------|
| `pre-commit` | `commit()` 之前 | P0 | 代码质量检查、格式化 |
| `prepare-commit-msg` | 提交消息准备后 | P2 | 修改提交消息 |
| `commit-msg` | 提交消息验证 | P0 | 验证提交消息格式 |
| `post-commit` | `commit()` 成功后 | P0 | 后置操作、通知 |
| `pre-push` | `push()` 之前 | P1 | 推送前验证 |
| `pre-rebase` | `rebase_onto()` 之前 | P2 | Rebase 前检查 |
| `post-merge` | `pull()`/`merge()` 成功后 | P1 | 合并后操作 |
| `post-checkout` | `checkout_branch()` 成功后 | P2 | 切换分支后操作 |

#### 1.2 核心功能

1. **Hook 发现**
   - 从 `.git/hooks/` 目录发现可执行脚本
   - 支持按 hook 名称匹配（如 `pre-commit`）
   - 跳过 `.sample` 文件
   - 检查文件可执行权限

2. **Hook 执行**
   - 执行外部脚本（bash、Python、Rust 二进制等）
   - 设置正确的环境变量（`GIT_DIR`、`GIT_WORK_TREE` 等）
   - 传递必要的上下文信息
   - 捕获并显示脚本输出

3. **结果处理**
   - `pre-*` hooks 失败：阻止操作
   - `post-*` hooks 失败：记录错误但不阻止操作
   - 支持超时控制（默认 30 秒）

4. **跳过机制**
   - 支持 `--no-verify` 参数跳过所有 hooks
   - 支持 `--skip-hook=<name>` 跳过特定 hook
   - 支持环境变量 `SKIP_HOOKS=pre-commit,pre-push`

5. **第三方工具兼容性**
   - 自动检测项目中使用的 hooks 管理工具
   - 优先执行工具管理的 hooks
   - 支持与标准 Git hooks 共存
   - 按优先级执行：工具管理的 hooks > 标准 hooks

### 2. 第三方工具兼容性需求

#### 2.1 支持的 Hooks 管理工具

| 工具名称 | 检测方式 | 执行方式 | 优先级 |
|---------|---------|---------|--------|
| **prek** | `.pre-commit-config.yaml` + `prek` 命令存在 | 调用 `prek run --hook {hook-name}` | P0（优先） |
| **pre-commit** | `.pre-commit-config.yaml` + `pre-commit` 命令存在 | 调用 `pre-commit run --hook {hook-name}` | P0（回退） |
| **标准 Git hooks** | `.git/hooks/` 或 `core.hooksPath` 指向的目录 | 直接执行脚本 | P1（兜底） |

#### 2.2 工具检测策略

**检测顺序**（按优先级）：
1. **prek/pre-commit**：检查 `.pre-commit-config.yaml` 是否存在
   - 如果存在，优先检查 `prek` 命令是否可用（性能更好，快 7 倍）
   - 如果 `prek` 不可用，检查 `pre-commit` 命令是否可用
   - 如果都不可用，但配置文件存在，记录警告但不阻止操作
   - 如果工具可用，调用 `prek run --hook {hook-name}` 或 `pre-commit run --hook {hook-name}`

2. **标准 Git hooks**：检查 hooks 目录（`.git/hooks/` 或 `core.hooksPath` 指向的目录）
   - `HookDiscoverer` 会自动检查 `core.hooksPath` 配置
   - 如果 `core.hooksPath` 已设置，使用该路径作为 hooks 目录
   - 如果 `core.hooksPath` 未设置，使用默认的 `.git/hooks/` 目录
   - 检查对应 hook 脚本是否存在且可执行
   - 这是兜底方案，确保标准 Git hooks 能正常工作

#### 2.3 执行策略

**执行优先级**：
```
1. prek/pre-commit hooks（如果检测到工具和配置文件）
   ├─ 优先使用 prek（如果可用）
   └─ 回退到 pre-commit（如果 prek 不可用）
   ↓
2. 标准 Git hooks（.git/hooks/ 或 core.hooksPath 指向的目录）
   ├─ 如果 core.hooksPath 已设置，从该路径读取
   └─ 如果 core.hooksPath 未设置，从 .git/hooks/ 读取
   ↓
3. 如果所有 hooks 都通过，继续操作
```

**重要说明**：
- prek/pre-commit 和标准 hooks 可以**共存**：先执行 prek/pre-commit，再执行标准 hooks
- 如果 prek/pre-commit hooks 失败，会阻止操作，不会继续执行标准 hooks
- 标准 hooks 支持 `core.hooksPath` 配置（兼容其他工具设置的路径）

**重要原则**：
- 如果检测到多个工具，按优先级执行（prek/pre-commit > 标准 hooks）
- 如果某个工具管理的 hook 失败，阻止操作（不继续执行后续 hooks）
- 如果某个工具管理的 hook 不存在，继续检查下一个工具
- 所有 hooks 必须全部通过才能继续操作

#### 2.4 工具特定实现

**prek/pre-commit**：
- 检测：检查 `.pre-commit-config.yaml` 是否存在
- 执行：调用命令行工具 `prek run --hook {hook-name}` 或 `pre-commit run --hook {hook-name}`
- 参数：`--hook {hook-name} --all-files`（检查所有文件）
- 环境变量：设置 `GIT_DIR`、`GIT_WORK_TREE` 等
- 退出码：非 0 退出码表示失败，阻止操作

**标准 Git hooks**：
- 检测：检查 hooks 目录（`.git/hooks/` 或 `core.hooksPath` 指向的目录）
- 执行：直接执行 hook 脚本
- 环境变量：设置 `GIT_DIR`、`GIT_WORK_TREE` 等，以及 hook 特定的环境变量
- 退出码：非 0 退出码表示失败，阻止操作

### 3. 非功能需求

#### 2.1 性能要求
- Hook 执行不应显著影响 Git 操作速度
- 支持 hook 超时控制（防止无限执行）
- Hook 发现结果可缓存

#### 3.2 兼容性要求
- 完全兼容标准 Git hooks 脚本（`.git/hooks/` 或 `core.hooksPath` 指向的目录）
- 兼容 prek/pre-commit 工具（通过 `.pre-commit-config.yaml` 配置）
- 不影响直接使用 `git` 命令的用户
- 支持跨平台（Unix、Windows）
- 支持工具共存（prek/pre-commit 和标准 hooks 可以同时存在，按优先级执行）

#### 2.3 安全性要求
- 验证 hook 脚本来源（仅执行 `.git/hooks/` 下的脚本）
- 限制 hook 执行权限（如果可能）
- 防止恶意 hook 脚本

---

## 🏗️ 架构设计

### 1. 模块结构

```
crates/storage/src/git/services/hooks/
├── mod.rs              // Hook 服务接口和模块导出
├── executor.rs         // Hook 执行器（核心逻辑）
├── discoverer.rs       // Hook 发现器（标准 Git hooks）
├── context.rs         // Hook 上下文信息
├── config.rs          // Hook 配置管理
├── error.rs           // Hook 相关错误类型
└── tools/              // 第三方工具兼容模块
    ├── mod.rs          // 工具检测和执行接口
    ├── pre_commit.rs   // pre-commit/prek 支持
    └── detector.rs     // 工具检测器
```

### 2. 类图设计

```
┌─────────────────────┐
│   HookService       │  Trait
│   + execute()       │
└─────────────────────┘
         ▲
         │
┌─────────────────────┐
│ HookServiceImpl     │
│ - executor          │
│ - discoverer        │
│ - tool_detector     │
│ - config            │
└─────────────────────┘
         │
         ├─────────────────┬──────────────────┐
         │                 │                  │
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ HookExecutor │  │HookDiscoverer│  │ToolDetector  │
│ + run()      │  │ + find()     │  │ + detect()   │
└──────────────┘  └──────────────┘  └──────────────┘
                                           │
                           ┌───────────────┼───────────────┐
                           │               │               │
                    ┌──────────┐                    ┌──────────┐
                    │PreCommit │                    │ Standard │
                    │Executor  │                    │Executor  │
                    └──────────┘                    └──────────┘
                    (prek/pre-commit)              (标准 hooks)
```

### 3. 执行流程

#### 3.1 Commit 流程（带 Hooks）

```
commit(message, all)
    ↓
[1] 写入暂存区 (index.write())
    ↓
[2] pre-commit hook
    ├─ 发现脚本: .git/hooks/pre-commit
    ├─ 执行脚本（传递暂存区文件列表）
    ├─ 检查结果: 失败则阻止提交
    └─ 如果脚本修改了文件，重新暂存
    ↓
[3] prepare-commit-msg hook（可选）
    ├─ 传递提交消息
    └─ 可以修改提交消息
    ↓
[4] 执行 git2 commit 操作
    ↓
[5] commit-msg hook
    ├─ 传递提交消息和 SHA
    └─ 验证消息格式，失败则回滚提交
    ↓
[6] post-commit hook
    ├─ 传递提交 SHA
    └─ 执行后置操作（记录错误但不阻止）
    ↓
[7] 返回提交 SHA
```

#### 3.2 Push 流程（带 Hooks）

```
push(branch_name, set_upstream)
    ↓
[1] pre-push hook
    ├─ 发现脚本: .git/hooks/pre-push
    ├─ 传递: 分支名、要推送的提交列表、远程 URL
    ├─ 执行脚本
    └─ 检查结果: 失败则阻止推送
    ↓
[2] 执行 git2 push 操作
    ↓
[3] 返回结果
```

---

## 💻 实现方案

### 1. Hook 服务接口

```rust
// crates/storage/src/git/services/hooks/mod.rs

use domain::git::GitError;

/// Hook 执行结果
#[derive(Debug, Clone)]
pub enum HookResult {
    /// Hook 执行成功
    Success,
    /// Hook 执行失败，包含错误消息
    Failure(String),
    /// Hook 修改了文件，需要重新暂存
    Modified,
}

/// Hook 上下文信息
#[derive(Debug, Clone)]
pub struct HookContext {
    /// 仓库路径
    pub repo_path: std::path::PathBuf,
    /// Git 目录路径
    pub git_dir: std::path::PathBuf,

    // Commit 相关
    /// 暂存区文件列表
    pub staged_files: Vec<String>,
    /// 提交消息（prepare-commit-msg, commit-msg）
    pub commit_message: Option<String>,
    /// 提交 SHA（commit-msg, post-commit）
    pub commit_sha: Option<String>,

    // Push 相关
    /// 分支名称
    pub branch_name: Option<String>,
    /// 远程 URL
    pub remote_url: Option<String>,
    /// 要推送的提交列表
    pub commits_to_push: Vec<String>,

    // Merge 相关
    /// 是否有冲突
    pub has_conflicts: Option<bool>,
}

/// Hook 服务接口
pub trait HookService: Send + Sync {
    /// 执行指定名称的 hook
    ///
    /// # 参数
    /// - `hook_name`: Hook 名称（如 "pre-commit"）
    /// - `context`: Hook 上下文信息
    /// - `skip_if_missing`: 如果 hook 不存在，是否跳过（默认 true）
    ///
    /// # 返回
    /// - `Ok(HookResult)`: Hook 执行结果
    /// - `Err(GitError)`: Hook 执行失败
    fn execute_hook(
        &self,
        hook_name: &str,
        context: &HookContext,
        skip_if_missing: bool,
    ) -> Result<HookResult, GitError>;

    /// 检查是否应该跳过 hooks
    fn should_skip_hooks(&self) -> bool;

    /// 检查是否应该跳过特定 hook
    fn should_skip_hook(&self, hook_name: &str) -> bool;
}
```

### 2. Hook 发现器

```rust
// crates/storage/src/git/services/hooks/discoverer.rs

use std::path::{Path, PathBuf};

pub struct HookDiscoverer {
    hooks_dir: PathBuf,
}

impl HookDiscoverer {
    /// 创建 HookDiscoverer
    ///
    /// # 参数
    /// - `git_dir`: Git 目录路径（.git）
    /// - `repo_path`: 仓库根目录路径
    ///
    /// # 说明
    /// 会检查 Git 配置 `core.hooksPath`，如果设置了则使用该路径
    /// 否则使用默认的 `.git/hooks/` 目录
    pub fn new(git_dir: PathBuf, repo_path: PathBuf) -> Self {
        // 检查 core.hooksPath 配置（某些工具会设置此配置）
        let hooks_dir = if let Ok(hooks_path) = Self::get_core_hooks_path(&git_dir, &repo_path) {
            hooks_path
        } else {
            // 默认使用 .git/hooks/
            git_dir.join("hooks")
        };

        Self { hooks_dir }
    }

    /// 获取 core.hooksPath 配置值
    fn get_core_hooks_path(git_dir: &Path, repo_path: &Path) -> Result<PathBuf, GitError> {
        use std::process::Command;

        // 使用 git config 命令读取配置
        let output = Command::new("git")
            .arg("config")
            .arg("core.hooksPath")
            .current_dir(repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed(format!("Failed to read git config: {}", e)))?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                // 如果是相对路径，相对于仓库根目录
                let hooks_path = if PathBuf::from(&path_str).is_absolute() {
                    PathBuf::from(path_str)
                } else {
                    repo_path.join(path_str)
                };
                return Ok(hooks_path);
            }
        }

        Err(GitError::OperationFailed("core.hooksPath not set".into()))
    }

    /// 发现指定名称的 hook
    pub fn find_hook(&self, hook_name: &str) -> Result<Option<PathBuf>, GitError> {
        let hook_path = self.hooks_dir.join(hook_name);

        // 检查文件是否存在
        if !hook_path.exists() {
            return Ok(None);
        }

        // 跳过 .sample 文件
        if hook_path.to_string_lossy().ends_with(".sample") {
            return Ok(None);
        }

        // 检查可执行权限
        if !self.is_executable(&hook_path)? {
            return Ok(None);
        }

        Ok(Some(hook_path))
    }

    /// 检查文件是否可执行
    fn is_executable(&self, path: &Path) -> Result<bool, GitError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            let mode = metadata.permissions().mode();
            Ok((mode & 0o111) != 0)
        }

        #[cfg(windows)]
        {
            // Windows 上通过扩展名判断
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                Ok(ext_str == "exe" || ext_str == "bat" || ext_str == "cmd" || ext_str == "ps1")
            } else {
                Ok(true) // 没有扩展名，假设可执行
            }
        }
    }
}
```

### 3. Hook 执行器

```rust
// crates/storage/src/git/services/hooks/executor.rs

use std::process::{Command, Stdio};
use std::time::Duration;
use domain::git::GitError;
use super::{HookContext, HookResult};

pub struct HookExecutor {
    timeout: Duration,
}

impl HookExecutor {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// 执行 hook 脚本
    pub fn execute(
        &self,
        hook_path: &std::path::Path,
        context: &HookContext,
    ) -> Result<HookResult, GitError> {
        // 准备环境变量
        let mut env_vars = self.prepare_env_vars(context);

        // 准备标准输入（某些 hook 需要）
        let stdin_content = self.prepare_stdin(context, hook_path)?;

        // 执行脚本
        let mut cmd = Command::new(hook_path);
        cmd.current_dir(&context.repo_path)
            .envs(&env_vars)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 设置超时（使用 spawn + wait_timeout）
        let mut child = cmd.spawn()
            .map_err(|e| GitError::OperationFailed(format!("Failed to execute hook: {}", e)))?;

        // 写入标准输入
        if let Some(ref stdin) = stdin_content {
            if let Some(mut stdin_handle) = child.stdin.take() {
                use std::io::Write;
                stdin_handle.write_all(stdin.as_bytes())
                    .map_err(|e| GitError::OperationFailed(format!("Failed to write stdin: {}", e)))?;
            }
        }

        // 等待执行完成（带超时）
        let output = self.wait_with_timeout(&mut child)?;

        // 处理输出
        self.handle_output(&output)?;

        // 检查结果
        if output.status.success() {
            Ok(HookResult::Success)
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(GitError::HookFailed(format!("Hook {} failed: {}",
                hook_path.display(), error_msg)))
        }
    }

    /// 准备环境变量
    fn prepare_env_vars(&self, context: &HookContext) -> Vec<(String, String)> {
        let mut env_vars = Vec::new();

        env_vars.push(("GIT_DIR".to_string(), context.git_dir.to_string_lossy().to_string()));
        env_vars.push(("GIT_WORK_TREE".to_string(), context.repo_path.to_string_lossy().to_string()));

        // 为 pre-commit 传递暂存区文件列表
        if !context.staged_files.is_empty() {
            env_vars.push(("GIT_STAGED_FILES".to_string(),
                context.staged_files.join("\n")));
        }

        // 为 commit-msg 传递提交消息
        if let Some(ref msg) = context.commit_message {
            env_vars.push(("GIT_COMMIT_MSG".to_string(), msg.clone()));
        }

        // 为 pre-push 传递分支和提交信息
        if let Some(ref branch) = context.branch_name {
            env_vars.push(("GIT_BRANCH".to_string(), branch.clone()));
        }

        if !context.commits_to_push.is_empty() {
            env_vars.push(("GIT_COMMITS_TO_PUSH".to_string(),
                context.commits_to_push.join("\n")));
        }

        env_vars
    }

    /// 准备标准输入内容
    fn prepare_stdin(
        &self,
        context: &HookContext,
        hook_path: &std::path::Path,
    ) -> Result<Option<String>, GitError> {
        let hook_name = hook_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // commit-msg hook 需要传递提交消息
        if hook_name == "commit-msg" {
            if let Some(ref msg) = context.commit_message {
                return Ok(Some(msg.clone()));
            }
        }

        // prepare-commit-msg hook 需要传递提交消息
        if hook_name == "prepare-commit-msg" {
            if let Some(ref msg) = context.commit_message {
                return Ok(Some(msg.clone()));
            }
        }

        Ok(None)
    }

    /// 等待进程完成（带超时）
    fn wait_with_timeout(
        &self,
        child: &mut std::process::Child,
    ) -> Result<std::process::Output, GitError> {
        // 简化实现：使用 blocking wait（实际应该使用异步或线程）
        // 注意：这里需要根据实际需求选择超时实现方式
        match child.wait() {
            Ok(status) => {
                let output = std::process::Output {
                    status,
                    stdout: child.stdout.take()
                        .and_then(|mut s| {
                            use std::io::Read;
                            let mut buf = Vec::new();
                            s.read_to_end(&mut buf).ok()?;
                            Some(buf)
                        })
                        .unwrap_or_default(),
                    stderr: child.stderr.take()
                        .and_then(|mut s| {
                            use std::io::Read;
                            let mut buf = Vec::new();
                            s.read_to_end(&mut buf).ok()?;
                            Some(buf)
                        })
                        .unwrap_or_default(),
                };
                Ok(output)
            }
            Err(e) => Err(GitError::OperationFailed(format!("Hook execution failed: {}", e))),
        }
    }

    /// 处理 hook 输出
    fn handle_output(&self, output: &std::process::Output) -> Result<(), GitError> {
        // 输出到 stderr（与 Git 行为一致）
        if !output.stdout.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }
}
```

### 4. Hook 配置

```rust
// crates/storage/src/git/services/hooks/config.rs

use std::collections::HashSet;

pub struct HookConfig {
    /// 是否跳过所有 hooks
    pub skip_all: bool,
    /// 要跳过的 hook 名称列表
    pub skip_hooks: HashSet<String>,
    /// Hook 超时时间（秒）
    pub timeout_seconds: u64,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            skip_all: false,
            skip_hooks: HashSet::new(),
            timeout_seconds: 30,
        }
    }
}

impl HookConfig {
    /// 从环境变量创建配置
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // 检查 SKIP_HOOKS 环境变量
        if let Ok(skip_hooks) = std::env::var("SKIP_HOOKS") {
            for hook_name in skip_hooks.split(',') {
                let hook_name = hook_name.trim().to_string();
                if hook_name == "all" {
                    config.skip_all = true;
                } else {
                    config.skip_hooks.insert(hook_name);
                }
            }
        }

        config
    }

    /// 检查是否应该跳过指定 hook
    pub fn should_skip(&self, hook_name: &str) -> bool {
        self.skip_all || self.skip_hooks.contains(hook_name)
    }
}
```

### 5. 工具检测器

```rust
// crates/storage/src/git/services/hooks/tools/detector.rs

use std::path::{Path, PathBuf};

/// 检测到的工具类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookTool {
    /// prek (Rust，pre-commit 的高性能替代)
    Prek,
    /// pre-commit (Python)
    PreCommit,
    /// 标准 Git hooks
    Standard,
}

/// 工具检测结果
#[derive(Debug, Clone)]
pub struct ToolDetectionResult {
    pub tool: HookTool,
    pub config_path: Option<PathBuf>,
    pub executable_path: Option<PathBuf>,
}

pub struct ToolDetector {
    repo_path: PathBuf,
}

impl ToolDetector {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    /// 检测项目中使用的 hooks 工具
    ///
    /// 按优先级返回检测结果（优先级高的在前）
    pub fn detect_tools(&self, hook_name: &str) -> Vec<ToolDetectionResult> {
        let mut results = Vec::new();

        // 1. 检测 prek/pre-commit
        if let Some(result) = self.detect_pre_commit_prek(hook_name) {
            results.push(result);
        }

        // 2. 标准 Git hooks（总是检查，作为兜底）
        // 注意：HookDiscoverer 会自动检查 core.hooksPath 配置
        // 如果 core.hooksPath 已设置，会使用该路径；否则使用 .git/hooks/
        results.push(ToolDetectionResult {
            tool: HookTool::Standard,
            config_path: None,
            executable_path: None,
        });

        results
    }

    /// 检测 prek/pre-commit
    fn detect_pre_commit_prek(&self, hook_name: &str) -> Option<ToolDetectionResult> {
        let config_path = self.repo_path.join(".pre-commit-config.yaml");

        if !config_path.exists() {
            return None;
        }

        // 优先使用 prek（性能更好，快 7 倍）
        // 注意：需要添加 which crate 依赖
        if let Ok(path) = which::which("prek") {
            return Some(ToolDetectionResult {
                tool: HookTool::Prek,
                config_path: Some(config_path),
                executable_path: Some(path),
            });
        }

        // 回退到 pre-commit
        if let Ok(path) = which::which("pre-commit") {
            return Some(ToolDetectionResult {
                tool: HookTool::PreCommit,
                config_path: Some(config_path),
                executable_path: Some(path),
            });
        }

        // 配置文件存在但工具不可用，记录警告但返回 None
        // 实际使用时会在日志中记录警告，但不阻止操作
        None
    }
}
```

### 6. pre-commit/prek 执行器

```rust
// crates/storage/src/git/services/hooks/tools/pre_commit.rs

use std::process::Command;
use std::path::PathBuf;
use domain::git::GitError;
use super::super::{HookContext, HookResult};

pub struct PreCommitExecutor {
    tool_path: PathBuf,
    config_path: PathBuf,
    is_prek: bool,
}

impl PreCommitExecutor {
    pub fn new(tool_path: PathBuf, config_path: PathBuf, is_prek: bool) -> Self {
        Self {
            tool_path,
            config_path,
            is_prek,
        }
    }

    /// 执行 pre-commit/prek hook
    pub fn execute(&self, hook_name: &str, context: &HookContext) -> Result<HookResult, GitError> {
        let mut cmd = Command::new(&self.tool_path);

        // pre-commit/prek 的命令行参数
        cmd.arg("run")
            .arg("--hook")
            .arg(hook_name)
            .arg("--all-files"); // 运行所有文件的检查

        // 设置工作目录
        cmd.current_dir(&context.repo_path);

        // 设置环境变量
        cmd.env("GIT_DIR", &context.git_dir);
        cmd.env("GIT_WORK_TREE", &context.repo_path);

        // 对于 pre-commit hook，传递暂存区文件列表
        if hook_name == "pre-commit" && !context.staged_files.is_empty() {
            // pre-commit 会自动检测暂存区文件，但我们可以通过环境变量传递
            cmd.env("PRE_COMMIT_ALL_FILES", "1");
        }

        // 执行命令
        let output = cmd.output()
            .map_err(|e| GitError::OperationFailed(format!("Failed to execute {}: {}",
                if self.is_prek { "prek" } else { "pre-commit" }, e)))?;

        // 处理输出
        if !output.stdout.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        // 检查结果
        if output.status.success() {
            Ok(HookResult::Success)
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(GitError::HookFailed(format!("{} hook failed: {}",
                if self.is_prek { "prek" } else { "pre-commit" }, error_msg)))
        }
    }
}
```

### 7. Hook 服务实现（更新版，支持工具兼容）

```rust
// crates/storage/src/git/services/hooks/mod.rs

use std::sync::Arc;
use std::time::Duration;
use domain::git::GitError;
use super::discoverer::HookDiscoverer;
use super::executor::HookExecutor;
use super::config::HookConfig;
use super::tools::detector::{ToolDetector, HookTool};
use super::tools::pre_commit::PreCommitExecutor;

pub struct HookServiceImpl {
    discoverer: HookDiscoverer,
    executor: HookExecutor,
    tool_detector: ToolDetector,
    config: HookConfig,
    repo_path: std::path::PathBuf,
}

impl HookServiceImpl {
    pub fn new(
        git_dir: std::path::PathBuf,
        repo_path: std::path::PathBuf,
        config: HookConfig,
    ) -> Self {
        let discoverer = HookDiscoverer::new(git_dir.clone(), repo_path.clone());
        let executor = HookExecutor::new(Duration::from_secs(config.timeout_seconds));
        let tool_detector = ToolDetector::new(repo_path.clone());

        Self {
            discoverer,
            executor,
            tool_detector,
            config,
            repo_path,
        }
    }
}

impl HookService for HookServiceImpl {
    fn execute_hook(
        &self,
        hook_name: &str,
        context: &HookContext,
        skip_if_missing: bool,
    ) -> Result<HookResult, GitError> {
        // 检查是否应该跳过
        if self.config.should_skip(hook_name) {
            return Ok(HookResult::Success);
        }

        // 检测工具（按优先级）
        let tool_results = self.tool_detector.detect_tools(hook_name);

        // 按优先级执行 hooks
        for tool_result in tool_results {
            match tool_result.tool {
                HookTool::Prek | HookTool::PreCommit => {
                    // 执行 prek/pre-commit
                    if let (Some(config_path), Some(executable_path)) =
                        (tool_result.config_path, tool_result.executable_path) {
                        let is_prek = tool_result.tool == HookTool::Prek;
                        let executor = PreCommitExecutor::new(
                            executable_path,
                            config_path,
                            is_prek,
                        );

                        match executor.execute(hook_name, context) {
                            Ok(_) => {
                                // prek/pre-commit 成功，继续检查标准 hooks
                                continue;
                            }
                            Err(e) => {
                                // prek/pre-commit 失败，阻止操作
                                return Err(e);
                            }
                        }
                    }
                }

                HookTool::Standard => {
                    // 执行标准 Git hooks
                    // HookDiscoverer 会自动检查 core.hooksPath 配置
                    // 如果 core.hooksPath 已设置，会使用该路径；否则使用 .git/hooks/
                    match self.discoverer.find_hook(hook_name)? {
                        Some(hook_path) => {
                            match self.executor.execute(&hook_path, context) {
                                Ok(result) => {
                                    return Ok(result);
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        None => {
                            // 标准 hook 不存在
                            if skip_if_missing {
                                return Ok(HookResult::Success);
                            } else {
                                return Err(GitError::OperationFailed(
                                    format!("Hook {} not found", hook_name)
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 所有工具都检查完毕，没有找到 hook
        if skip_if_missing {
            Ok(HookResult::Success)
        } else {
            Err(GitError::OperationFailed(
                format!("Hook {} not found", hook_name)
            ))
        }
    }

    fn should_skip_hooks(&self) -> bool {
        self.config.skip_all
    }

    fn should_skip_hook(&self, hook_name: &str) -> bool {
        self.config.should_skip(hook_name)
    }
}
```

---

## 🔌 集成方案

### 1. 在 CommitService 中集成

```rust
// crates/storage/src/git/services/commit.rs

impl CommitService for CommitServiceImpl {
    fn commit(&self, message: &str, all: bool) -> Result<String, GitError> {
        let repo = self.ctx.repository();
        let git_dir = repo.path().to_path_buf();
        let repo_path = repo.workdir()
            .ok_or_else(|| GitError::OperationFailed("Not a work tree".into()))?
            .to_path_buf();

        // 创建 hook 服务
        let hook_config = HookConfig::from_env();
        let hook_service = HookServiceImpl::new(git_dir.clone(), repo_path.clone(), hook_config);

        let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        // 添加更改到暂存区
        if all {
            // ... 现有代码 ...
        }

        index.write().map_err(|e| GitError::IndexError(e.to_string()))?;

        // [1] pre-commit hook
        let staged_files = self.get_staged_files(&index)?;
        let hook_context = HookContext {
            repo_path: repo_path.clone(),
            git_dir: git_dir.clone(),
            staged_files,
            commit_message: None,
            commit_sha: None,
            branch_name: None,
            remote_url: None,
            commits_to_push: Vec::new(),
            has_conflicts: None,
        };

        match hook_service.execute_hook("pre-commit", &hook_context, true)? {
            HookResult::Failure(msg) => {
                return Err(GitError::HookFailed(format!("pre-commit hook failed: {}", msg)));
            }
            HookResult::Modified => {
                // Hook 修改了文件，重新读取索引
                index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;
                index.write().map_err(|e| GitError::IndexError(e.to_string()))?;
            }
            HookResult::Success => {}
        }

        // [2] prepare-commit-msg hook（可选）
        let mut final_message = message.to_string();
        let mut prepare_context = hook_context.clone();
        prepare_context.commit_message = Some(message.to_string());

        if let Ok(HookResult::Success) = hook_service.execute_hook("prepare-commit-msg", &prepare_context, true) {
            // 如果 hook 修改了消息，可以从环境变量或文件读取
            // 这里简化处理，实际需要根据 hook 规范实现
        }

        // [3] 执行实际的 git2 commit
        let tree_id = index.write_tree().map_err(|e| GitError::IndexError(e.to_string()))?;
        let tree = repo.find_tree(tree_id).map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let parent_commit = repo.head().and_then(|head| head.peel_to_commit()).ok();

        if let Some(ref parent) = parent_commit {
            let parent_tree = parent.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;
            if tree_id == parent_tree.id() {
                return Err(GitError::OperationFailed("Nothing to commit".into()));
            }
        }

        let signature = self.ctx.get_signature()?;
        let oid = if let Some(parent) = parent_commit {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &final_message,
                &tree,
                &[&parent],
            )
            .map_err(|e| GitError::OperationFailed(e.to_string()))?
        } else {
            repo.commit(Some("HEAD"), &signature, &signature, &final_message, &tree, &[])
                .map_err(|e| GitError::OperationFailed(e.to_string()))?
        };

        // [4] commit-msg hook
        let mut commit_msg_context = hook_context.clone();
        commit_msg_context.commit_message = Some(final_message.clone());
        commit_msg_context.commit_sha = Some(oid.to_string());

        match hook_service.execute_hook("commit-msg", &commit_msg_context, true)? {
            HookResult::Failure(msg) => {
                // commit-msg 失败，需要回滚提交（这里简化处理）
                return Err(GitError::HookFailed(format!("commit-msg hook failed: {}", msg)));
            }
            _ => {}
        }

        // [5] post-commit hook
        let mut post_commit_context = hook_context;
        post_commit_context.commit_sha = Some(oid.to_string());

        // post-commit 失败不影响提交（已提交成功）
        let _ = hook_service.execute_hook("post-commit", &post_commit_context, true);

        Ok(oid.to_string())
    }

    fn get_staged_files(&self, index: &git2::Index) -> Result<Vec<String>, GitError> {
        let mut files = Vec::new();
        for entry in index.iter() {
            if let Some(path) = entry.path {
                files.push(path.to_string_lossy().to_string());
            }
        }
        Ok(files)
    }
}
```

### 2. 在 RemoteService 中集成

```rust
// crates/storage/src/git/services/remote.rs

impl RemoteService for RemoteServiceImpl {
    fn push(&self, branch_name: &str, set_upstream: bool) -> Result<(), GitError> {
        let repo = self.ctx.repository();
        let git_dir = repo.path().to_path_buf();
        let repo_path = repo.workdir()
            .ok_or_else(|| GitError::OperationFailed("Not a work tree".into()))?
            .to_path_buf();

        // 创建 hook 服务
        let hook_config = HookConfig::from_env();
        let hook_service = HookServiceImpl::new(git_dir.clone(), repo_path.clone(), hook_config);

        // [1] pre-push hook
        let commits_to_push = self.get_commits_to_push(&repo, branch_name)?;
        let remote_url = self.get_remote_url(&repo)?;

        let hook_context = HookContext {
            repo_path: repo_path.clone(),
            git_dir: git_dir.clone(),
            staged_files: Vec::new(),
            commit_message: None,
            commit_sha: None,
            branch_name: Some(branch_name.to_string()),
            remote_url: Some(remote_url),
            commits_to_push,
            has_conflicts: None,
        };

        match hook_service.execute_hook("pre-push", &hook_context, true)? {
            HookResult::Failure(msg) => {
                return Err(GitError::HookFailed(format!("pre-push hook failed: {}", msg)));
            }
            _ => {}
        }

        // [2] 执行实际的 git2 push
        let mut remote = repo.find_remote("origin")
            .map_err(|e| GitError::RemoteError(e.to_string()))?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        let callbacks = GitContext::create_callbacks();
        let mut opts = PushOptions::new();
        opts.remote_callbacks(callbacks);

        remote.push(&[&refspec], Some(&mut opts))
            .map_err(|e| GitError::RemoteError(e.to_string()))?;

        // 设置上游跟踪分支
        if set_upstream {
            let mut branch = repo.find_branch(branch_name, git2::BranchType::Local)
                .map_err(|_| GitError::BranchNotFound(branch_name.to_string()))?;
            let upstream_name = format!("origin/{}", branch_name);
            branch.set_upstream(Some(&upstream_name))
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        }

        Ok(())
    }
}
```

---

## 🧪 测试计划

### 1. 单元测试

#### 1.1 HookDiscoverer 测试
- ✅ 发现存在的 hook
- ✅ 跳过不存在的 hook
- ✅ 跳过 .sample 文件
- ✅ 检查可执行权限

#### 1.2 HookExecutor 测试
- ✅ 执行简单的 bash 脚本
- ✅ 处理脚本输出
- ✅ 处理脚本失败
- ✅ 超时控制

#### 1.3 HookConfig 测试
- ✅ 从环境变量读取配置
- ✅ 跳过特定 hook
- ✅ 跳过所有 hooks

### 2. 集成测试

#### 2.1 Commit 流程测试
- ✅ pre-commit hook 成功，提交继续
- ✅ pre-commit hook 失败，提交被阻止
- ✅ pre-commit hook 修改文件，文件被重新暂存
- ✅ commit-msg hook 验证消息格式
- ✅ post-commit hook 执行后置操作

#### 2.2 Push 流程测试
- ✅ pre-push hook 成功，推送继续
- ✅ pre-push hook 失败，推送被阻止

#### 2.3 跳过机制测试
- ✅ `--no-verify` 跳过所有 hooks
- ✅ 环境变量 `SKIP_HOOKS=pre-commit` 跳过特定 hook

### 3. 兼容性测试

#### 3.1 标准 Git Hooks
- ✅ 执行项目中现有的 `pre-commit` 脚本
- ✅ 验证与标准 Git 命令行为一致
- ✅ 跨平台测试（Unix、Windows）

#### 3.2 第三方工具兼容性测试
- ✅ **prek/pre-commit**：
  - 检测 `.pre-commit-config.yaml` 配置文件
  - 优先使用 prek（如果可用）
  - 回退到 pre-commit（如果 prek 不可用）
  - 执行 hooks 并验证结果
  - 验证工具不可用时警告但不阻止操作
- ✅ **标准 hooks 与 core.hooksPath**：
  - 验证 `HookDiscoverer` 能正确检查 `core.hooksPath` 配置
  - 如果 `core.hooksPath` 已设置，从该路径读取 hooks
  - 如果 `core.hooksPath` 未设置，从 `.git/hooks/` 读取
- ✅ **工具共存**：
  - 测试 prek/pre-commit 和标准 hooks 同时存在的情况
  - 验证执行优先级正确（prek/pre-commit 先执行）
  - 验证所有 hooks 都执行

---

## 📅 实施计划

### 阶段 1：核心功能（P0）

**目标**：实现基本的 hook 发现和执行机制

**任务**：
1. ✅ 创建 `hooks` 模块结构
2. ✅ 实现 `HookDiscoverer`（支持 `core.hooksPath` 配置）
3. ✅ 实现 `HookExecutor`
4. ✅ 实现 `HookConfig`
5. ✅ 实现 `ToolDetector`（工具检测器，仅检测 prek/pre-commit）
6. ✅ 实现 `PreCommitExecutor`
7. ✅ 实现 `HookService` trait 和 `HookServiceImpl`（支持 prek/pre-commit 和标准 hooks）

**预计时间**：2-3 天

### 阶段 2：Commit 集成（P0）

**目标**：在 `CommitService` 中集成 hooks

**任务**：
1. ✅ 集成 `pre-commit` hook
2. ✅ 集成 `commit-msg` hook
3. ✅ 集成 `post-commit` hook
4. ✅ 处理 hook 修改文件的情况

**预计时间**：2-3 天

### 阶段 3：Push 集成（P1）

**目标**：在 `RemoteService` 中集成 hooks

**任务**：
1. ✅ 集成 `pre-push` hook
2. ✅ 实现获取要推送的提交列表

**预计时间**：1-2 天

### 阶段 4：其他 Hooks（P2）

**目标**：实现其他 hooks

**任务**：
1. ✅ 集成 `prepare-commit-msg` hook
2. ✅ 集成 `post-merge` hook
3. ✅ 集成 `post-checkout` hook
4. ✅ 集成 `pre-rebase` hook

**预计时间**：2-3 天

### 阶段 5：测试和优化（P0）

**目标**：完善测试和优化性能

**任务**：
1. ✅ 编写单元测试
2. ✅ 编写集成测试
3. ✅ 性能优化
4. ✅ 文档完善

**预计时间**：2-3 天

**总计预计时间**：8-12 天（包含 prek/pre-commit 兼容性实现）

---

## 📝 使用示例

### 1. 基本使用

```rust
// 用户执行 commit，hooks 自动执行
let git_repo = registry::get_git_repository();
git_repo.commit("feat: add new feature", true)?;
// pre-commit hook 自动执行
// commit-msg hook 自动执行
// post-commit hook 自动执行
```

### 2. 跳过 Hooks

```rust
// 方式 1: 使用环境变量
std::env::set_var("SKIP_HOOKS", "pre-commit");
git_repo.commit("feat: add new feature", true)?;

// 方式 2: 在 CLI 中添加 --no-verify 参数（需要实现）
// workflow commit create --no-verify "feat: add new feature"
```

### 3. Hook 脚本示例

```bash
#!/bin/bash
# .git/hooks/pre-commit

# 运行代码格式化
cargo fmt --all

# 运行 Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 如果检查失败，退出码 1 会阻止提交
```

### 4. 第三方工具兼容性示例

#### 4.1 使用 pre-commit/prek

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
```

```rust
// Workflow CLI 会自动检测并执行 pre-commit/prek hooks
let git_repo = registry::get_git_repository();
git_repo.commit("feat: add new feature", true)?;
// 自动执行: prek run --hook pre-commit（如果 prek 可用）
// 或: pre-commit run --hook pre-commit（如果 prek 不可用）
```

#### 4.2 工具优先级示例

假设项目中同时存在：
- `.pre-commit-config.yaml`（prek/pre-commit 配置）
- `.git/hooks/pre-commit`（标准 Git hook）

执行顺序：
1. ✅ 首先执行 prek/pre-commit hooks（如果工具可用）
   - 优先使用 prek（如果已安装）
   - 回退到 pre-commit（如果 prek 不可用）
2. ✅ 如果通过，执行标准 Git hooks
   - `HookDiscoverer` 检查 `core.hooksPath` 配置
   - 如果 `core.hooksPath` 已设置，从该路径读取 hooks
   - 如果 `core.hooksPath` 未设置，从 `.git/hooks/pre-commit` 读取
3. ✅ 所有 hooks 都通过后，继续提交

**重要说明**：
- prek/pre-commit 和标准 hooks 可以**共存**：先执行 prek/pre-commit，再执行标准 hooks
- 如果 prek/pre-commit hooks 失败，会阻止操作，不会继续执行标准 hooks
- 标准 hooks 支持 `core.hooksPath` 配置（兼容其他工具设置的路径）
- 如果任何 hook 失败，操作会被阻止

---

## 📦 依赖要求

### 新增依赖

需要在 `crates/storage/Cargo.toml` 中添加：

```toml
[dependencies]
# 用于检测命令行工具是否存在
which = "6.0"
```

### 可选依赖

如果需要在 Windows 上更好地处理路径，可以考虑：

```toml
[dependencies]
# Windows 路径处理（如果需要）
path-slash = "0.2"  # 可选
```

## 🔗 相关文档

- [Git Hooks 官方文档](https://git-scm.com/docs/githooks)
- [pre-commit 文档](https://pre-commit.com/)
- [prek 文档](https://prek.j178.dev/)
- [项目架构文档](../architecture.md)
- [开发规范](../development.md)

---

## 📌 注意事项

1. **兼容性**：
   - 确保与标准 Git hooks 完全兼容（`.git/hooks/` 或 `core.hooksPath` 指向的目录）
   - 兼容 prek/pre-commit 工具（通过 `.pre-commit-config.yaml` 配置）
   - 支持工具共存，按优先级执行（prek/pre-commit > 标准 hooks）

2. **工具检测**：
   - 自动检测 prek/pre-commit 工具（通过 `.pre-commit-config.yaml` 配置文件）
   - 优先使用性能更好的工具（prek > pre-commit）
   - 如果工具不可用但配置文件存在，记录警告但不阻止操作

3. **执行优先级**：
   - prek/pre-commit（最高优先级，如果检测到工具和配置文件）
   - 标准 Git hooks（.git/hooks/ 或 `core.hooksPath` 指向的目录）
   - 所有 hooks 必须全部通过才能继续操作

4. **性能**：
   - Hook 执行不应显著影响操作速度
   - prek 比 pre-commit 快 7 倍，优先使用 prek
   - 支持 hook 超时控制（防止无限执行）

5. **安全性**：
   - 只执行 hooks 目录下的脚本（`.git/hooks/` 或 `core.hooksPath` 指向的目录）
   - 只调用已安装的命令行工具（prek/pre-commit）
   - 验证工具路径和配置文件路径

6. **错误处理**：
   - `pre-*` hooks 失败：阻止操作
   - `post-*` hooks 失败：记录错误但不阻止操作（操作已完成）
   - 工具不可用：记录警告但不阻止操作

7. **跨平台**：
   - 考虑 Unix 和 Windows 的差异（可执行权限、脚本类型）
   - 使用 `which` crate 检测命令行工具
   - 正确处理路径分隔符

---

**最后更新**: 2025-02-05
