# PR Close 命令实现方案

## 一、功能需求

实现 `pr close` 命令，包含以下功能：
1. 关闭当前 PR（远程）
2. 删除远程分支
3. 删除本地分支并切换到 default branch

## 二、架构设计

### 2.1 职责划分

- **Git 模块** (`src/lib/git/branch.rs`): 通用 Git 操作
  - `delete_remote()`: 删除远程分支（使用 `git push origin --delete`）

- **PlatformProvider trait** (`src/lib/pr/provider.rs`): 平台特定 API 操作
  - `close_pull_request()`: 关闭 PR（通过平台 API）

- **Close 命令** (`src/commands/pr/close.rs`): 业务逻辑编排
  - 协调 Git 操作和平台 API 操作
  - 处理错误和边界情况

### 2.2 实现流程

```
pr close
  ├─ 1. 获取仓库类型和 PR ID
  ├─ 2. 检查 PR 状态（如果已关闭，跳过关闭步骤）
  ├─ 3. 关闭 PR（远程，通过 PlatformProvider）
  ├─ 4. 删除远程分支（通过 Git::delete_remote）
  └─ 5. 清理本地：切换到默认分支并删除当前分支
```

## 三、文件修改清单

### 3.1 新增文件

1. **`src/commands/pr/close.rs`** - Close 命令实现

### 3.2 修改文件

1. **`src/bin/pr.rs`** - 添加 Close 子命令
2. **`src/commands/pr/mod.rs`** - 导出 close 模块
3. **`src/lib/pr/provider.rs`** - 添加 `close_pull_request` 方法
4. **`src/lib/pr/github.rs`** - 实现 GitHub 的关闭 PR
5. **`src/lib/pr/codeup.rs`** - 实现 Codeup 的关闭 PR
6. **`src/lib/git/branch.rs`** - 添加 `delete_remote` 方法
7. **`src/commands/install.rs`** - 更新 completion 生成

## 四、详细实现

### 4.1 Git 模块扩展

**文件**: `src/lib/git/branch.rs`

```rust
/// 删除远程分支
///
/// 使用 `git push origin --delete <branch_name>` 删除远程分支。
///
/// # 参数
///
/// * `branch_name` - 要删除的远程分支名称
///
/// # 错误
///
/// 如果删除失败，返回相应的错误信息。
pub fn delete_remote(branch_name: &str) -> Result<()> {
    cmd("git", &["push", "origin", "--delete", branch_name])
        .run()
        .with_context(|| format!("Failed to delete remote branch: {}", branch_name))?;
    Ok(())
}
```

### 4.2 PlatformProvider trait 扩展

**文件**: `src/lib/pr/provider.rs`

```rust
/// 关闭 Pull Request
///
/// # Arguments
/// * `pull_request_id` - PR ID
fn close_pull_request(pull_request_id: &str) -> Result<()>;
```

### 4.3 GitHub 实现

**文件**: `src/lib/pr/github.rs`

```rust
impl PlatformProvider for GitHub {
    /// 关闭 Pull Request
    fn close_pull_request(pull_request_id: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        // GitHub API: PATCH /repos/{owner}/{repo}/pulls/{pull_number}
        // Body: {"state": "closed"}
        #[derive(Debug, Serialize)]
        struct UpdatePullRequestRequest {
            state: String,
        }

        let request = UpdatePullRequestRequest {
            state: "closed".to_string(),
        };

        let client = Self::get_client()?;
        let headers = Self::get_headers()?;
        let response: HttpResponse<serde_json::Value> = client
            .patch(&url, &request, None, Some(headers))
            .context(format!("Failed to close PR: {}", pull_request_id))?;

        if !response.is_success() {
            return Err(Self::handle_api_error_json(&response));
        }

        Ok(())
    }
}
```

### 4.4 Codeup 实现

**文件**: `src/lib/pr/codeup.rs`

```rust
impl PlatformProvider for Codeup {
    /// 关闭 Pull Request
    fn close_pull_request(pull_request_id: &str) -> Result<()> {
        let (project_id, cookie) = Self::get_env_vars()?;
        let settings = Settings::get();
        let csrf_token = settings
            .codeup_csrf_token
            .as_ref()
            .context("CODEUP_CSRF_TOKEN environment variable not set")?;

        // 先获取 PR 信息以确定实际的 PR ID
        let actual_pull_request_id = if pull_request_id.parse::<u64>().is_ok() {
            pull_request_id.to_string()
        } else {
            // 可能是分支名或 URL，先查找 PR
            let pull_request_info = Self::get_pull_request_by_branch(pull_request_id)?;
            match pull_request_info {
                Some(pr) => {
                    if let Some(ref detail_url) = pr.detail_url {
                        Self::extract_pull_request_id_from_url(detail_url)
                            .context("Failed to extract PR ID from URL")?
                    } else if let Some(iid) = pr.pull_request_number {
                        iid.to_string()
                    } else {
                        anyhow::bail!("Cannot determine PR ID")
                    }
                }
                None => anyhow::bail!("PR not found: {}", pull_request_id),
            }
        };

        // Codeup API: 关闭 PR 的端点（需要查看 Codeup API 文档确认）
        // 可能需要使用 PATCH 或 PUT 请求更新 PR 状态为 closed
        #[derive(Debug, Serialize)]
        struct ClosePullRequestRequest {
            state: String,
        }

        let request = ClosePullRequestRequest {
            state: "closed".to_string(),
        };

        let url = format!(
            "https://codeup.aliyun.com/api/v4/projects/{}/code_reviews/{}/close?_csrf={}&_input_charset=utf-8",
            project_id, actual_pull_request_id, csrf_token
        );

        let client = Self::get_client()?;
        let headers = Self::get_headers(&cookie, Some("application/json"))?;
        let response: HttpResponse<serde_json::Value> = client
            .put(&url, &request, None, Some(&headers))
            .context("Failed to close PR via Codeup API")?;

        if !response.is_success() {
            anyhow::bail!(
                "Codeup close PR request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        Ok(())
    }
}
```

### 4.5 Close 命令实现

**文件**: `src/commands/pr/close.rs`

```rust
use crate::{
    log_info, log_success, log_warning, Codeup, Git, GitHub, PlatformProvider, RepoType,
};
use anyhow::{Context, Result};

/// PR 关闭命令
#[allow(dead_code)]
pub struct PullRequestCloseCommand;

impl PullRequestCloseCommand {
    /// 关闭 PR
    #[allow(dead_code)]
    pub fn close(pull_request_id: Option<String>) -> Result<()> {
        // 1. 获取仓库类型和 PR ID
        let repo_type = Git::detect_repo_type()?;
        let pull_request_id = Self::get_pull_request_id(pull_request_id, &repo_type)?;

        log_success!("\nClosing PR: #{}", pull_request_id);

        // 2. 获取当前分支名（关闭前保存）
        let current_branch = Git::current_branch()?;

        // 3. 获取默认分支
        let default_branch = Git::get_default_branch()
            .context("Failed to get default branch")?;

        // 4. 检查 PR 状态（如果已关闭，跳过关闭步骤）
        let was_already_closed = Self::check_if_already_closed(&pull_request_id, &repo_type)?;

        if !was_already_closed {
            // 5. 关闭 PR（远程）
            Self::close_pull_request(&pull_request_id, &repo_type)?;
        }

        // 6. 删除远程分支
        Self::delete_remote_branch(&current_branch)?;

        // 7. 清理本地：切换到默认分支并删除当前分支
        Self::cleanup_after_close(&current_branch, &default_branch)?;

        Ok(())
    }

    /// 获取 PR ID（从参数或当前分支）
    fn get_pull_request_id(
        pull_request_id: Option<String>,
        repo_type: &RepoType,
    ) -> Result<String> {
        if let Some(id) = pull_request_id {
            return Ok(id);
        }

        // 从当前分支获取 PR
        let pr_id = match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::get_current_branch_pull_request()?
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::get_current_branch_pull_request()?
            }
            _ => {
                anyhow::bail!(
                    "Auto-detection of PR ID is only supported for GitHub and Codeup repositories."
                );
            }
        };

        match pr_id {
            Some(id) => {
                log_success!("Found PR for current branch: #{}", id);
                Ok(id)
            }
            None => {
                let error_msg = match repo_type {
                    RepoType::GitHub => "No PR found for current branch. Please specify PR ID.",
                    RepoType::Codeup => {
                        "No PR found for current branch. Please specify PR ID or branch name."
                    }
                    _ => "No PR found for current branch.",
                };
                anyhow::bail!("{}", error_msg);
            }
        }
    }

    /// 检查 PR 是否已经关闭
    fn check_if_already_closed(
        pull_request_id: &str,
        repo_type: &RepoType,
    ) -> Result<bool> {
        let status = match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::get_pull_request_status(pull_request_id)?
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::get_pull_request_status(pull_request_id)?
            }
            _ => {
                anyhow::bail!(
                    "PR close is currently only supported for GitHub and Codeup repositories."
                );
            }
        };

        // 如果状态是 closed 或 merged，说明已经关闭
        if status.state == "closed" || status.state == "merged" {
            log_warning!("PR #{} is already closed (state: {})", pull_request_id, status.state);
            return Ok(true);
        }

        Ok(false)
    }

    /// 关闭 PR（根据仓库类型调用对应的实现）
    fn close_pull_request(pull_request_id: &str, repo_type: &RepoType) -> Result<()> {
        match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::close_pull_request(pull_request_id)
                    .context("Failed to close PR via GitHub API")?;
                log_success!("PR closed successfully");
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::close_pull_request(pull_request_id)
                    .context("Failed to close PR via Codeup API")?;
                log_success!("PR closed successfully");
            }
            _ => {
                anyhow::bail!(
                    "PR close is currently only supported for GitHub and Codeup repositories."
                );
            }
        }
        Ok(())
    }

    /// 删除远程分支
    fn delete_remote_branch(branch_name: &str) -> Result<()> {
        // 检查远程分支是否存在
        let (_, exists_remote) = Git::is_branch_exists(branch_name)
            .context("Failed to check if remote branch exists")?;

        if !exists_remote {
            log_info!("Remote branch '{}' does not exist, skipping deletion", branch_name);
            return Ok(());
        }

        log_info!("Deleting remote branch: {}", branch_name);

        // 尝试删除远程分支
        match Git::delete_remote(branch_name) {
            Ok(()) => {
                log_success!("Remote branch deleted: {}", branch_name);
            }
            Err(e) => {
                // 如果分支已经被 API 删除，忽略错误
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("remote ref does not exist")
                    || error_msg.contains("not found")
                    || error_msg.contains("does not exist")
                {
                    log_info!("Remote branch '{}' may have already been deleted", branch_name);
                } else {
                    // 其他错误，记录警告但继续执行
                    log_warning!("Failed to delete remote branch: {}", e);
                    log_warning!("You may need to delete it manually");
                }
            }
        }

        Ok(())
    }

    /// 关闭后清理：切换到默认分支并删除当前分支
    fn cleanup_after_close(current_branch: &str, default_branch: &str) -> Result<()> {
        // 如果当前分支已经是默认分支，不需要清理
        if current_branch == default_branch {
            log_info!("Already on default branch: {}", default_branch);
            return Ok(());
        }

        log_info!("Switching to default branch: {}", default_branch);

        // 1. 先更新远程分支信息（这会同步远程删除的分支）
        Git::fetch()?;

        // 2. 检查是否有未提交的更改，如果有就先 stash
        let has_stashed = Git::has_commit()?;
        if has_stashed {
            log_info!("Stashing local changes before switching branches...");
            Git::stash_push(Some("Auto-stash before PR close cleanup"))?;
        }

        // 3. 切换到默认分支
        Git::checkout_branch(default_branch)
            .with_context(|| format!("Failed to checkout default branch: {}", default_branch))?;

        // 4. 更新本地默认分支
        Git::pull(default_branch)
            .with_context(|| format!("Failed to pull latest changes from {}", default_branch))?;

        // 5. 删除本地分支（如果还存在）
        if Self::branch_exists_locally(current_branch)? {
            log_info!("Deleting local branch: {}", current_branch);
            Git::delete(current_branch, false)
                .or_else(|_| {
                    // 如果分支未完全合并，尝试强制删除
                    log_info!("Branch may not be fully merged, trying force delete...");
                    Git::delete(current_branch, true)
                })
                .context("Failed to delete local branch")?;
            log_success!("Local branch deleted: {}", current_branch);
        } else {
            log_info!("Local branch already deleted: {}", current_branch);
        }

        // 6. 恢复 stash（如果有）
        if has_stashed {
            log_info!("Restoring stashed changes...");
            if let Err(e) = Git::stash_pop() {
                log_warning!("Failed to restore stashed changes: {}", e);
                log_warning!("You can manually restore them with: git stash pop");
                log_warning!("Or view the stash list with: git stash list");
            } else {
                log_success!("Stashed changes restored successfully");
            }
        }

        // 7. 清理远程分支引用（prune，移除已删除的远程分支引用）
        if let Err(e) = Git::prune_remote() {
            log_info!("Warning: Failed to prune remote references: {}", e);
            log_info!("This is a non-critical cleanup operation. Local cleanup is complete.");
        }

        log_success!(
            "Cleanup completed: switched to {} and deleted local branch {}",
            default_branch,
            current_branch
        );

        Ok(())
    }

    /// 检查本地分支是否存在
    fn branch_exists_locally(branch_name: &str) -> Result<bool> {
        let (exists_local, _) =
            Git::is_branch_exists(branch_name).context("Failed to check if branch exists")?;
        Ok(exists_local)
    }
}
```

### 4.6 CLI 入口修改

**文件**: `src/bin/pr.rs`

```rust
use workflow::commands::pr::{create, list, merge, status, update, close};

#[derive(Subcommand)]
enum PRCommands {
    // ... existing commands ...

    /// 关闭 Pull Request
    ///
    /// 关闭当前分支对应的 PR，删除远程分支，并切换到默认分支。
    Close {
        /// PR ID（可选，不提供时自动检测当前分支）
        #[arg(value_name = "PR_ID")]
        pull_request_id: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        // ... existing matches ...
        PRCommands::Close { pull_request_id } => {
            close::PullRequestCloseCommand::close(pull_request_id)?;
        }
    }

    Ok(())
}
```

### 4.7 模块导出

**文件**: `src/commands/pr/mod.rs`

```rust
pub mod create;
pub mod list;
pub mod merge;
pub mod status;
pub mod update;
pub mod close;
```

### 4.8 Completion 更新

**文件**: `src/commands/install.rs`

在 `generate_pr_completion` 函数中添加：

```rust
.subcommand(
    Command::new("close")
        .about("Close a Pull Request")
        .arg(clap::Arg::new("PR_ID").value_name("PR_ID")),
)
```

## 五、错误处理策略

### 5.1 PR 状态检查
- 关闭前检查 PR 状态，如果已关闭则跳过关闭步骤
- 参考 `merge.rs` 中的状态检查逻辑

### 5.2 远程分支删除
- 检查远程分支是否存在，不存在则跳过
- 如果删除失败（可能已被 API 删除），记录警告但继续执行
- 忽略 "remote ref does not exist" 错误

### 5.3 本地分支删除
- 如果分支未完全合并，尝试强制删除
- 如果分支不存在，跳过删除

### 5.4 边界情况
- 当前分支是默认分支：跳过删除，只关闭 PR
- 未提交的更改：自动 stash，切换分支后恢复
- 网络错误：返回错误，不继续执行后续步骤

## 六、实现步骤

1. **第一步**: 在 `Git` 模块中添加 `delete_remote` 方法
2. **第二步**: 在 `PlatformProvider` trait 中添加 `close_pull_request` 方法
3. **第三步**: 实现 GitHub 的 `close_pull_request`
4. **第四步**: 实现 Codeup 的 `close_pull_request`（需要确认 Codeup API）
5. **第五步**: 创建 `close.rs` 命令实现文件
6. **第六步**: 更新 CLI 入口和模块导出
7. **第七步**: 更新 completion 生成
8. **第八步**: 测试和错误处理优化

## 七、注意事项

1. **Codeup API**: 需要确认 Codeup 关闭 PR 的具体 API 端点和请求格式
2. **错误处理**: 确保所有错误情况都有适当的处理和日志
3. **幂等性**: 确保命令可以安全地重复执行（已关闭的 PR 不会报错）
4. **用户体验**: 提供清晰的日志输出，告知用户每个步骤的执行情况

