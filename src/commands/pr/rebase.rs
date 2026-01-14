use crate::commands::check;
use crate::commands::pr::helpers::{detect_base_branch, handle_stash_pop_result};
use crate::git::{GitBranch, GitCommit, GitRepo, GitStash};
use crate::pr::create_provider_auto;
use crate::pr::helpers::get_current_branch_pr_id;
use crate::prompt::spinner;
use crate::spinner;
use crate::{br, error, info, success, warning};
use color_eyre::{eyre::WrapErr, Result};

/// PR Rebase 命令
///
/// 将当前分支 rebase 到目标分支并更新 PR 的 base 分支。
///
/// # 完整执行流程图
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │  开始: workflow pr rebase <TARGET_BRANCH> [OPTIONS]              │
/// └────────────────────────────┬────────────────────────────────────┘
///                              │
///                              ▼
///                    ┌─────────────────┐
///                    │  1. 运行预检查   │
///                    │  (pre-flight)    │
///                    └────────┬─────────┘
///                             │
///                    ┌────────┴────────┐
///                    │                 │
///                通过 │                 │ 失败
///                    │                 │
///                    ▼                 ▼
///            ┌──────────────┐  ┌──────────────┐
///            │  继续执行    │  │ 询问用户确认 │
///            └──────┬───────┘  └──────┬───────┘
///                   │                 │
///                   │            ┌────┴────┐
///                   │            │         │
///                   │         是 │         │ 否
///                   │            │         │
///                   │            ▼         ▼
///                   │      ┌─────────┐  [结束]
///                   │      │继续执行 │
///                   │      └────┬────┘
///                   │            │
///                   └────────────┘
///                          │
///                          ▼
///                    ┌─────────────────┐
///                    │  2. 获取当前分支 │
///                    └────────┬────────┘
///                             │
///                             ▼
///                    ┌─────────────────┐
///                    │  3. 验证目标分支 │
///                    │     是否存在     │
///                    └────────┬────────┘
///                             │
///                    ┌────────┴────────┐
///                    │                 │
///                存在 │                 │ 不存在
///                    │                 │
///                    ▼                 ▼
///            ┌──────────────┐  ┌──────────────┐
///            │  继续执行    │  │ 错误并退出   │
///            └──────┬───────┘  └──────────────┘
///                   │
///                   ▼
///            ┌─────────────────┐
///            │  4. 检查工作区状态│
///            └────────┬──────────┘
///                     │
///            ┌────────┴────────┐
///            │                │
///        有更改 │                │ 无更改
///            │                │
///            ▼                ▼
///    ┌──────────────┐  ┌──────────────┐
///    │询问用户stash? │  │  继续执行    │
///    └──────┬───────┘  └──────┬───────┘
///           │                 │
///      ┌────┴────┐            │
///      │         │            │
///    是 │         │ 否         │
///      │         │            │
///      ▼         ▼            │
///  ┌──────┐  [结束]            │
///  │Stash│                    │
///  └──┬──┘                    │
///     │                       │
///     └───────────────────────┘
///              │
///              ▼
///     ┌──────────────────┐
///     │  5. Fetch 最新代码│
///     └────────┬──────────┘
///              │
///              ▼
///     ┌──────────────────┐
///     │  6. Dry-run 检查? │
///     └────────┬──────────┘
///              │
///      ┌───────┴───────┐
///      │               │
///    是 │               │ 否
///      │               │
///      ▼               ▼
/// ┌─────────┐  ┌──────────────┐
/// │显示预览 │  │  7. 执行 rebase│
/// │信息     │  │  git rebase   │
/// └────┬────┘  └──────┬───────┘
///      │              │
///      ▼              │
///   [结束]            │
///                     │
///            ┌────────┴────────┐
///            │                 │
///        成功 │                 │ 失败
///            │                 │
///            ▼                 ▼
///    ┌──────────────┐  ┌──────────────┐
///    │  继续执行    │  │检查是否冲突? │
///    └──────┬───────┘  └──────┬───────┘
///           │                 │
///           │          ┌───────┴───────┐
///           │          │               │
///           │       是 │               │ 否
///           │          │               │
///           │          ▼               ▼
///           │    ┌──────────┐  ┌──────────┐
///           │    │处理冲突: │  │恢复stash │
///           │    │暂停并提示│  │返回错误  │
///           │    └────┬─────┘  └────┬─────┘
///           │         │              │
///           │         ▼              ▼
///           │      [结束]         [结束]
///           │   等待用户解决
///           │
///           ▼
///    ┌─────────────────┐
///    │  8. 检查并更新   │
///    │     PR base     │
///    └────────┬──────────┘
///             │
///     ┌───────┴───────┐
///     │               │
///     ▼               ▼
/// ┌──────────┐  ┌──────────┐
/// │获取PR ID │  │未找到PR  │
/// └────┬─────┘  └────┬─────┘
///      │             │
/// ┌────┴────┐        │
/// │         │        │
///PR存在 │         │ 不存在  │
///      │         │        │
///      ▼         ▼        │
/// ┌──────────┐ ┌──────────┐ │
/// │提示用户  │ │记录:跳过 │ │
/// │确认更新? │ │PR更新    │ │
/// └────┬─────┘ └────┬─────┘ │
///      │            │       │
/// ┌────┴────┐       │       │
/// │         │       │       │
/// 是 │         │ 否   │       │
///      │         │       │       │
///      ▼         ▼       │       │
/// ┌──────┐  [跳过]      │       │
/// │更新  │              │       │
/// │PR base│              │       │
/// └───┬───┘              │       │
///     │                  │       │
///     │    ┌─────────────┴───────┘
///     │    │
/// 成功 │    │ 失败
///     │    │
///     ▼    ▼
/// ┌──────┐┌──────────┐
/// │记录  ││记录:更新 │
/// │成功  ││失败      │
/// └───┬──┘└────┬─────┘
///     │       │
///     └───────┘
///              │
///              ▼
///     ┌─────────────────┐
///     │  9. 恢复 stash? │
///     └────────┬──────────┘
///              │
///      ┌───────┴───────┐
///      │               │
///    是 │               │ 否
///      │               │
///      ▼               ▼
/// ┌──────────┐  ┌──────────┐
/// │恢复stash │  │跳过恢复  │
/// └────┬─────┘  └────┬─────┘
///      │             │
/// ┌────┴────┐        │
/// │         │        │
///成功 │         │ 失败   │
///      │         │        │
///      ▼         ▼        │
/// ┌──────┐ ┌──────────┐  │
/// │继续  │ │警告:恢复 │  │
/// │      │ │失败      │  │
/// └──┬───┘ └────┬─────┘  │
///    │          │        │
///    └──────────┴────────┘
///              │
///              ▼
///     ┌─────────────────┐
///     │ 10. 推送(默认)   │
///     └────────┬─────────┘
///              │
///      ┌───────┴───────┐
///      │               │
///    是 │               │ 否 (--no-push)
///      │               │
///      ▼               ▼
/// ┌──────────────┐ ┌──────────┐
/// │推送          │ │记录:跳过 │
/// │force-with-   │ │推送      │
/// │lease         │ └────┬─────┘
/// └──────┬───────┘      │
///        │              │
///   ┌────┴────┐         │
///   │         │         │
///成功 │         │ 失败     │
///   │         │         │
///   ▼         ▼         │
/// ┌──────┐ ┌──────────┐ │
/// │成功  │ │错误:推送  │ │
/// │完成  │ │失败      │ │
/// └──┬───┘ └────┬─────┘ │
///    │          │       │
///    │          ▼       │
///    │      [结束]      │
///    │   需要重新rebase │
///    │                  │
///    └──────────────────┘
///              │
///              ▼
///        ┌──────────┐
///        │ 操作成功  │
///        └────┬─────┘
///             │
///             ▼
///          [结束]
/// ```
///
/// # 关键决策点
///
/// 1. **预检查失败** → 询问用户是否继续
/// 2. **工作区有更改** → 询问用户是否 stash
/// 3. **Rebase 冲突** → 暂停操作，提示用户解决
/// 4. **PR 不存在** → 跳过更新（不报错）
/// 5. **更新 PR base** → 总是提示用户确认（如果找到 PR）
/// 6. **推送失败** → 提示用户重新 rebase
/// 7. **默认推送** → 默认推送到远程（除非使用 --no-push）
///
/// # 错误处理
///
/// - **目标分支不存在** → 错误退出
/// - **Rebase 冲突** → 暂停，等待用户解决
/// - **PR 更新失败** → 记录警告，继续执行
/// - **推送失败** → 错误提示，建议重新 rebase
pub struct PullRequestRebaseCommand;

impl PullRequestRebaseCommand {
    /// 将当前分支 rebase 到目标分支并更新 PR base
    ///
    /// # 参数
    ///
    /// * `target_branch` - 目标分支名称
    /// * `push` - 是否推送到远程
    /// * `dry_run` - 预览模式
    ///
    /// 注意：PR ID 会自动从当前分支检测，如果找到 PR，会提示用户确认是否更新 PR base
    pub fn rebase(target_branch: String, push: bool, dry_run: bool) -> Result<()> {
        // 1. 运行预检查
        info!("Running pre-flight checks...");
        if let Err(e) = check::CheckCommand::run_all() {
            warning!("Pre-flight checks failed: {}", e);
            crate::confirm!("Continue anyway?").default(false).prompt()?;
        }

        // 2. 获取当前分支
        let current_branch = GitBranch::current_branch()?;
        success!("Current branch: {}", current_branch);

        // 3. 验证目标分支存在
        Self::validate_target_branch(&target_branch)?;

        // 4. 检查工作区状态并 stash
        let has_stashed = Self::check_working_directory()?;

        // 5. 拉取目标分支最新代码
        spinner("Fetching latest changes...").with(GitRepo::fetch)?;

        // 6. 检测当前分支的基础分支（用于智能 rebase）
        let detected_base = detect_base_branch(&current_branch, &target_branch)?;

        // 7. 预览模式
        if dry_run {
            Self::dry_run(
                &current_branch,
                &target_branch,
                push,
                detected_base.as_deref(),
            )?;
            return Ok(());
        }

        // 8. 执行 rebase（智能选择 rebase 方式）
        let rebase_result = if let Some(base_branch) = &detected_base {
            success!(
                "Detected that '{}' is based on '{}', will only rebase unique commits",
                current_branch,
                base_branch
            );
            info!(
                "Rebasing commits from '{}' (excluding '{}' changes) onto '{}'",
                current_branch, base_branch, target_branch
            );
            spinner!("Rebasing '{}' onto '{}'...", current_branch, target_branch).with(|| {
                GitBranch::rebase_onto_with_upstream(&target_branch, base_branch, &current_branch)
            })
        } else {
            spinner!("Rebasing '{}' onto '{}'...", current_branch, target_branch)
                .with(|| GitBranch::rebase_onto(&target_branch))
        };

        // 9. 处理 rebase 结果
        if let Err(e) = rebase_result {
            // 如果 rebase 失败，恢复 stash
            if has_stashed {
                info!("Rebase failed, attempting to restore stashed changes...");
                handle_stash_pop_result(GitStash::stash_pop(None));
            }

            // 检查是否是冲突
            if Self::is_rebase_conflict(&e) {
                Self::handle_rebase_conflict()?;
                return Ok(()); // 冲突需要用户手动解决
            }

            return Err(e);
        }

        success!("Rebase completed successfully");

        // 10. 更新 PR base（如果找到 PR，会提示用户确认）
        Self::update_pr_base(&current_branch, &target_branch)?;

        // 11. 恢复 stash
        if has_stashed {
            info!("Restoring stashed changes...");
            handle_stash_pop_result(GitStash::stash_pop(None));
        }

        // 12. 推送到远程（默认推送）
        if push {
            Self::push_with_force_lease(&current_branch)?;
        } else {
            info!("Skipping push (use without --no-push to push by default)");
        }

        br!();
        success!("Rebase operation completed successfully");
        Ok(())
    }

    /// 验证目标分支存在
    fn validate_target_branch(target_branch: &str) -> Result<()> {
        // 检查本地分支
        if GitBranch::has_local_branch(target_branch)? {
            return Ok(());
        }

        // 检查远程分支
        if GitBranch::has_remote_branch(target_branch)? {
            return Ok(());
        }

        color_eyre::eyre::bail!(
            "Target branch '{}' does not exist. Please check the branch name.",
            target_branch
        );
    }

    /// 检查工作区状态并 stash
    fn check_working_directory() -> Result<bool> {
        let has_uncommitted =
            GitCommit::has_commit().wrap_err("Failed to check working directory status")?;

        if has_uncommitted {
            warning!("Working directory has uncommitted changes");
            let options = vec![
                "Stash changes and continue".to_string(),
                "Abort operation".to_string(),
            ];
            let selected = crate::select!("How would you like to proceed?", options)
                .default(0)
                .prompt()
                .wrap_err("Failed to get user choice")?;
            let choice = if selected == "Stash changes and continue" {
                0
            } else {
                1
            };

            match choice {
                0 => {
                    info!("Stashing uncommitted changes...");
                    GitStash::stash_push(Some("Auto-stash before rebase"))?;
                    success!("Changes stashed successfully");
                    Ok(true)
                }
                1 => {
                    color_eyre::eyre::bail!("Operation cancelled by user");
                }
                _ => unreachable!(),
            }
        } else {
            Ok(false)
        }
    }

    /// 检查是否是 rebase 冲突
    fn is_rebase_conflict(error: &color_eyre::eyre::Report) -> bool {
        let error_msg = error.to_string().to_lowercase();
        error_msg.contains("conflict") || error_msg.contains("could not apply")
    }

    /// 处理 rebase 冲突
    fn handle_rebase_conflict() -> Result<()> {
        error!("Rebase conflicts detected!");
        info!("Please resolve the conflicts manually:");
        info!("  1. Review conflicted files");
        info!("  2. Resolve conflicts");
        info!("  3. Stage resolved files: git add <files>");
        info!("  4. Continue rebase: git rebase --continue");
        info!("  5. Push when ready: git push --force-with-lease");
        info!("\nTo abort: git rebase --abort");

        color_eyre::eyre::bail!("Rebase conflicts detected. Please resolve manually.");
    }

    /// 更新 PR base 分支
    ///
    /// 自动检测当前分支的 PR，如果找到则提示用户确认更新
    fn update_pr_base(current_branch: &str, target_branch: &str) -> Result<()> {
        // 自动检测当前分支的 PR ID
        let pr_id = match get_current_branch_pr_id()? {
            Some(id) => id,
            None => {
                warning!("No PR found for current branch '{}'", current_branch);
                info!("Skipping PR base update");
                return Ok(());
            }
        };

        info!(
            "PR #{} found for current branch '{}'",
            pr_id, current_branch
        );
        info!(
            "Will update PR base branch from current base to '{}'",
            target_branch
        );

        // 提示用户确认
        crate::confirm!("Update PR #{} base branch to '{}'?", pr_id, target_branch)
            .default(true)
            .prompt()?;

        // 创建 PR provider
        let provider = create_provider_auto()?;

        // 更新 PR base
        spinner!(
            "Updating PR #{} base branch to '{}'...",
            pr_id,
            target_branch
        )
        .with(|| provider.update_pr_base(&pr_id, target_branch))
        .wrap_err("Failed to update PR base branch")?;

        success!("PR #{} base branch updated to '{}'", pr_id, target_branch);
        Ok(())
    }

    /// 使用 force-with-lease 推送
    fn push_with_force_lease(current_branch: &str) -> Result<()> {
        // 使用 force-with-lease 推送
        info!("Pushing to remote (force-with-lease)...");
        GitBranch::push_force_with_lease(current_branch)
            .wrap_err("Failed to push to remote (force-with-lease)")?;

        success!("Pushed to remote successfully");
        Ok(())
    }

    /// 预览模式
    fn dry_run(
        current_branch: &str,
        target_branch: &str,
        push: bool,
        detected_base: Option<&str>,
    ) -> Result<()> {
        info!("=== Dry Run Mode ===");
        info!("Current branch: {}", current_branch);
        info!("Target branch: {}", target_branch);

        if let Some(base_branch) = detected_base {
            info!(
                "Detected base branch: '{}' (will only rebase unique commits)",
                base_branch
            );
            info!(
                "Will rebase commits from '{}' (excluding '{}' changes) onto '{}'",
                current_branch, base_branch, target_branch
            );
        } else {
            info!("No base branch detected, will rebase all commits");
            info!("Will rebase '{}' onto '{}'", current_branch, target_branch);
        }

        info!("Will check for PR and prompt to update PR base if found (with user confirmation)");

        if push {
            info!("Will push to remote (force-with-lease)");
        } else {
            info!("Will NOT push to remote");
        }

        info!("=== End Dry Run ===");
        Ok(())
    }
}
