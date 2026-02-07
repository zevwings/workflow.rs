//! Pull Request 服务实现
//!
//! 实现 `PullRequestService` trait，提供 PR 相关的业务用例编排，
//! 协调多个底层服务（GitHub、LLM、Git）。

use std::sync::Arc;

use domain::{
    errors::ServiceError, CodePlatform, CommitSummaryService, GitHubRepository, GitRepository,
    PrStatus, PullRequestInfo, PullRequestService,
};

/// Pull Request 服务实现
///
/// 组合 GitHub 仓储、LLM 服务和 Git 仓储，实现完整的 PR 业务用例。
pub struct PullRequestServiceImpl {
    git_repo: Arc<dyn GitRepository>,
    github_repo: Arc<dyn GitHubRepository>,
    commit_summary_service: Arc<dyn CommitSummaryService>,
}

impl PullRequestServiceImpl {
    pub fn new(
        git_repo: Arc<dyn GitRepository>,
        github_repo: Arc<dyn GitHubRepository>,
        commit_summary_service: Arc<dyn CommitSummaryService>,
    ) -> Self {
        Self {
            git_repo,
            github_repo,
            commit_summary_service,
        }
    }

    /// 获取 PR repository，同时检查仓库类型是否支持 PR 操作（当前仅支持 GitHub）
    fn get_pr_repository(&self) -> Result<Arc<dyn GitHubRepository>, ServiceError> {
        let repo_info = self.git_repo.get_repo_info();
        match repo_info.kind.unwrap_or(CodePlatform::Unknown) {
            CodePlatform::GitHub => Ok(self.github_repo.clone()),
            _ => Err(ServiceError::UnsupportedOperation(
                "PR operations are not supported for this repository".to_string(),
            )),
        }
    }

    /// 解析 PR ID（支持自动检测当前分支的 PR）
    fn resolve_pr_id(&self, pr_id_or_branch: Option<&str>) -> Result<String, ServiceError> {
        match pr_id_or_branch {
            Some(id) => Ok(id.to_string()),
            None => {
                let current_branch = self.git_repo.get_current_branch()?;
                self.get_current_branch_pull_request(&current_branch)?.ok_or_else(|| {
                    ServiceError::NotFound(format!(
                        "No PR found for current branch '{}'",
                        current_branch
                    ))
                })
            }
        }
    }
}

fn map_github_err<T, E: std::fmt::Display>(r: Result<T, E>) -> Result<T, ServiceError> {
    r.map_err(|e| ServiceError::Other(format!("GitHub API error: {}", e)))
}

impl PullRequestService for PullRequestServiceImpl {
    fn create_pull_request(
        &self,
        jira_id: Option<&str>,
        title: Option<&str>,
        description: Option<&str>,
        target_branch: Option<&str>,
    ) -> Result<String, ServiceError> {
        // 获取当前分支信息
        let current_branch = self.git_repo.get_current_branch()?;

        // 使用提供的目标分支或获取默认分支
        let final_target_branch = match target_branch {
            Some(branch) => branch.to_string(),
            None => self
                .git_repo
                .get_default_branch()
                .unwrap_or_else(|_| "main".to_string()),
        };

        // 生成 PR 标题和描述
        let (final_title, final_description) = if title.is_none() {
            // 使用 CommitSummaryService 生成 PR 内容
            let analysis = self
                .commit_summary_service
                .run_analysis(Some(&final_target_branch))
                .map_err(|e| {
                    ServiceError::Other(format!("Failed to generate PR content: {}", e))
                })?;

            // 从 commit message 生成 PR 标题
            let pr_title = if !analysis.commit_message.title.is_empty() {
                analysis.commit_message.title.clone()
            } else {
                // Fallback: 使用 type + scope + subject
                let type_ = &analysis.structured_summary.type_;
                let scope = &analysis.structured_summary.scope;
                let subject = &analysis.structured_summary.subject;

                if scope.is_empty() {
                    format!("{}: {}", type_, subject)
                } else {
                    format!("{}({}): {}", type_, scope, subject)
                }
            };

            // 生成 PR 描述（使用结构化的摘要信息）
            let mut pr_desc = String::new();

            // Summary section
            if !analysis.structured_summary.main_purpose.is_empty() {
                pr_desc.push_str("## Summary\n\n");
                pr_desc.push_str(&analysis.structured_summary.main_purpose);
                pr_desc.push_str("\n\n");
            }

            // Key changes
            if !analysis.structured_summary.key_changes.is_empty() {
                pr_desc.push_str("### Key Changes\n\n");
                for change in &analysis.structured_summary.key_changes {
                    pr_desc.push_str(&format!("- {}\n", change));
                }
                pr_desc.push('\n');
            }

            // Breaking changes (if any)
            if analysis.impact_analysis.breaking_changes.has_breaking {
                pr_desc.push_str("## ⚠️ Breaking Changes\n\n");
                pr_desc.push_str(&analysis.impact_analysis.breaking_changes.description);
                pr_desc.push_str("\n\n");
                if !analysis
                    .impact_analysis
                    .breaking_changes
                    .migration_guide
                    .is_empty()
                {
                    pr_desc.push_str("### Migration Guide\n\n");
                    pr_desc.push_str(
                        &analysis
                            .impact_analysis
                            .breaking_changes
                            .migration_guide,
                    );
                    pr_desc.push_str("\n\n");
                }
            }

            // Testing suggestions
            if !analysis.impact_analysis.testing_suggestions.is_empty() {
                pr_desc.push_str("## Testing\n\n");
                for suggestion in &analysis.impact_analysis.testing_suggestions {
                    pr_desc.push_str(&format!("- {}\n", suggestion));
                }
                pr_desc.push('\n');
            }

            // 如果提供了 jira_id，添加到描述中
            if let Some(jid) = jira_id {
                pr_desc.push_str("---\n\n");
                pr_desc.push_str(&format!("**Jira**: {}\n", jid));
            }

            (pr_title, pr_desc.trim_end().to_string())
        } else {
            // 使用用户提供的标题和描述
            let pr_title = title.unwrap_or("Untitled PR").to_string();
            let mut pr_desc = description.unwrap_or("").to_string();

            // 如果提供了 jira_id，添加到描述中
            if let Some(jid) = jira_id {
                if !pr_desc.is_empty() {
                    pr_desc.push_str("\n\n");
                }
                pr_desc.push_str("---\n\n");
                pr_desc.push_str(&format!("**Jira**: {}\n", jid));
            }

            (pr_title, pr_desc)
        };

        let repo = self.get_pr_repository()?;
        let pr_id = map_github_err(repo.create_pull_request(
            &final_title,
            &final_description,
            &current_branch,
            &final_target_branch,
        ))?;

        Ok(pr_id)
    }

    fn merge_pull_request(&self, pr_id: &str, force: bool) -> Result<(), ServiceError> {
        let repo = self.get_pr_repository()?;
        map_github_err(repo.merge_pull_request(pr_id, force))
    }

    fn get_pr_status(&self, pr_id_or_branch: Option<&str>) -> Result<PrStatus, ServiceError> {
        let pr_id = self.resolve_pr_id(pr_id_or_branch)?;
        let repo = self.get_pr_repository()?;
        let (state, merged, _merged_at) = map_github_err(repo.get_pull_request_status(&pr_id))?;
        let pr_info = map_github_err(repo.get_pull_request(&pr_id))?;

        Ok(PrStatus {
            id: pr_id,
            title: pr_info.title,
            state,
            merged,
        })
    }

    fn close_pull_request(&self, pr_id: &str) -> Result<(), ServiceError> {
        let repo = self.get_pr_repository()?;
        map_github_err(repo.close_pull_request(pr_id))
    }

    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PrStatus>, ServiceError> {
        let repo = self.get_pr_repository()?;
        let prs = map_github_err(repo.list_pull_requests(state, limit))?;

        Ok(prs
            .into_iter()
            .map(|pr| PrStatus {
                id: pr.id,
                title: pr.title,
                state: pr.status.state,
                merged: pr.status.merged,
            })
            .collect())
    }

    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), ServiceError> {
        let repo = self.get_pr_repository()?;
        map_github_err(repo.update_pull_request(pr_id, title, body))
    }

    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), ServiceError> {
        if comment.is_empty() {
            return Err(ServiceError::InvalidInput(
                "Comment cannot be empty".to_string(),
            ));
        }
        let repo = self.get_pr_repository()?;
        map_github_err(repo.add_comment(pr_id, comment))
    }

    fn approve_pull_request(&self, pr_id: &str) -> Result<(), ServiceError> {
        let repo = self.get_pr_repository()?;
        map_github_err(repo.approve_pull_request(pr_id))
    }

    fn get_pr_diff(&self, pr_id: &str) -> Result<String, ServiceError> {
        let repo = self.get_pr_repository()?;
        map_github_err(repo.get_pr_diff(pr_id))
    }

    fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, ServiceError> {
        let repo = self.get_pr_repository()?;
        map_github_err(repo.get_pull_request(pr_id))
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, ServiceError> {
        let repo = self.get_pr_repository()?;
        map_github_err(repo.get_current_branch_pull_request(current_branch))
    }
}
