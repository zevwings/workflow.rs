//! workflow commit commit-diff：测试 GitRepository::get_commit_diff

use prompt::info;

use crate::registry::get_git_repository;

/// 获取指定 commit 的 diff 内容
pub struct CommitDiffCommand {
    ref_or_sha: String,
}

impl CommitDiffCommand {
    pub fn new(ref_or_sha: String) -> Self {
        Self { ref_or_sha }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let git_repo = get_git_repository();
        let diff = git_repo.get_commit_diff(&self.ref_or_sha)?;
        if let Some(patch) = diff {
            info!("{}", patch);
        }
        Ok(())
    }
}
