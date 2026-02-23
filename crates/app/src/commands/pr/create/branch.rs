//! 分支处理逻辑

use domain::{get_change_types_by_branch_type, BranchType, GitRepository};
use prompt::{info, select, spinner, success, warning};

use crate::commands::pr::create::{
    commit::{check_needs_push, commit_changes, push_branch},
    pr::{confirm_target_branch, create_pull_request, format_pr_title, generate_pr_summary},
    types::{BranchHandleContext, BranchHandleOption, ConfirmOption, TargetBranchOption},
};
use crate::commands::pr::utils::{generate_pull_request_body, generate_pull_request_title};
use crate::{
    bootstrap,
    util::{branch_type_from_branch_name, safe_pull, PullOptions},
};

/// 处理非默认分支的情况
///
/// 返回 (Option<String>, Option<String>)：
/// - (None, None): 使用当前分支，不需要创建新分支，PR 已处理
/// - (Some(branch_name), Some(target)): 需要创建的新分支名和目标分支
pub fn handle_non_default_branch(
    ctx: &BranchHandleContext<'_>,
    dry_run: bool,
) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    let options = vec![
        BranchHandleOption::UseCurrentBranch(ctx.current_branch.to_string()),
        BranchHandleOption::CreateFromCurrent(ctx.generated_branch_name.to_string()),
        BranchHandleOption::SwitchToDefault(ctx.generated_branch_name.to_string()),
    ];

    let selected = select!("Please select how to handle branches:", options)
        .prompt()
        .map_err(|e| format!("Failed to select branch option: {}", e))?;

    match selected {
        BranchHandleOption::UseCurrentBranch(_) => handle_use_current_branch(ctx, dry_run),
        BranchHandleOption::CreateFromCurrent(_) => handle_create_from_current(ctx),
        BranchHandleOption::SwitchToDefault(_) => handle_switch_to_default(ctx, dry_run),
    }
}

/// 处理"直接使用当前分支"选项
fn handle_use_current_branch(
    ctx: &BranchHandleContext<'_>,
    dry_run: bool,
) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    let pr_service = bootstrap::get_pull_request_service();

    // 检查当前分支是否已有 PR
    let existing_pr_id = pr_service
        .get_current_branch_pull_request(ctx.current_branch)
        .map_err(|e| format!("Failed to check existing PR: {}", e))?;

    if let Some(pr_id) = existing_pr_id {
        // 已有 PR，询问是否更新
        handle_existing_pr(ctx, &pr_id, dry_run)?;
        Ok((None, None))
    } else {
        // 没有 PR，根据分支状态处理
        handle_no_existing_pr(ctx, dry_run)?;
        Ok((None, None))
    }
}

/// 处理已存在 PR 的情况
fn handle_existing_pr(
    ctx: &BranchHandleContext<'_>,
    pr_id: &str,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let update_options = vec![ConfirmOption::Yes, ConfirmOption::No];
    let update_selected = select!(
        format!(
            "Branch '{}' already has PR #{}. Update it?",
            ctx.current_branch, pr_id
        ),
        update_options
    )
    .prompt()
    .map_err(|e| format!("Failed to select update option: {}", e))?;

    if update_selected == ConfirmOption::Yes {
        let pr_service = bootstrap::get_pull_request_service();

        // 生成 commit message（用于组合 PR 标题）
        let commit_message = build_commit_message(ctx.jira_id, ctx.description)?;

        // 获取该 PR 的 base 分支，使摘要分析基于正确基准（与 handle_no_existing_pr 一致）
        let base_branch = pr_service.get_pull_request(pr_id).ok().map(|pr| pr.target_branch);

        // 生成 PR 摘要（三阶段分析，基于 PR 的 target 分支做 diff；拿不到则用 None 由服务推断）
        let pr_summary = generate_pr_summary(base_branch.as_deref())?;

        // 使用模板生成 PR body（PR Ready + Types of changes + LLM 描述 + Jira 链接）
        let branch_type =
            branch_type_from_branch_name(ctx.current_branch).unwrap_or(BranchType::Feature);
        let selected_change_types = get_change_types_by_branch_type(branch_type);
        let pr_body = generate_pull_request_body(
            &selected_change_types,
            Some(&pr_summary.pr_body),
            ctx.jira_id.as_deref(),
            None,
            None,
        )?;

        // 组合 PR 标题：优先使用模板，否则使用内置格式
        let pr_title = generate_pull_request_title(
            &pr_summary.type_,
            pr_summary.scope.as_deref(),
            ctx.jira_id.as_deref(),
            &commit_message,
        )
        .unwrap_or_else(|| {
            format_pr_title(
                &pr_summary.type_,
                pr_summary.scope.as_deref(),
                ctx.jira_id.as_deref(),
                &commit_message,
            )
        });

        // 更新 PR
        if dry_run {
            info!("[DRY RUN] Would update PR #{}", pr_id);
            info!("  Title: {}", pr_title);
            info!("  Description:\n{}", pr_body);
        } else {
            info!("Updating PR #{}...", pr_id);
            pr_service
                .update_pull_request(pr_id, Some(&pr_title), Some(&pr_body))
                .map_err(|e| format!("Failed to update PR: {}", e))?;
            success!("PR #{} updated successfully!", pr_id);
        }
    } else {
        info!("Operation cancelled");
    }

    Ok(())
}

/// 处理不存在 PR 的情况
fn handle_no_existing_pr(
    ctx: &BranchHandleContext<'_>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 根据分支状态处理：
    // 1. 如果有未提交的代码 -> 提交，push，创建 PR
    // 2. 如果有提交但未 push -> push，创建 PR
    // 3. 如果已 push -> 直接创建 PR

    if !dry_run {
        // 检查是否有未提交的更改
        let status = ctx
            .branch_repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to check working tree status: {}", e))?;

        if !status.is_clean() {
            // 有未提交的更改，执行提交
            commit_changes(ctx.branch_repo, ctx.jira_id, ctx.description)?;
        } else {
            // 没有未提交的更改，检查是否需要 push
            let needs_push = check_needs_push(ctx.branch_repo, ctx.current_branch)?;
            if needs_push {
                push_branch(ctx.branch_repo)?;
            }
        }
    } else {
        let status = ctx
            .branch_repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to check working tree status: {}", e))?;

        if !status.is_clean() {
            info!("[DRY RUN] Would commit changes");
            info!("[DRY RUN] Would push branch to remote");
        } else {
            let needs_push = check_needs_push(ctx.branch_repo, ctx.current_branch)?;
            if needs_push {
                info!("[DRY RUN] Would push branch to remote");
            } else {
                info!("[DRY RUN] Branch is up to date with remote");
            }
        }
    }

    // 生成 commit message（用于组合 PR 标题）
    let commit_message = build_commit_message(ctx.jira_id, ctx.description)?;

    // 先确定目标分支（在生成 summary 之前，以便用正确的 base 做 diff 分析）
    let target_branch = if dry_run {
        // 直接使用默认分支，跳过耗时的推断和交互
        let default_branch = ctx
            .branch_repo
            .get_default_branch()
            .map_err(|e| format!("Failed to get default branch: {}", e))?;
        info!("[DRY RUN] Target branch: {}", default_branch);
        default_branch
    } else {
        // 非 dry-run 模式：推断目标分支并询问用户确认
        let inferred_target = ctx
            .branch_repo
            .infer_target_branch(ctx.current_branch)
            .map_err(|e| format!("Failed to infer target branch: {}", e))?;

        confirm_target_branch(ctx.branch_repo, inferred_target.as_deref())?
    };

    // 生成 PR 摘要（三阶段分析，基于已选目标分支的 diff）
    let pr_summary = generate_pr_summary(Some(&target_branch))?;

    // 使用模板生成 PR body（PR Ready + Types of changes + LLM 描述 + Jira 链接）
    let branch_type =
        branch_type_from_branch_name(ctx.current_branch).unwrap_or(BranchType::Feature);
    let selected_change_types = get_change_types_by_branch_type(branch_type);
    let pr_body = generate_pull_request_body(
        &selected_change_types,
        Some(&pr_summary.pr_body),
        ctx.jira_id.as_deref(),
        None,
        None,
    )?;

    // 组合 PR 标题：优先使用模板，否则使用内置格式
    let pr_title = generate_pull_request_title(
        &pr_summary.type_,
        pr_summary.scope.as_deref(),
        ctx.jira_id.as_deref(),
        &commit_message,
    )
    .unwrap_or_else(|| {
        format_pr_title(
            &pr_summary.type_,
            pr_summary.scope.as_deref(),
            ctx.jira_id.as_deref(),
            &commit_message,
        )
    });

    create_pull_request(
        ctx.branch_repo,
        ctx.current_branch,
        &pr_title,
        &pr_body,
        Some(&target_branch),
        dry_run,
    )?;

    Ok(())
}

/// 处理"基于当前分支创建新分支"选项
fn handle_create_from_current(
    ctx: &BranchHandleContext<'_>,
) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    // 推断当前分支的源分支，让用户选择目标分支
    let target_branch = if ctx.current_branch != ctx.default_branch {
        // 当前分支不是默认分支，推断其源分支
        let inferred_source = ctx
            .branch_repo
            .infer_target_branch(ctx.current_branch)
            .map_err(|e| format!("Failed to infer source branch: {}", e))?;

        // 根据推断结果，让用户选择目标分支
        if let Some(source) = inferred_source {
            // 成功推断出源分支
            if source == ctx.default_branch {
                // 源分支就是默认分支，提供 current_branch 或 默认分支 两个选项
                let options = vec![
                    TargetBranchOption::Current(ctx.current_branch.to_string()),
                    TargetBranchOption::Default(ctx.default_branch.to_string()),
                ];
                let selected = select!("Please select the target branch for PR:", options)
                    .prompt()
                    .map_err(|e| format!("Failed to select target branch: {}", e))?;

                selected.branch_name().to_string()
            } else {
                // 源分支不是默认分支，提供三个选项
                let options = vec![
                    TargetBranchOption::Current(ctx.current_branch.to_string()),
                    TargetBranchOption::Inferred(source),
                    TargetBranchOption::Default(ctx.default_branch.to_string()),
                ];
                let selected = select!("Please select the target branch for PR:", options)
                    .prompt()
                    .map_err(|e| format!("Failed to select target branch: {}", e))?;

                selected.branch_name().to_string()
            }
        } else {
            // 无法推断源分支，只提供 current_branch 或 默认分支 两个选项
            let options = vec![
                TargetBranchOption::Current(ctx.current_branch.to_string()),
                TargetBranchOption::Default(ctx.default_branch.to_string()),
            ];
            let selected = select!("Please select the target branch for PR:", options)
                .prompt()
                .map_err(|e| format!("Failed to select target branch: {}", e))?;

            selected.branch_name().to_string()
        }
    } else {
        // 当前分支是默认分支，直接使用默认分支作为目标
        ctx.default_branch.to_string()
    };

    Ok((
        Some(ctx.generated_branch_name.to_string()),
        Some(target_branch),
    ))
}

/// 处理"切换到默认分支"选项
fn handle_switch_to_default(
    ctx: &BranchHandleContext<'_>,
    dry_run: bool,
) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    if dry_run {
        info!(
            "[DRY RUN] Would stash changes, switch to '{}', and pull latest",
            ctx.default_branch
        );
        let status = ctx
            .branch_repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to check working tree status: {}", e))?;
        if !status.is_clean() {
            info!("[DRY RUN] Would stash changes before switching");
        }
    } else {
        prepare_default_branch(ctx.branch_repo, ctx.current_branch, ctx.default_branch)?;
    }
    // 目标分支就是默认分支
    Ok((
        Some(ctx.generated_branch_name.to_string()),
        Some(ctx.default_branch.to_string()),
    ))
}

/// 处理默认分支的情况
///
/// 返回 (Option<String>, Option<String>)：
/// - (Some(branch_name), Some(target)): 需要创建的新分支名和目标分支
pub fn handle_default_branch(
    default_branch: &str,
    generated_branch_name: &str,
) -> (Option<String>, Option<String>) {
    // 情况2: 是默认分支，需要创建新分支
    // 目标分支就是默认分支
    // 分支创建和后续操作在主逻辑中统一处理
    (
        Some(generated_branch_name.to_string()),
        Some(default_branch.to_string()),
    )
}

/// 生成 commit message 字符串
///
/// 与 `commit_changes` 中的逻辑一致，用于组合 PR 标题。
fn build_commit_message(
    jira_id: &Option<String>,
    description: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(jira_id) = jira_id {
        if jira_id.trim().is_empty() {
            return Ok(description.unwrap_or("Update").to_string());
        }
        let jira_repo = bootstrap::get_jira_repository();
        match spinner!("Fetching JIRA ticket '{}'...", jira_id)
            .with(|| jira_repo.get_issue_info(jira_id))
        {
            Ok(issue) => Ok(format!("{}: {}", jira_id, issue.fields.summary)),
            Err(e) => {
                warning!("Failed to fetch JIRA ticket '{}': {}", jira_id, e);
                Ok(jira_id.clone())
            }
        }
    } else {
        Ok(description.unwrap_or("Update").to_string())
    }
}

/// 准备默认分支的辅助方法
///
/// 处理 stash、切换分支、拉取最新代码等操作
///
/// # 返回
/// 返回是否需要在新分支上恢复 stash
pub fn prepare_default_branch(
    branch_repo: &dyn GitRepository,
    _current_branch: &str,
    default_branch: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // 检查工作区状态
    let status = branch_repo
        .get_working_tree_status()
        .map_err(|e| format!("Failed to check working tree status: {}", e))?;

    let needs_stash = !status.is_clean();

    // 如果有未提交的更改，先 stash
    if needs_stash {
        info!("Working tree has uncommitted changes, stashing...");
        branch_repo
            .stash_push(Some("Auto-stash before creating branch from default"))
            .map_err(|e| format!("Failed to stash changes: {}", e))?;
    }

    // 切换到默认分支
    info!("Switching to default branch '{}'...", default_branch);
    branch_repo
        .checkout_branch(default_branch)
        .map_err(|e| format!("Failed to switch to branch '{}': {}", default_branch, e))?;

    // 拉取最新代码（工作区已 stash 故无需再 stash）
    spinner!("Pulling latest changes from '{}'...", default_branch)
        .with(|| safe_pull(default_branch, &PullOptions::no_stash()))
        .map_err(|e| format!("Failed to pull latest changes: {}", e))?;

    // 返回是否需要恢复 stash（将在新分支上恢复）
    Ok(needs_stash)
}
