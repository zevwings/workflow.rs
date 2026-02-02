//! Jira 仓储接口

use crate::jira::entity::{JiraAttachment, JiraIssue};
use crate::jira::error::JiraError;
use crate::jira::status::JiraStatusConfig;
use crate::JiraUser;
use std::path::{Path, PathBuf};

/// 附件下载结果
#[derive(Debug, Clone)]
pub struct AttachmentDownloadResult {
    /// 基础目录路径
    pub base_dir: PathBuf,
    /// 成功下载的文件列表
    pub downloaded_files: Vec<PathBuf>,
    /// 失败的文件列表（文件名，错误信息）
    pub failed_files: Vec<(String, String)>,
}

/// Jira 仓储接口
///
/// 提供 Jira API 操作的接口定义。
pub trait JiraRepository: Send + Sync {
    /// 获取 Jira 用户信息
    fn get_user_info(&self) -> Result<JiraUser, JiraError>;

    /// 获取 Issue 信息
    fn get_issue_info(&self, issue_id: &str) -> Result<JiraIssue, JiraError>;

    /// 更新 Issue 状态
    fn update_issue_status(&self, issue_id: &str, status: &str) -> Result<(), JiraError>;

    /// 添加评论
    fn add_comment(&self, issue_id: &str, comment: &str) -> Result<(), JiraError>;

    /// 获取附件列表
    fn get_attachments(&self, issue_id: &str) -> Result<Vec<JiraAttachment>, JiraError>;

    /// 下载附件
    ///
    /// # 参数
    ///
    /// * `issue_id` - Jira ticket ID
    /// * `base_dir` - 基础目录路径（用于创建下载目录）
    ///
    /// # 返回
    ///
    /// 返回下载结果，包含基础目录路径、成功下载的文件列表和失败的文件列表。
    fn download_attachments(
        &self,
        issue_id: &str,
        base_dir: &Path,
    ) -> Result<AttachmentDownloadResult, JiraError>;

    /// 清理附件目录
    ///
    /// 清理指定 JIRA ID 的附件目录，或清理所有附件目录。
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ID（如 "PROJ-123"）。如果为 `None`，清理所有附件目录
    ///
    /// # 返回
    ///
    /// 返回清理结果，包含是否成功删除、目录是否存在等信息。
    fn clean_attachments(&self, jira_id: Option<&str>) -> Result<(), JiraError>;

    /// 获取项目状态列表
    ///
    /// 从 Jira API 获取指定项目的所有可用状态列表。
    ///
    /// # 参数
    ///
    /// * `project` - 项目名称（如 `"PROJ"`）
    ///
    /// # 返回
    ///
    /// 返回项目状态名称列表。
    fn get_project_statuses(&self, project: &str) -> Result<Vec<String>, JiraError>;

    /// 写入 Jira 状态配置
    ///
    /// 将状态配置写入 `jira.toml` 配置文件。
    /// 如果项目配置已存在，则更新；如果不存在，则创建新配置。
    ///
    /// # 参数
    ///
    /// * `config` - Jira 状态配置结构体
    fn write_status_config(&self, config: &JiraStatusConfig) -> Result<(), JiraError>;

    /// 读取 PR 创建时的状态
    ///
    /// 从配置文件中读取指定 Jira ticket 所属项目的 PR 创建时的目标状态。
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    ///
    /// # 返回
    ///
    /// 返回 PR 创建时的目标状态名称（如果已配置），否则返回 `None`。
    fn read_pull_request_created_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError>;

    /// 读取 PR 合并时的状态
    ///
    /// 从配置文件中读取指定 Jira ticket 所属项目的 PR 合并时的目标状态。
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    ///
    /// # 返回
    ///
    /// 返回 PR 合并时的目标状态名称（如果已配置），否则返回 `None`。
    fn read_pull_request_merged_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError>;
}
