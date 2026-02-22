//! 分支工作流模块

use crate::util::branch::{strip_branch_type_prefix, to_slug};
use domain::{BranchTemplateVars, BranchType, JiraIssue};
use prompt::{info, select, spinner};
use toolkit::TemplateEngine;

use crate::bootstrap;

pub struct GenerateBranchNameResult {
    pub branch_name: String,
    pub branch_type: BranchType,
    pub jira_issue: JiraIssue,
}

/// 使用模板生成分支名
///
/// 根据分支类型、JIRA 信息和用户配置，使用模板渲染生成完整的分支名。
/// 模板格式：`{{#if prefix}}{{prefix}}/{{/if}}{branch_type}/{{jira_key}}-{{summary_slug}}`
///
/// # Arguments
///
/// * `branch_type` - 分支类型（feature/bugfix/hotfix/refactoring/chore）
/// * `summary_slug` - URL 友好的摘要（如 "chat-unified-entry"）
/// * `jira_key` - JIRA ticket key（可选，如 "IOSNAT-30271"）
///
/// # Returns
///
/// 格式化后的完整分支名
///
/// # Examples
///
/// ```text
/// // 有 prefix 配置（如 "zw"）和 jira_key
/// // 输出：zw/feature/iosnat-30271-chat-unified-entry
///
/// // 无 prefix 配置，无 jira_key
/// // 输出：feature/chat-unified-entry
/// ```
pub fn generate_branch_name_from_template(
    branch_type: BranchType,
    summary_slug: &str,
    jira_key: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    // 获取配置
    let config_repo = bootstrap::get_repo_config_repository();
    let repo_config = config_repo
        .load()
        .map_err(|e| format!("Failed to load repository configuration: {}", e))?;

    // 获取对应类型的模板
    let template = match branch_type {
        BranchType::Feature => &repo_config.project.template.branch.feature,
        BranchType::Bugfix => &repo_config.project.template.branch.bugfix,
        BranchType::Hotfix => &repo_config.project.template.branch.hotfix,
        BranchType::Refactoring => &repo_config.project.template.branch.refactoring,
        BranchType::Chore => &repo_config.project.template.branch.chore,
    };

    // 构建模板变量
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

    // 渲染模板
    let engine = TemplateEngine::new();
    let branch_name = engine
        .render_string(template, &vars)
        .map_err(|e| format!("Failed to render template: {}", e))?;

    Ok(branch_name)
}

/// 选择分支类型
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

/// 内部辅助函数：生成分支名的核心逻辑
///
/// 提取公共逻辑，包括选择分支类型、获取已存在分支、使用 LLM 生成基础分支名。
pub fn generate_branch_name_by_summary(
    summary: &str,
) -> Result<(BranchType, String), Box<dyn std::error::Error>> {
    // 让用户选择分支类型（在 Spinner 之外进行，避免 raw mode 冲突）
    let branch_type = select_branch_type()?;

    // 获取所有已存在的分支名（失败则为空）
    let branch_repo = bootstrap::get_git_repository();
    let exists_branches: Vec<String> = branch_repo
        .list_branches(false, true)
        .map(|branches| branches.iter().map(|b| b.name.clone()).collect())
        .unwrap_or_default();

    // 使用 LLM 生成基础分支名（不包含 branch_type 前缀）
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

/// 从 JIRA ID 生成分支名
///
/// 根据 JIRA ticket 信息，使用 LLM 生成分支名，并应用分支模板。
///
/// # Arguments
///
/// * `jira_id` - JIRA ticket ID（如 "PROJ-123"）
///
/// # Returns
///
/// 生成的分支名（如 "feature/proj-123-chat-unified-entry"）
pub fn generate_branch_name_from_jira(
    jira_id: impl AsRef<str>,
) -> Result<GenerateBranchNameResult, Box<dyn std::error::Error>> {
    let jira_id = jira_id.as_ref();
    // 获取 JiraRepository
    let jira_repo = bootstrap::get_jira_repository();

    // 获取 JIRA ticket 信息
    let issue = spinner!("Fetching JIRA ticket '{}'...", jira_id)
        .with(|| jira_repo.get_issue_info(jira_id))
        .map_err(|e| format!("Failed to fetch JIRA ticket: {}", e))?;

    let summary = issue.fields.summary.clone();

    // 使用核心逻辑生成分支类型和基础分支名
    let (branch_type, base_branch_name) = generate_branch_name_by_summary(&summary)?;

    // 使用模板将基础分支名与 branch_type 组合成完整分支名
    let branch_name =
        generate_branch_name_from_template(branch_type, &base_branch_name, Some(jira_id))?;

    Ok(GenerateBranchNameResult {
        branch_name,
        branch_type,
        jira_issue: issue,
    })
}
