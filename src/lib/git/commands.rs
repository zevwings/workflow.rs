use anyhow::{Context, Result};
use duct::cmd;
use std::path::Path;
use std::process::Command;

/// Git 操作模块
pub struct Git;

impl Git {
    /// 检查 Git 状态
    pub fn status() -> Result<String> {
        cmd("git", &["status"])
            .read()
            .context("Failed to run git status")
    }

    /// 获取当前分支名
    pub fn current_branch() -> Result<String> {
        let output = cmd("git", &["branch", "--show-current"])
            .read()
            .context("Failed to get current branch")?;

        Ok(output.trim().to_string())
    }

    /// 添加所有文件到暂存区
    pub fn add_all() -> Result<()> {
        cmd("git", &["add", "--all"])
            .run()
            .context("Failed to add all files")?;
        Ok(())
    }

    /// 检查分支是否存在（本地或远程）
    /// 返回 (本地存在, 远程存在)
    pub fn branch_exists(branch_name: &str) -> Result<(bool, bool)> {
        // 检查本地分支
        let local_branches = cmd("git", &["branch", "--list"])
            .read()
            .context("Failed to list local branches")?;
        let exists_local = local_branches
            .lines()
            .any(|line| line.trim().trim_start_matches('*').trim() == branch_name);

        // 检查远程分支
        let remote_branches = cmd("git", &["branch", "-r", "--list"])
            .read()
            .context("Failed to list remote branches")?;
        let exists_remote = remote_branches.lines().any(|line| {
            line.trim()
                .strip_prefix("origin/")
                .map(|b| b == branch_name)
                .unwrap_or(false)
        });

        Ok((exists_local, exists_remote))
    }

    /// 创建或切换到分支
    ///
    /// 如果分支已存在且是当前分支，则跳过
    /// 如果分支已存在但不是当前分支，则切换到该分支
    /// 如果分支只存在于远程，则创建本地分支并跟踪远程分支
    /// 如果分支不存在，则创建新分支
    pub fn checkout_branch(branch_name: &str) -> Result<()> {
        // 检查是否是当前分支
        let current_branch = Self::current_branch()?;
        if current_branch == branch_name {
            // 已经是当前分支，无需操作
            return Ok(());
        }

        // 检查分支是否存在
        let (exists_local, exists_remote) = Self::branch_exists(branch_name)?;

        if exists_local {
            // 分支已存在于本地，切换到它
            cmd("git", &["checkout", branch_name])
                .run()
                .context(format!("Failed to checkout branch: {}", branch_name))?;
        } else if exists_remote {
            // 分支只存在于远程，创建本地分支并跟踪远程分支
            cmd(
                "git",
                &[
                    "checkout",
                    "-b",
                    branch_name,
                    &format!("origin/{}", branch_name),
                ],
            )
            .run()
            .context(format!("Failed to checkout remote branch: {}", branch_name))?;
        } else {
            // 分支不存在，创建新分支
            cmd("git", &["checkout", "-b", branch_name])
                .run()
                .context(format!("Failed to create branch: {}", branch_name))?;
        }
        Ok(())
    }

    /// 检查工程中是否存在 pre-commit hooks
    ///
    /// 检查以下位置：
    /// 1. `.git/hooks/pre-commit` - Git hooks
    /// 2. `.pre-commit-config.yaml` - pre-commit 工具配置文件
    /// 3. `pre-commit` 命令是否可用（pre-commit 工具）
    fn has_pre_commit() -> bool {
        // 检查 .git/hooks/pre-commit
        if let Ok(git_dir) = cmd("git", &["rev-parse", "--git-dir"]).read() {
            let hooks_path = Path::new(&git_dir.trim()).join("hooks").join("pre-commit");
            if hooks_path.exists() && hooks_path.is_file() {
                return true;
            }
        }

        // 检查 .pre-commit-config.yaml
        if Path::new(".pre-commit-config.yaml").exists() {
            return true;
        }

        // 检查 pre-commit 命令是否可用
        if cmd("which", &["pre-commit"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok()
        {
            return true;
        }

        false
    }

    /// 执行 pre-commit hooks（如果存在）
    ///
    /// 如果有 pre-commit 工具配置，使用 `pre-commit run`
    /// 如果有 Git hooks，直接执行 `.git/hooks/pre-commit` 脚本
    fn run_pre_commit() -> Result<()> {
        use crate::{log_info, log_success};

        // 检查是否有 staged 的文件
        // 使用 git status --porcelain 检查暂存的文件（更可靠）
        let has_staged = cmd("git", &["status", "--porcelain"])
            .read()
            .map(|output| {
                output
                    .lines()
                    .any(|line| {
                        if line.is_empty() {
                            return false;
                        }
                        // 第一列（X）表示暂存区状态，如果不是空格，说明有暂存的文件
                        // 注意：不能 trim，因为需要保留前导空格来判断
                        let first_char = line.chars().next().unwrap_or(' ');
                        first_char != ' ' && first_char != '?'
                    })
            })
            .unwrap_or(false);

        if !has_staged {
            log_info!("No staged files, skipping pre-commit");
            return Ok(());
        }

        // 优先使用 pre-commit 工具
        if Path::new(".pre-commit-config.yaml").exists() {
            log_info!("Running pre-commit hooks...");
            let output = cmd("pre-commit", &["run"])
                .stdout_capture()
                .stderr_capture()
                .run()
                .context("Failed to run pre-commit")?;

            if output.status.success() {
                log_success!("Pre-commit checks passed");
                Ok(())
            } else {
                anyhow::bail!("Pre-commit checks failed");
            }
        } else if let Ok(git_dir) = cmd("git", &["rev-parse", "--git-dir"]).read() {
            // 检查并执行 Git hooks
            let hooks_path = Path::new(&git_dir.trim()).join("hooks").join("pre-commit");
            if hooks_path.exists() && hooks_path.is_file() {
                log_info!("Running Git pre-commit hooks...");
                let output = Command::new(&hooks_path)
                    .output()
                    .context("Failed to run pre-commit hooks")?;

                if output.status.success() {
                    log_success!("Pre-commit checks passed");
                    Ok(())
                } else {
                    anyhow::bail!("Pre-commit checks failed");
                }
            } else {
                // 没有 pre-commit hooks，跳过
                log_info!("No pre-commit hooks found, skipping");
                Ok(())
            }
        } else {
            // 无法获取 git 目录，跳过
            log_info!("Not in a git repository, skipping pre-commit");
            Ok(())
        }
    }

    /// 提交更改
    ///
    /// 自动暂存所有已修改的文件，然后提交
    pub fn commit(message: &str, no_verify: bool) -> Result<()> {
        use crate::{log_info, log_warning};

        // 1. 使用 git status --porcelain 检查是否有更改（更可靠）
        let status_output = cmd("git", &["status", "--porcelain"])
            .read()
            .context("Failed to check git status")?;

        let has_changes = !status_output.trim().is_empty();

        // 2. 如果没有更改，直接返回
        if !has_changes {
            log_info!("Nothing to commit, working tree clean");
            return Ok(());
        }

        // 3. 暂存所有已修改的文件
        // 注意：即使文件已经在暂存区，执行 add_all() 也是安全的，不会造成问题
        // 这样可以确保所有更改都被暂存，包括未暂存和已暂存的更改
        Self::add_all().context("Failed to stage changes")?;

        // 5. 再次检查是否有暂存的文件（在 add_all 之后）
        // 使用 git status --porcelain 检查暂存的文件（更可靠）
        let staged_output = cmd("git", &["status", "--porcelain"])
            .read()
            .context("Failed to check staged changes")?;

        // 检查是否有暂存的文件
        // git status --porcelain 格式: "XY filename"
        // X = 暂存区状态, Y = 工作区状态
        // 如果 X 不是空格，说明有暂存的文件
        let has_staged_after = staged_output
            .lines()
            .any(|line| {
                if line.is_empty() {
                    return false;
                }
                // 第一列（X）表示暂存区状态，如果不是空格，说明有暂存的文件
                // 注意：不能 trim，因为需要保留前导空格来判断
                let first_char = line.chars().next().unwrap_or(' ');
                first_char != ' ' && first_char != '?'
            });

        if !has_staged_after {
            // 如果 add_all 后仍然没有暂存的文件，可能是所有更改都被忽略了
            log_warning!("No changes to commit after staging.");
            log_warning!("This might be due to:");
            log_warning!("  - All changes are ignored by .gitignore");
            log_warning!("  - Files are already committed");
            log_warning!("  - No actual changes detected");
            log_info!("Nothing to commit, working tree clean");
            return Ok(());
        }

        // 6. 如果不需要跳过验证，且存在 pre-commit，则先执行 pre-commit
        if !no_verify && Self::has_pre_commit() {
            Self::run_pre_commit()?;
        }

        // 7. 执行提交
        let mut args = vec!["commit", "-m", message];
        if no_verify {
            args.push("--no-verify");
        }

        cmd("git", &args).run().context("Failed to commit")?;
        Ok(())
    }

    /// 推送到远程仓库
    pub fn push(branch_name: &str, set_upstream: bool) -> Result<()> {
        let mut args = vec!["push"];
        if set_upstream {
            args.push("-u");
        }
        args.push("origin");
        args.push(branch_name);

        cmd("git", &args)
            .run()
            .context(format!("Failed to push branch: {}", branch_name))?;
        Ok(())
    }

    /// 快速更新代码
    ///
    /// # Arguments
    /// * `commit_message` - 提交消息。如果为 None，使用默认消息 "update"
    ///
    /// # 执行的操作
    /// 1. 执行 `git commit`（会自动暂存所有文件，使用指定的提交消息）
    /// 2. 执行 `git push`
    ///
    /// # Note
    /// 此方法只负责 Git 操作，不关心提交消息的来源（可以是 PR 标题、自定义消息等）
    /// `commit` 方法会自动暂存所有已修改的文件，无需手动执行 `git add`
    pub fn update(commit_message: Option<String>) -> Result<()> {
        use crate::{log_success, log_warning};

        // 1. 确定提交消息
        let message = commit_message.unwrap_or_else(|| {
            log_warning!("No commit message provided, using default message");
            "update".to_string()
        });

        log_success!("Using commit message: {}", message);

        // 2. 执行 git commit（会自动暂存所有文件）
        log_success!("Staging and committing changes...");
        Self::commit(&message, false)?; // 不使用 --no-verify（commit 方法内部会自动暂存）

        // 4. 执行 git push
        let current_branch = Self::current_branch()?;
        log_success!("Pushing to remote...");
        Self::push(&current_branch, false)?; // 不使用 -u（分支应该已经存在）

        log_success!("Update completed successfully!");
        Ok(())
    }

    /// 获取默认分支
    ///
    /// 根据仓库类型获取默认分支：
    /// - GitHub: 通过 API 获取
    /// - Codeup: 从远程获取
    /// - 其他: 从远程获取
    pub fn get_default_branch() -> Result<String> {
        use crate::git::{detect_repo_type, RepoType};
        use crate::pr::github::GitHub;

        let repo_type = detect_repo_type()?;
        match repo_type {
            RepoType::GitHub => {
                let (owner, repo_name) = GitHub::get_owner_and_repo()
                    .context("Failed to get owner and repo for GitHub")?;
                GitHub::get_default_branch(&owner, &repo_name)
                    .context("Failed to get default branch from GitHub")
            }
            RepoType::Codeup | RepoType::Unknown => {
                Self::get_default_branch_from_remote()
            }
        }
    }

    /// 从远程获取默认分支
    ///
    /// 尝试通过 Git 命令获取远程默认分支
    fn get_default_branch_from_remote() -> Result<String> {
        // 尝试获取远程默认分支
        let output = cmd("git", &["symbolic-ref", "refs/remotes/origin/HEAD"])
            .read()
            .or_else(|_| {
                // 如果上面失败，尝试从 remote show origin 获取
                let remote_info = cmd("git", &["remote", "show", "origin"]).read()?;
                for line in remote_info.lines() {
                    if line.contains("HEAD branch:") {
                        if let Some(branch) = line.split("HEAD branch:").nth(1) {
                            return Ok(branch.trim().to_string());
                        }
                    }
                }
                anyhow::bail!("Could not determine default branch")
            })?;

        // 提取分支名：refs/remotes/origin/main -> main
        let branch = output
            .trim()
            .strip_prefix("refs/remotes/origin/")
            .unwrap_or(output.trim())
            .to_string();

        Ok(branch)
    }

    /// 保存未提交的修改到 stash
    pub fn stash(message: Option<&str>) -> Result<()> {
        let mut args = vec!["stash", "push"];
        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }
        cmd("git", &args)
            .run()
            .context("Failed to stash changes")?;
        Ok(())
    }

    /// 检查分支是否有提交（相对于默认分支）
    ///
    /// 返回 true 如果分支有提交，false 如果分支为空或与默认分支相同
    pub fn branch_has_commits(branch_name: &str, default_branch: &str) -> Result<bool> {
        // 检查分支是否有提交（相对于默认分支）
        // 使用 git rev-list 来检查是否有提交
        let output = cmd("git", &["rev-list", "--count", &format!("{}..{}", default_branch, branch_name)])
            .read()
            .context("Failed to check branch commits")?;

        let count: u32 = output.trim().parse()
            .context("Failed to parse commit count")?;

        Ok(count > 0)
    }

    /// 恢复 stash 中的修改
    ///
    /// 如果遇到合并冲突，会保留 stash entry 并返回错误，提示用户手动解决冲突
    pub fn stash_pop() -> Result<()> {
        use crate::log_warning;

        // 尝试执行 git stash pop
        let result = cmd("git", &["stash", "pop"]).run();

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // 检查是否有未合并的路径（冲突文件）
                let has_unmerged = cmd("git", &["ls-files", "-u"])
                    .read()
                    .map(|output| !output.trim().is_empty())
                    .unwrap_or(false);

                if has_unmerged {
                    log_warning!("Merge conflicts detected when restoring stashed changes.");
                    log_warning!("The stash entry is kept in case you need it again.");
                    log_warning!("Please resolve the conflicts manually and then:");
                    log_warning!("  1. Resolve conflicts in the affected files");
                    log_warning!("  2. Stage the resolved files with: git add <file>");
                    log_warning!("  3. Continue with your workflow");
                    anyhow::bail!(
                        "Failed to pop stash due to merge conflicts. Please resolve conflicts manually."
                    );
                } else {
                    // 没有冲突但失败了，返回原始错误
                    Err(e).context("Failed to pop stash")
                }
            }
        }
    }
}
