use crate::base::settings::Settings;
use crate::git::GitRepo;
use crate::git::RepoType;
use anyhow::{Context, Result};
use regex::Regex;

use super::platform::{create_provider, TYPES_OF_CHANGES};

/// 从 PR URL 提取 PR ID
///
/// # 示例
/// ```
/// use workflow::pr::helpers::extract_pull_request_id_from_url;
/// assert_eq!(extract_pull_request_id_from_url("https://github.com/owner/repo/pull/123").unwrap(), "123".to_string());
/// assert_eq!(extract_pull_request_id_from_url("https://codeup.aliyun.com/xxx/project/xxx/code_reviews/12345").unwrap(), "12345".to_string());
/// ```
pub fn extract_pull_request_id_from_url(url: &str) -> Result<String> {
    let re = Regex::new(r"/(\d+)(?:/|$)").context("Invalid regex pattern")?;
    let caps = re.captures(url).context("Failed to extract PR ID from URL")?;

    Ok(caps.get(1).unwrap().as_str().to_string())
}

/// 从 Git remote URL 提取 GitHub 仓库的 owner/repo
///
/// 支持标准格式和 SSH host 别名格式（如 github-brainim）
///
/// # 示例
/// ```
/// use workflow::pr::helpers::extract_github_repo_from_url;
/// assert_eq!(extract_github_repo_from_url("git@github.com:owner/repo.git").unwrap(), "owner/repo");
/// assert_eq!(extract_github_repo_from_url("git@github-brainim:owner/repo.git").unwrap(), "owner/repo");
/// assert_eq!(extract_github_repo_from_url("https://github.com/owner/repo.git").unwrap(), "owner/repo");
/// ```
pub fn extract_github_repo_from_url(url: &str) -> Result<String> {
    // 匹配 SSH 格式: git@github.com:owner/repo.git 或 git@github-xxx:owner/repo.git (支持 SSH host 别名)
    // 使用 git@github[^:]*: 来匹配 git@github 开头的所有 SSH host（包括别名）
    let ssh_re =
        Regex::new(r"git@github[^:]*:(.+?)(?:\.git)?$").context("Invalid regex pattern")?;
    if let Some(caps) = ssh_re.captures(url) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    // 匹配 HTTPS 格式: https://github.com/owner/repo.git
    let https_re = Regex::new(r"https?://(?:www\.)?github\.com/(.+?)(?:\.git)?/?$")
        .context("Invalid regex pattern")?;
    if let Some(caps) = https_re.captures(url) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    anyhow::bail!("Failed to extract GitHub repo from URL: {}", url)
}

/// 生成 PR body（使用模板系统）
///
/// # Arguments
/// * `selected_change_types` - 选中的变更类型数组
/// * `short_description` - 简短描述（可选）
/// * `jira_ticket` - Jira ticket ID（可选）
/// * `dependency` - 依赖信息（可选）
/// * `jira_info` - Optional JIRA issue information (for template variables)
pub fn generate_pull_request_body(
    selected_change_types: &[bool],
    short_description: Option<&str>,
    jira_ticket: Option<&str>,
    dependency: Option<&str>,
    jira_info: Option<&crate::jira::JiraIssue>,
) -> Result<String> {
    use crate::template::{
        default_pull_request_template, load_pull_request_template, ChangeTypeItem,
        PullRequestTemplateVars, TemplateEngine,
    };

    // Load PR template
    let template_str = load_pull_request_template().unwrap_or_else(|_| {
        // Fallback to default template if loading fails
        default_pull_request_template()
    });

    // Prepare change types
    let change_types: Vec<ChangeTypeItem> = TYPES_OF_CHANGES
        .iter()
        .enumerate()
        .map(|(i, name)| ChangeTypeItem {
            name: name.to_string(),
            selected: i < selected_change_types.len() && selected_change_types[i],
        })
        .collect();

    // Get JIRA service address
    let jira_service_address = Settings::get().jira.service_address.clone();

    // Prepare template variables
    let vars = PullRequestTemplateVars {
        jira_key: jira_ticket.map(|s| s.to_string()),
        jira_summary: jira_info.as_ref().map(|i| i.fields.summary.clone()),
        jira_description: jira_info.as_ref().and_then(|i| i.fields.description.clone()),
        jira_type: None, // TODO: Extract from jira_info if available
        jira_service_address,
        change_types,
        short_description: short_description.map(|s| s.to_string()),
        dependency: dependency.map(|s| s.to_string()),
    };

    // Render template
    let engine = TemplateEngine::new();
    engine
        .render_string(&template_str, &vars)
        .context("Failed to render PR body template")
}

// Branch naming functions have been moved to lib/branch module
// Use branch::BranchNaming::from_jira_ticket() and branch::BranchNaming::from_title() instead

/// 生成 commit 标题（使用模板系统）
///
/// # Arguments
/// * `jira_ticket` - Jira ticket ID（可选）
/// * `title` - PR 标题
/// * `commit_type` - Optional commit type (e.g., "feat", "fix")
/// * `scope` - Optional commit scope
/// * `body` - Optional commit body
pub fn generate_commit_title(
    jira_ticket: Option<&str>,
    title: &str,
    commit_type: Option<&str>,
    scope: Option<&str>,
    body: Option<&str>,
) -> Result<String> {
    use crate::template::{load_commit_template, CommitTemplateVars, TemplateEngine};

    use crate::template::loader::default_commit_template;

    // Load commit template
    let template_str = load_commit_template().unwrap_or_else(|_| {
        // Fallback to default template
        default_commit_template()
    });

    // Prepare template variables
    let vars = CommitTemplateVars {
        commit_type: commit_type.unwrap_or("feat").to_string(),
        scope: scope.map(|s| s.to_string()),
        subject: title.to_string(),
        body: body.map(|s| s.to_string()),
        jira_key: jira_ticket.map(|s| s.to_string()),
    };

    // Render template
    let engine = TemplateEngine::new();
    engine
        .render_string(&template_str, &vars)
        .context("Failed to render commit title template")
}

// transform_to_branch_name has been moved to lib/branch module
// Use branch::BranchNaming::sanitize() instead

/// 根据仓库类型调用平台提供者方法
///
/// 这是一个统一的调度函数，根据仓库类型自动选择对应的平台提供者实现。
/// 它接受一个闭包，闭包接收 RepoType 并根据类型调用对应的 PlatformProvider 方法。
///
/// # 参数
///
/// * `f` - 一个闭包，接收 RepoType 并根据类型调用对应的 PlatformProvider 方法
/// * `operation_name` - 操作名称（用于错误消息）
///
/// # 返回
///
/// 返回闭包执行的结果
///
/// # 错误
///
/// 如果仓库类型未知或不支持，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::pr::{PlatformProvider, detect_repo_type};
/// use workflow::pr::github::GitHub;
/// use workflow::RepoType;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 获取当前分支的 PR ID
/// let pr_id = detect_repo_type(
///     |repo_type| match repo_type {
///         RepoType::GitHub => {
///             let github = GitHub;
///             github.get_current_branch_pull_request()
///         },
///         RepoType::Codeup => Ok(None),  // Codeup support has been removed
///         RepoType::Unknown => Ok(None),
///     },
///     "get current branch PR"
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn detect_repo_type<F, T>(f: F, operation_name: &str) -> Result<T>
where
    F: FnOnce(RepoType) -> Result<T>,
{
    let repo_type = GitRepo::detect_repo_type().context("Failed to detect repository type")?;
    match repo_type {
        RepoType::GitHub => f(repo_type),
        RepoType::Codeup => {
            anyhow::bail!(
                "{} is not supported for Codeup repositories. Codeup support has been removed. Only GitHub is currently supported.",
                operation_name
            );
        }
        RepoType::Unknown => {
            anyhow::bail!(
                "{} is currently only supported for GitHub repositories.",
                operation_name
            );
        }
    }
}

/// 根据仓库类型调用平台提供者方法（获取当前分支的 PR ID）
///
/// 这是一个便捷函数，专门用于获取当前分支的 PR ID。
///
/// # 返回
///
/// 返回当前分支的 PR ID（如果存在），否则返回 None
pub fn get_current_branch_pr_id() -> Result<Option<String>> {
    let provider = create_provider()?;
    provider.get_current_branch_pull_request()
}

/// 解析 PR ID（从参数或当前分支）
///
/// 如果提供了 `pull_request_id`，直接返回。
/// 否则，尝试从当前分支自动检测 PR ID。
///
/// # 参数
///
/// * `pull_request_id` - 可选的 PR ID（从命令行参数传入）
///
/// # 返回
///
/// 返回解析后的 PR ID 字符串
///
/// # 错误
///
/// 如果无法自动检测 PR ID 且未提供参数，返回错误。
pub fn resolve_pull_request_id(pull_request_id: Option<String>) -> Result<String> {
    if let Some(id) = pull_request_id {
        return Ok(id);
    }

    let provider = create_provider()?;
    match provider.get_current_branch_pull_request()? {
        Some(id) => Ok(id),
        None => {
            let repo_type = GitRepo::detect_repo_type()?;
            let error_msg = match repo_type {
                RepoType::GitHub => "No PR found for current branch. Please specify PR ID.",
                RepoType::Codeup => {
                    "Codeup support has been removed. Only GitHub is currently supported."
                }
                _ => "No PR found for current branch.",
            };
            anyhow::bail!("{}", error_msg);
        }
    }
}
