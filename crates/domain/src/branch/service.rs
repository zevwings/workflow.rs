//! 分支服务接口

use crate::branch::entity::{BranchSyncOptions, BranchSyncResult};
use crate::errors::ServiceError;

/// 分支服务接口
pub trait BranchService: Send + Sync {
    /// 创建分支
    fn create_branch(
        &self,
        jira_id: Option<&str>,
        branch_type: Option<&str>,
    ) -> Result<String, ServiceError>;

    /// 同步分支
    fn sync_branch(
        &self,
        source_branch: Option<&str>,
        options: &BranchSyncOptions,
    ) -> Result<BranchSyncResult, ServiceError>;

    /// 重命名分支
    fn rename_branch(&self, new_name: &str) -> Result<(), ServiceError>;
}
