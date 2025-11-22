//! Jira 工作历史记录管理
//!
//! 本模块提供了 PR 创建和合并的工作历史记录管理功能：
//! - 读取工作历史记录（通过 PR ID 查找 Jira ticket）
//! - 根据分支名查找 PR ID
//! - 写入工作历史记录
//! - 更新工作历史记录的合并时间
//! - 删除工作历史记录条目

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::base::settings::paths::Paths;
use crate::{log_info, log_warning};

/// 工作历史记录条目
///
/// 记录 PR 的创建和合并信息，包括 Jira ticket、PR URL、时间戳等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHistoryEntry {
    /// Jira ticket ID（如 `"PROJ-123"`）
    pub jira_ticket: String,
    /// Pull Request URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    /// PR 创建时间（ISO 8601 格式，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// PR 合并时间（ISO 8601 格式，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
    /// 仓库地址（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// 分支名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// 工作历史记录存储格式（JSON 对象，PR ID -> Entry）
type WorkHistoryMap = HashMap<String, WorkHistoryEntry>;

/// Jira 工作历史记录管理
///
/// 提供 PR 创建和合并的工作历史记录读写功能。
pub struct JiraWorkHistory;

impl JiraWorkHistory {
    /// 读取工作历史记录（通过 PR ID 查找 Jira ticket）
    ///
    /// 从工作历史记录文件中查找指定 PR ID 对应的 Jira ticket。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `repository` - 仓库地址（可选，如果提供则从仓库特定的文件读取）
    ///
    /// # 返回
    ///
    /// 返回 Jira ticket ID（如果找到），否则返回 `None`。
    ///
    /// # 文件格式
    ///
    /// 工作历史记录文件格式（JSON）：
    /// ```json
    /// {
    ///   "456": {
    ///     "jira_ticket": "PROJ-123",
    ///     "pull_request_url": "https://github.com/xxx/pull/456",
    ///     "created_at": "2024-01-15T10:30:00Z",
    ///     "merged_at": null,
    ///     "repository": "github.com/xxx/yyy",
    ///     "branch": "feature/PROJ-123-add-feature"
    ///   }
    /// }
    /// ```
    ///
    /// # 错误
    ///
    /// 如果读取文件失败，返回相应的错误信息。
    pub fn read_work_history(
        pull_request_id: &str,
        repository: Option<&str>,
    ) -> Result<Option<String>> {
        let entry = Self::read_work_history_entry(pull_request_id, repository)?;
        Ok(entry.map(|e| e.jira_ticket))
    }

    /// 根据分支名从工作历史记录中查找 PR ID
    ///
    /// 从工作历史记录文件中查找指定分支名对应的 PR ID。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 分支名称（如 `"feature/PROJ-123-add-feature"`）
    /// * `repository` - 仓库地址（必需，用于确定存储文件）
    ///
    /// # 返回
    ///
    /// 返回 PR ID（如果找到），否则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果读取文件失败，返回相应的错误信息。
    pub fn find_pr_id_by_branch(
        branch_name: &str,
        repository: Option<&str>,
    ) -> Result<Option<String>> {
        let repo_url = repository.context("Repository URL is required for work history")?;
        let repo_file = Self::get_repo_work_history_path(repo_url)?;

        if !repo_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&repo_file).context("Failed to read work-history file")?;
        let history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history file")?;
        for (pr_id, entry) in history_map.iter() {
            if let Some(ref branch) = entry.branch {
                if branch == branch_name {
                    return Ok(Some(pr_id.clone()));
                }
            }
        }

        Ok(None)
    }

    /// 写入工作历史记录
    ///
    /// 将 PR 创建信息写入工作历史记录文件。
    /// 如果记录已存在，则更新；如果不存在，则创建新记录。
    ///
    /// 根据仓库 URL 将记录写入到对应的仓库特定文件中。
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `pull_request_url` - Pull Request URL（可选）
    /// * `repository` - 仓库地址（必需，用于确定存储文件）
    /// * `branch` - 分支名称（可选）
    ///
    /// # 行为
    ///
    /// 1. 根据仓库 URL 确定存储文件路径
    /// 2. 读取现有的工作历史记录（如果文件存在）
    /// 3. 创建或更新指定 PR ID 的记录
    /// 4. 设置 `created_at` 为当前时间（ISO 8601 格式）
    /// 5. 将更新后的记录写入仓库特定的文件
    ///
    /// # 错误
    ///
    /// 如果读取或写入文件失败，返回相应的错误信息。
    pub fn write_work_history(
        jira_ticket: &str,
        pull_request_id: &str,
        pull_request_url: Option<&str>,
        repository: Option<&str>,
        branch: Option<&str>,
    ) -> Result<()> {
        let repo_url = repository.context("Repository URL is required for work history")?;
        let repo_file = Self::get_repo_work_history_path(repo_url)?;

        let mut history_map: WorkHistoryMap = if repo_file.exists() {
            let content = fs::read_to_string(&repo_file)
                .context("Failed to read existing work-history file")?;
            serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };

        let created_at = Utc::now().to_rfc3339();
        history_map.insert(
            pull_request_id.to_string(),
            WorkHistoryEntry {
                jira_ticket: jira_ticket.to_string(),
                pull_request_url: pull_request_url.map(|s| s.to_string()),
                created_at: Some(created_at),
                merged_at: None,
                repository: repository.map(|s| s.to_string()),
                branch: branch.map(|s| s.to_string()),
            },
        );

        let json = serde_json::to_string_pretty(&history_map)
            .context("Failed to serialize work-history")?;

        fs::write(&repo_file, json).context("Failed to write work-history file")?;

        Ok(())
    }

    /// 更新工作历史记录的合并时间
    ///
    /// 更新指定 PR ID 的工作历史记录，设置 `merged_at` 为当前时间。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `repository` - 仓库地址（必需，用于确定存储文件）
    ///
    /// # 行为
    ///
    /// 1. 根据仓库 URL 确定存储文件路径
    /// 2. 读取现有的工作历史记录
    /// 3. 查找指定 PR ID 的记录
    /// 4. 如果找到，更新 `merged_at` 为当前时间（ISO 8601 格式）
    /// 5. 将更新后的记录写入仓库特定的文件
    ///
    /// # 返回
    ///
    /// 如果文件不存在或记录不存在，返回 `Ok(())`（不报错）。
    ///
    /// # 错误
    ///
    /// 如果读取或写入文件失败，返回相应的错误信息。
    pub fn update_work_history_merged(
        pull_request_id: &str,
        repository: Option<&str>,
    ) -> Result<()> {
        let repo_url = repository.context("Repository URL is required for work history")?;
        let repo_file = Self::get_repo_work_history_path(repo_url)?;

        if !repo_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&repo_file).context("Failed to read work-history file")?;

        let mut history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history file")?;

        if let Some(entry) = history_map.get_mut(pull_request_id) {
            entry.merged_at = Some(Utc::now().to_rfc3339());
        }

        let json = serde_json::to_string_pretty(&history_map)
            .context("Failed to serialize work-history file")?;

        fs::write(&repo_file, json).context("Failed to write work-history file")?;

        Ok(())
    }

    /// 删除工作历史记录中的 PR ID 条目
    ///
    /// 从工作历史记录文件中删除指定 PR ID 的条目。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"157"`）
    /// * `repository` - 仓库地址（必需，用于确定存储文件）
    ///
    /// # 行为
    ///
    /// 1. 根据仓库 URL 确定存储文件路径
    /// 2. 读取现有的工作历史记录
    /// 3. 删除指定 PR ID 的条目
    /// 4. 将更新后的记录写入仓库特定的文件
    ///
    /// # 返回
    ///
    /// 如果文件不存在或记录不存在，返回 `Ok(())`（不报错）。
    ///
    /// # 错误
    ///
    /// 如果读取或写入文件失败，返回相应的错误信息。
    pub fn delete_work_history_entry(
        pull_request_id: &str,
        repository: Option<&str>,
    ) -> Result<()> {
        let repo_url = repository.context("Repository URL is required for work history")?;
        let repo_file = Self::get_repo_work_history_path(repo_url)?;

        if !repo_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&repo_file).context("Failed to read work-history file")?;

        let mut history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history file")?;

        if history_map.remove(pull_request_id).is_some() {
            log_info!("Removed PR #{} from work-history", pull_request_id);

            if history_map.is_empty() {
                fs::remove_file(&repo_file).context("Failed to remove empty work-history file")?;
                log_info!("Removed empty work-history file");
            } else {
                let json = serde_json::to_string_pretty(&history_map)
                    .context("Failed to serialize work-history file")?;

                fs::write(&repo_file, json).context("Failed to write work-history file")?;
            }
        } else {
            log_warning!(
                "PR #{} not found in work-history, skipping deletion",
                pull_request_id
            );
        }

        Ok(())
    }

    /// 读取完整的工作历史记录条目（内部方法）
    ///
    /// 从工作历史记录文件中读取指定 PR ID 的完整记录。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - Pull Request ID（如 `"456"`）
    /// * `repository` - 仓库地址（可选，如果提供则从仓库特定的文件读取）
    ///
    /// # 返回
    ///
    /// 返回 `WorkHistoryEntry` 结构体（如果找到），否则返回 `None`。
    /// 如果文件不存在，返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果读取或解析文件失败，返回相应的错误信息。
    fn read_work_history_entry(
        pull_request_id: &str,
        repository: Option<&str>,
    ) -> Result<Option<WorkHistoryEntry>> {
        let repo_url = repository.context("Repository URL is required for work history")?;
        let repo_file = Self::get_repo_work_history_path(repo_url)?;

        if !repo_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&repo_file).context("Failed to read work-history file")?;
        let history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history file")?;

        Ok(history_map.get(pull_request_id).cloned())
    }

    /// 规范化仓库 URL 为文件名
    ///
    /// 将仓库 URL 转换为可用于文件名的字符串。
    ///
    /// # 转换规则
    ///
    /// - `git@github.com:owner/repo.git` → `github.com-owner-repo`
    /// - `https://github.com/owner/repo.git` → `github.com-owner-repo`
    /// - `http://github.com/owner/repo.git` → `github.com-owner-repo`
    fn normalize_repo_to_filename(repo_url: &str) -> String {
        repo_url
            .replace("git@", "")
            .replace("https://", "")
            .replace("http://", "")
            .replace(":", "-")
            .replace("/", "-")
            .replace(".git", "")
            .replace(".", "-")
    }

    /// 获取仓库特定的工作历史文件路径
    ///
    /// 根据仓库 URL 生成对应的历史文件路径。
    /// 文件存储在 `${HOME}/.workflow/work-history/` 目录下。
    ///
    /// # 参数
    ///
    /// * `repo_url` - 仓库 URL（如 `"git@github.com:owner/repo.git"`）
    ///
    /// # 返回
    ///
    /// 返回仓库特定的历史文件路径。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录或获取路径，返回相应的错误信息。
    fn get_repo_work_history_path(repo_url: &str) -> Result<PathBuf> {
        let history_dir = Paths::work_history_dir()?;
        let repo_id = Self::normalize_repo_to_filename(repo_url);
        Ok(history_dir.join(format!("{}.json", repo_id)))
    }
}

#[cfg(test)]
mod tests {}
