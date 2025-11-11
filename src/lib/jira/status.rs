//! Jira 状态管理
//!
//! 本模块提供了 Jira 状态相关的完整功能：
//! - 获取项目状态列表（通过 REST API）
//! - 交互式配置状态映射（PR 创建/合并时的状态）
//! - 读取状态配置
//! - 工作历史记录管理（PR 创建/合并记录）

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::http::{Authorization, HttpClient};
use crate::{log_debug, log_info, log_success};
use dialoguer::Select;

use super::helpers::{extract_jira_project, get_auth, get_base_url};

/// 配置文件路径管理
///
/// 管理 Jira 状态配置和工作历史记录的存储路径。
/// 所有文件都存储在 `${HOME}/.workflow` 目录下。
pub struct ConfigPaths {
    /// Jira 状态配置文件路径（`${HOME}/.workflow/jira-status.json`）
    pub jira_status: PathBuf,
    /// 工作历史记录文件路径（`${HOME}/.workflow/work-history.json`）
    pub work_history: PathBuf,
}

impl ConfigPaths {
    /// 创建配置文件路径管理器
    ///
    /// 初始化配置文件路径，确保 `.workflow` 目录存在。
    ///
    /// # 返回
    ///
    /// 返回 `ConfigPaths` 结构体，包含所有配置文件的路径。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或无法创建目录，返回相应的错误信息。
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
///
/// 存储单个项目的状态配置，包括 PR 创建和合并时的目标状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusConfig {
    /// PR 创建时的目标状态（JSON 字段名：`created-pr`）
    #[serde(rename = "created-pr")]
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态（JSON 字段名：`merged-pr`）
    #[serde(rename = "merged-pr")]
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置（兼容现有接口）
///
/// 包含项目名称和对应的状态配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusConfig {
    /// 项目名称（如 `"PROJ"`）
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置存储格式（JSON 对象）
type JiraStatusMap = HashMap<String, ProjectStatusConfig>;

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

/// 规范化仓库 URL 为文件名
///
/// 将仓库 URL 转换为可用于文件名的字符串。
///
/// # 示例
/// ```
/// assert_eq!(
///     normalize_repo_to_filename("git@github.com:owner/repo.git"),
///     "github.com-owner-repo"
/// );
/// assert_eq!(
///     normalize_repo_to_filename("https://github.com/owner/repo.git"),
///     "github.com-owner-repo"
/// );
/// ```
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
    let paths = ConfigPaths::new()?;
    let repo_id = normalize_repo_to_filename(repo_url);
    let history_dir = paths
        .work_history
        .parent()
        .context("Failed to get workflow directory")?
        .join("work-history");

    // 确保目录存在
    fs::create_dir_all(&history_dir).context("Failed to create work-history directory")?;

    Ok(history_dir.join(format!("{}.json", repo_id)))
}

/// 获取项目状态列表
///
/// 通过 Jira REST API 获取指定项目的所有可用状态。
///
/// # 参数
///
/// * `project` - Jira 项目名称，如 `"PROJ"`
///
/// # 返回
///
/// 返回状态名称列表。
///
/// # 错误
///
/// 如果项目不存在、无访问权限或 API 调用失败，返回相应的错误信息。
pub fn get_project_statuses(project: &str) -> Result<Vec<String>> {
    let (email, api_token) = get_auth()?;
    let base_url = get_base_url()?;
    let url = format!("{}/project/{}/statuses", base_url, project);

    let client = HttpClient::new()?;
    let auth = Authorization::new(&email, &api_token);
    let response = client
        .get::<serde_json::Value>(&url, Some(&auth), None)
        .context("Failed to fetch project statuses")?;

    if !response.is_success() {
        // 提供更详细的错误信息
        if response.status == 404 {
            anyhow::bail!(
                "Project '{}' not found in Jira. Please check:\n  - The project name is correct\n  - The project exists in your Jira instance\n  - You have access to this project\n  - The project name format is correct (e.g., 'PROJ', not 'zw/修改打包脚本问题')",
                project
            );
        } else if response.status == 403 {
            anyhow::bail!(
                "Access denied for project '{}'. Please check your Jira API token permissions.",
                project
            );
        } else {
            anyhow::bail!(
                "Failed to fetch project statuses for '{}': HTTP {}. Please check the project name and your Jira API access.",
                project,
                response.status
            );
        }
    }

    // 检查响应数据是否是有效的 JSON
    if response.data.is_null() {
        anyhow::bail!(
            "Empty response from Jira API for project '{}'. The project may not exist or you may not have access.",
            project
        );
    }

    // 解析状态列表（取第一个版本的 statuses）
    let statuses = response
        .data
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|version| version.get("statuses"))
        .and_then(|s| s.as_array())
        .with_context(|| {
            format!(
                "Invalid statuses JSON structure for project '{}'. The API response format may have changed. Response: {}",
                project,
                serde_json::to_string_pretty(&response.data).unwrap_or_else(|_| "Unable to serialize response".to_string())
            )
        })?;

    let status_names: Vec<String> = statuses
        .iter()
        .filter_map(|s| s.get("name"))
        .filter_map(|n| n.as_str())
        .map(|s| s.to_string())
        .collect();

    Ok(status_names)
}

/// Jira 状态管理（用于 PR 流程）
///
/// 提供 PR 创建和合并时的状态自动更新功能，
/// 以及工作历史记录的读写功能。
pub struct JiraStatus;

impl JiraStatus {
    /// 交互式配置 Jira 状态
    ///
    /// 通过交互式界面配置指定项目的 PR 创建和合并时的目标状态。
    /// 会从 Jira API 获取项目状态列表，然后让用户选择。
    ///
    /// # 参数
    ///
    /// * `jira_ticket_or_project` - Jira ticket ID（如 `"PROJ-123"`）或项目名称（如 `"PROJ"`）
    ///
    /// # 行为
    ///
    /// 1. 从 ticket 或项目名中提取项目名
    /// 2. 验证项目名格式
    /// 3. 从 Jira API 获取项目状态列表
    /// 4. 交互式选择 PR 创建时的状态
    /// 5. 交互式选择 PR 合并时的状态
    /// 6. 保存配置到本地文件
    ///
    /// # 错误
    ///
    /// 如果项目名格式无效、无法获取状态列表或保存配置失败，返回相应的错误信息。
    pub fn configure_interactive(jira_ticket_or_project: &str) -> Result<()> {
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
            if jira_ticket_or_project
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                jira_ticket_or_project
            } else {
                anyhow::bail!(
                    "Invalid Jira ticket or project name: '{}'. Expected format: 'PROJ-123' (ticket) or 'PROJ' (project name). The input contains invalid characters (like '/', Chinese characters, etc.).",
                    jira_ticket_or_project
                );
            }
        };

        // 获取项目状态列表
        log_debug!("Fetching status list for project: {}", project);
        let statuses = get_project_statuses(project).context("Failed to get project statuses")?;

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
        log_debug!("  PR created status: {}", created_pull_request_status);
        log_debug!("  PR merged status: {}", merged_pull_request_status);

        Ok(())
    }

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
    ///
    /// # 错误
    ///
    /// 如果 ticket 格式无效或读取配置失败，返回相应的错误信息。
    pub fn read_pull_request_created_status(jira_ticket: &str) -> Result<Option<String>> {
        let project = extract_jira_project(jira_ticket).context("Invalid Jira ticket format")?;

        let config = Self::read_status_config(project)
            .context("Failed to read Jira status configuration")?;

        Ok(config.created_pull_request_status)
    }

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
    ///
    /// # 错误
    ///
    /// 如果 ticket 格式无效或读取配置失败，返回相应的错误信息。
    pub fn read_pull_request_merged_status(jira_ticket: &str) -> Result<Option<String>> {
        let project = extract_jira_project(jira_ticket).context("Invalid Jira ticket format")?;

        let config = Self::read_status_config(project)
            .context("Failed to read Jira status configuration")?;

        Ok(config.merged_pull_request_status)
    }

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
        let repo_file = get_repo_work_history_path(repo_url)?;

        if !repo_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&repo_file).context("Failed to read work-history file")?;

        // 解析 JSON
        let history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history file")?;

        // 遍历所有记录，查找匹配的分支名
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
        let repo_file = get_repo_work_history_path(repo_url)?;

        // 读取现有配置
        let mut history_map: WorkHistoryMap = if repo_file.exists() {
            let content = fs::read_to_string(&repo_file)
                .context("Failed to read existing work-history file")?;
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
        let repo_file = get_repo_work_history_path(repo_url)?;

        if !repo_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&repo_file).context("Failed to read work-history file")?;

        let mut history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history file")?;

        // 更新合并时间
        if let Some(entry) = history_map.get_mut(pull_request_id) {
            entry.merged_at = Some(Utc::now().to_rfc3339());
        }

        // 写入 JSON
        let json = serde_json::to_string_pretty(&history_map)
            .context("Failed to serialize work-history file")?;

        fs::write(&repo_file, json).context("Failed to write work-history file")?;

        Ok(())
    }

    /// 读取 Jira 状态配置（内部方法）
    ///
    /// 从配置文件中读取指定项目的状态配置。
    ///
    /// # 参数
    ///
    /// * `project` - 项目名称（如 `"PROJ"`）
    ///
    /// # 返回
    ///
    /// 返回 `JiraStatusConfig` 结构体。
    /// 如果文件不存在或项目配置不存在，返回空配置（所有字段为 `None`）。
    ///
    /// # 错误
    ///
    /// 如果读取或解析文件失败，返回相应的错误信息。
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
    ///
    /// 将状态配置写入配置文件。
    /// 如果项目配置已存在，则更新；如果不存在，则创建新配置。
    ///
    /// # 参数
    ///
    /// * `config` - Jira 状态配置结构体
    ///
    /// # 行为
    ///
    /// 1. 读取现有的状态配置（如果文件存在）
    /// 2. 更新或插入项目配置
    /// 3. 将更新后的配置写入文件（使用 pretty print 格式）
    ///
    /// # 错误
    ///
    /// 如果读取或写入文件失败，返回相应的错误信息。
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
        let repo_file = get_repo_work_history_path(repo_url)?;

        if !repo_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&repo_file).context("Failed to read work-history file")?;

        // 解析 JSON
        let history_map: WorkHistoryMap =
            serde_json::from_str(&content).context("Failed to parse work-history file")?;

        // 查找 PR ID
        Ok(history_map.get(pull_request_id).cloned())
    }
}

#[cfg(test)]
mod tests {}
