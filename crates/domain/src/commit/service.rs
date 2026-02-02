//! 提交服务接口

use crate::commit::entity::AmendPreview;
use crate::errors::ServiceError;

/// 提交服务接口
pub trait CommitService: Send + Sync {
    /// Amend 提交
    fn amend_commit(
        &self,
        message: Option<&str>,
        files: &[String],
    ) -> Result<AmendPreview, ServiceError>;

    /// Reword 提交
    fn reword_commit(&self, commit_id: &str, new_message: &str) -> Result<(), ServiceError>;

    /// Squash 提交
    fn squash_commits(&self, count: usize) -> Result<(), ServiceError>;
}
