//! 提交和推送相关逻辑

use domain::GitRepository;
use prompt::{error, info, spinner, success};

use crate::bootstrap;
use crate::util::safe_push;

/// 提交代码更改
///
/// 如果有 JIRA ID，使用 `{jira-id}: {summary}` 作为 commit message
/// 如果没有 JIRA ID，使用输入的 description 作为 commit message
///
/// # 返回
/// 返回提交的 SHA（如果有提交），否则返回 None
pub fn commit_changes(
    branch_repo: &dyn GitRepository,
    jira_id: &Option<String>,
    description: Option<&str>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
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
            let jira_repo = bootstrap::get_jira_repository();

            // 尝试获取 JIRA ticket 信息，如果失败则使用 JIRA ID 作为降级方案
            match spinner!("Fetching JIRA ticket '{}'...", jira_id)
                .with(|| jira_repo.get_issue_info(jira_id))
            {
                Ok(issue) => {
                    info!("Successfully fetched JIRA ticket '{}'", jira_id);
                    format!("{}: {}", jira_id, issue.fields.summary)
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

    info!("Committing changes with message: {}", commit_message);
    // 直接尝试提交所有更改（包括未暂存的）
    // commit 函数会处理 .gitignore 并检查是否有实际更改
    let commit_sha = branch_repo.commit(&commit_message, true).map_err(|e| {
        let err_msg = e.to_string();
        if err_msg.contains("nothing to commit") {
            return "No changes to commit".into();
        }
        format!("Failed to commit changes: {}", e)
    })?;

    success!("Committed changes: {}", &commit_sha[..7]);

    // 推送代码到远端
    push_branch(branch_repo)?;

    Ok(Some(commit_sha))
}

/// 推送分支到远端
pub fn push_branch(branch_repo: &dyn GitRepository) -> Result<(), Box<dyn std::error::Error>> {
    let current_branch = branch_repo
        .get_current_branch()
        .map_err(|e| format!("Failed to get current branch: {}", e))?;

    spinner!("Pushing branch '{}' to remote...", current_branch)
        .with(|| safe_push(&current_branch, true))
        .map_err(|e| format!("Failed to push branch: {}", e))?;

    success!("Pushed branch '{}' to remote", current_branch);
    Ok(())
}

/// 检查是否需要推送分支到远端
///
/// 返回 true 如果：
/// - 远程分支不存在
/// - 本地有未推送到远程的提交
pub fn check_needs_push(
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
