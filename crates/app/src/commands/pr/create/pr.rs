//! PR 创建和摘要生成逻辑

use domain::git::CodePlatform;
use domain::{GitRepository, PullRequestContent};
use prompt::{error, info, select, spinner, success, warning};
use toolkit::BrowserExt;

use crate::registry;

use super::types::TargetBranchOption;

/// 创建 Pull Request
///
/// 使用生成的 PR 内容创建 Pull Request
///
/// # 参数
/// - `target_branch`: 可选的目标分支，如果为 None 则使用默认分支
pub fn create_pull_request(
    branch_repo: &dyn GitRepository,
    branch_name: &str,
    pr_content: &PullRequestContent,
    target_branch: Option<&str>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 使用提供的目标分支或默认分支
    let default_branch = branch_repo
        .get_default_branch()
        .map_err(|e| format!("Failed to get default branch: {}", e))?;

    let target = target_branch.unwrap_or(&default_branch);

    // 构建 PR 描述
    let mut pr_body = String::new();
    if let Some(ref description) = pr_content.description {
        pr_body.push_str(description);
    }
    if let Some(ref summary) = pr_content.summary {
        if !pr_body.is_empty() {
            pr_body.push_str("\n\n");
        }
        pr_body.push_str("## Summary\n\n");
        pr_body.push_str(summary);
    }

    if dry_run {
        info!("[DRY RUN] Would create Pull Request:");
        info!("  Title: {}", pr_content.pr_title);
        info!("  Source branch: {}", branch_name);
        info!("  Target branch: {}", target);
        if !pr_body.is_empty() {
            info!("  Description:\n{}", pr_body);
        }
        return Ok(());
    }

    // 创建 PR
    info!("Creating Pull Request...");
    let pr_service = registry::get_pull_request_service();
    let pr_id = spinner!("Creating Pull Request...")
        .with(|| {
            pr_service.create_pull_request(
                None, // jira_id
                Some(&pr_content.pr_title),
                Some(&pr_body),
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

    if let Some(url) = pr_url {
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
    } else {
        warning!(
            "Could not generate PR URL. Platform: {:?}, Repo: {:?}",
            repo_info.kind,
            repo_info.name
        );
    }

    Ok(())
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

/// 生成 PR 详细总结
///
/// 获取当前工作区和暂存区相对于默认分支的 diff，然后调用 LLM 生成详细的 PR 总结。
/// 在提交代码之前调用此方法。
///
/// # 返回
/// 返回生成的 PR 内容（如果有更改），否则返回 None
pub fn generate_pr_summary(
    branch_repo: &dyn GitRepository,
    _branch_name: &str,
    jira_id: &Option<String>,
    description: Option<&str>,
) -> Result<Option<PullRequestContent>, Box<dyn std::error::Error>> {
    // 获取默认分支
    let default_branch = branch_repo
        .get_default_branch()
        .map_err(|e| format!("Failed to get default branch: {}", e))?;

    // 获取工作区和暂存区相对于默认分支的 diff
    // storage 层会自动应用 .gitignore 忽略规则和大小限制
    // 这个 diff 包括：已提交的更改、暂存区更改、工作区未暂存更改
    let git_diff = branch_repo
        .get_working_tree_diff(&default_branch)
        .map_err(|e| format!("Failed to get working tree diff: {}", e))?;

    // 如果没有 diff（既没有已提交的 commits，也没有未提交的更改），跳过生成总结
    let git_diff = match git_diff {
        Some(diff) if !diff.trim().is_empty() => diff,
        _ => {
            info!("No changes to generate PR summary");
            return Ok(None);
        }
    };

    // 生成 commit title（用于生成 PR 内容）
    let commit_title = if let Some(jira_id) = jira_id {
        // 获取 JIRA summary
        let jira_repo = registry::get_jira_repository();
        let issue = spinner!("Fetching JIRA ticket '{}'...", jira_id)
            .with(|| jira_repo.get_issue_info(jira_id))
            .map_err(|e| format!("Failed to fetch JIRA ticket: {}", e))?;
        format!("{}: {}", jira_id, issue.summary)
    } else if let Some(desc) = description {
        desc.to_string()
    } else {
        // 使用 description 或默认消息
        description.unwrap_or("Update").to_string()
    };

    // 获取已存在的分支列表（用于避免重复分支名）
    let existing_branches = branch_repo
        .list_branches(false, true)
        .map_err(|e| format!("Failed to list branches: {}", e))?;
    let branch_names: Vec<String> =
        existing_branches.iter().map(|b| b.display_name.clone()).collect();

    // 调用 LLM 生成 PR 内容（包括详细总结）
    info!("Generating PR summary...");
    let llm_repo = registry::get_llm_repository();
    let pr_content = spinner!("Generating PR content and summary...")
        .with(|| {
            llm_repo.create_pr_content(&commit_title, Some(branch_names), Some(git_diff.clone()))
        })
        .map_err(|e| format!("Failed to generate PR content: {}", e))?;

    // 显示 PR 内容
    info!("PR Title: {}", pr_content.pr_title);
    if let Some(ref desc) = pr_content.description {
        info!("PR Description:\n{}", desc);
    }
    if let Some(ref scope) = pr_content.scope {
        info!("Scope: {}", scope);
    }

    // 显示详细总结
    if let Some(ref summary) = pr_content.summary {
        success!("PR Summary generated successfully!");
        println!("\n{}", summary);
    } else {
        info!("No detailed summary generated (this is normal if git diff is empty or too large)");
    }

    Ok(Some(pr_content))
}
