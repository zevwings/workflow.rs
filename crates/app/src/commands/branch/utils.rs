//! 分支工作流模块

use domain::{sanitize_branch_name, BranchTemplateVars, BranchType, JiraIssue};
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
    let repo_config = config_repo.load().map_err(|e| format!("加载仓库配置失败: {}", e))?;

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
        .map_err(|e| format!("渲染模板失败: {}", e))?;

    Ok(branch_name)
}

/// 将 JIRA summary 转换为 URL 友好的 slug 格式
///
/// # Arguments
///
/// * `summary` - JIRA ticket 摘要
///
/// # Returns
///
/// Slug 格式的字符串（小写、连字符分隔、只包含 ASCII 字符）
///
/// # Examples
///
/// ```
/// use app::commands::branch::to_slug;
///
/// assert_eq!(to_slug("Chat Unified Entry"), "chat-unified-entry");
/// assert_eq!(to_slug("Fix: Auth Issue"), "fix-auth-issue");
/// ```
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
                // 跳过其他特殊字符
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        // 移除多余的连字符
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // 清理结果
    sanitize_branch_name(&slug)
}

/// 从分支名解析分支类型
///
/// 按 `/` 分割分支名，依次尝试每个片段与 `BranchType::parse` 匹配（如 `feature`、`bugfix`）。
/// 适用于 `feature/xxx`、`zw/feature/xxx` 等格式。
pub fn branch_type_from_branch_name(branch_name: &str) -> Option<BranchType> {
    for segment in branch_name.split('/') {
        if let Some(bt) = BranchType::parse(segment) {
            return Some(bt);
        }
    }
    None
}

/// 选择分支类型
pub fn select_branch_type() -> Result<BranchType, Box<dyn std::error::Error>> {
    let branch_type_options: Vec<String> =
        BranchType::all().iter().map(|t| t.as_str().to_string()).collect();

    let selected_type = select!("选择分支类型", branch_type_options)
        .prompt()
        .map_err(|e| format!("选择分支类型失败: {}", e))?;

    let branch_type = BranchType::parse(&selected_type)
        .ok_or_else(|| format!("无效的分支类型: {}", selected_type))?;

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
    let base_branch_name = match spinner!("正在生成分支名...")
        .with(|| branch_service.generate_branch_name(Some(summary), &exists_branches))
    {
        Ok(name) => strip_branch_type_prefix(&name),
        Err(e) => {
            info!("LLM 生成失败: {}, 使用备用方法", e);
            to_slug(summary)
        }
    };

    Ok((branch_type, base_branch_name))
}

/// 移除分支名中可能存在的类型前缀
///
/// 防御性处理：如果 LLM 返回了带类型前缀的分支名，移除它。
///
/// # Examples
///
/// ```ignore
/// // `strip_branch_type_prefix` 是内部辅助函数（非公有 API）。
/// assert_eq!(strip_branch_type_prefix("feature/my-branch"), "my-branch");
/// assert_eq!(strip_branch_type_prefix("my-branch"), "my-branch");
/// assert_eq!(strip_branch_type_prefix("bugfix/fix-issue"), "fix-issue");
/// ```
fn strip_branch_type_prefix(name: &str) -> String {
    let prefixes = ["feature/", "bugfix/", "hotfix/", "refactoring/", "chore/"];

    for prefix in prefixes {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }

    name.to_string()
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
    let issue = spinner!("正在获取 JIRA 工单 '{}'...", jira_id)
        .with(|| jira_repo.get_issue_info(jira_id))
        .map_err(|e| format!("获取 JIRA 工单失败: {}", e))?;

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
