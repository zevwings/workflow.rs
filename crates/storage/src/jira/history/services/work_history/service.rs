//! 工作历史记录服务
//!
//! 提供 PR 创建和合并的工作历史记录管理功能，包括：
//! - 读取工作历史记录（通过 PR ID 查找 Jira ticket）
//! - 根据分支名查找 PR ID
//! - 写入工作历史记录
//! - 更新工作历史记录的合并时间
//! - 删除工作历史记录条目

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use domain::{DeleteHistoryResult, JiraError, PathService, WorkHistoryEntry};
use toolkit::{file, log_warn};

/// 工作历史记录存储格式（JSON 对象，PR ID -> Entry）
type WorkHistoryMap = HashMap<String, WorkHistoryEntry>;

/// 工作历史记录服务 trait
pub trait WorkHistoryService: Send + Sync {
    /// 读取工作历史记录（通过 PR ID 查找 Jira ticket）
    fn read_work_history(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError>;

    /// 读取完整的工作历史记录条目
    fn read_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<WorkHistoryEntry>, JiraError>;

    /// 根据分支名从工作历史记录中查找 PR ID
    fn find_pr_id_by_branch(
        &self,
        branch_name: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError>;

    /// 写入工作历史记录
    fn write_work_history(
        &self,
        jira_ticket: &str,
        pull_request_id: &str,
        pull_request_url: Option<&str>,
        repository: &str,
        branch: Option<&str>,
    ) -> Result<(), JiraError>;

    /// 更新工作历史记录的合并时间
    fn update_work_history_merged(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<(), JiraError>;

    /// 删除工作历史记录中的 PR ID 条目
    fn delete_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<DeleteHistoryResult, JiraError>;

    /// 根据 JIRA ticket 查找关联的 PR 列表
    fn find_prs_by_jira_ticket(
        &self,
        jira_ticket: &str,
    ) -> Result<Vec<WorkHistoryEntry>, JiraError>;

    /// 根据 JIRA ticket 查找关联的分支列表
    fn find_branches_by_jira_ticket(&self, jira_ticket: &str) -> Result<Vec<String>, JiraError>;
}

/// 工作历史记录服务实现
pub struct WorkHistoryServiceImpl {
    /// 工作历史目录路径提供者
    // history_dir_provider: Arc<dyn Fn() -> Result<PathBuf, JiraError> + Send + Sync + 'static>,
    path_service: Arc<dyn PathService>,
}

impl WorkHistoryServiceImpl {
    /// 创建新的工作历史记录服务实例
    pub fn new(path_service: Arc<dyn PathService>) -> Self {
        Self { path_service }
    }

    // /// 获取工作历史目录路径
    // fn history_dir(&self) -> Result<PathBuf, JiraError> {
    //     (self.history_dir_provider)()
    // }

    /// 规范化仓库 URL 为文件名
    ///
    /// 将仓库 URL 转换为可用于文件名的字符串。
    ///
    /// # 转换规则
    ///
    /// - `git@github.com:owner/repo.git` → `github-com-owner-repo`
    /// - `https://github.com/owner/repo.git` → `github-com-owner-repo`
    /// - `http://github.com/owner/repo.git` → `github-com-owner-repo`
    fn normalize_repo_to_filename(repo_url: &str) -> String {
        repo_url
            .replace("git@", "")
            .replace("https://", "")
            .replace("http://", "")
            .replace([':', '/'], "-")
            .replace(".git", "")
            .replace('.', "-")
    }

    /// 获取仓库特定的工作历史文件路径
    fn get_repo_work_history_path(&self, repo_url: &str) -> Result<PathBuf, JiraError> {
        let history_dir = self
            .path_service
            .get_jira_work_history_dir()
            .map_err(|e| JiraError::ApiError(format!("Failed to get history dir: {}", e)))?;
        let repo_id = Self::normalize_repo_to_filename(repo_url);
        Ok(history_dir.join(format!("{}.json", repo_id)))
    }

    /// 读取工作历史文件
    fn read_history_map(&self, path: &Path) -> Result<WorkHistoryMap, JiraError> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = file::read_string(path)
            .map_err(|e| JiraError::ApiError(format!("Failed to read work history file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| JiraError::ApiError(format!("Failed to parse work history JSON: {}", e)))
    }

    /// 写入工作历史文件
    fn write_history_map(&self, path: &Path, map: &WorkHistoryMap) -> Result<(), JiraError> {
        let content = serde_json::to_string_pretty(map)
            .map_err(|e| JiraError::ApiError(format!("Failed to serialize work history: {}", e)))?;

        file::write_string(path, &content)
            .map_err(|e| JiraError::ApiError(format!("Failed to write work history file: {}", e)))
    }
}

impl WorkHistoryService for WorkHistoryServiceImpl {
    fn read_work_history(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError> {
        let entry = self.read_work_history_entry(pull_request_id, repository)?;
        Ok(entry.map(|e| e.jira_ticket))
    }

    fn read_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<Option<WorkHistoryEntry>, JiraError> {
        let repo_file = self.get_repo_work_history_path(repository)?;

        if !repo_file.exists() {
            return Ok(None);
        }

        let history_map = self.read_history_map(&repo_file)?;
        Ok(history_map.get(pull_request_id).cloned())
    }

    fn find_pr_id_by_branch(
        &self,
        branch_name: &str,
        repository: &str,
    ) -> Result<Option<String>, JiraError> {
        let repo_file = self.get_repo_work_history_path(repository)?;

        if !repo_file.exists() {
            return Ok(None);
        }

        let history_map = self.read_history_map(&repo_file)?;
        for (pr_id, entry) in history_map.iter() {
            if let Some(ref branch) = entry.branch {
                if branch == branch_name {
                    return Ok(Some(pr_id.to_string()));
                }
            }
        }

        Ok(None)
    }

    fn write_work_history(
        &self,
        jira_ticket: &str,
        pull_request_id: &str,
        pull_request_url: Option<&str>,
        repository: &str,
        branch: Option<&str>,
    ) -> Result<(), JiraError> {
        let repo_file = self.get_repo_work_history_path(repository)?;

        let mut history_map = if repo_file.exists() {
            self.read_history_map(&repo_file).unwrap_or_default()
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
                repository: Some(repository.to_string()),
                branch: branch.map(|s| s.to_string()),
            },
        );

        self.write_history_map(&repo_file, &history_map)?;

        Ok(())
    }

    fn update_work_history_merged(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<(), JiraError> {
        let repo_file = self.get_repo_work_history_path(repository)?;

        if !repo_file.exists() {
            return Ok(());
        }

        let mut history_map = self.read_history_map(&repo_file)?;

        if let Some(entry) = history_map.get_mut(pull_request_id) {
            entry.merged_at = Some(Utc::now().to_rfc3339());
        }

        self.write_history_map(&repo_file, &history_map)?;

        Ok(())
    }

    fn delete_work_history_entry(
        &self,
        pull_request_id: &str,
        repository: &str,
    ) -> Result<DeleteHistoryResult, JiraError> {
        let repo_file = self.get_repo_work_history_path(repository)?;

        if !repo_file.exists() {
            return Ok(DeleteHistoryResult {
                messages: vec![],
                warnings: vec![],
            });
        }

        let mut history_map = self.read_history_map(&repo_file)?;

        let mut messages = Vec::new();
        let mut warnings = Vec::new();

        if history_map.remove(pull_request_id).is_some() {
            messages.push(format!("Removed PR #{} from work-history", pull_request_id));

            if history_map.is_empty() {
                fs::remove_file(&repo_file).map_err(|e| {
                    JiraError::ApiError(format!("Failed to remove empty work-history file: {}", e))
                })?;
                messages.push("Removed empty work-history file".to_string());
            } else {
                self.write_history_map(&repo_file, &history_map)?;
            }
        } else {
            let warning_msg = format!(
                "PR #{} not found in work-history, skipping deletion",
                pull_request_id
            );
            log_warn!("{}", warning_msg);
            warnings.push(warning_msg);
        }

        Ok(DeleteHistoryResult { messages, warnings })
    }

    fn find_prs_by_jira_ticket(
        &self,
        jira_ticket: &str,
    ) -> Result<Vec<WorkHistoryEntry>, JiraError> {
        let history_dir = self
            .path_service
            .get_jira_work_history_dir()
            .map_err(|e| JiraError::ApiError(format!("Failed to get history dir: {}", e)))?;
        let mut results = Vec::new();

        if !history_dir.exists() {
            return Ok(results);
        }

        let entries = fs::read_dir(&history_dir).map_err(|e| {
            JiraError::ApiError(format!("Failed to read work-history directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                JiraError::ApiError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let history_map = self.read_history_map(&path)?;

                for (_, entry) in history_map.iter() {
                    if entry.jira_ticket == jira_ticket {
                        results.push(entry.clone());
                    }
                }
            }
        }

        Ok(results)
    }

    fn find_branches_by_jira_ticket(&self, jira_ticket: &str) -> Result<Vec<String>, JiraError> {
        let entries = self.find_prs_by_jira_ticket(jira_ticket)?;
        let branches: Vec<String> = entries.iter().filter_map(|e| e.branch.clone()).collect();
        Ok(branches)
    }
}
