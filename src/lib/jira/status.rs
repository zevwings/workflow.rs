use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 配置文件路径管理
pub struct ConfigPaths {
    pub jira_status: PathBuf,
    pub work_history: PathBuf,
}

impl ConfigPaths {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(&home);

        // 使用 ${HOME}/.workflow 目录
        let workflow_dir = home_dir.join(".workflow");

        // 确保目录存在
        fs::create_dir_all(&workflow_dir).context("Failed to create .workflow directory")?;

        Ok(Self {
            jira_status: workflow_dir.join("jira-status.json"),
            work_history: workflow_dir.join("work-history.json"),
        })
    }
}

/// 项目状态配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusConfig {
    #[serde(rename = "created-pr")]
    pub created_pull_request_status: Option<String>,
    #[serde(rename = "merged-pr")]
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置（兼容现有接口）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusConfig {
    pub project: String,
    pub created_pull_request_status: Option<String>,
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置存储格式（JSON 对象）
type JiraStatusMap = HashMap<String, ProjectStatusConfig>;

/// 工作历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHistoryEntry {
    pub jira_ticket: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>, // ISO 8601 格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// 工作历史记录存储格式（JSON 对象，PR ID -> Entry）
type WorkHistoryMap = HashMap<String, WorkHistoryEntry>;

/// Jira 状态管理（用于 PR 流程）
pub struct JiraStatus;

impl JiraStatus {
    /// 交互式配置 Jira 状态
    pub fn configure_interactive(jira_ticket_or_project: &str) -> Result<()> {
        // 需要导入的依赖
        use super::helpers::extract_jira_project;
        use crate::{log_info, log_success, Jira};
        use dialoguer::Select;

        // 从 ticket 或项目名中提取项目名
        let project = extract_jira_project(jira_ticket_or_project);

        // 验证项目名格式（Jira 项目名通常只包含字母、数字、下划线，且不包含斜杠）
        let project = if let Some(proj) = project {
            // 如果提取到了项目名，验证格式
            if proj.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                proj
            } else {
                anyhow::bail!(
                    "Invalid Jira project name format: '{}'. Jira project names should contain only ASCII letters, numbers, and underscores (e.g., 'PROJ', 'PROJ-123').",
                    proj
                );
            }
        } else {
            // 如果没有提取到项目名（说明不是 ticket 格式），验证输入是否为有效的项目名
            if jira_ticket_or_project.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                jira_ticket_or_project
            } else {
                anyhow::bail!(
                    "Invalid Jira ticket or project name: '{}'. Expected format: 'PROJ-123' (ticket) or 'PROJ' (project name). The input contains invalid characters (like '/', Chinese characters, etc.).",
                    jira_ticket_or_project
                );
            }
        };

        // 获取项目状态列表
        log_info!("Fetching status list for project: {}", project);
        let statuses =
            Jira::get_project_statuses(project).context("Failed to get project statuses")?;

        if statuses.is_empty() {
            anyhow::bail!("No statuses found for project: {}", project);
        }

        // 选择 PR 创建时的状态
        log_info!(
            "\nSelect one of the following states to change when PR is ready or In progress:"
        );

        // 使用 Select 进行单选
        let selection = Select::new()
            .with_prompt("Select status for PR created")
            .items(&statuses)
            .interact()
            .context("Failed to select status")?;

        let created_pull_request_status = &statuses[selection];

        // 选择 PR 合并时的状态
        log_info!("\nSelect one of the following states to change when PR is merged or Done:");
        let selection = Select::new()
            .with_prompt("Select status for PR merged")
            .items(&statuses)
            .interact()
            .context("Failed to select status")?;

        let merged_pull_request_status = &statuses[selection];

        // 写入配置
        let jira_config = JiraStatusConfig {
            project: project.to_string(),
            created_pull_request_status: Some(created_pull_request_status.clone()),
            merged_pull_request_status: Some(merged_pull_request_status.clone()),
        };

        Self::write_status_config(&jira_config)
            .context("Failed to write Jira status configuration")?;

        log_success!("Jira status configuration saved");
        log_info!("  PR created status: {}", created_pull_request_status);
        log_info!("  PR merged status: {}", merged_pull_request_status);

        Ok(())
    }

    /// 读取 PR 创建时的状态
    pub fn read_pull_request_created_status(jira_ticket: &str) -> Result<Option<String>> {
        use super::helpers::extract_jira_project;

        let project = extract_jira_project(jira_ticket).context("Invalid Jira ticket format")?;

        let config = Self::read_status_config(project)
            .context("Failed to read Jira status configuration")?;

        Ok(config.created_pull_request_status)
    }

    /// 读取 PR 合并时的状态
    pub fn read_pull_request_merged_status(jira_ticket: &str) -> Result<Option<String>> {
        use super::helpers::extract_jira_project;

        let project = extract_jira_project(jira_ticket).context("Invalid Jira ticket format")?;

        let config = Self::read_status_config(project)
            .context("Failed to read Jira status configuration")?;

        Ok(config.merged_pull_request_status)
    }

    /// 读取工作历史记录（通过 PR ID 查找 Jira ticket）
    ///
    /// 文件格式（JSON）：
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
    pub fn read_work_history(pull_request_id: &str) -> Result<Option<String>> {
        let entry = Self::read_work_history_entry(pull_request_id)?;
        Ok(entry.map(|e| e.jira_ticket))
    }

    /// 写入工作历史记录
    pub fn write_work_history(
        jira_ticket: &str,
        pull_request_id: &str,
        pull_request_url: Option<&str>,
        repository: Option<&str>,
        branch: Option<&str>,
    ) -> Result<()> {
        let paths = ConfigPaths::new()?;

        // 读取现有配置
        let mut history_map: WorkHistoryMap = if paths.work_history.exists() {
            let content = fs::read_to_string(&paths.work_history)
                .context("Failed to read existing work-history.json")?;
            serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };

        // 获取当前时间（ISO 8601 格式）
        let created_at = Utc::now().to_rfc3339();

        // 更新或插入记录
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

        // 写入 JSON（使用 pretty print 以便阅读）
        let json = serde_json::to_string_pretty(&history_map)
            .context("Failed to serialize work-history.json")?;

        fs::write(&paths.work_history, json).context("Failed to write work-history.json")?;

        Ok(())
    }

    /// 更新工作历史记录的合并时间
    pub fn update_work_history_merged(pull_request_id: &str) -> Result<()> {
        let paths = ConfigPaths::new()?;
        if !paths.work_history.exists() {
            return Ok(());
        }

        let content =
            fs::read_to_string(&paths.work_history).context("Failed to read work-history.json")?;

        let mut history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history.json")?;

        // 更新合并时间
        if let Some(entry) = history_map.get_mut(pull_request_id) {
            entry.merged_at = Some(Utc::now().to_rfc3339());
        }

        // 写入 JSON
        let json = serde_json::to_string_pretty(&history_map)
            .context("Failed to serialize work-history.json")?;

        fs::write(&paths.work_history, json).context("Failed to write work-history.json")?;

        Ok(())
    }

    /// 读取 Jira 状态配置（内部方法）
    fn read_status_config(project: &str) -> Result<JiraStatusConfig> {
        let paths = ConfigPaths::new()?;

        // 如果文件不存在，返回空配置
        if !paths.jira_status.exists() {
            return Ok(JiraStatusConfig {
                project: project.to_string(),
                created_pull_request_status: None,
                merged_pull_request_status: None,
            });
        }

        let content =
            fs::read_to_string(&paths.jira_status).context("Failed to read jira-status.json")?;

        // 解析 JSON
        let status_map: JiraStatusMap =
            serde_json::from_str(&content).context("Failed to parse jira-status.json")?;

        // 查找项目配置
        let project_config = status_map.get(project);

        Ok(JiraStatusConfig {
            project: project.to_string(),
            created_pull_request_status: project_config
                .and_then(|c| c.created_pull_request_status.clone()),
            merged_pull_request_status: project_config
                .and_then(|c| c.merged_pull_request_status.clone()),
        })
    }

    /// 写入 Jira 状态配置（内部方法）
    fn write_status_config(config: &JiraStatusConfig) -> Result<()> {
        let paths = ConfigPaths::new()?;

        // 读取现有配置
        let mut status_map: JiraStatusMap = if paths.jira_status.exists() {
            let content = fs::read_to_string(&paths.jira_status)
                .context("Failed to read existing jira-status.json")?;
            serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };

        // 更新或插入项目配置
        status_map.insert(
            config.project.clone(),
            ProjectStatusConfig {
                created_pull_request_status: config.created_pull_request_status.clone(),
                merged_pull_request_status: config.merged_pull_request_status.clone(),
            },
        );

        // 写入 JSON（使用 pretty print 以便阅读）
        let json = serde_json::to_string_pretty(&status_map)
            .context("Failed to serialize jira-status.json")?;

        fs::write(&paths.jira_status, json).context("Failed to write jira-status.json")?;

        Ok(())
    }

    /// 读取完整的工作历史记录条目（内部方法）
    fn read_work_history_entry(pull_request_id: &str) -> Result<Option<WorkHistoryEntry>> {
        let paths = ConfigPaths::new()?;
        if !paths.work_history.exists() {
            return Ok(None);
        }

        let content =
            fs::read_to_string(&paths.work_history).context("Failed to read work-history.json")?;

        // 解析 JSON
        let history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history.json")?;

        // 查找 PR ID
        Ok(history_map.get(pull_request_id).cloned())
    }
}

#[cfg(test)]
mod tests {}
