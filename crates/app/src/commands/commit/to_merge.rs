//! workflow commit to-merge：测试 GitRepository::commits_to_merge

use prompt::info;

use crate::bootstrap::get_git_repository;

/// 列出将源分支合并到目标分支时会引入的 commit
pub struct CommitToMergeCommand {
    source_branch: String,
    target_branch: String,
}

impl CommitToMergeCommand {
    pub fn new(source_branch: String, target_branch: String) -> Self {
        Self {
            source_branch,
            target_branch,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let git_repo = get_git_repository();
        let shas = git_repo.commits_to_merge(&self.source_branch, &self.target_branch)?;
        for sha in &shas {
            info!("{}", sha);
        }
        Ok(())
    }
}
