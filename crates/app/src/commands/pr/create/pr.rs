//! PR 创建和摘要生成逻辑

use domain::git::CodePlatform;
use domain::summary::entity::{
    AffectedModule, CommitSummaryAnalysis, DetailsByCategory, ImpactAnalysis,
};
use domain::GitRepository;
use prompt::{error, info, select, spinner, success, warning};
use toolkit::BrowserExt;

use crate::registry;

use super::types::TargetBranchOption;

/// PR 创建结果
#[derive(Debug, Clone)]
pub struct PrCreateResult {
    /// PR ID
    pub pr_id: String,
    /// PR URL
    pub pr_url: String,
}

/// PR 摘要结果
///
/// 由三阶段提交分析生成，包含 PR body 和 Conventional Commits 所需的 type/scope。
#[derive(Debug, Clone)]
pub struct PrSummaryResult {
    /// Conventional Commits type（feat / fix / refactor / docs / style / test / chore / perf）
    pub type_: String,
    /// Commit scope（变更涉及的模块或功能区域，如 "api", "auth"）
    pub scope: Option<String>,
    /// PR body（Markdown 格式，包含总结、变更详情、影响分析和统计信息）
    pub pr_body: String,
}

/// 组合 PR 标题
///
/// 使用 Conventional Commits 格式将 type、scope 和 commit message 组合为 PR 标题。
///
/// # 示例
///
/// - 有 scope: `feat(auth): PROJ-123: 用户登录功能`
/// - 无 scope: `feat: PROJ-123: 用户登录功能`
pub fn format_pr_title(type_: &str, scope: Option<&str>, commit_message: &str) -> String {
    match scope {
        Some(s) if !s.is_empty() => format!("{}({}): {}", type_, s, commit_message),
        _ => format!("{}: {}", type_, commit_message),
    }
}

/// 创建 Pull Request
///
/// 使用生成的 PR 标题和内容创建 Pull Request
///
/// # 参数
/// - `branch_repo`: Git 仓库
/// - `branch_name`: 源分支名称
/// - `pr_title`: PR 标题
/// - `pr_body`: PR 描述内容（Markdown 格式）
/// - `target_branch`: 可选的目标分支，如果为 None 则使用默认分支
/// - `dry_run`: 是否为 dry-run 模式
///
/// # 返回
/// - `Ok(Some(PrCreateResult))` - PR 创建成功，返回 PR ID 和 URL
/// - `Ok(None)` - dry_run 模式，未实际创建 PR
/// - `Err(...)` - 创建失败
pub fn create_pull_request(
    branch_repo: &dyn GitRepository,
    branch_name: &str,
    pr_title: &str,
    pr_body: &str,
    target_branch: Option<&str>,
    dry_run: bool,
) -> Result<Option<PrCreateResult>, Box<dyn std::error::Error>> {
    // 使用提供的目标分支或默认分支
    let default_branch = branch_repo
        .get_default_branch()
        .map_err(|e| format!("Failed to get default branch: {}", e))?;

    let target = target_branch.unwrap_or(&default_branch);

    if dry_run {
        info!("[DRY RUN] Would create Pull Request:");
        info!("  Title: {}", pr_title);
        info!("  Source branch: {}", branch_name);
        info!("  Target branch: {}", target);
        if !pr_body.is_empty() {
            info!("  Description:\n{}", pr_body);
        }
        return Ok(None);
    }

    // 创建 PR
    info!("Creating Pull Request...");
    let pr_service = registry::get_pull_request_service();
    let pr_id = spinner!("Creating Pull Request...")
        .with(|| {
            pr_service.create_pull_request(
                None, // jira_id
                Some(pr_title),
                Some(pr_body),
                Some(target), // 使用用户选择的目标分支
            )
        })
        .map_err(|e| format!("Failed to create Pull Request: {}", e))?;

    success!("Pull Request created successfully!");
    info!("PR ID: {}", pr_id);

    // 获取 PR URL 并打开浏览器
    let repo_info = branch_repo.get_repo_info();
    // 使用 repo_info.name（owner/repo 格式）直接构建 PR URL
    let pr_url = match (&repo_info.name, repo_info.kind) {
        (Some(repo_name), Some(CodePlatform::GitHub)) => {
            Some(format!("https://github.com/{}/pull/{}", repo_name, pr_id))
        }
        // 将来可以添加其他平台支持
        _ => None,
    };

    if let Some(ref url) = pr_url {
        info!("PR URL: {}", url);

        // 使用默认浏览器打开 PR 页面
        match url.open_in_browser() {
            Ok(()) => {
                success!("Opened PR in browser");
            }
            Err(e) => {
                // 打开浏览器失败不应该阻止整个流程
                error!("Failed to open PR in browser: {}", e);
            }
        }

        Ok(Some(PrCreateResult {
            pr_id: pr_id.clone(),
            pr_url: url.clone(),
        }))
    } else {
        warning!(
            "Could not generate PR URL. Platform: {:?}, Repo: {:?}",
            repo_info.kind,
            repo_info.name
        );
        // 即使没有 URL，PR 也已经创建成功，返回 PR ID
        Ok(Some(PrCreateResult {
            pr_id: pr_id.clone(),
            pr_url: String::new(),
        }))
    }
}

/// 询问用户确认目标分支
///
/// # 参数
/// - `branch_repo`: Git 仓库
/// - `inferred_target`: 推断出的目标分支（可能为 None）
///
/// # 返回
/// 用户选择的目标分支名称
pub fn confirm_target_branch(
    branch_repo: &dyn GitRepository,
    inferred_target: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let default_branch = branch_repo
        .get_default_branch()
        .map_err(|e| format!("Failed to get default branch: {}", e))?;

    // 如果推断出了目标分支，询问用户确认
    if let Some(inferred) = inferred_target {
        // 如果推断的分支和默认分支相同，直接使用，无需询问
        if inferred == default_branch {
            info!("Target branch: {}", inferred);
            return Ok(inferred.to_string());
        }

        let options = vec![
            TargetBranchOption::Inferred(inferred.to_string()),
            TargetBranchOption::Default(default_branch.clone()),
        ];

        let selected = select!("Please select the target branch for PR:", options)
            .prompt()
            .map_err(|e| format!("Failed to select target branch: {}", e))?;

        Ok(selected.branch_name().to_string())
    } else {
        // 无法推断，直接使用默认分支
        info!(
            "Cannot infer target branch, using default: {}",
            default_branch
        );
        Ok(default_branch)
    }
}

/// 生成 PR 摘要
///
/// 调用三阶段提交分析服务（文件分类 → 分类分析 → 全局总结），
/// 将分析结果渲染为 Markdown 格式的 PR body，并提取 type/scope。
///
/// 必须在 `commit_changes` 之后调用，确保变更已提交，`get_merge_diff` 能获取到 diff。
///
/// # 参数
/// - `base_branch`: 可选的基准分支，为 None 时由服务自动推断
///
/// # 返回
/// 返回 `PrSummaryResult`（包含 type、scope 和 PR body）
pub fn generate_pr_summary(
    base_branch: Option<&str>,
) -> Result<PrSummaryResult, Box<dyn std::error::Error>> {
    info!("Generating PR summary...");

    let summary_service = registry::get_commit_summary_service();
    let analysis = spinner!("Analyzing commit changes (3-stage analysis)...")
        .with(|| summary_service.run_analysis(base_branch))
        .map_err(|e| format!("Failed to generate PR summary: {}", e))?;

    // 提取 type 和 scope
    let type_ = analysis.structured_summary.type_.clone();
    let scope = if analysis.structured_summary.scope.is_empty() {
        None
    } else {
        Some(analysis.structured_summary.scope.clone())
    };

    // 渲染 PR body
    let pr_body = render_pr_body(&analysis);

    // 显示摘要信息
    info!("Type: {}", type_);
    if let Some(ref s) = scope {
        info!("Scope: {}", s);
    }
    success!("PR Summary generated successfully!");
    println!("\n{}", pr_body);

    Ok(PrSummaryResult {
        type_,
        scope,
        pr_body,
    })
}

/// 将 `CommitSummaryAnalysis` 渲染为 Markdown 格式的 PR body
fn render_pr_body(analysis: &CommitSummaryAnalysis) -> String {
    let mut body = String::new();

    // == Summary ==
    body.push_str("## Summary\n\n");
    if !analysis.structured_summary.main_purpose.is_empty() {
        body.push_str(&analysis.structured_summary.main_purpose);
        body.push_str("\n\n");
    }

    // Key changes
    if !analysis.structured_summary.key_changes.is_empty() {
        body.push_str("### Key Changes\n\n");
        for change in &analysis.structured_summary.key_changes {
            body.push_str(&format!("- {}\n", change));
        }
        body.push('\n');
    }

    // == Changes by Category ==
    render_details_by_category(&mut body, &analysis.structured_summary.details_by_category);

    // == Impact Analysis ==
    render_impact_analysis(&mut body, &analysis.impact_analysis);

    // == Statistics ==
    let stats = &analysis.statistics;
    body.push_str("## Statistics\n\n");
    body.push_str(&format!(
        "| Metric | Value |\n|--------|-------|\n| Total files | {} |\n| Additions | +{} |\n| Deletions | -{} |\n| Net change | {} |\n",
        stats.total_files,
        stats.additions,
        stats.deletions,
        stats.net_change,
    ));

    let fb = &stats.file_breakdown;
    if fb.added > 0 || fb.modified > 0 || fb.deleted > 0 || fb.renamed > 0 {
        body.push_str(&format!(
            "| Added files | {} |\n| Modified files | {} |\n| Deleted files | {} |\n| Renamed files | {} |\n",
            fb.added, fb.modified, fb.deleted, fb.renamed,
        ));
    }
    body.push('\n');

    // == Metadata ==
    let meta = &analysis.metadata;
    if !meta.complexity.is_empty() || !meta.review_priority.is_empty() {
        body.push_str("## Review Info\n\n");
        if !meta.complexity.is_empty() {
            body.push_str(&format!("- **Complexity**: {}\n", meta.complexity));
        }
        if !meta.review_priority.is_empty() {
            body.push_str(&format!("- **Review priority**: {}\n", meta.review_priority));
        }
        if !meta.estimated_review_time.is_empty() {
            body.push_str(&format!(
                "- **Estimated review time**: {}\n",
                meta.estimated_review_time
            ));
        }
        if !meta.tags.is_empty() {
            body.push_str(&format!("- **Tags**: {}\n", meta.tags.join(", ")));
        }
        body.push('\n');
    }

    body.trim_end().to_string()
}

/// 渲染按类别划分的变更详情
fn render_details_by_category(body: &mut String, details: &DetailsByCategory) {
    let categories: Vec<(&str, &[String])> = vec![
        ("Features", &details.features),
        ("Bug Fixes", &details.fixes),
        ("Refactors", &details.refactors),
        ("Configuration", &details.config),
        ("Documentation", &details.docs),
        ("Tests", &details.tests),
        ("Others", &details.others),
    ];

    let has_any = categories.iter().any(|(_, items)| !items.is_empty());
    if !has_any {
        return;
    }

    body.push_str("## Changes\n\n");
    for (label, items) in &categories {
        if items.is_empty() {
            continue;
        }
        body.push_str(&format!("### {}\n\n", label));
        for item in *items {
            body.push_str(&format!("- {}\n", item));
        }
        body.push('\n');
    }
}

/// 渲染影响分析
fn render_impact_analysis(body: &mut String, impact: &ImpactAnalysis) {
    let has_breaking = impact.breaking_changes.has_breaking;
    let has_modules = !impact.affected_modules.is_empty();
    let has_risk = !impact.risk_assessment.overall_risk.is_empty();
    let has_testing = !impact.testing_suggestions.is_empty();

    if !has_breaking && !has_modules && !has_risk && !has_testing {
        return;
    }

    body.push_str("## Impact Analysis\n\n");

    // Breaking changes
    if has_breaking {
        body.push_str("### Breaking Changes\n\n");
        if !impact.breaking_changes.description.is_empty() {
            body.push_str(&format!("{}\n\n", impact.breaking_changes.description));
        }
        if !impact.breaking_changes.migration_guide.is_empty() {
            body.push_str(&format!(
                "**Migration guide**: {}\n\n",
                impact.breaking_changes.migration_guide
            ));
        }
    }

    // Affected modules
    if has_modules {
        render_affected_modules(body, &impact.affected_modules);
    }

    // Risk assessment
    if has_risk {
        body.push_str(&format!(
            "### Risk Assessment\n\n**Overall risk**: {}\n\n",
            impact.risk_assessment.overall_risk
        ));
        if !impact.risk_assessment.risk_factors.is_empty() {
            body.push_str("**Risk factors**:\n");
            for factor in &impact.risk_assessment.risk_factors {
                body.push_str(&format!("- {}\n", factor));
            }
            body.push('\n');
        }
        if !impact.risk_assessment.mitigation.is_empty() {
            body.push_str("**Mitigation**:\n");
            for m in &impact.risk_assessment.mitigation {
                body.push_str(&format!("- {}\n", m));
            }
            body.push('\n');
        }
    }

    // Testing suggestions
    if has_testing {
        body.push_str("### Testing Suggestions\n\n");
        for suggestion in &impact.testing_suggestions {
            body.push_str(&format!("- {}\n", suggestion));
        }
        body.push('\n');
    }
}

/// 渲染受影响模块表格
fn render_affected_modules(body: &mut String, modules: &[AffectedModule]) {
    body.push_str("### Affected Modules\n\n");
    body.push_str("| Module | Impact | Severity |\n|--------|--------|----------|\n");
    for m in modules {
        body.push_str(&format!("| {} | {} | {} |\n", m.module, m.impact, m.severity));
    }
    body.push('\n');
}
