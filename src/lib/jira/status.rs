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
        fs::create_dir_all(&workflow_dir)
            .context("Failed to create .workflow directory")?;

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
    pub created_pr_status: Option<String>,
    #[serde(rename = "merged-pr")]
    pub merged_pr_status: Option<String>,
}

/// Jira 状态配置（兼容现有接口）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusConfig {
    pub project: String,
    pub created_pr_status: Option<String>,
    pub merged_pr_status: Option<String>,
}

/// Jira 状态配置存储格式（JSON 对象）
type JiraStatusMap = HashMap<String, ProjectStatusConfig>;

/// 工作历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHistoryEntry {
    pub jira_ticket: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
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

/// 读取环境变量
#[allow(dead_code)]
pub fn get_env_var(key: &str) -> Result<String> {
    std::env::var(key).with_context(|| format!("Environment variable {} not set", key))
}

/// 读取可选的环境变量
#[allow(dead_code)]
pub fn get_env_var_opt(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// 读取 Jira 状态配置
///
/// 文件格式（JSON）：
/// ```json
/// {
///   "PROJ": {
///     "created-pr": "In Review",
///     "merged-pr": "Done"
///   }
/// }
/// ```
pub fn read_jira_status(project: &str) -> Result<JiraStatusConfig> {
    let paths = ConfigPaths::new()?;

    // 如果文件不存在，返回空配置
    if !paths.jira_status.exists() {
        return Ok(JiraStatusConfig {
            project: project.to_string(),
            created_pr_status: None,
            merged_pr_status: None,
        });
    }

    let content = fs::read_to_string(&paths.jira_status)
        .context("Failed to read jira-status.json")?;

    // 解析 JSON
    let status_map: JiraStatusMap = serde_json::from_str(&content)
        .context("Failed to parse jira-status.json")?;

    // 查找项目配置
    let project_config = status_map.get(project);

    Ok(JiraStatusConfig {
        project: project.to_string(),
        created_pr_status: project_config
            .and_then(|c| c.created_pr_status.clone()),
        merged_pr_status: project_config
            .and_then(|c| c.merged_pr_status.clone()),
    })
}

/// 写入 Jira 状态配置
pub fn write_jira_status(config: &JiraStatusConfig) -> Result<()> {
    let paths = ConfigPaths::new()?;

    // 读取现有配置
    let mut status_map: JiraStatusMap = if paths.jira_status.exists() {
        let content = fs::read_to_string(&paths.jira_status)
            .context("Failed to read existing jira-status.json")?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    };

    // 更新或插入项目配置
    status_map.insert(
        config.project.clone(),
        ProjectStatusConfig {
            created_pr_status: config.created_pr_status.clone(),
            merged_pr_status: config.merged_pr_status.clone(),
        },
    );

    // 写入 JSON（使用 pretty print 以便阅读）
    let json = serde_json::to_string_pretty(&status_map)
        .context("Failed to serialize jira-status.json")?;

    fs::write(&paths.jira_status, json)
        .context("Failed to write jira-status.json")?;

    Ok(())
}

/// 读取工作历史记录（通过 PR ID 查找 Jira ticket）
///
/// 文件格式（JSON）：
/// ```json
/// {
///   "456": {
///     "jira_ticket": "PROJ-123",
///     "pr_url": "https://github.com/xxx/pull/456",
///     "created_at": "2024-01-15T10:30:00Z",
///     "merged_at": null,
///     "repository": "github.com/xxx/yyy",
///     "branch": "feature/PROJ-123-add-feature"
///   }
/// }
/// ```
pub fn read_work_history(pr_id: &str) -> Result<Option<String>> {
    let entry = read_work_history_entry(pr_id)?;
    Ok(entry.map(|e| e.jira_ticket))
}

/// 读取完整的工作历史记录条目
pub fn read_work_history_entry(pr_id: &str) -> Result<Option<WorkHistoryEntry>> {
    let paths = ConfigPaths::new()?;
    if !paths.work_history.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&paths.work_history)
        .context("Failed to read work-history.json")?;

    // 解析 JSON
    let history_map: WorkHistoryMap = serde_json::from_str(&content)
        .context("Failed to parse work-history.json")?;

    // 查找 PR ID
    Ok(history_map.get(pr_id).cloned())
}

/// 写入工作历史记录
pub fn write_work_history(
    jira_ticket: &str,
    pr_id: &str,
    pr_url: Option<&str>,
    repository: Option<&str>,
    branch: Option<&str>,
) -> Result<()> {
    let paths = ConfigPaths::new()?;

    // 读取现有配置
    let mut history_map: WorkHistoryMap = if paths.work_history.exists() {
        let content = fs::read_to_string(&paths.work_history)
            .context("Failed to read existing work-history.json")?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    };

    // 获取当前时间（ISO 8601 格式）
    let created_at = Utc::now().to_rfc3339();

    // 更新或插入记录
    history_map.insert(
        pr_id.to_string(),
        WorkHistoryEntry {
            jira_ticket: jira_ticket.to_string(),
            pr_url: pr_url.map(|s| s.to_string()),
            created_at: Some(created_at),
            merged_at: None,
            repository: repository.map(|s| s.to_string()),
            branch: branch.map(|s| s.to_string()),
        },
    );

    // 写入 JSON（使用 pretty print 以便阅读）
    let json = serde_json::to_string_pretty(&history_map)
        .context("Failed to serialize work-history.json")?;

    fs::write(&paths.work_history, json)
        .context("Failed to write work-history.json")?;

    Ok(())
}

/// 更新工作历史记录的合并时间
pub fn update_work_history_merged(pr_id: &str) -> Result<()> {
    let paths = ConfigPaths::new()?;
    if !paths.work_history.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&paths.work_history)
        .context("Failed to read work-history.json")?;

    let mut history_map: WorkHistoryMap = serde_json::from_str(&content)
        .context("Failed to parse work-history.json")?;

    // 更新合并时间
    if let Some(entry) = history_map.get_mut(pr_id) {
        entry.merged_at = Some(Utc::now().to_rfc3339());
    }

    // 写入 JSON
    let json = serde_json::to_string_pretty(&history_map)
        .context("Failed to serialize work-history.json")?;

    fs::write(&paths.work_history, json)
        .context("Failed to write work-history.json")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
