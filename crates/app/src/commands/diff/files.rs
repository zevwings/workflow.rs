//! workflow commit files：测试 GitRepository::get_commit_changed_files

use domain::CommitChangeType;
use prompt::info;

use crate::bootstrap::get_git_repository;

/// 获取指定 commit 变更的文件列表
pub struct CommitFilesCommand {
    ref_or_sha: String,
}

impl CommitFilesCommand {
    pub fn new(ref_or_sha: String) -> Self {
        Self { ref_or_sha }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let git_repo = get_git_repository();
        let files = git_repo.get_commit_changed_files(&self.ref_or_sha)?;
        for f in &files {
            let kind = match f.change_type {
                CommitChangeType::Added => "A",
                CommitChangeType::Modified => "M",
                CommitChangeType::Deleted => "D",
                CommitChangeType::Renamed => "R",
                CommitChangeType::Copied => "C",
                CommitChangeType::TypeChanged => "T",
            };
            if let Some(ref old) = f.old_path {
                info!("{} {} -> {}", kind, old, f.path);
            } else {
                info!("{} {}", kind, f.path);
            }
        }
        Ok(())
    }
}
