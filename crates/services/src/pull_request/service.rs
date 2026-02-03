//! Pull Request 服务实现
//!
//! 实现 `PullRequestService` trait，提供 PR 相关的业务用例编排，
//! 协调多个底层服务（GitHub、LLM、Git）。

use std::sync::Arc;

use domain::{
    errors::ServiceError, CodePlatform, GitHubRepository, GitRepository, LLMRepository, PrStatus,
    PullRequestInfo, PullRequestService,
};

/// Pull Request 服务实现
///
/// 组合 GitHub 仓储、LLM 服务和 Git 仓储，实现完整的 PR 业务用例。
pub struct PullRequestServiceImpl {
    git_repo: Arc<dyn GitRepository>,
    github_repo: Arc<dyn GitHubRepository>,
    llm_repo: Arc<dyn LLMRepository>,
}

impl PullRequestServiceImpl {
    pub fn new(
        git_repo: Arc<dyn GitRepository>,
        github_repo: Arc<dyn GitHubRepository>,
        llm_repo: Arc<dyn LLMRepository>,
    ) -> Self {
        Self {
            git_repo,
            github_repo,
            llm_repo,
        }
    }

    /// 检查仓库类型是否支持 PR 操作
    fn check_pr_support(&self) -> Result<(), ServiceError> {
        let repo_info = self.git_repo.get_repo_info();
        match repo_info.kind.unwrap_or(CodePlatform::Unknown) {
            CodePlatform::GitHub => Ok(()),
            _ => Err(ServiceError::UnsupportedOperation(
                "PR operations are not supported for this repository".to_string(),
            )),
        }
    }

    /// 根据仓库类型获取 PR repository（当前仅支持 GitHub）
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
                self.get_current_branch_pull_request(&current_branch)?
                    .ok_or_else(|| {
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
        self.check_pr_support()?;

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

        // 如果未提供标题，使用 LLM 生成 PR 内容
        let (final_title, final_description) = if title.is_none() {
            // 使用 LLM 根据当前分支名生成 PR 内容
            // 这里我们使用 create_pr_content，它会使用分支名来生成标题和描述
            let pr_content = self.llm_repo.create_pr_content(
                &current_branch,
                None, // exists_branches
                None, // git_diff
            )?;

            let mut desc = pr_content
                .description
                .unwrap_or_else(|| String::from("No description provided"));

            // 如果提供了 jira_id，添加到描述中
            if let Some(jid) = jira_id {
                let jira_link = format!("\n\nJira: {}", jid);
                desc = desc + &jira_link;
            }

            (pr_content.pr_title, desc)
        } else {
            let title = title.unwrap_or("Untitled PR").to_string();
            let mut desc = description.unwrap_or("").to_string();

            // 如果提供了 jira_id，添加到描述中
            if let Some(jid) = jira_id {
                let jira_link = format!("\n\nJira: {}", jid);
                desc = desc + &jira_link;
            }

            (title, desc)
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
        self.check_pr_support()?;
        let repo = self.get_pr_repository()?;
        map_github_err(repo.merge_pull_request(pr_id, force))
    }

    fn get_pr_status(&self, pr_id_or_branch: Option<&str>) -> Result<PrStatus, ServiceError> {
        self.check_pr_support()?;

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
        self.check_pr_support()?;
        let repo = self.get_pr_repository()?;
        map_github_err(repo.close_pull_request(pr_id))
    }

    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PrStatus>, ServiceError> {
        self.check_pr_support()?;

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
        self.check_pr_support()?;
        let repo = self.get_pr_repository()?;
        map_github_err(repo.update_pull_request(pr_id, title, body))
    }

    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), ServiceError> {
        self.check_pr_support()?;
        if comment.is_empty() {
            return Err(ServiceError::InvalidInput(
                "Comment cannot be empty".to_string(),
            ));
        }
        let repo = self.get_pr_repository()?;
        map_github_err(repo.add_comment(pr_id, comment))
    }

    fn approve_pull_request(&self, pr_id: &str) -> Result<(), ServiceError> {
        self.check_pr_support()?;
        let repo = self.get_pr_repository()?;
        map_github_err(repo.approve_pull_request(pr_id))
    }

    fn get_pr_diff(&self, pr_id: &str) -> Result<String, ServiceError> {
        self.check_pr_support()?;
        let repo = self.get_pr_repository()?;
        map_github_err(repo.get_pr_diff(pr_id))
    }

    fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, ServiceError> {
        self.check_pr_support()?;
        let repo = self.get_pr_repository()?;
        map_github_err(repo.get_pull_request(pr_id))
    }

    fn summarize_pull_request(
        &self,
        pr_id: Option<&str>,
    ) -> Result<domain::llm::entity::PullRequestSummary, ServiceError> {
        self.check_pr_support()?;
        let pr_id = self.resolve_pr_id(pr_id)?;
        let repo = self.get_pr_repository()?;
        let pr_info = map_github_err(repo.get_pull_request(&pr_id))?;
        let pr_diff = map_github_err(repo.get_pr_diff(&pr_id))?;

        // 调用 LLM 生成总结
        let summary = self.llm_repo.summarize_pr(&pr_info.title, &pr_diff)?;

        Ok(summary)
    }

    fn reword_pull_request(
        &self,
        pr_id: Option<&str>,
    ) -> Result<domain::llm::entity::PullRequestReword, ServiceError> {
        self.check_pr_support()?;
        let pr_id = self.resolve_pr_id(pr_id)?;
        let repo = self.get_pr_repository()?;
        let pr_info = map_github_err(repo.get_pull_request(&pr_id))?;
        let pr_diff = map_github_err(repo.get_pr_diff(&pr_id))?;

        // 调用 LLM 重写 PR
        let reword = self.llm_repo.reword_pr(&pr_diff, Some(&pr_info.title))?;

        Ok(reword)
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, ServiceError> {
        self.check_pr_support()?;
        let repo = self.get_pr_repository()?;
        map_github_err(repo.get_current_branch_pull_request(current_branch))
    }
}
