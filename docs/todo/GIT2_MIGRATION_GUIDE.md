# Git 命令到 git2 迁移指南

## 概述

本文档记录了将项目中的 Git 模块从命令行调用方式迁移到 `git2` (libgit2 Rust 绑定) 的详细计划和实施指南。

## 迁移目标

- **性能提升**：消除进程启动开销，提升 10-100 倍性能
- **部署简化**：减少对系统 Git 安装的依赖（运行时）
- **类型安全**：使用强类型 API 减少运行时错误
- **功能完整性**：覆盖约 95% 的 Git 操作（包括 rebase 和 cherry-pick）

## 迁移范围

### 需要迁移的模块

1. **提交管理模块** (`src/lib/git/commit.rs`) - ~20 个操作
2. **分支管理模块** (`src/lib/git/branch.rs`) - ~25 个操作
3. **Tag 管理模块** (`src/lib/git/tag.rs`) - ~8 个操作
4. **配置管理模块** (`src/lib/git/config.rs`) - ~4 个操作
5. **Stash 管理模块** (`src/lib/git/stash.rs`) - ~6 个操作

### 可以迁移到 git2 的高级操作

1. **Rebase 操作** - ✅ git2 原生支持
   - `src/lib/git/branch.rs` - `rebase_onto()`, `rebase_onto_with_upstream()`
   - 使用 `Repository::rebase()` API 实现
   - ⚠️ 交互式 rebase（squash、reword）可能需要额外实现或保留命令行

2. **Cherry-pick 操作** - ✅ git2 支持（通过 `merge_commits()`）
   - `src/lib/git/cherry_pick.rs` - 所有操作
   - 使用 `Repository::merge_commits()` 实现三向合并

### 需要保留命令行的操作

1. **Pre-commit 工具** - 独立 Python 工具
   - `src/lib/git/pre_commit.rs` - 保留外部调用

2. **Git Hooks 执行** - git2 不自动执行
   - 需要手动检测并执行 hooks，或回退到命令行

## 环境准备

### 1. 添加 git2 依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
git2 = "0.18"  # 或更新版本
```

### 2. 系统依赖

**编译时依赖：**
- C 编译器（GCC、Clang 或 MSVC）
- OpenSSL 开发库（用于 HTTPS 连接）
- libssh2 开发库（用于 SSH 连接，可选）
- zlib 开发库（压缩支持）

**运行时依赖：**
- 不需要系统 git 命令
- 需要链接编译好的 libgit2 库

### 3. 跨平台支持

git2 支持跨平台，但需要：
- Linux/macOS：通常通过包管理器安装依赖
- Windows：可能需要 vcpkg 或手动安装依赖

## 迁移实施计划

### 阶段 1：基础设置和简单操作（优先级：高）

#### 1.1 添加 git2 依赖和基础工具函数

**目标：** 建立 git2 的基础设施

**步骤：**
1. 添加 git2 依赖到 `Cargo.toml`
2. 创建辅助函数用于打开仓库
3. 添加错误处理工具函数

**示例代码：**

```rust
// src/lib/git/git2_utils.rs (新建文件)
use git2::Repository;
use color_eyre::{eyre::WrapErr, Result};

/// 打开当前目录的 Git 仓库
pub fn open_repo() -> Result<Repository> {
    Repository::open(".")
        .wrap_err("Failed to open repository")
}

/// 打开指定路径的 Git 仓库
pub fn open_repo_at(path: &str) -> Result<Repository> {
    Repository::open(path)
        .wrap_err_with(|| format!("Failed to open repository at: {}", path))
}
```

#### 1.2 迁移提交管理模块 - 简单操作

**优先级：高**

| 操作 | 当前实现 | git2 实现 | 难度 |
|------|---------|----------|------|
| `get_last_commit_info()` | `git log -1` | `repo.head().peel_to_commit()` | ⭐ 简单 |
| `get_last_commit_sha()` | `git log -1 --format=%H` | `repo.head().peel_to_commit()?.id()` | ⭐ 简单 |
| `get_last_commit_message()` | `git log -1 --format=%s` | `commit.message()` | ⭐ 简单 |
| `has_last_commit()` | `git log -1 --oneline` | `repo.head().is_some()` | ⭐ 简单 |
| `parse_commit_ref()` | `git rev-parse` | `repo.revparse_single()` | ⭐ 简单 |
| `get_commit_info()` | `git log -1` | `repo.find_commit()` | ⭐ 简单 |
| `get_parent_commit()` | `git rev-parse <sha>^` | `commit.parent(0)` | ⭐ 简单 |

**迁移示例：**

```rust
// 迁移前
pub fn get_last_commit_info() -> Result<CommitInfo> {
    let sha = GitCommand::new(["log", "-1", "--format=%H"]).read()?;
    let message = GitCommand::new(["log", "-1", "--format=%s"]).read()?;
    let author = GitCommand::new(["log", "-1", "--format=%an <%ae>"]).read()?;
    let date = GitCommand::new(["log", "-1", "--format=%ai"]).read()?;
    // ...
}

// 迁移后
pub fn get_last_commit_info() -> Result<CommitInfo> {
    use crate::git::git2_utils::open_repo;

    let repo = open_repo()?;
    let head = repo.head().wrap_err("Failed to get HEAD")?;
    let commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

    let sha = commit.id().to_string();
    let message = commit
        .message()
        .ok_or_else(|| color_eyre::eyre::eyre!("Commit has no message"))?
        .to_string();
    let author = commit.author();
    let author_str = format!("{} <{}>", author.name().unwrap_or(""), author.email().unwrap_or(""));
    let date = commit.time();
    let date_str = format!("{}", chrono::DateTime::<chrono::FixedOffset>::from_timestamp(
        date.seconds(),
        date.sign().offset_minutes() as i32 * 60
    ).unwrap_or_default());

    Ok(CommitInfo {
        sha,
        message: message.lines().next().unwrap_or("").to_string(),
        author: author_str,
        date: date_str,
    })
}
```

#### 1.3 迁移配置管理模块

**优先级：高**

| 操作 | 当前实现 | git2 实现 | 难度 |
|------|---------|----------|------|
| `set_user_email()` | `git config --global user.email` | `Config::open_default()?.set_str()` | ⭐ 简单 |
| `set_user_name()` | `git config --global user.name` | `Config::open_default()?.set_str()` | ⭐ 简单 |
| `get_user_email()` | `git config --global user.email` | `Config::open_default()?.get_string()` | ⭐ 简单 |
| `get_user_name()` | `git config --global user.name` | `Config::open_default()?.get_string()` | ⭐ 简单 |

**迁移示例：**

```rust
// 迁移前
pub fn set_user_email(email: &str) -> Result<()> {
    GitCommand::new(["config", "--global", "user.email", email])
        .run()
        .wrap_err("Failed to set user email")
}

// 迁移后
pub fn set_user_email(email: &str) -> Result<()> {
    use git2::Config;

    let mut config = Config::open_default()
        .wrap_err("Failed to open git config")?;
    config.set_str("user.email", email)
        .wrap_err("Failed to set user.email")?;
    Ok(())
}
```

### 阶段 2：核心操作（优先级：高）

#### 2.1 迁移提交管理模块 - 核心操作

**优先级：高**

| 操作 | 当前实现 | git2 实现 | 难度 |
|------|---------|----------|------|
| `add_all()` | `git add --all` | `index.add_all()` | ⭐ 简单 |
| `add_files()` | `git add <files>` | `index.add_path()` | ⭐ 简单 |
| `has_staged()` | `git diff --cached --quiet` | `index.write_tree()` 比较 | ⭐⭐ 中等 |
| `status()` | `git status --porcelain` | `repo.status()` 或 `repo.diff_index_to_workdir()` | ⭐⭐ 中等 |
| `has_commit()` | `git diff --quiet` | `repo.status()` 检查 | ⭐⭐ 中等 |
| `commit()` | `git commit -m` | `repo.commit()` | ⭐⭐ 中等 |
| `amend()` | `git commit --amend` | `repo.commit()` + `repo.reset()` | ⭐⭐⭐ 复杂 |

**迁移示例 - add_all()：**

```rust
// 迁移前
pub fn add_all() -> Result<()> {
    GitCommand::new(["add", "--all"]).run().wrap_err("Failed to add all files")
}

// 迁移后
pub fn add_all() -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::IndexAddOption;

    let repo = open_repo()?;
    let mut index = repo.index().wrap_err("Failed to open index")?;

    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .wrap_err("Failed to add all files to index")?;
    index.write().wrap_err("Failed to write index")?;

    Ok(())
}
```

**迁移示例 - status()：**

```rust
// 迁移前
pub fn status() -> Result<String> {
    GitCommand::new(["status", "--porcelain"])
        .read()
        .wrap_err("Failed to run git status")
}

// 迁移后
pub fn status() -> Result<String> {
    use crate::git::git2_utils::open_repo;
    use git2::StatusOptions;

    let repo = open_repo()?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))
        .wrap_err("Failed to get status")?;

    let mut output = Vec::new();
    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("");

        // 转换为 porcelain 格式
        let status_str = if status.is_index_new() || status.is_index_modified() {
            if status.is_index_new() { "A " } else { "M " }
        } else if status.is_index_deleted() {
            "D "
        } else if status.is_wt_modified() {
            " M"
        } else if status.is_wt_deleted() {
            " D"
        } else if status.is_wt_new() {
            "??"
        } else {
            "  "
        };

        output.push(format!("{}{}", status_str, path));
    }

    Ok(output.join("\n"))
}
```

**迁移示例 - commit()：**

```rust
// 迁移前
pub fn commit(message: &str, no_verify: bool) -> Result<CommitResult> {
    // ... 检查更改 ...
    Self::add_all().wrap_err("Failed to stage changes")?;
    // ... pre-commit hooks ...
    let mut args = vec!["commit", "-m", message];
    if no_verify {
        args.push("--no-verify");
    }
    GitCommand::new(args).run().wrap_err("Failed to commit")?;
    // ...
}

// 迁移后
pub fn commit(message: &str, no_verify: bool) -> Result<CommitResult> {
    use crate::git::git2_utils::open_repo;
    use git2::{Signature, IndexAddOption};

    // 1. 检查是否有更改
    let has_changes = Self::has_commit()?;
    if !has_changes {
        return Ok(CommitResult {
            committed: false,
            message: Some("Nothing to commit, working tree clean".to_string()),
        });
    }

    let repo = open_repo()?;

    // 2. 暂存所有文件
    Self::add_all().wrap_err("Failed to stage changes")?;

    // 3. Pre-commit hooks（如果启用）
    let should_skip_hook = if !no_verify && GitPreCommit::has_pre_commit() {
        GitPreCommit::run_pre_commit()?;
        true
    } else {
        false
    };

    // 4. 如果 no_verify 为 false 且存在 Git hooks，使用命令行以触发 hooks
    let has_git_hooks = repo.path().join("hooks").join("pre-commit").exists();
    if !no_verify && has_git_hooks && !should_skip_hook {
        // 使用命令行执行提交以触发 Git hooks
        let mut args = vec!["commit", "-m", message];
        GitCommand::new(args).run().wrap_err("Failed to commit")?;
    } else {
        // 使用 git2 执行提交
        let mut index = repo.index().wrap_err("Failed to open index")?;
        let tree_id = index.write_tree().wrap_err("Failed to write tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

        // 获取 HEAD commit（如果存在）
        let parent_commit = repo.head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok());

        // 获取签名
        let config = repo.config().ok();
        let name = config.as_ref()
            .and_then(|c| c.get_string("user.name").ok())
            .unwrap_or_else(|| "Unknown".to_string());
        let email = config.as_ref()
            .and_then(|c| c.get_string("user.email").ok())
            .unwrap_or_else(|| "unknown@example.com".to_string());
        let signature = Signature::now(&name, &email)
            .wrap_err("Failed to create signature")?;

        // 创建 commit
        let parents = if let Some(parent) = parent_commit {
            vec![&parent]
        } else {
            vec![]
        };

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )
        .wrap_err("Failed to create commit")?;
    }

    Ok(CommitResult {
        committed: true,
        message: None,
    })
}
```

#### 2.2 迁移 Tag 管理模块

**优先级：中**

| 操作 | 当前实现 | git2 实现 | 难度 |
|------|---------|----------|------|
| `list_local_tags()` | `git tag -l` | `repo.tag_names()` | ⭐ 简单 |
| `list_remote_tags()` | `git ls-remote --tags` | `repo.find_remote()` + `remote.ls()` | ⭐⭐ 中等 |
| `tag_exists()` | `git rev-parse --verify` | `repo.find_reference()` | ⭐ 简单 |
| `delete_local_tag()` | `git tag -d` | `repo.tag_delete()` | ⭐ 简单 |
| `delete_remote_tag()` | `git push origin --delete` | `remote.push()` | ⭐⭐ 中等 |

**迁移示例：**

```rust
// 迁移前
pub fn list_local_tags() -> Result<Vec<String>> {
    let output = GitCommand::new(["tag", "-l"]).read()?;
    // ...
}

// 迁移后
pub fn list_local_tags() -> Result<Vec<String>> {
    use crate::git::git2_utils::open_repo;

    let repo = open_repo()?;
    let tag_names = repo.tag_names(None)
        .wrap_err("Failed to get tag names")?;

    let mut tags = Vec::new();
    for name in tag_names.iter() {
        if let Some(tag) = name {
            tags.push(tag.to_string());
        }
    }

    Ok(tags)
}
```

#### 2.3 迁移 Stash 管理模块

**优先级：中**

| 操作 | 当前实现 | git2 实现 | 难度 |
|------|---------|----------|------|
| `stash_push()` | `git stash push` | `repo.stash_save()` | ⭐ 简单 |
| `stash_list()` | `git stash list` | `repo.stash_foreach()` | ⭐⭐ 中等 |
| `stash_apply()` | `git stash apply` | `repo.stash_apply()` | ⭐⭐ 中等 |
| `stash_drop()` | `git stash drop` | `repo.stash_drop()` | ⭐ 简单 |
| `stash_pop()` | `git stash pop` | `repo.stash_pop()` | ⭐⭐ 中等 |

**迁移示例：**

```rust
// 迁移前
pub fn stash_push(message: Option<&str>) -> Result<String> {
    let mut args = vec!["stash", "push"];
    if let Some(msg) = message {
        args.push("-m");
        args.push(msg);
    }
    GitCommand::new(args).run()?;
    // ...
}

// 迁移后
pub fn stash_push(message: Option<&str>) -> Result<String> {
    use crate::git::git2_utils::open_repo;
    use git2::Signature;

    let repo = open_repo()?;

    // 获取签名
    let config = repo.config().ok();
    let name = config.as_ref()
        .and_then(|c| c.get_string("user.name").ok())
        .unwrap_or_else(|| "Unknown".to_string());
    let email = config.as_ref()
        .and_then(|c| c.get_string("user.email").ok())
        .unwrap_or_else(|| "unknown@example.com".to_string());
    let signature = Signature::now(&name, &email)
        .wrap_err("Failed to create signature")?;

    let stash_id = repo.stash_save(
        &signature,
        message.unwrap_or(""),
        None,
    )
    .wrap_err("Failed to stash changes")?;

    Ok(format!("stash@{{{}}}", stash_id))
}
```

### 阶段 3：复杂操作（优先级：中）

#### 3.1 迁移分支管理模块

**优先级：中**

| 操作 | 当前实现 | git2 实现 | 难度 |
|------|---------|----------|------|
| `get_current_branch()` | `git branch --show-current` | `repo.head().shorthand()` | ⭐ 简单 |
| `branch_exists()` | `git branch -l` | `repo.find_branch()` | ⭐ 简单 |
| `create_and_switch()` | `git checkout -b` | `repo.branch()` + `repo.checkout_tree()` | ⭐⭐ 中等 |
| `switch_to_branch()` | `git checkout` | `repo.checkout_tree()` + `repo.set_head()` | ⭐⭐ 中等 |
| `get_default_branch()` | `git ls-remote --symref` | `repo.find_remote()` + `remote.ls()` | ⭐⭐⭐ 复杂 |
| `pull_latest()` | `git pull origin` | `remote.fetch()` + `repo.merge()` | ⭐⭐⭐ 复杂 |
| `force_push()` | `git push --force-with-lease` | `remote.push()` | ⭐⭐ 中等 |
| `delete_remote_branch()` | `git push origin --delete` | `remote.push()` | ⭐⭐ 中等 |
| `merge()` | `git merge` | `repo.merge()` | ⭐⭐ 中等 |
| `rebase_onto()` | `git rebase` | `repo.rebase()` | ⭐⭐⭐ 复杂 |
| `rebase_onto_with_upstream()` | `git rebase --onto` | `repo.rebase()` | ⭐⭐⭐ 复杂 |

**迁移示例 - get_current_branch()：**

```rust
// 迁移前
pub fn get_current_branch() -> Result<String> {
    GitCommand::new(["branch", "--show-current"])
        .read()
        .wrap_err("Failed to get current branch")
}

// 迁移后
pub fn get_current_branch() -> Result<String> {
    use crate::git::git2_utils::open_repo;

    let repo = open_repo()?;
    let head = repo.head().wrap_err("Failed to get HEAD")?;
    let branch_name = head.shorthand()
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get branch name"))?;

    Ok(branch_name.to_string())
}
```

**迁移示例 - switch_to_branch()：**

```rust
// 迁移前
pub fn switch_to_branch(branch_name: &str) -> Result<()> {
    GitCommand::new(["checkout", branch_name])
        .run()
        .wrap_err_with(|| format!("Failed to switch to branch: {}", branch_name))
}

// 迁移后
pub fn switch_to_branch(branch_name: &str) -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::build::CheckoutBuilder;

    let repo = open_repo()?;
    let branch = repo.find_branch(branch_name, git2::BranchType::Local)
        .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;

    let branch_ref = branch.get();
    let commit = branch_ref.peel_to_commit()
        .wrap_err_with(|| format!("Failed to get commit for branch: {}", branch_name))?;
    let tree = repo.find_tree(commit.tree_id())
        .wrap_err("Failed to find tree")?;

    // Checkout tree
    let mut checkout_opts = CheckoutBuilder::new();
    checkout_opts.force();
    repo.checkout_tree(&tree, Some(&mut checkout_opts))
        .wrap_err("Failed to checkout tree")?;

    // Update HEAD
    repo.set_head(branch_ref.name().unwrap())
        .wrap_err("Failed to set HEAD")?;

    Ok(())
}
```

**迁移示例 - rebase_onto()：**

```rust
// 迁移前
pub fn rebase_onto(target_branch: &str) -> Result<()> {
    GitCommand::new(["rebase", target_branch])
        .run()
        .wrap_err_with(|| format!("Failed to rebase onto branch: {}", target_branch))
}

// 迁移后
pub fn rebase_onto(target_branch: &str) -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::{RebaseOptions, RebaseOperationType};

    let repo = open_repo()?;

    // 1. 获取当前分支的 HEAD
    let head = repo.head().wrap_err("Failed to get HEAD")?;
    let head_commit = head.peel_to_commit()?;
    let branch_annotated = repo.find_annotated_commit(head_commit.id())?;

    // 2. 获取目标分支
    let target_branch_ref = repo.find_branch(target_branch, git2::BranchType::Local)
        .wrap_err_with(|| format!("Failed to find branch: {}", target_branch))?;
    let target_commit = target_branch_ref.get().peel_to_commit()?;
    let target_annotated = repo.find_annotated_commit(target_commit.id())?;

    // 3. 获取上游分支（当前分支的 merge-base）
    let merge_base_oid = repo.merge_base(head_commit.id(), target_commit.id())
        .wrap_err("Failed to find merge base")?;
    let merge_base_commit = repo.find_commit(merge_base_oid)?;
    let upstream_annotated = repo.find_annotated_commit(merge_base_oid)?;

    // 4. 初始化 rebase
    let mut rebase_opts = RebaseOptions::new();
    rebase_opts.inmemory(true); // 在内存中执行，更安全
    let mut rebase = repo.rebase(
        Some(&branch_annotated),
        Some(&upstream_annotated),
        Some(&target_annotated),
        Some(&mut rebase_opts),
    )
    .wrap_err("Failed to initialize rebase")?;

    // 5. 执行 rebase 操作
    while let Some(op) = rebase.next()? {
        match op.kind() {
            RebaseOperationType::Pick => {
                // 应用提交
                let commit_id = op.id();
                let commit = repo.find_commit(commit_id)?;

                // 获取签名（使用原提交的作者和提交者）
                let author = commit.author();
                let committer = commit.committer();

                // 提交 rebase 操作
                rebase.commit(None, &author, None)
                    .wrap_err("Failed to commit rebase operation")?;
            }
            RebaseOperationType::Reword => {
                // 如果需要修改提交消息，可以在这里处理
                let commit_id = op.id();
                let commit = repo.find_commit(commit_id)?;
                let author = commit.author();
                rebase.commit(None, &author, None)?;
            }
            RebaseOperationType::Edit => {
                // 编辑操作：应用提交但不自动完成
                // 需要用户手动处理
                return Err(color_eyre::eyre::eyre!(
                    "Edit operation in rebase requires manual intervention"
                ));
            }
            _ => {
                // 其他操作类型
                return Err(color_eyre::eyre::eyre!(
                    "Unsupported rebase operation type: {:?}",
                    op.kind()
                ));
            }
        }
    }

    // 6. 完成 rebase
    rebase.finish(None)
        .wrap_err("Failed to finish rebase")?;

    Ok(())
}
```

**迁移示例 - rebase_onto_with_upstream()：**

```rust
// 迁移前
pub fn rebase_onto_with_upstream(newbase: &str, upstream: &str, branch: &str) -> Result<()> {
    GitCommand::new(["rebase", "--onto", newbase, upstream, branch])
        .run()
        .wrap_err_with(|| {
            format!(
                "Failed to rebase '{}' onto '{}' (excluding '{}' commits)",
                branch, newbase, upstream
            )
        })
}

// 迁移后
pub fn rebase_onto_with_upstream(newbase: &str, upstream: &str, branch: &str) -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::{RebaseOptions, RebaseOperationType};

    let repo = open_repo()?;

    // 1. 获取分支、上游和新基础分支
    let branch_ref = repo.find_branch(branch, git2::BranchType::Local)
        .wrap_err_with(|| format!("Failed to find branch: {}", branch))?;
    let branch_commit = branch_ref.get().peel_to_commit()?;
    let branch_annotated = repo.find_annotated_commit(branch_commit.id())?;

    let upstream_ref = repo.find_branch(upstream, git2::BranchType::Local)
        .wrap_err_with(|| format!("Failed to find upstream branch: {}", upstream))?;
    let upstream_commit = upstream_ref.get().peel_to_commit()?;
    let upstream_annotated = repo.find_annotated_commit(upstream_commit.id())?;

    let newbase_ref = repo.find_branch(newbase, git2::BranchType::Local)
        .wrap_err_with(|| format!("Failed to find newbase branch: {}", newbase))?;
    let newbase_commit = newbase_ref.get().peel_to_commit()?;
    let newbase_annotated = repo.find_annotated_commit(newbase_commit.id())?;

    // 2. 初始化 rebase（指定 onto 参数）
    let mut rebase_opts = RebaseOptions::new();
    rebase_opts.inmemory(true);
    let mut rebase = repo.rebase(
        Some(&branch_annotated),
        Some(&upstream_annotated),
        Some(&newbase_annotated),
        Some(&mut rebase_opts),
    )
    .wrap_err("Failed to initialize rebase")?;

    // 3. 执行 rebase 操作（与 rebase_onto 类似）
    while let Some(op) = rebase.next()? {
        match op.kind() {
            RebaseOperationType::Pick => {
                let commit_id = op.id();
                let commit = repo.find_commit(commit_id)?;
                let author = commit.author();
                rebase.commit(None, &author, None)
                    .wrap_err("Failed to commit rebase operation")?;
            }
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "Unsupported rebase operation type: {:?}",
                    op.kind()
                ));
            }
        }
    }

    // 4. 完成 rebase
    rebase.finish(None)
        .wrap_err("Failed to finish rebase")?;

    Ok(())
}
```

#### 3.2 迁移 Cherry-pick 操作模块

**优先级：中**

| 操作 | 当前实现 | git2 实现 | 难度 |
|------|---------|----------|------|
| `cherry_pick()` | `git cherry-pick` | `repo.merge_commits()` | ⭐⭐⭐ 复杂 |
| `cherry_pick_no_commit()` | `git cherry-pick --no-commit` | `repo.merge_commits()` + 不提交 | ⭐⭐⭐ 复杂 |
| `cherry_pick_continue()` | `git cherry-pick --continue` | 处理冲突后继续 | ⭐⭐⭐ 复杂 |
| `cherry_pick_abort()` | `git cherry-pick --abort` | `repo.state_cleanup()` | ⭐⭐ 中等 |

**迁移示例 - cherry_pick()：**

```rust
// 迁移前
pub fn cherry_pick(commit: &str) -> Result<()> {
    GitCommand::new(["cherry-pick", commit])
        .run()
        .wrap_err_with(|| format!("Failed to cherry-pick commit: {}", commit))
}

// 迁移后
pub fn cherry_pick(commit: &str) -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::{IndexAddOption, Signature};

    let repo = open_repo()?;

    // 1. 解析提交 SHA
    let commit_oid = repo.revparse_single(commit)
        .wrap_err_with(|| format!("Failed to parse commit: {}", commit))?
        .id();

    // 2. 获取要 cherry-pick 的提交
    let commit_to_pick = repo.find_commit(commit_oid)
        .wrap_err_with(|| format!("Failed to find commit: {}", commit))?;

    // 3. 获取当前 HEAD
    let head = repo.head()
        .wrap_err("Failed to get HEAD")?;
    let head_commit = head.peel_to_commit()
        .wrap_err("Failed to get HEAD commit")?;

    // 4. 执行三向合并（cherry-pick 的本质）
    let merge_opts = None; // 使用默认合并选项
    let mut index = repo.merge_commits(&head_commit, &commit_to_pick, merge_opts)
        .wrap_err("Failed to merge commits for cherry-pick")?;

    // 5. 检查冲突
    if index.has_conflicts() {
        // 写入索引以便用户解决冲突
        index.write()
            .wrap_err("Failed to write index with conflicts")?;

        // 创建 CHERRY_PICK_HEAD 文件（模拟 git cherry-pick 的行为）
        let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
        std::fs::write(&cherry_pick_head, commit_oid.to_string())
            .wrap_err("Failed to create CHERRY_PICK_HEAD")?;

        return Err(color_eyre::eyre::eyre!(
            "Cherry-pick conflict detected. Please resolve conflicts and use cherry_pick_continue()"
        ));
    }

    // 6. 写入 tree
    let tree_id = index.write_tree_to(&repo)
        .wrap_err("Failed to write tree")?;
    let tree = repo.find_tree(tree_id)
        .wrap_err("Failed to find tree")?;

    // 7. 获取签名（优先使用配置，否则使用原提交的作者）
    let config = repo.config().ok();
    let name = config.as_ref()
        .and_then(|c| c.get_string("user.name").ok())
        .unwrap_or_else(|| {
            commit_to_pick.author().name().unwrap_or("Unknown").to_string()
        });
    let email = config.as_ref()
        .and_then(|c| c.get_string("user.email").ok())
        .unwrap_or_else(|| {
            commit_to_pick.author().email().unwrap_or("unknown@example.com").to_string()
        });
    let signature = Signature::now(&name, &email)
        .wrap_err("Failed to create signature")?;

    // 8. 创建新提交
    let message = commit_to_pick.message()
        .unwrap_or("Cherry-pick commit");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&head_commit],
    )
    .wrap_err("Failed to create commit")?;

    Ok(())
}
```

**迁移示例 - cherry_pick_no_commit()：**

```rust
// 迁移前
pub fn cherry_pick_no_commit(commit: &str) -> Result<()> {
    GitCommand::new(["cherry-pick", "--no-commit", commit])
        .run()
        .wrap_err_with(|| format!("Failed to cherry-pick commit (no-commit): {}", commit))
}

// 迁移后
pub fn cherry_pick_no_commit(commit: &str) -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::build::CheckoutBuilder;

    let repo = open_repo()?;

    // 1-4. 与 cherry_pick() 相同，获取提交并执行合并
    let commit_oid = repo.revparse_single(commit)?.id();
    let commit_to_pick = repo.find_commit(commit_oid)?;
    let head_commit = repo.head()?.peel_to_commit()?;
    let mut index = repo.merge_commits(&head_commit, &commit_to_pick, None)?;

    // 2. 检查冲突
    if index.has_conflicts() {
        index.write()?;
        return Err(color_eyre::eyre::eyre!(
            "Cherry-pick conflict detected. Please resolve conflicts manually."
        ));
    }

    // 3. 写入索引（但不创建提交）
    index.write()
        .wrap_err("Failed to write index")?;

    // 4. Checkout 到工作区（应用更改）
    let tree_id = index.write_tree_to(&repo)?;
    let tree = repo.find_tree(tree_id)?;
    let mut checkout_opts = CheckoutBuilder::new();
    checkout_opts.force();
    repo.checkout_tree(&tree.as_object(), Some(&mut checkout_opts))
        .wrap_err("Failed to checkout tree")?;

    // 5. 创建 CHERRY_PICK_HEAD（标记正在进行 cherry-pick）
    let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
    std::fs::write(&cherry_pick_head, commit_oid.to_string())
        .wrap_err("Failed to create CHERRY_PICK_HEAD")?;

    Ok(())
}
```

**迁移示例 - cherry_pick_continue()：**

```rust
// 迁移前
pub fn cherry_pick_continue() -> Result<()> {
    GitCommand::new(["cherry-pick", "--continue"])
        .run()
        .wrap_err("Failed to continue cherry-pick")
}

// 迁移后
pub fn cherry_pick_continue() -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::{IndexAddOption, Signature};

    let repo = open_repo()?;

    // 1. 检查是否有 CHERRY_PICK_HEAD
    let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
    if !cherry_pick_head.exists() {
        return Err(color_eyre::eyre::eyre!("No cherry-pick in progress"));
    }

    // 2. 读取要 cherry-pick 的提交 SHA
    let commit_sha = std::fs::read_to_string(&cherry_pick_head)
        .wrap_err("Failed to read CHERRY_PICK_HEAD")?;
    let commit_oid = commit_sha.trim().parse::<git2::Oid>()
        .wrap_err("Failed to parse commit SHA")?;
    let commit_to_pick = repo.find_commit(commit_oid)?;

    // 3. 检查是否还有冲突
    let mut index = repo.index()
        .wrap_err("Failed to open index")?;
    if index.has_conflicts() {
        return Err(color_eyre::eyre::eyre!(
            "Conflicts still exist. Please resolve all conflicts before continuing."
        ));
    }

    // 4. 获取当前 HEAD
    let head_commit = repo.head()?.peel_to_commit()?;

    // 5. 写入 tree 并创建提交
    let tree_id = index.write_tree_to(&repo)?;
    let tree = repo.find_tree(tree_id)?;

    let config = repo.config().ok();
    let name = config.as_ref()
        .and_then(|c| c.get_string("user.name").ok())
        .unwrap_or_else(|| {
            commit_to_pick.author().name().unwrap_or("Unknown").to_string()
        });
    let email = config.as_ref()
        .and_then(|c| c.get_string("user.email").ok())
        .unwrap_or_else(|| {
            commit_to_pick.author().email().unwrap_or("unknown@example.com").to_string()
        });
    let signature = Signature::now(&name, &email)?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_to_pick.message().unwrap_or("Cherry-pick commit"),
        &tree,
        &[&head_commit],
    )?;

    // 6. 删除 CHERRY_PICK_HEAD
    std::fs::remove_file(&cherry_pick_head)
        .wrap_err("Failed to remove CHERRY_PICK_HEAD")?;

    Ok(())
}
```

**迁移示例 - cherry_pick_abort()：**

```rust
// 迁移前
pub fn cherry_pick_abort() -> Result<()> {
    GitCommand::new(["cherry-pick", "--abort"])
        .run()
        .wrap_err("Failed to abort cherry-pick")
}

// 迁移后
pub fn cherry_pick_abort() -> Result<()> {
    use crate::git::git2_utils::open_repo;
    use git2::build::CheckoutBuilder;

    let repo = open_repo()?;

    // 1. 检查是否有 CHERRY_PICK_HEAD
    let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
    if !cherry_pick_head.exists() {
        return Err(color_eyre::eyre::eyre!("No cherry-pick in progress"));
    }

    // 2. 重置到 ORIG_HEAD（如果存在）或当前 HEAD
    let orig_head = repo.path().join("ORIG_HEAD");
    let target = if orig_head.exists() {
        std::fs::read_to_string(&orig_head)?.trim().to_string()
    } else {
        repo.head()?.target().unwrap().to_string()
    };

    let target_oid = target.parse::<git2::Oid>()
        .wrap_err("Failed to parse target commit SHA")?;
    let target_commit = repo.find_commit(target_oid)?;
    let target_tree = repo.find_tree(target_commit.tree_id())?;

    // 3. 重置索引和工作区
    repo.reset(&target_commit.as_object(), git2::ResetType::Hard, None)
        .wrap_err("Failed to reset repository")?;

    // 4. 删除 CHERRY_PICK_HEAD
    std::fs::remove_file(&cherry_pick_head)
        .wrap_err("Failed to remove CHERRY_PICK_HEAD")?;

    Ok(())
}
```

## 常见问题和解决方案

### 1. 错误处理

**问题：** git2 使用 `git2::Error`，需要转换为项目的 `Result` 类型

**解决方案：**

```rust
use color_eyre::{eyre::WrapErr, Result};
use git2::Error as Git2Error;

// 使用 wrap_err 转换错误
let repo = Repository::open(".")
    .wrap_err("Failed to open repository")?;

// 或者自定义错误转换
impl From<Git2Error> for color_eyre::eyre::Error {
    fn from(err: Git2Error) -> Self {
        color_eyre::eyre::eyre!("Git error: {}", err.message())
    }
}
```

### 2. 时间格式转换

**问题：** git2 的 `Time` 类型需要转换为字符串

**解决方案：**

```rust
use git2::Time;
use chrono::{DateTime, FixedOffset, TimeZone};

fn format_git_time(time: Time) -> String {
    let offset = FixedOffset::east_opt(time.sign().offset_minutes() * 60)
        .unwrap_or(FixedOffset::east_opt(0).unwrap());
    let dt = offset.timestamp_opt(time.seconds(), 0)
        .single()
        .unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S %z").to_string()
}
```

### 3. Pre-commit Hooks 处理

**问题：** git2 的 `commit()` 不会自动执行 Git hooks

**解决方案：** 可以直接执行 hook 脚本，但需要设置正确的环境变量和工作目录

**方案 1：直接执行 hook 脚本（推荐，无需 git 命令）**

```rust
use std::process::Command;
use std::path::Path;

fn execute_pre_commit_hook(repo: &git2::Repository, no_verify: bool) -> Result<()> {
    if no_verify {
        return Ok(()); // 跳过 hooks
    }

    let hooks_dir = repo.path().join("hooks");
    let pre_commit_hook = hooks_dir.join("pre-commit");

    // 检查 hook 是否存在且可执行
    if !pre_commit_hook.exists() {
        return Ok(()); // 没有 hook，直接返回
    }

    // 获取仓库的工作目录（非 bare 仓库）
    let workdir = repo.workdir()
        .ok_or_else(|| color_eyre::eyre::eyre!("Bare repository, cannot execute hooks"))?;

    // 获取 Git 目录路径
    let git_dir = repo.path().to_path_buf();
    let index_file = git_dir.join("index");

    // 执行 hook 脚本
    let status = Command::new(&pre_commit_hook)
        .current_dir(workdir) // 设置工作目录为仓库根目录
        .env("GIT_DIR", git_dir) // 设置 GIT_DIR 环境变量
        .env("GIT_INDEX_FILE", index_file) // 设置 GIT_INDEX_FILE 环境变量
        .env("GIT_WORK_TREE", workdir) // 设置 GIT_WORK_TREE（某些 hooks 需要）
        // 可选：设置其他 Git 环境变量
        .env("GIT_EDITOR", "true") // 防止某些 hooks 尝试打开编辑器
        .status()
        .wrap_err("Failed to execute pre-commit hook")?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!(
            "Pre-commit hook failed with exit code: {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

// 在 commit 函数中使用
pub fn commit(message: &str, no_verify: bool) -> Result<CommitResult> {
    let repo = open_repo()?;

    // 1. 检查是否有更改
    // ... 省略其他检查 ...

    // 2. 暂存文件
    // ... 省略暂存逻辑 ...

    // 3. 执行 pre-commit hook（如果存在）
    if !no_verify {
        execute_pre_commit_hook(&repo, no_verify)?;
    }

    // 4. 使用 git2 执行提交
    // ... 使用 repo.commit() 创建提交 ...
}
```

**方案 2：回退到命令行（简单但需要 git 命令）**

```rust
// 检查是否有 Git hooks
let has_git_hooks = repo.path().join("hooks").join("pre-commit").exists();

if !no_verify && has_git_hooks && !should_skip_hook {
    // 使用命令行执行提交以触发 Git hooks
    GitCommand::new(["commit", "-m", message]).run()?;
} else {
    // 使用 git2 执行提交
    repo.commit(/* ... */)?;
}
```

**重要环境变量说明：**

- `GIT_DIR`: 指向 `.git` 目录，hook 脚本中的 Git 命令需要这个变量
- `GIT_INDEX_FILE`: 指向索引文件（通常是 `$GIT_DIR/index`），某些 hooks 需要访问暂存区
- `GIT_WORK_TREE`: 指向工作目录，某些 hooks 需要这个变量
- 工作目录：应该设置为仓库根目录（非 bare 仓库）或 `$GIT_DIR`（bare 仓库）

**注意事项：**

1. ✅ **可以直接执行 hook 脚本**，不需要通过 `git` 命令
2. ⚠️ **必须设置正确的环境变量**，否则 hook 脚本中的 Git 命令可能无法正常工作
3. ⚠️ **工作目录必须正确**，hook 脚本可能依赖相对路径
4. ⚠️ **某些复杂的 hooks**（如 `pre-commit` 工具）可能需要额外的环境变量或配置

### 4. 状态检查的 Porcelain 格式

**问题：** git2 的 `status()` 返回 `Statuses`，需要转换为 porcelain 格式

**解决方案：**

```rust
use git2::Status;

fn status_to_porcelain(status: Status) -> &'static str {
    if status.is_index_new() {
        "A "
    } else if status.is_index_modified() {
        "M "
    } else if status.is_index_deleted() {
        "D "
    } else if status.is_wt_modified() {
        " M"
    } else if status.is_wt_deleted() {
        " D"
    } else if status.is_wt_new() {
        "??"
    } else {
        "  "
    }
}
```

## 测试策略

### 1. 单元测试

为每个迁移的方法添加单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_add_all() {
        // 创建临时仓库
        // 添加文件
        // 测试 add_all()
        // 验证文件已暂存
    }
}
```

### 2. 集成测试

确保迁移后的行为与命令行实现一致：

```rust
#[test]
fn test_commit_behavior_consistency() {
    // 使用命令行创建 commit
    // 使用 git2 创建 commit
    // 比较结果
}
```

### 3. 性能测试

对比迁移前后的性能：

```rust
#[test]
fn test_performance_improvement() {
    // 测量命令行执行时间
    // 测量 git2 执行时间
    // 验证性能提升
}
```

## 迁移检查清单

### 提交管理模块 (`src/lib/git/commit.rs`)

- [ ] `status()` - 使用 `repo.status()` 或 `repo.diff_index_to_workdir()`
- [ ] `has_commit()` - 使用 `repo.status()` 检查
- [ ] `has_staged()` - 使用 `index.write_tree()` 比较
- [ ] `add_all()` - 使用 `index.add_all()`
- [ ] `add_files()` - 使用 `index.add_path()`
- [ ] `commit()` - 使用 `repo.commit()`
- [ ] `amend()` - 使用 `repo.commit()` + `repo.reset()`
- [ ] `get_last_commit_info()` - 使用 `repo.head().peel_to_commit()`
- [ ] `get_last_commit_sha()` - 使用 `repo.head().peel_to_commit()?.id()`
- [ ] `get_last_commit_message()` - 使用 `commit.message()`
- [ ] `has_last_commit()` - 使用 `repo.head().is_some()`
- [ ] `parse_commit_ref()` - 使用 `repo.revparse_single()`
- [ ] `get_commit_info()` - 使用 `repo.find_commit()`
- [ ] `get_parent_commit()` - 使用 `commit.parent(0)`
- [ ] `get_branch_commits()` - 使用 `repo.revwalk()`
- [ ] `get_commits_from_to_head()` - 使用 `repo.revwalk()`
- [ ] `is_commit_in_current_branch()` - 使用 `repo.merge_base()`
- [ ] `get_modified_files()` - 使用 `repo.diff_index_to_workdir()`
- [ ] `get_untracked_files()` - 使用 `repo.status()` 过滤
- [ ] `reset_hard()` - 使用 `repo.reset()`
- [ ] `get_worktree_status()` - 使用 `repo.status()` 统计

### 分支管理模块 (`src/lib/git/branch.rs`)

- [ ] `get_current_branch()` - 使用 `repo.head().shorthand()`
- [ ] `branch_exists()` - 使用 `repo.find_branch()`
- [ ] `create_and_switch()` - 使用 `repo.branch()` + `repo.checkout_tree()`
- [ ] `switch_to_branch()` - 使用 `repo.checkout_tree()` + `repo.set_head()`
- [ ] `get_default_branch()` - 使用 `repo.find_remote()` + `remote.ls()`
- [ ] `pull_latest()` - 使用 `remote.fetch()` + `repo.merge()`
- [ ] `force_push()` - 使用 `remote.push()`
- [ ] `delete_remote_branch()` - 使用 `remote.push()`
- [ ] `merge()` - 使用 `repo.merge()`
- [ ] `get_merged_branches()` - 使用 `repo.revwalk()` 遍历
- [ ] `get_commit_count()` - 使用 `repo.revwalk()` 计数
- [ ] `get_merge_base()` - 使用 `repo.merge_base()`
- [ ] `is_commit_in_remote()` - 使用 `repo.find_remote()` + 遍历引用
- [ ] `rebase_onto()` - 使用 `repo.rebase()` API
- [ ] `rebase_onto_with_upstream()` - 使用 `repo.rebase()` API（指定 onto 参数）

### Tag 管理模块 (`src/lib/git/tag.rs`)

- [ ] `list_local_tags()` - 使用 `repo.tag_names()`
- [ ] `list_remote_tags()` - 使用 `repo.find_remote()` + `remote.ls()`
- [ ] `tag_exists()` - 使用 `repo.find_reference()`
- [ ] `delete_local_tag()` - 使用 `repo.tag_delete()`
- [ ] `delete_remote_tag()` - 使用 `remote.push()`

### 配置管理模块 (`src/lib/git/config.rs`)

- [ ] `set_user_email()` - 使用 `Config::open_default()?.set_str()`
- [ ] `set_user_name()` - 使用 `Config::open_default()?.set_str()`
- [ ] `get_user_email()` - 使用 `Config::open_default()?.get_string()`
- [ ] `get_user_name()` - 使用 `Config::open_default()?.get_string()`

### Stash 管理模块 (`src/lib/git/stash.rs`)

- [ ] `stash_push()` - 使用 `repo.stash_save()`
- [ ] `stash_list()` - 使用 `repo.stash_foreach()`
- [ ] `stash_apply()` - 使用 `repo.stash_apply()`
- [ ] `stash_drop()` - 使用 `repo.stash_drop()`
- [ ] `stash_pop()` - 使用 `repo.stash_pop()`
- [ ] `stash_show()` - 使用 `repo.find_commit()` + `commit.tree()`

### Cherry-pick 操作模块 (`src/lib/git/cherry_pick.rs`)

- [ ] `cherry_pick()` - 使用 `repo.merge_commits()` 实现三向合并
- [ ] `cherry_pick_no_commit()` - 使用 `repo.merge_commits()` + checkout，不提交
- [ ] `cherry_pick_continue()` - 处理冲突后继续提交
- [ ] `cherry_pick_abort()` - 使用 `repo.reset()` 重置状态
- [ ] `is_cherry_pick_in_progress()` - 检查 `CHERRY_PICK_HEAD` 文件是否存在

## 迁移步骤

### 步骤 1：准备阶段

1. 添加 git2 依赖到 `Cargo.toml`
2. 创建 `src/lib/git/git2_utils.rs` 工具模块
3. 运行 `cargo check` 确保依赖正确

### 步骤 2：迁移简单操作

1. 从 `commit.rs` 的简单操作开始
2. 迁移 `get_last_commit_info()` 等读取操作
3. 添加测试确保功能正常

### 步骤 3：迁移核心操作

1. 迁移 `add_all()`, `status()`, `commit()` 等核心操作
2. 处理 pre-commit hooks 的特殊情况
3. 添加集成测试

### 步骤 4：迁移其他模块

1. 迁移 Tag 管理模块
2. 迁移配置管理模块
3. 迁移 Stash 管理模块
4. 迁移分支管理模块（包括 rebase 操作）
5. 迁移 Cherry-pick 操作模块

### 步骤 5：测试和优化

1. 运行所有测试确保通过
2. 性能测试和优化
3. 更新文档

## 注意事项

### 1. 向后兼容性

- 保持公共 API 不变
- 只改变内部实现
- 确保返回值格式一致

### 2. 错误处理

- 统一错误消息格式
- 保持错误上下文信息
- 使用 `wrap_err` 添加上下文

### 3. 性能考虑

- git2 操作通常比命令行快 10-100 倍
- 但某些操作（如 status）可能需要优化
- 使用缓存避免重复操作

### 4. 特殊场景处理

- **Pre-commit hooks**：可以直接执行，无需 git 命令
  - git2 的 `commit()` 不会自动执行 Git hooks
  - **推荐方案**：直接执行 hook 脚本，设置正确的环境变量（`GIT_DIR`、`GIT_INDEX_FILE`、`GIT_WORK_TREE`）和工作目录
  - **备选方案**：回退到命令行执行提交以触发 hooks
  - 必须设置正确的环境变量，否则 hook 脚本中的 Git 命令可能无法正常工作

- **Git hooks 执行环境**：
  - 工作目录：非 bare 仓库设置为仓库根目录，bare 仓库设置为 `$GIT_DIR`
  - 必需环境变量：`GIT_DIR`（指向 `.git` 目录）、`GIT_INDEX_FILE`（指向索引文件）
  - 可选环境变量：`GIT_WORK_TREE`（某些 hooks 需要）

- **Rebase 冲突处理**：
  - git2 的 rebase API 支持冲突检测
  - 需要手动处理冲突并调用 `rebase.commit()` 继续

- **Cherry-pick 冲突处理**：
  - 使用 `index.has_conflicts()` 检测冲突
  - 创建 `CHERRY_PICK_HEAD` 文件标记状态
  - 用户解决冲突后调用 `cherry_pick_continue()` 继续

## 参考资源

- [git2 官方文档](https://docs.rs/git2/)
- [git2 示例代码](https://github.com/rust-lang/git2-rs/tree/master/examples)
- [libgit2 文档](https://libgit2.org/docs/)
