//! 清理本地分支命令

use domain::{GitError, GitRepository};
use prompt::{confirm, error, info, success, warning};

use crate::registry;

/// 保护分支列表（不可删除）
const PROTECTED_BRANCHES: &[&str] = &["develop", "dev"];

/// 删除分支的结果：(成功数量, 失败列表)
type DeleteBranchesResult = (usize, Vec<(String, GitError)>);

/// Branch Clean 命令
pub struct BranchCleanCommand {
    dry_run: bool,
}

impl BranchCleanCommand {
    /// 创建新的 BranchCleanCommand
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// 运行 `workflow branch clean` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let git_repo = registry::get_git_repository();

        let default_branch = git_repo.get_default_branch()?;
        let current_branch = git_repo.get_current_branch()?;

        // 获取忽略列表
        let ignore_list = registry::get_repo_config_repository()
            .load_user_config()
            .map(|c| c.branch.ignore)
            .unwrap_or_default();

        // 获取所有本地分支并过滤
        let branches_to_clean: Vec<String> = git_repo
            .list_branches(false, false)?
            .into_iter()
            .map(|item| item.name)
            .filter(|name| self.should_clean(name, &default_branch, &current_branch, &ignore_list))
            .collect();

        if branches_to_clean.is_empty() {
            info!("No branches to clean");
            return Ok(());
        }

        // 显示将要删除的分支
        info!("Found {} branch(es) to clean:", branches_to_clean.len());
        for branch in &branches_to_clean {
            info!("  - {}", branch);
        }

        if self.dry_run {
            info!(
                "[DRY RUN] Would delete {} branch(es)",
                branches_to_clean.len()
            );
            return Ok(());
        }

        // 确认删除
        if !confirm!("Delete {} branch(es)?", branches_to_clean.len())
            .default(true)
            .prompt()?
        {
            info!("Operation cancelled");
            return Ok(());
        }

        // 执行删除
        let (deleted, failed) = self.delete_branches(git_repo.as_ref(), &branches_to_clean)?;

        if deleted > 0 {
            success!("Cleaned {} branch(es)", deleted);
        }

        if !failed.is_empty() {
            warning!("Failed to delete {} branch(es)", failed.len());
            for (branch, err) in failed {
                error!("  - {}: {}", branch, err);
            }
        }

        Ok(())
    }

    /// 判断分支是否应该被清理
    fn should_clean(
        &self,
        name: &str,
        default_branch: &str,
        current_branch: &str,
        ignore_list: &[String],
    ) -> bool {
        // 保留默认分支、当前分支、保护分支和忽略列表中的分支
        name != default_branch
            && name != current_branch
            && !PROTECTED_BRANCHES.contains(&name)
            && !ignore_list.iter().any(|i| i == name)
    }

    /// 删除分支列表，返回 (成功数量, 失败列表)
    fn delete_branches(
        &self,
        git_repo: &dyn GitRepository,
        branches: &[String],
    ) -> Result<DeleteBranchesResult, Box<dyn std::error::Error>> {
        let mut deleted = 0;
        let mut failed = Vec::new();

        for branch in branches {
            match git_repo.delete_local_branch(branch, false) {
                Ok(()) => {
                    deleted += 1;
                    info!("Deleted branch '{}'", branch);
                }
                Err(GitError::BranchNotFullyMerged(_)) => {
                    warning!("Branch '{}' is not fully merged", branch);
                    if confirm!("Force delete branch '{}'?", branch).default(false).prompt()? {
                        match git_repo.delete_local_branch(branch, true) {
                            Ok(()) => {
                                deleted += 1;
                                info!("Force deleted branch '{}'", branch);
                            }
                            Err(e) => {
                                warning!("Failed to force delete branch '{}': {}", branch, e);
                                failed.push((branch.clone(), e));
                            }
                        }
                    } else {
                        info!("Skipped branch '{}'", branch);
                    }
                }
                Err(e) => {
                    warning!("Failed to delete branch '{}': {}", branch, e);
                    failed.push((branch.clone(), e));
                }
            }
        }

        Ok((deleted, failed))
    }
}
