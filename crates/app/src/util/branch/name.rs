use domain::{sanitize_branch_name, BranchTemplateVars, BranchType, JiraIssue};
use prompt::{info, select, spinner};
use toolkit::TemplateEngine;

use crate::bootstrap;

/// 根据 summary 生成分支类型和基础分支名（选择类型、LLM 生成或 fallback）。
pub fn generate_branch_name_by_summary(
    summary: &str,
) -> Result<(BranchType, String), Box<dyn std::error::Error>> {
    let branch_type = select_branch_type()?;
    let branch_repo = bootstrap::get_git_repository();
    let exists_branches: Vec<String> = branch_repo
        .list_branches(false, true)
        .map(|branches| branches.iter().map(|b| b.name.clone()).collect())
        .unwrap_or_default();

    let branch_service = bootstrap::get_branch_service();
    let base_branch_name = match spinner!("Generating branch name...")
        .with(|| branch_service.generate_branch_name(Some(summary), &exists_branches))
    {
        Ok(name) => strip_branch_type_prefix(&name),
        Err(e) => {
            info!("LLM generation failed: {}, using fallback method", e);
            to_slug(summary)
        }
    };

    Ok((branch_type, base_branch_name))
}

/// 从 JIRA ID 生成分支名，包含 JIRA 信息获取、LLM 生成、模板渲染。
pub fn generate_branch_name_from_jira(
    jira_id: impl AsRef<str>,
) -> Result<GenerateBranchNameResult, Box<dyn std::error::Error>> {
    let jira_id = jira_id.as_ref();
    let jira_repo = bootstrap::get_jira_repository();
    let issue = spinner!("Fetching JIRA ticket '{}'...", jira_id)
        .with(|| jira_repo.get_issue_info(jira_id))
        .map_err(|e| format!("Failed to fetch JIRA ticket: {}", e))?;

    let summary = issue.fields.summary.clone();

    let (branch_type, base_branch_name) = generate_branch_name_by_summary(&summary)?;
    let branch_name =
        generate_branch_name_from_template(branch_type, &base_branch_name, Some(jira_id))?;

    Ok(GenerateBranchNameResult {
        branch_name,
        branch_type,
        jira_issue: issue,
    })
}

pub struct GenerateBranchNameResult {
    pub branch_name: String,
    pub branch_type: BranchType,
    pub jira_issue: JiraIssue,
}

/// 使用用户配置的模板渲染分支名，支持 prefix、branch_type、jira_key、summary_slug。
pub fn generate_branch_name_from_template(
    branch_type: BranchType,
    summary_slug: &str,
    jira_key: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let config_repo = bootstrap::get_repo_config_repository();
    let repo_config = config_repo
        .load()
        .map_err(|e| format!("Failed to load repository configuration: {}", e))?;

    let template = match branch_type {
        BranchType::Feature => &repo_config.project.template.branch.feature,
        BranchType::Bugfix => &repo_config.project.template.branch.bugfix,
        BranchType::Hotfix => &repo_config.project.template.branch.hotfix,
        BranchType::Refactoring => &repo_config.project.template.branch.refactoring,
        BranchType::Chore => &repo_config.project.template.branch.chore,
    };

    let vars = BranchTemplateVars {
        prefix: if repo_config.user.branch.prefix.is_empty() {
            None
        } else {
            Some(repo_config.user.branch.prefix.clone())
        },
        jira_key: jira_key.map(|s| s.to_string()),
        summary_slug: Some(summary_slug.to_string()),
        jira_summary: None,
        jira_type: None,
    };

    let engine = TemplateEngine::new();
    let branch_name = engine
        .render_string(template, &vars)
        .map_err(|e| format!("Failed to render template: {}", e))?;

    Ok(branch_name)
}

/// 将文本转为 slug（小写、连字符分隔、仅 ASCII 字母数字）。
pub fn to_slug(summary: impl AsRef<str>) -> String {
    let slug = summary
        .as_ref()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    sanitize_branch_name(&slug)
}

/// 从分支名解析分支类型，支持 `feature/xxx`、`zw/feature/xxx` 等格式。
pub fn branch_type_from_branch_name(branch_name: &str) -> Option<BranchType> {
    for segment in branch_name.split('/') {
        if let Some(bt) = BranchType::parse(segment) {
            return Some(bt);
        }
    }
    None
}

/// 移除 LLM 可能返回的类型前缀（如 feature/、bugfix/）。
pub fn strip_branch_type_prefix(name: &str) -> String {
    let prefixes = ["feature/", "bugfix/", "hotfix/", "refactoring/", "chore/"];

    for prefix in prefixes {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }

    name.to_string()
}

/// 交互式选择分支类型。
pub fn select_branch_type() -> Result<BranchType, Box<dyn std::error::Error>> {
    let branch_type_options: Vec<String> =
        BranchType::all().iter().map(|t| t.as_str().to_string()).collect();

    let selected_type = select!("Select branch type", branch_type_options)
        .prompt()
        .map_err(|e| format!("Failed to select branch type: {}", e))?;

    let branch_type = BranchType::parse(&selected_type)
        .ok_or_else(|| format!("Invalid branch type: {}", selected_type))?;

    Ok(branch_type)
}
