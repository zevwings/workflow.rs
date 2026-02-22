//! 创建分支命令

use domain::GitRepository;
use prompt::{error, info, input, select, success};

use crate::util::{
    generate_branch_name_from_jira, generate_branch_name_from_template, select_branch_type, to_slug,
};
use crate::util::{safe_pull, PullOptions};
use crate::{bootstrap, commands::jira::utils::get_jira_id_interactive_optional};

/// 源分支选项
#[derive(Clone)]
enum SourceBranchOption {
    /// 从当前分支创建
    FromCurrent(String),
    /// 从默认分支创建
    FromDefault(String),
}

impl std::fmt::Display for SourceBranchOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceBranchOption::FromCurrent(branch) => {
                write!(f, "Create from current branch ({})", branch)
            }
            SourceBranchOption::FromDefault(branch) => {
                write!(f, "Create from default branch ({})", branch)
            }
        }
    }
}

/// Branch Create 命令
pub struct BranchCreateCommand {
    jira_id: Option<String>,
    from_default: bool,
    dry_run: bool,
}

impl BranchCreateCommand {
    /// 创建新的 BranchCreateCommand
    pub fn new(jira_id: Option<String>, from_default: bool, dry_run: bool) -> Self {
        Self {
            jira_id,
            from_default,
            dry_run,
        }
    }

    /// 运行 `workflow branch create` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = bootstrap::get_git_repository();

        // 确定源分支
        let source_branch = if self.from_default {
            branch_repo
                .get_default_branch()
                .map_err(|e| format!("Failed to get default branch: {}", e))?
        } else {
            branch_repo
                .get_current_branch()
                .map_err(|e| format!("Failed to get current branch: {}", e))?
        };

        let jira_id = get_jira_id_interactive_optional(self.jira_id.clone())?;

        // 生成分支名
        let branch_name = if let Some(jira_id) = jira_id {
            let result = generate_branch_name_from_jira(&jira_id)?;
            result.branch_name
        } else {
            self.generate_branch_name_manual()?
        };

        if self.dry_run {
            info!(
                "[DRY RUN] Would create branch '{}' from '{}'",
                branch_name, source_branch
            );
            return Ok(());
        }

        // 检查分支是否已存在
        let (exists_local, exists_remote) = branch_repo
            .has_branch(&branch_name)
            .map_err(|e| format!("Failed to check branch existence: {}", e))?;

        if exists_local || exists_remote {
            error!("Branch '{}' already exists", branch_name);
            if exists_local {
                info!("  Local branch exists");
            }
            if exists_remote {
                info!("  Remote branch exists");
            }
            return Err(format!("Branch '{}' already exists", branch_name).into());
        }

        // 获取当前分支和默认分支
        let current_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        let default_branch = branch_repo
            .get_default_branch()
            .map_err(|e| format!("Failed to get default branch: {}", e))?;

        // 确定源分支和是否需要恢复 stash
        let (final_source_branch, needs_stash_restore) = if self.from_default {
            // 如果指定了 from_default，强制从默认分支创建
            if default_branch != current_branch {
                let needs_stash =
                    self.prepare_default_branch(branch_repo.as_ref(), &default_branch)?;
                (default_branch.clone(), needs_stash)
            } else {
                (default_branch.clone(), false)
            }
        } else if default_branch != current_branch {
            // 如果没有指定 from_default，且当前分支不是默认分支，询问用户从哪里创建
            let options = vec![
                SourceBranchOption::FromCurrent(current_branch.clone()),
                SourceBranchOption::FromDefault(default_branch.clone()),
            ];

            let selected = select!("Please select where to create the new branch:", options)
                .prompt()
                .map_err(|e| format!("Failed to select source branch: {}", e))?;

            match selected {
                SourceBranchOption::FromCurrent(_) => {
                    // 从当前分支创建
                    (current_branch.clone(), false)
                }
                SourceBranchOption::FromDefault(_) => {
                    // 从默认分支创建，需要 stash、切换、拉取
                    let needs_stash =
                        self.prepare_default_branch(branch_repo.as_ref(), &default_branch)?;
                    (default_branch.clone(), needs_stash)
                }
            }
        } else {
            // 当前分支就是默认分支，直接使用
            (source_branch.clone(), false)
        };

        // 创建分支
        info!(
            "Creating branch '{}' from '{}'...",
            branch_name, final_source_branch
        );
        branch_repo
            .create_branch(&branch_name)
            .map_err(|e| format!("Failed to create branch: {}", e))?;

        // 切换到新分支
        branch_repo
            .checkout_branch(&branch_name)
            .map_err(|e| format!("Failed to checkout branch: {}", e))?;

        // 如果之前 stash 了代码，在新分支上恢复
        if needs_stash_restore {
            info!("Restoring stashed changes on new branch...");
            branch_repo
                .stash_pop(0)
                .map_err(|e| format!("Failed to restore stashed changes: {}", e))?;
        }

        success!("Created and switched to branch '{}'", branch_name);
        Ok(())
    }

    fn generate_branch_name_manual(&self) -> Result<String, Box<dyn std::error::Error>> {
        let branch_name = input!("Please enter your new branch name:")
            .validator(|input: &str| {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    Err("Branch name cannot be empty".to_string())
                } else if to_slug(trimmed).is_empty() {
                    Err("Branch name must contain at least one ASCII letter or number (a-z, A-Z, 0-9)".to_string())
                } else {
                    Ok(())
                }
            })
            .prompt()
            .map(|s: String| s.trim().to_string())
            .map_err(|e| format!("Failed to get branch name: {}", e))?;

        let branch_name_slug = to_slug(&branch_name);
        let branch_type = select_branch_type()?;
        let full_branch_name =
            generate_branch_name_from_template(branch_type, &branch_name_slug, None)?;
        Ok(full_branch_name)
    }

    /// 准备默认分支：stash、切换、拉取。返回是否需在新分支上恢复 stash。
    fn prepare_default_branch(
        &self,
        branch_repo: &dyn GitRepository,
        default_branch: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // 检查工作区状态
        let status = branch_repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to check working tree status: {}", e))?;

        let needs_stash = !status.is_clean();

        // 如果有未提交的更改，先 stash
        if needs_stash {
            info!("Working tree has uncommitted changes, stashing...");
            branch_repo
                .stash_push(Some("Auto-stash before creating branch from default"))
                .map_err(|e| format!("Failed to stash changes: {}", e))?;
        }

        // 切换到默认分支
        info!("Switching to default branch '{}'...", default_branch);
        branch_repo
            .checkout_branch(default_branch)
            .map_err(|e| format!("Failed to switch to branch '{}': {}", default_branch, e))?;

        // 拉取最新代码（工作区已 stash 故无需再 stash）
        info!("Pulling latest changes from '{}'...", default_branch);
        safe_pull(default_branch, &PullOptions::no_stash())?;

        // 返回是否需要恢复 stash（将在新分支上恢复）
        Ok(needs_stash)
    }
}
