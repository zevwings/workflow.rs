//! 创建 Pull Request 命令

use color_eyre::Result;
use domain::{GitRepository, PullRequestContent};
use prompt::{error, info, input, select, spinner, success, warning};
use toolkit::BrowserExt;

use crate::registry;
use crate::workflows::utils::branch::{
    generate_branch_name_from_jira, generate_branch_name_from_template,
};
use crate::workflows::utils::jira::get_jira_id_interactive_optional;

/// 分支处理方式选项
#[derive(Clone)]
enum BranchHandleOption {
    /// 直接使用当前分支
    UseCurrentBranch(String),
    /// 基于当前分支创建新分支
    CreateFromCurrent(String),
    /// 切换到默认分支，创建新分支
    SwitchToDefault(String),
}

impl std::fmt::Display for BranchHandleOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchHandleOption::UseCurrentBranch(branch) => {
                write!(f, "Use current branch directly ({})", branch)
            }
            BranchHandleOption::CreateFromCurrent(branch) => {
                write!(f, "Create new branch from current ({})", branch)
            }
            BranchHandleOption::SwitchToDefault(branch) => {
                write!(f, "Switch to default branch and create new ({})", branch)
            }
        }
    }
}

/// 目标分支选项
#[derive(Clone)]
enum TargetBranchOption {
    /// 合并到当前分支
    Current(String),
    /// 合并到推断的分支
    Inferred(String),
    /// 合并到默认分支
    Default(String),
}

impl std::fmt::Display for TargetBranchOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetBranchOption::Current(branch) => {
                write!(f, "Merge to current branch: {}", branch)
            }
            TargetBranchOption::Inferred(branch) => {
                write!(f, "Merge to inferred branch: {}", branch)
            }
            TargetBranchOption::Default(branch) => {
                write!(f, "Merge to default branch: {}", branch)
            }
        }
    }
}

impl TargetBranchOption {
    /// 获取分支名
    fn branch_name(&self) -> &str {
        match self {
            TargetBranchOption::Current(branch)
            | TargetBranchOption::Inferred(branch)
            | TargetBranchOption::Default(branch) => branch,
        }
    }
}

/// 确认操作选项
#[derive(Clone, PartialEq)]
enum ConfirmOption {
    /// 确认执行
    Yes,
    /// 取消操作
    No,
}

impl std::fmt::Display for ConfirmOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfirmOption::Yes => write!(f, "Yes, update PR"),
            ConfirmOption::No => write!(f, "No, cancel"),
        }
    }
}

/// Pull Request Create 命令
pub struct PullRequestCreateCommand {
    jira_id: Option<String>,
    #[allow(dead_code)]
    title: Option<String>,
    #[allow(dead_code)]
    description: Option<String>,
    dry_run: bool,
}

impl PullRequestCreateCommand {
    /// 创建新的 PullRequestCreateCommand
    pub fn new(
        jira_id: Option<String>,
        title: Option<String>,
        description: Option<String>,
        dry_run: bool,
    ) -> Self {
        Self {
            jira_id,
            title,
            description,
            dry_run,
        }
    }

    /// 运行 `workflow pr create` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = registry::get_git_repository();

        // 获取 JIRA ID（交互式或从参数）
        let jira_id = get_jira_id_interactive_optional(self.jira_id.clone())?;

        // 保存 description 用于后续提交
        let mut description_for_commit = None;

        // 第一步：创建分支
        let branch_name = if let Some(ref jira_id) = jira_id {
            // 从 JIRA ID 生成分支名
            generate_branch_name_from_jira(jira_id)?
        } else {
            // 如果没有 JIRA ID，让用户输入描述
            let description = input!("Please enter ticket description:")
                .validator(|input: &str| {
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        Err("Description cannot be empty".to_string())
                    } else {
                        Ok(())
                    }
                })
                .prompt()
                .map(|s: String| s.trim().to_string())
                .map_err(|e| format!("Failed to get description: {}", e))?;

            // 保存 description 用于后续提交
            description_for_commit = Some(description.clone());

            // 先让用户选择分支类型（在 Spinner 之外，避免 raw mode 冲突）
            let branch_type = crate::workflows::utils::branch::select_branch_type()
                .map_err(|e| format!("Failed to select branch type: {}", e))?;

            // 然后使用 Spinner 生成分支名称
            let base_branch_name = {
                let branch_repo = registry::get_git_repository();
                let exists_branches: Option<Vec<String>> = branch_repo
                    .list_branches(false, true)
                    .map(|branches| branches.iter().map(|b| b.name.clone()).collect())
                    .ok();

                let llm_repo = registry::get_llm_repository();
                match spinner!("Generating branch name...").with(|| {
                    llm_repo.generate_branch_name(Some(description.as_str()), exists_branches)
                }) {
                    Ok(name) => name,
                    Err(e) => {
                        info!("LLM generation failed: {}, using fallback method", e);
                        crate::workflows::utils::branch::to_slug(description.as_str())
                    }
                }
            };

            // 使用模板将基础分支名与 branch_type 组合成完整分支名（不包含 JIRA ID）

            generate_branch_name_from_template(branch_type, &base_branch_name, None)?
        };

        // 获取默认分支和当前分支
        let default_branch = branch_repo
            .get_default_branch()
            .map_err(|e| format!("Failed to get default branch: {}", e))?;

        let current_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 根据当前分支和默认分支的关系，处理不同的逻辑
        // 返回: (新分支名, 目标分支)
        let (final_branch_name, target_branch) = if default_branch != current_branch {
            // 情况1: 不是默认分支，询问用户如何处理
            self.handle_non_default_branch(
                branch_repo.as_ref(),
                &current_branch,
                &default_branch,
                &branch_name,
                &jira_id,
                description_for_commit.as_deref(),
            )?
        } else {
            // 情况2: 是默认分支，提交代码并创建新 PR
            self.handle_default_branch(
                branch_repo.as_ref(),
                &default_branch,
                &branch_name,
                &jira_id,
                description_for_commit.as_deref(),
            )?
        };

        // 如果返回了分支名，说明需要创建新分支并创建 PR
        if let Some(new_branch_name) = final_branch_name {
            // 检查分支是否已存在
            let (exists_local, exists_remote) = branch_repo
                .has_branch(&new_branch_name)
                .map_err(|e| format!("Failed to check branch existence: {}", e))?;

            if exists_local || exists_remote {
                error!("Branch '{}' already exists", new_branch_name);
                if exists_local {
                    info!("  Local branch exists");
                }
                if exists_remote {
                    info!("  Remote branch exists");
                }
                return Err(format!("Branch '{}' already exists", new_branch_name).into());
            }

            // 获取当前分支作为源分支（可能在 handle_non_default_branch 中已经切换）
            let source_branch = branch_repo
                .get_current_branch()
                .map_err(|e| format!("Failed to get current branch: {}", e))?;

            // 创建新分支
            if self.dry_run {
                info!(
                    "[DRY RUN] Would create branch '{}' from '{}'",
                    new_branch_name, source_branch
                );
                info!("[DRY RUN] Would switch to branch '{}'", new_branch_name);
            } else {
                info!(
                    "Creating branch '{}' from '{}'...",
                    new_branch_name, source_branch
                );
                branch_repo
                    .create_branch(&new_branch_name)
                    .map_err(|e| format!("Failed to create branch: {}", e))?;

                branch_repo
                    .checkout_branch(&new_branch_name)
                    .map_err(|e| format!("Failed to checkout branch: {}", e))?;

                success!("Created and switched to branch '{}'", new_branch_name);
            }

            // 生成 PR 详细总结（在提交之前）
            let pr_content = self.generate_pr_summary(
                branch_repo.as_ref(),
                &new_branch_name,
                &jira_id,
                description_for_commit.as_deref(),
            )?;

            // 提交代码（如果有更改）
            if !self.dry_run {
                self.commit_changes(
                    branch_repo.as_ref(),
                    &jira_id,
                    description_for_commit.as_deref(),
                )?;
            } else {
                let status = branch_repo
                    .get_working_tree_status()
                    .map_err(|e| format!("Failed to check working tree status: {}", e))?;

                if !status.is_clean() {
                    info!("[DRY RUN] Would commit changes");
                    let commit_message = if let Some(jira_id) = &jira_id {
                        format!(
                            "[DRY RUN] Commit message would be: {}: <JIRA summary>",
                            jira_id
                        )
                    } else if let Some(desc) = description_for_commit.as_deref() {
                        format!("[DRY RUN] Commit message would be: {}", desc)
                    } else {
                        "[DRY RUN] Commit message would be generated".to_string()
                    };
                    info!("{}", commit_message);
                    info!(
                        "[DRY RUN] Would push branch '{}' to remote",
                        new_branch_name
                    );
                } else {
                    info!("[DRY RUN] No changes to commit");
                }
            }

            // 创建 PR（如果有 PR 内容）
            if let Some(pr_content) = pr_content {
                self.create_pull_request(
                    branch_repo.as_ref(),
                    &new_branch_name,
                    &pr_content,
                    target_branch.as_deref(),
                )?;
            }
        }

        Ok(())
    }

    /// 处理非默认分支的情况
    ///
    /// 返回 (Option<String>, Option<String>)：
    /// - (None, None): 使用当前分支，不需要创建新分支，PR 已处理
    /// - (Some(branch_name), Some(target)): 需要创建的新分支名和目标分支
    fn handle_non_default_branch(
        &self,
        branch_repo: &dyn GitRepository,
        current_branch: &str,
        default_branch: &str,
        generated_branch_name: &str,
        jira_id: &Option<String>,
        description: Option<&str>,
    ) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
        let options = vec![
            BranchHandleOption::UseCurrentBranch(current_branch.to_string()),
            BranchHandleOption::CreateFromCurrent(generated_branch_name.to_string()),
            BranchHandleOption::SwitchToDefault(generated_branch_name.to_string()),
        ];

        let selected = select!("Please select how to handle branches:", options)
            .prompt()
            .map_err(|e| format!("Failed to select branch option: {}", e))?;

        match selected {
            BranchHandleOption::UseCurrentBranch(_) => {
                // 1.1 直接使用当前分支
                let pr_service = registry::get_pull_request_service();

                // 检查当前分支是否已有 PR
                let existing_pr_id = pr_service
                    .get_current_branch_pull_request(current_branch)
                    .map_err(|e| format!("Failed to check existing PR: {}", e))?;

                if let Some(pr_id) = existing_pr_id {
                    // 已有 PR，询问是否更新
                    let update_options = vec![ConfirmOption::Yes, ConfirmOption::No];
                    let update_selected = select!(
                        format!(
                            "Branch '{}' already has PR #{}. Update it?",
                            current_branch, pr_id
                        ),
                        update_options
                    )
                    .prompt()
                    .map_err(|e| format!("Failed to select update option: {}", e))?;

                    if update_selected == ConfirmOption::Yes {
                        // 生成 PR 内容
                        let pr_content = self.generate_pr_summary(
                            branch_repo,
                            current_branch,
                            jira_id,
                            description,
                        )?;

                        if let Some(pr_content) = pr_content {
                            // 更新 PR
                            if self.dry_run {
                                info!("[DRY RUN] Would update PR #{}", pr_id);
                                info!("  Title: {}", pr_content.pr_title);
                                if let Some(ref desc) = pr_content.description {
                                    info!("  Description:\n{}", desc);
                                }
                            } else {
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

                                info!("Updating PR #{}...", pr_id);
                                pr_service
                                    .update_pull_request(
                                        &pr_id,
                                        Some(&pr_content.pr_title),
                                        Some(&pr_body),
                                    )
                                    .map_err(|e| format!("Failed to update PR: {}", e))?;
                                success!("PR #{} updated successfully!", pr_id);
                            }
                        }
                    } else {
                        info!("Operation cancelled");
                    }
                    Ok((None, None))
                } else {
                    // 没有 PR，根据分支状态处理：
                    // 1. 如果有未提交的代码 -> 提交，push，创建 PR
                    // 2. 如果有提交但未 push -> push，创建 PR
                    // 3. 如果已 push -> 直接创建 PR

                    if !self.dry_run {
                        // 检查是否有未提交的更改
                        let status = branch_repo
                            .get_working_tree_status()
                            .map_err(|e| format!("Failed to check working tree status: {}", e))?;

                        if !status.is_clean() {
                            // 有未提交的更改，执行提交
                            self.commit_changes(branch_repo, jira_id, description)?;
                        } else {
                            // 没有未提交的更改，检查是否需要 push
                            let needs_push = self.check_needs_push(branch_repo, current_branch)?;
                            if needs_push {
                                self.push_branch(branch_repo)?;
                            }
                        }
                    } else {
                        let status = branch_repo
                            .get_working_tree_status()
                            .map_err(|e| format!("Failed to check working tree status: {}", e))?;

                        if !status.is_clean() {
                            info!("[DRY RUN] Would commit changes");
                            info!("[DRY RUN] Would push branch to remote");
                        } else {
                            let needs_push = self.check_needs_push(branch_repo, current_branch)?;
                            if needs_push {
                                info!("[DRY RUN] Would push branch to remote");
                            } else {
                                info!("[DRY RUN] Branch is up to date with remote");
                            }
                        }
                    }

                    // 生成 PR 内容并创建 PR
                    let pr_content = self.generate_pr_summary(
                        branch_repo,
                        current_branch,
                        jira_id,
                        description,
                    )?;

                    if let Some(pr_content) = pr_content {
                        // 在 dry-run 模式下，简化目标分支推断逻辑
                        let target_branch = if self.dry_run {
                            // 直接使用默认分支，跳过耗时的推断和交互
                            let default_branch = branch_repo
                                .get_default_branch()
                                .map_err(|e| format!("Failed to get default branch: {}", e))?;
                            info!("[DRY RUN] Target branch: {}", default_branch);
                            default_branch
                        } else {
                            // 非 dry-run 模式：推断目标分支并询问用户确认
                            let inferred_target =
                                branch_repo
                                    .infer_target_branch(current_branch)
                                    .map_err(|e| format!("Failed to infer target branch: {}", e))?;

                            self.confirm_target_branch(branch_repo, inferred_target.as_deref())?
                        };

                        self.create_pull_request(
                            branch_repo,
                            current_branch,
                            &pr_content,
                            Some(&target_branch),
                        )?;
                    }
                    Ok((None, None))
                }
            }
            BranchHandleOption::CreateFromCurrent(_) => {
                // 1.2 基于当前分支创建新分支
                // 推断当前分支的源分支，让用户选择目标分支
                let target_branch = if current_branch != default_branch {
                    // 当前分支不是默认分支，推断其源分支
                    let inferred_source = branch_repo
                        .infer_target_branch(current_branch)
                        .map_err(|e| format!("Failed to infer source branch: {}", e))?;

                    // 根据推断结果，让用户选择目标分支
                    if let Some(source) = inferred_source {
                        // 成功推断出源分支
                        if source == default_branch {
                            // 源分支就是默认分支，提供 current_branch 或 默认分支 两个选项
                            let options = vec![
                                TargetBranchOption::Current(current_branch.to_string()),
                                TargetBranchOption::Default(default_branch.to_string()),
                            ];
                            let selected =
                                select!("Please select the target branch for PR:", options)
                                    .prompt()
                                    .map_err(|e| {
                                        format!("Failed to select target branch: {}", e)
                                    })?;

                            selected.branch_name().to_string()
                        } else {
                            // 源分支不是默认分支，提供三个选项
                            let options = vec![
                                TargetBranchOption::Current(current_branch.to_string()),
                                TargetBranchOption::Inferred(source),
                                TargetBranchOption::Default(default_branch.to_string()),
                            ];
                            let selected =
                                select!("Please select the target branch for PR:", options)
                                    .prompt()
                                    .map_err(|e| {
                                        format!("Failed to select target branch: {}", e)
                                    })?;

                            selected.branch_name().to_string()
                        }
                    } else {
                        // 无法推断源分支，只提供 current_branch 或 默认分支 两个选项
                        let options = vec![
                            TargetBranchOption::Current(current_branch.to_string()),
                            TargetBranchOption::Default(default_branch.to_string()),
                        ];
                        let selected = select!("Please select the target branch for PR:", options)
                            .prompt()
                            .map_err(|e| format!("Failed to select target branch: {}", e))?;

                        selected.branch_name().to_string()
                    }
                } else {
                    // 当前分支是默认分支，直接使用默认分支作为目标
                    default_branch.to_string()
                };

                Ok((Some(generated_branch_name.to_string()), Some(target_branch)))
            }
            BranchHandleOption::SwitchToDefault(_) => {
                // 1.3 切换到默认分支，创建新分支
                if self.dry_run {
                    info!(
                        "[DRY RUN] Would stash changes, switch to '{}', and pull latest",
                        default_branch
                    );
                    let status = branch_repo
                        .get_working_tree_status()
                        .map_err(|e| format!("Failed to check working tree status: {}", e))?;
                    if !status.is_clean() {
                        info!("[DRY RUN] Would stash changes before switching");
                    }
                } else {
                    self.prepare_default_branch(branch_repo, current_branch, default_branch)?;
                }
                // 目标分支就是默认分支
                Ok((
                    Some(generated_branch_name.to_string()),
                    Some(default_branch.to_string()),
                ))
            }
        }
    }

    /// 处理默认分支的情况
    ///
    /// 返回 (Option<String>, Option<String>)：
    /// - (Some(branch_name), Some(target)): 需要创建的新分支名和目标分支
    fn handle_default_branch(
        &self,
        _branch_repo: &dyn GitRepository,
        default_branch: &str,
        generated_branch_name: &str,
        _jira_id: &Option<String>,
        _description: Option<&str>,
    ) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
        // 情况2: 是默认分支，需要创建新分支
        // 目标分支就是默认分支
        // 分支创建和后续操作在主逻辑中统一处理
        Ok((
            Some(generated_branch_name.to_string()),
            Some(default_branch.to_string()),
        ))
    }

    /// 提交代码更改
    ///
    /// 如果有 JIRA ID，使用 `{jira-id}: {summary}` 作为 commit message
    /// 如果没有 JIRA ID，使用输入的 description 作为 commit message
    ///
    /// # 返回
    /// 返回提交的 SHA（如果有提交），否则返回 None
    fn commit_changes(
        &self,
        branch_repo: &dyn GitRepository,
        jira_id: &Option<String>,
        description: Option<&str>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // 性能优化：移除 get_working_tree_status() 调用
        // 直接尝试提交，如果没有更改会在 commit 方法中处理

        // 生成 commit message
        let commit_message = if let Some(jira_id) = jira_id {
            // 检查 JIRA ID 是否为空字符串
            if jira_id.trim().is_empty() {
                // JIRA ID 为空，使用 description
                if let Some(desc) = description {
                    desc.to_string()
                } else {
                    return Err("No commit message available".into());
                }
            } else {
                // 获取 JIRA summary
                info!("Fetching JIRA ticket '{}'...", jira_id);
                let jira_repo = registry::get_jira_repository();

                // 尝试获取 JIRA ticket 信息，如果失败则使用 JIRA ID 作为降级方案
                match spinner!("Fetching JIRA ticket '{}'...", jira_id)
                    .with(|| jira_repo.get_issue_info(jira_id))
                {
                    Ok(issue) => {
                        info!("Successfully fetched JIRA ticket '{}'", jira_id);
                        format!("{}: {}", jira_id, issue.summary)
                    }
                    Err(e) => {
                        error!("Failed to fetch JIRA ticket '{}': {}", jira_id, e);
                        info!("Using JIRA ID as commit message: {}", jira_id);
                        jira_id.clone()
                    }
                }
            }
        } else if let Some(desc) = description {
            desc.to_string()
        } else {
            return Err("No commit message available".into());
        };

        // 提交更改
        info!("Committing changes with message: {}", commit_message);

        // 直接尝试提交所有更改（包括未暂存的）
        // commit 函数会处理 .gitignore 并检查是否有实际更改
        let commit_sha = match spinner!("Committing changes...")
            .with(|| branch_repo.commit(&commit_message, true))
        {
            Ok(sha) => sha,
            Err(e) => {
                let err_msg = e.to_string();
                // 检查是否是"没有更改需要提交"的错误（支持中英文）
                if err_msg.contains("nothing to commit") || err_msg.contains("没有更改需要提交")
                {
                    info!("No changes to commit");
                    return Ok(None);
                }
                return Err(format!("Failed to commit changes: {}", e).into());
            }
        };

        success!("Committed changes: {}", &commit_sha[..7]);

        // 推送代码到远端
        self.push_branch(branch_repo)?;

        Ok(Some(commit_sha))
    }

    /// 创建 Pull Request
    ///
    /// 使用生成的 PR 内容创建 Pull Request
    ///
    /// # 参数
    /// - `target_branch`: 可选的目标分支，如果为 None 则使用默认分支
    fn create_pull_request(
        &self,
        branch_repo: &dyn GitRepository,
        branch_name: &str,
        pr_content: &PullRequestContent,
        target_branch: Option<&str>,
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

        if self.dry_run {
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
        if let Some(ref origin_url) = repo_info.origin_url {
            // 从 origin_url 提取 owner/repo 并构建 PR URL
            // 例如: https://github.com/owner/repo.git 或 git@github.com:owner/repo.git
            if let Some(pr_url) = extract_pr_url(origin_url, &pr_id) {
                info!("PR URL: {}", pr_url);

                // 使用默认浏览器打开 PR 页面
                match pr_url.open_in_browser() {
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
                    "Could not extract PR URL from origin: {}. Only GitHub URLs are supported.",
                    origin_url
                );
            }
        } else {
            warning!("No origin URL found, cannot generate PR URL");
        }

        Ok(())
    }

    /// 推送分支到远端
    fn push_branch(
        &self,
        branch_repo: &dyn GitRepository,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取当前分支名
        let current_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 推送分支到远端
        info!("Pushing branch '{}' to remote...", current_branch);
        branch_repo
            .push(&current_branch, true)
            .map_err(|e| format!("Failed to push branch: {}", e))?;

        success!("Pushed branch '{}' to remote", current_branch);

        Ok(())
    }

    /// 检查是否需要推送分支到远端
    ///
    /// 返回 true 如果：
    /// - 远程分支不存在
    /// - 本地有未推送到远程的提交
    fn check_needs_push(
        &self,
        branch_repo: &dyn GitRepository,
        branch_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // 检查远程分支是否存在
        let (_, remote_exists) = branch_repo
            .has_branch(branch_name)
            .map_err(|e| format!("Failed to check branch existence: {}", e))?;

        if !remote_exists {
            // 远程分支不存在，需要 push
            return Ok(true);
        }

        // 远程分支存在，检查本地 HEAD 是否已在远程
        let head_commit = branch_repo
            .get_commit_info("HEAD")
            .map_err(|e| format!("Failed to get HEAD commit: {}", e))?;

        let is_in_remote = branch_repo
            .is_commit_in_remote_branch(branch_name, &head_commit.sha)
            .map_err(|e| format!("Failed to check commit in remote: {}", e))?;

        // 如果本地 HEAD 不在远程，需要 push
        Ok(!is_in_remote)
    }

    /// 准备默认分支的辅助方法
    ///
    /// 处理 stash、切换分支、拉取最新代码等操作
    ///
    /// # 返回
    /// 返回是否需要在新分支上恢复 stash
    fn prepare_default_branch(
        &self,
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

        // 拉取最新代码
        info!("Pulling latest changes from '{}'...", default_branch);
        branch_repo
            .pull(default_branch)
            .map_err(|e| format!("Failed to pull latest changes: {}", e))?;

        // 返回是否需要恢复 stash（将在新分支上恢复）
        Ok(needs_stash)
    }

    /// 询问用户确认目标分支
    ///
    /// # 参数
    /// - `branch_repo`: Git 仓库
    /// - `inferred_target`: 推断出的目标分支（可能为 None）
    ///
    /// # 返回
    /// 用户选择的目标分支名称
    fn confirm_target_branch(
        &self,
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
    fn generate_pr_summary(
        &self,
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
        if git_diff.is_none() || git_diff.as_ref().unwrap().trim().is_empty() {
            info!("No changes to generate PR summary");
            return Ok(None);
        }

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
            .with(|| llm_repo.create_pr_content(&commit_title, Some(branch_names), git_diff))
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
            info!(
                "No detailed summary generated (this is normal if git diff is empty or too large)"
            );
        }

        Ok(Some(pr_content))
    }
}

/// 从远程 URL 提取 PR URL
///
/// 支持以下格式：
/// - https://github.com/owner/repo.git
/// - https://github.com/owner/repo
/// - git@github.com:owner/repo.git
/// - git@github.com:owner/repo
fn extract_pr_url(remote_url: &str, pr_id: &str) -> Option<String> {
    // 处理 https:// 格式
    if let Some(stripped) = remote_url.strip_prefix("https://github.com/") {
        // 移除可能的 .git 后缀
        let repo = stripped.strip_suffix(".git").unwrap_or(stripped);
        if !repo.is_empty() {
            return Some(format!("https://github.com/{}/pull/{}", repo, pr_id));
        }
    }

    // 处理 git@ 格式
    if let Some(stripped) = remote_url.strip_prefix("git@github.com:") {
        // 移除可能的 .git 后缀
        let repo = stripped.strip_suffix(".git").unwrap_or(stripped);
        if !repo.is_empty() {
            return Some(format!("https://github.com/{}/pull/{}", repo, pr_id));
        }
    }

    None
}
