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
            None => self.git_repo.get_default_branch().unwrap_or_else(|_| "main".to_string()),
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
                desc.push_str("\n\nJira: ");
                desc.push_str(jid);
            }

            (pr_content.pr_title, desc)
        } else {
            let title = title.unwrap_or("Untitled PR").to_string();
            let mut desc = description.unwrap_or("").to_string();

            // 如果提供了 jira_id，添加到描述中
            if let Some(jid) = jira_id {
                desc.push_str("\n\nJira: ");
                desc.push_str(jid);
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

    fn summarize_pull_request(
        &self,
        pr_id: Option<&str>,
    ) -> Result<domain::llm::entity::PullRequestSummary, ServiceError> {
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
        let repo = self.get_pr_repository()?;
        map_github_err(repo.get_current_branch_pull_request(current_branch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use domain::git::error::GitError;
    use domain::git::{StashApplyResult, StashEntry, StashPopResult};
    use domain::github::error::GitHubError;
    use domain::llm::entity::{PullRequestContent, PullRequestReword, PullRequestSummary};
    use domain::llm::error::LLMError;
    use domain::pr::entity::{PullRequestInfo, PullRequestStatus};
    use domain::GitRepository;
    use domain::RepoInfo;

    struct MockGitRepository {
        repo_info: RepoInfo,
        current_branch: Mutex<String>,
        default_branch: Option<String>,
    }

    impl MockGitRepository {
        fn new(repo_info: RepoInfo, current_branch: &str, default_branch: Option<String>) -> Self {
            Self {
                repo_info,
                current_branch: Mutex::new(current_branch.to_string()),
                default_branch,
            }
        }
    }

    impl GitRepository for MockGitRepository {
        fn get_repo_info(&self) -> RepoInfo {
            self.repo_info.clone()
        }

        fn get_ignore_directory_patterns(&self) -> Vec<String> {
            Vec::new()
        }

        fn get_working_tree_diff(&self, _base_branch: &str) -> Result<Option<String>, GitError> {
            unimplemented!("test-only")
        }

        fn create_branch(&self, _name: &str) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn delete_local_branch(&self, _name: &str, _force: bool) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn delete_remote_branch(&self, _name: &str) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn rename_branch(&self, _old_name: Option<&str>, _new_name: &str) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn list_branches(
            &self,
            _remove_prefix: bool,
            _all: bool,
        ) -> Result<Vec<domain::BranchInfo>, GitError> {
            unimplemented!("test-only")
        }

        fn checkout_branch(&self, _name: &str) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn get_current_branch(&self) -> Result<String, GitError> {
            self.current_branch
                .lock()
                .map(|name| name.clone())
                .map_err(|_| GitError::Other("Failed to lock current branch".to_string()))
        }

        fn has_branch(&self, _name: &str) -> Result<(bool, bool), GitError> {
            unimplemented!("test-only")
        }

        fn get_default_branch(&self) -> Result<String, GitError> {
            self.default_branch.clone().ok_or_else(|| {
                GitError::BranchNotFound("default branch not configured".to_string())
            })
        }

        fn infer_target_branch(&self, _current_branch: &str) -> Result<Option<String>, GitError> {
            unimplemented!("test-only")
        }

        fn get_commit_info(&self, _ref_or_sha: &str) -> Result<domain::CommitInfo, GitError> {
            unimplemented!("test-only")
        }

        fn get_commit_changed_files(
            &self,
            _ref_or_sha: &str,
        ) -> Result<Vec<domain::CommitFileChange>, GitError> {
            unimplemented!("test-only")
        }

        fn get_commit_diff(&self, _ref_or_sha: &str) -> Result<Option<String>, GitError> {
            unimplemented!("test-only")
        }

        fn get_working_tree_status(&self) -> Result<domain::WorkingTreeStatus, GitError> {
            unimplemented!("test-only")
        }

        fn commit(&self, _message: &str, _all: bool) -> Result<String, GitError> {
            unimplemented!("test-only")
        }

        fn merge_branch(
            &self,
            _source_branch: &str,
            _strategy: domain::MergeStrategy,
        ) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn has_merge_conflicts(&self) -> Result<bool, GitError> {
            unimplemented!("test-only")
        }

        fn is_branch_merged(&self, _branch: &str, _base_branch: &str) -> Result<bool, GitError> {
            unimplemented!("test-only")
        }

        fn merge_base(&self, _branch1: &str, _branch2: &str) -> Result<String, GitError> {
            unimplemented!("test-only")
        }

        fn commits_to_merge(
            &self,
            _source_branch: &str,
            _target_branch: &str,
        ) -> Result<Vec<String>, GitError> {
            unimplemented!("test-only")
        }

        fn rebase_onto(&self, _target_branch: &str) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn rebase_onto_with_upstream(
            &self,
            _newbase: &str,
            _upstream: &str,
            _branch: &str,
        ) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn push(&self, _branch_name: &str, _set_upstream: bool) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn pull(&self, _branch_name: &str) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn is_commit_in_remote_branch(
            &self,
            _branch: &str,
            _commit_sha: &str,
        ) -> Result<bool, GitError> {
            unimplemented!("test-only")
        }

        fn stash_push(&self, _message: Option<&str>) -> Result<usize, GitError> {
            unimplemented!("test-only")
        }

        fn stash_pop(&self, _index: usize) -> Result<StashPopResult, GitError> {
            unimplemented!("test-only")
        }

        fn stash_apply(&self, _index: usize) -> Result<StashApplyResult, GitError> {
            unimplemented!("test-only")
        }

        fn stash_list(&self) -> Result<Vec<StashEntry>, GitError> {
            unimplemented!("test-only")
        }

        fn stash_drop(&self, _index: usize) -> Result<(), GitError> {
            unimplemented!("test-only")
        }

        fn create_tag(
            &self,
            _name: &str,
            _target: Option<&str>,
            _message: Option<&str>,
            _scope: domain::TagCreateScope,
            _force: bool,
        ) -> Result<domain::TagCreateInfo, GitError> {
            unimplemented!("test-only")
        }

        fn delete_tag(
            &self,
            _name: &str,
            _scope: domain::TagDeleteScope,
            _force: bool,
        ) -> Result<domain::TagDeleteInfo, GitError> {
            unimplemented!("test-only")
        }

        fn delete_tags_by_pattern(
            &self,
            _pattern: &str,
            _scope: domain::TagDeleteScope,
            _force: bool,
        ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
            unimplemented!("test-only")
        }

        fn list_tags(&self, _include_remote: bool) -> Result<Vec<String>, GitError> {
            unimplemented!("test-only")
        }

        fn has_tag(&self, _name: &str) -> Result<(bool, bool), GitError> {
            unimplemented!("test-only")
        }

        fn preview_delete(
            &self,
            _name: Option<&str>,
            _pattern: Option<&str>,
            _scope: domain::TagDeleteScope,
        ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
            unimplemented!("test-only")
        }

        fn get_file_blame(
            &self,
            _file_path: &str,
            _revision: Option<&str>,
        ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
            unimplemented!("test-only")
        }

        fn get_file_blame_range(
            &self,
            _file_path: &str,
            _start_line: usize,
            _end_line: usize,
            _revision: Option<&str>,
        ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
            unimplemented!("test-only")
        }
    }

    struct MockGitHubRepository {
        created: Mutex<Vec<(String, String, String, String)>>,
        create_pr_id: String,
        pull_requests: Mutex<HashMap<String, PullRequestInfo>>,
        statuses: Mutex<HashMap<String, (String, bool, Option<String>)>>,
        list_result: Mutex<Vec<PullRequestInfo>>,
        current_branch_pr: Mutex<Option<String>>,
        merge_calls: Mutex<Vec<(String, bool)>>,
        close_calls: Mutex<Vec<String>>,
        update_calls: Mutex<Vec<(String, Option<String>, Option<String>)>>,
        comment_calls: Mutex<Vec<(String, String)>>,
        approve_calls: Mutex<Vec<String>>,
    }

    impl MockGitHubRepository {
        fn new(create_pr_id: &str) -> Self {
            Self {
                created: Mutex::new(Vec::new()),
                create_pr_id: create_pr_id.to_string(),
                pull_requests: Mutex::new(HashMap::new()),
                statuses: Mutex::new(HashMap::new()),
                list_result: Mutex::new(Vec::new()),
                current_branch_pr: Mutex::new(None),
                merge_calls: Mutex::new(Vec::new()),
                close_calls: Mutex::new(Vec::new()),
                update_calls: Mutex::new(Vec::new()),
                comment_calls: Mutex::new(Vec::new()),
                approve_calls: Mutex::new(Vec::new()),
            }
        }

        fn set_pull_request(&self, pr: PullRequestInfo, status: (String, bool, Option<String>)) {
            let pr_id = pr.id.clone();
            self.pull_requests.lock().expect("lock pull_requests").insert(pr_id.clone(), pr);
            self.statuses
                .lock()
                .expect("lock statuses")
                .insert(pr_id, (status.0, status.1, status.2.clone()));
        }

        fn set_list_result(&self, prs: Vec<PullRequestInfo>) {
            *self.list_result.lock().expect("lock list_result") = prs;
        }

        fn set_current_branch_pr(&self, pr_id: Option<String>) {
            *self.current_branch_pr.lock().expect("lock current_branch_pr") = pr_id;
        }

        fn last_create_args(&self) -> (String, String, String, String) {
            self.created
                .lock()
                .expect("lock created")
                .last()
                .cloned()
                .expect("missing create args")
        }

        fn last_merge_args(&self) -> (String, bool) {
            self.merge_calls
                .lock()
                .expect("lock merge_calls")
                .last()
                .cloned()
                .expect("missing merge args")
        }

        fn last_close_args(&self) -> String {
            self.close_calls
                .lock()
                .expect("lock close_calls")
                .last()
                .cloned()
                .expect("missing close args")
        }

        fn last_update_args(&self) -> (String, Option<String>, Option<String>) {
            self.update_calls
                .lock()
                .expect("lock update_calls")
                .last()
                .cloned()
                .expect("missing update args")
        }

        fn last_comment_args(&self) -> (String, String) {
            self.comment_calls
                .lock()
                .expect("lock comment_calls")
                .last()
                .cloned()
                .expect("missing comment args")
        }

        fn last_approve_args(&self) -> String {
            self.approve_calls
                .lock()
                .expect("lock approve_calls")
                .last()
                .cloned()
                .expect("missing approve args")
        }
    }

    impl GitHubRepository for MockGitHubRepository {
        fn create_pull_request(
            &self,
            title: &str,
            body: &str,
            source_branch: &str,
            target_branch: &str,
        ) -> Result<String, GitHubError> {
            self.created.lock().expect("lock created").push((
                title.to_string(),
                body.to_string(),
                source_branch.to_string(),
                target_branch.to_string(),
            ));
            Ok(self.create_pr_id.clone())
        }

        fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, GitHubError> {
            self.pull_requests
                .lock()
                .expect("lock pull_requests")
                .get(pr_id)
                .cloned()
                .ok_or_else(|| GitHubError::NotFound(pr_id.to_string()))
        }

        fn merge_pull_request(&self, _pr_id: &str, _force: bool) -> Result<(), GitHubError> {
            self.merge_calls
                .lock()
                .expect("lock merge_calls")
                .push((_pr_id.to_string(), _force));
            Ok(())
        }

        fn get_user_info(&self) -> Result<domain::github::entity::GitHubUser, GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn close_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
            self.close_calls.lock().expect("lock close_calls").push(_pr_id.to_string());
            Ok(())
        }

        fn list_pull_requests(
            &self,
            _state: Option<&str>,
            _limit: Option<usize>,
        ) -> Result<Vec<PullRequestInfo>, GitHubError> {
            Ok(self.list_result.lock().expect("lock list_result").clone())
        }

        fn update_pull_request(
            &self,
            _pr_id: &str,
            _title: Option<&str>,
            _body: Option<&str>,
        ) -> Result<(), GitHubError> {
            self.update_calls.lock().expect("lock update_calls").push((
                _pr_id.to_string(),
                _title.map(|s| s.to_string()),
                _body.map(|s| s.to_string()),
            ));
            Ok(())
        }

        fn add_comment(&self, _pr_id: &str, _comment: &str) -> Result<(), GitHubError> {
            self.comment_calls
                .lock()
                .expect("lock comment_calls")
                .push((_pr_id.to_string(), _comment.to_string()));
            Ok(())
        }

        fn approve_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
            self.approve_calls.lock().expect("lock approve_calls").push(_pr_id.to_string());
            Ok(())
        }

        fn get_pr_diff(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Ok("diff".to_string())
        }

        fn get_pull_request_info(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn get_pull_request_url(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn get_pull_request_title(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn get_pull_request_body(&self, _pr_id: &str) -> Result<Option<String>, GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn get_pull_request_status(
            &self,
            pr_id: &str,
        ) -> Result<(String, bool, Option<String>), GitHubError> {
            self.statuses
                .lock()
                .expect("lock statuses")
                .get(pr_id)
                .cloned()
                .ok_or_else(|| GitHubError::NotFound(pr_id.to_string()))
        }

        fn update_pr_base(&self, _pr_id: &str, _new_base: &str) -> Result<(), GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn get_current_branch_pull_request(
            &self,
            _current_branch: &str,
        ) -> Result<Option<String>, GitHubError> {
            Ok(self.current_branch_pr.lock().expect("lock current_branch_pr").clone())
        }
    }

    struct FailingGitHubRepository;

    impl GitHubRepository for FailingGitHubRepository {
        fn create_pull_request(
            &self,
            _title: &str,
            _body: &str,
            _source_branch: &str,
            _target_branch: &str,
        ) -> Result<String, GitHubError> {
            unimplemented!("test-only")
        }

        fn get_pull_request(&self, _pr_id: &str) -> Result<PullRequestInfo, GitHubError> {
            unimplemented!("test-only")
        }

        fn merge_pull_request(&self, _pr_id: &str, _force: bool) -> Result<(), GitHubError> {
            Err(GitHubError::Other("boom".to_string()))
        }

        fn get_user_info(&self) -> Result<domain::github::entity::GitHubUser, GitHubError> {
            unimplemented!("test-only")
        }

        fn close_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
            unimplemented!("test-only")
        }

        fn list_pull_requests(
            &self,
            _state: Option<&str>,
            _limit: Option<usize>,
        ) -> Result<Vec<PullRequestInfo>, GitHubError> {
            unimplemented!("test-only")
        }

        fn update_pull_request(
            &self,
            _pr_id: &str,
            _title: Option<&str>,
            _body: Option<&str>,
        ) -> Result<(), GitHubError> {
            unimplemented!("test-only")
        }

        fn add_comment(&self, _pr_id: &str, _comment: &str) -> Result<(), GitHubError> {
            unimplemented!("test-only")
        }

        fn approve_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
            unimplemented!("test-only")
        }

        fn get_pr_diff(&self, _pr_id: &str) -> Result<String, GitHubError> {
            unimplemented!("test-only")
        }

        fn get_pull_request_info(&self, _pr_id: &str) -> Result<String, GitHubError> {
            unimplemented!("test-only")
        }

        fn get_pull_request_url(&self, _pr_id: &str) -> Result<String, GitHubError> {
            unimplemented!("test-only")
        }

        fn get_pull_request_title(&self, _pr_id: &str) -> Result<String, GitHubError> {
            unimplemented!("test-only")
        }

        fn get_pull_request_body(&self, _pr_id: &str) -> Result<Option<String>, GitHubError> {
            unimplemented!("test-only")
        }

        fn get_pull_request_status(
            &self,
            _pr_id: &str,
        ) -> Result<(String, bool, Option<String>), GitHubError> {
            unimplemented!("test-only")
        }

        fn update_pr_base(&self, _pr_id: &str, _new_base: &str) -> Result<(), GitHubError> {
            unimplemented!("test-only")
        }

        fn get_current_branch_pull_request(
            &self,
            _current_branch: &str,
        ) -> Result<Option<String>, GitHubError> {
            unimplemented!("test-only")
        }
    }

    struct MockLLMRepository {
        create_content: PullRequestContent,
        reword: PullRequestReword,
        summary: PullRequestSummary,
        create_calls: Mutex<usize>,
    }

    impl MockLLMRepository {
        fn new(create_content: PullRequestContent) -> Self {
            Self {
                create_content,
                reword: PullRequestReword {
                    pr_title: "Reworded".to_string(),
                    description: Some("Reworded body".to_string()),
                },
                summary: PullRequestSummary {
                    summary: "Summary".to_string(),
                    filename: "summary".to_string(),
                },
                create_calls: Mutex::new(0),
            }
        }

        fn create_call_count(&self) -> usize {
            *self.create_calls.lock().expect("lock create_calls")
        }
    }

    impl LLMRepository for MockLLMRepository {
        fn verify_config(&self) -> Result<String, LLMError> {
            Err(LLMError::Other("test-only".to_string()))
        }

        fn generate_branch_name(
            &self,
            _title: Option<&str>,
            _exists_branches: Option<Vec<String>>,
        ) -> Result<String, LLMError> {
            Err(LLMError::Other("test-only".to_string()))
        }

        fn generate_pr_content(
            &self,
            _branch_name: &str,
            _commits: &[String],
        ) -> Result<domain::pr::entity::PrContent, LLMError> {
            Err(LLMError::Other("test-only".to_string()))
        }

        fn generate_commit_message(&self, _changes: &str) -> Result<String, LLMError> {
            Err(LLMError::Other("test-only".to_string()))
        }

        fn translate_to_english(&self, _text: &str) -> Result<String, LLMError> {
            Err(LLMError::Other("test-only".to_string()))
        }

        fn create_pr_content(
            &self,
            _commit_title: &str,
            _exists_branches: Option<Vec<String>>,
            _git_diff: Option<String>,
        ) -> Result<PullRequestContent, LLMError> {
            let mut calls = self.create_calls.lock().expect("lock create_calls");
            *calls += 1;
            Ok(self.create_content.clone())
        }

        fn reword_pr(
            &self,
            _pr_diff: &str,
            _current_title: Option<&str>,
        ) -> Result<PullRequestReword, LLMError> {
            Ok(self.reword.clone())
        }

        fn summarize_pr(
            &self,
            _pr_title: &str,
            _pr_diff: &str,
        ) -> Result<PullRequestSummary, LLMError> {
            Ok(self.summary.clone())
        }

        fn summarize_file_change(
            &self,
            _file_path: &str,
            _file_diff: &str,
        ) -> Result<String, LLMError> {
            Err(LLMError::Other("test-only".to_string()))
        }
    }

    fn build_service(
        repo_info: RepoInfo,
        github_repo: Arc<dyn GitHubRepository>,
        llm_repo: Arc<dyn LLMRepository>,
        default_branch: Option<String>,
    ) -> PullRequestServiceImpl {
        let git_repo = Arc::new(MockGitRepository::new(
            repo_info,
            "feature/test",
            default_branch,
        ));
        PullRequestServiceImpl::new(git_repo, github_repo, llm_repo)
    }

    fn build_repo_info(kind: CodePlatform) -> RepoInfo {
        RepoInfo {
            is_valid: true,
            kind: Some(kind),
            origin_url: Some("https://example.com/org/repo.git".to_string()),
            directory: Some("/tmp/repo/.git".to_string()),
            name: Some("org/repo".to_string()),
            owner: Some("org".to_string()),
        }
    }

    fn build_llm_repo() -> Arc<dyn LLMRepository> {
        Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }))
    }

    #[test]
    fn create_pull_request_uses_llm_when_title_missing() {
        let github_repo = Arc::new(MockGitHubRepository::new("123"));
        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "Auto title".to_string(),
            description: Some("Auto description".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo.clone(),
            llm_repo.clone(),
            None,
        );
        let pr_id = service.create_pull_request(Some("JIRA-1"), None, None, None).unwrap();

        assert_eq!(pr_id, "123");
        let (title, body, source_branch, target_branch) = github_repo.last_create_args();
        assert_eq!(title, "Auto title");
        assert_eq!(body, "Auto description\n\nJira: JIRA-1");
        assert_eq!(source_branch, "feature/test");
        assert_eq!(target_branch, "main");
        assert_eq!(llm_repo.create_call_count(), 1);
    }

    #[test]
    fn create_pull_request_uses_provided_title_and_description() {
        let github_repo = Arc::new(MockGitHubRepository::new("456"));
        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo.clone(),
            llm_repo.clone(),
            Some("develop".to_string()),
        );

        let pr_id = service
            .create_pull_request(
                None,
                Some("Custom title"),
                Some("Custom body"),
                Some("release"),
            )
            .unwrap();

        assert_eq!(pr_id, "456");
        let (title, body, _source_branch, target_branch) = github_repo.last_create_args();
        assert_eq!(title, "Custom title");
        assert_eq!(body, "Custom body");
        assert_eq!(target_branch, "release");
        assert_eq!(llm_repo.create_call_count(), 0);
    }

    #[test]
    fn get_pr_status_resolves_current_branch() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_current_branch_pr(Some("77".to_string()));
        github_repo.set_pull_request(
            PullRequestInfo {
                id: "77".to_string(),
                title: "PR title".to_string(),
                body: "PR body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/test".to_string(),
                target_branch: "main".to_string(),
            },
            ("open".to_string(), false, None),
        );

        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let status = service.get_pr_status(None).unwrap();
        assert_eq!(status.id, "77");
        assert_eq!(status.title, "PR title");
        assert_eq!(status.state, "open");
        assert!(!status.merged);
    }

    #[test]
    fn get_pr_status_returns_not_found_when_no_pr_for_current_branch() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_current_branch_pr(None);
        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let err = service.get_pr_status(None).unwrap_err();
        assert!(matches!(err, ServiceError::NotFound(_)));
    }

    #[test]
    fn add_comment_rejects_empty_input() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let err = service.add_comment("1", "").unwrap_err();
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }

    #[test]
    fn create_pull_request_rejects_non_github_repo() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::Codeup),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let err = service
            .create_pull_request(None, Some("title"), Some("body"), None)
            .unwrap_err();
        assert!(matches!(err, ServiceError::UnsupportedOperation(_)));
    }

    #[test]
    fn list_pull_requests_maps_status_fields() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_list_result(vec![
            PullRequestInfo {
                id: "1".to_string(),
                title: "First".to_string(),
                body: "body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/a".to_string(),
                target_branch: "main".to_string(),
            },
            PullRequestInfo {
                id: "2".to_string(),
                title: "Second".to_string(),
                body: "body".to_string(),
                status: PullRequestStatus {
                    state: "merged".to_string(),
                    merged: true,
                    merged_at: Some("2025-01-01".to_string()),
                },
                source_branch: "feature/b".to_string(),
                target_branch: "main".to_string(),
            },
        ]);

        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let prs = service.list_pull_requests(None, None).unwrap();
        assert_eq!(prs.len(), 2);
        assert_eq!(prs[0].id, "1");
        assert_eq!(prs[0].state, "open");
        assert!(!prs[0].merged);
        assert_eq!(prs[1].id, "2");
        assert_eq!(prs[1].state, "merged");
        assert!(prs[1].merged);
    }

    #[test]
    fn test_get_pr_status_uses_explicit_id() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_pull_request(
            PullRequestInfo {
                id: "99".to_string(),
                title: "Explicit".to_string(),
                body: "body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/explicit".to_string(),
                target_branch: "main".to_string(),
            },
            ("open".to_string(), false, None),
        );

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            build_llm_repo(),
            Some("main".to_string()),
        );

        let status = service.get_pr_status(Some("99")).unwrap();
        assert_eq!(status.id, "99");
        assert_eq!(status.title, "Explicit");
    }

    #[test]
    fn test_merge_pull_request_forwards_to_repo() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo.clone(),
            build_llm_repo(),
            Some("main".to_string()),
        );

        service.merge_pull_request("77", true).unwrap();
        let (pr_id, force) = github_repo.last_merge_args();
        assert_eq!(pr_id, "77");
        assert!(force);
    }

    #[test]
    fn test_merge_pull_request_maps_github_error() {
        let github_repo = Arc::new(FailingGitHubRepository);
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            build_llm_repo(),
            Some("main".to_string()),
        );

        let err = service.merge_pull_request("1", false).unwrap_err();
        assert!(matches!(err, ServiceError::Other(_)));
        assert!(err.to_string().contains("GitHub API error"));
    }

    #[test]
    fn test_close_pull_request_forwards_to_repo() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo.clone(),
            build_llm_repo(),
            Some("main".to_string()),
        );

        service.close_pull_request("22").unwrap();
        let pr_id = github_repo.last_close_args();
        assert_eq!(pr_id, "22");
    }

    #[test]
    fn test_update_pull_request_forwards_to_repo() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo.clone(),
            build_llm_repo(),
            Some("main".to_string()),
        );

        service.update_pull_request("33", Some("Title"), Some("Body")).unwrap();
        let (pr_id, title, body) = github_repo.last_update_args();
        assert_eq!(pr_id, "33");
        assert_eq!(title, Some("Title".to_string()));
        assert_eq!(body, Some("Body".to_string()));
    }

    #[test]
    fn test_add_comment_forwards_to_repo() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo.clone(),
            build_llm_repo(),
            Some("main".to_string()),
        );

        service.add_comment("44", "hello").unwrap();
        let (pr_id, comment) = github_repo.last_comment_args();
        assert_eq!(pr_id, "44");
        assert_eq!(comment, "hello");
    }

    #[test]
    fn test_approve_pull_request_forwards_to_repo() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo.clone(),
            build_llm_repo(),
            Some("main".to_string()),
        );

        service.approve_pull_request("55").unwrap();
        let pr_id = github_repo.last_approve_args();
        assert_eq!(pr_id, "55");
    }

    #[test]
    fn test_get_pr_diff_returns_content() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            build_llm_repo(),
            Some("main".to_string()),
        );

        let diff = service.get_pr_diff("66").unwrap();
        assert_eq!(diff, "diff");
    }

    #[test]
    fn test_get_pull_request_returns_info() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_pull_request(
            PullRequestInfo {
                id: "77".to_string(),
                title: "PR title".to_string(),
                body: "PR body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/test".to_string(),
                target_branch: "main".to_string(),
            },
            ("open".to_string(), false, None),
        );

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            build_llm_repo(),
            Some("main".to_string()),
        );

        let pr = service.get_pull_request("77").unwrap();
        assert_eq!(pr.id, "77");
        assert_eq!(pr.title, "PR title");
    }

    #[test]
    fn test_summarize_pull_request_returns_llm_result() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_pull_request(
            PullRequestInfo {
                id: "88".to_string(),
                title: "PR title".to_string(),
                body: "PR body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/test".to_string(),
                target_branch: "main".to_string(),
            },
            ("open".to_string(), false, None),
        );

        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let summary = service.summarize_pull_request(Some("88")).unwrap();
        assert_eq!(summary.summary, "Summary");
        assert_eq!(summary.filename, "summary");
    }

    #[test]
    fn test_reword_pull_request_returns_llm_result() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_pull_request(
            PullRequestInfo {
                id: "99".to_string(),
                title: "PR title".to_string(),
                body: "PR body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/test".to_string(),
                target_branch: "main".to_string(),
            },
            ("open".to_string(), false, None),
        );

        let llm_repo = Arc::new(MockLLMRepository::new(PullRequestContent {
            branch_name: "feature/test".to_string(),
            pr_title: "unused".to_string(),
            description: Some("unused".to_string()),
            scope: None,
            summary: None,
        }));

        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let reword = service.reword_pull_request(Some("99")).unwrap();
        assert_eq!(reword.pr_title, "Reworded");
        assert_eq!(reword.description, Some("Reworded body".to_string()));
    }

    #[test]
    fn test_list_pull_requests_maps_github_error() {
        // FailingGitHubRepository 的 list_pull_requests 返回 unimplemented
        // 但我们需要一个真正失败的实现
        struct FailingListGitHubRepository;

        impl GitHubRepository for FailingListGitHubRepository {
            fn create_pull_request(
                &self,
                _title: &str,
                _body: &str,
                _source_branch: &str,
                _target_branch: &str,
            ) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request(&self, _pr_id: &str) -> Result<PullRequestInfo, GitHubError> {
                unimplemented!("test-only")
            }

            fn merge_pull_request(&self, _pr_id: &str, _force: bool) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_user_info(&self) -> Result<domain::github::entity::GitHubUser, GitHubError> {
                unimplemented!("test-only")
            }

            fn close_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn list_pull_requests(
                &self,
                _state: Option<&str>,
                _limit: Option<usize>,
            ) -> Result<Vec<PullRequestInfo>, GitHubError> {
                Err(GitHubError::Other("list failed".to_string()))
            }

            fn update_pull_request(
                &self,
                _pr_id: &str,
                _title: Option<&str>,
                _body: Option<&str>,
            ) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn add_comment(&self, _pr_id: &str, _comment: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn approve_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pr_diff(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_info(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_url(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_title(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_body(&self, _pr_id: &str) -> Result<Option<String>, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_status(
                &self,
                _pr_id: &str,
            ) -> Result<(String, bool, Option<String>), GitHubError> {
                unimplemented!("test-only")
            }

            fn update_pr_base(&self, _pr_id: &str, _new_base: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_current_branch_pull_request(
                &self,
                _current_branch: &str,
            ) -> Result<Option<String>, GitHubError> {
                unimplemented!("test-only")
            }
        }

        let github_repo = Arc::new(FailingListGitHubRepository);
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            build_llm_repo(),
            Some("main".to_string()),
        );

        let err = service.list_pull_requests(None, None).unwrap_err();
        assert!(matches!(err, ServiceError::Other(_)));
        assert!(err.to_string().contains("GitHub API error"));
    }

    #[test]
    fn test_get_pull_request_maps_github_error() {
        struct FailingGetGitHubRepository;

        impl GitHubRepository for FailingGetGitHubRepository {
            fn create_pull_request(
                &self,
                _title: &str,
                _body: &str,
                _source_branch: &str,
                _target_branch: &str,
            ) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request(&self, _pr_id: &str) -> Result<PullRequestInfo, GitHubError> {
                Err(GitHubError::NotFound("PR not found".to_string()))
            }

            fn merge_pull_request(&self, _pr_id: &str, _force: bool) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_user_info(&self) -> Result<domain::github::entity::GitHubUser, GitHubError> {
                unimplemented!("test-only")
            }

            fn close_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn list_pull_requests(
                &self,
                _state: Option<&str>,
                _limit: Option<usize>,
            ) -> Result<Vec<PullRequestInfo>, GitHubError> {
                unimplemented!("test-only")
            }

            fn update_pull_request(
                &self,
                _pr_id: &str,
                _title: Option<&str>,
                _body: Option<&str>,
            ) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn add_comment(&self, _pr_id: &str, _comment: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn approve_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pr_diff(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_info(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_url(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_title(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_body(&self, _pr_id: &str) -> Result<Option<String>, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_status(
                &self,
                _pr_id: &str,
            ) -> Result<(String, bool, Option<String>), GitHubError> {
                unimplemented!("test-only")
            }

            fn update_pr_base(&self, _pr_id: &str, _new_base: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_current_branch_pull_request(
                &self,
                _current_branch: &str,
            ) -> Result<Option<String>, GitHubError> {
                unimplemented!("test-only")
            }
        }

        let github_repo = Arc::new(FailingGetGitHubRepository);
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            build_llm_repo(),
            Some("main".to_string()),
        );

        let err = service.get_pull_request("999").unwrap_err();
        assert!(matches!(err, ServiceError::Other(_)));
        assert!(err.to_string().contains("GitHub API error"));
    }

    #[test]
    fn test_get_pr_status_maps_github_error() {
        struct FailingStatusGitHubRepository;

        impl GitHubRepository for FailingStatusGitHubRepository {
            fn create_pull_request(
                &self,
                _title: &str,
                _body: &str,
                _source_branch: &str,
                _target_branch: &str,
            ) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request(&self, _pr_id: &str) -> Result<PullRequestInfo, GitHubError> {
                unimplemented!("test-only")
            }

            fn merge_pull_request(&self, _pr_id: &str, _force: bool) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_user_info(&self) -> Result<domain::github::entity::GitHubUser, GitHubError> {
                unimplemented!("test-only")
            }

            fn close_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn list_pull_requests(
                &self,
                _state: Option<&str>,
                _limit: Option<usize>,
            ) -> Result<Vec<PullRequestInfo>, GitHubError> {
                unimplemented!("test-only")
            }

            fn update_pull_request(
                &self,
                _pr_id: &str,
                _title: Option<&str>,
                _body: Option<&str>,
            ) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn add_comment(&self, _pr_id: &str, _comment: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn approve_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pr_diff(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_info(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_url(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_title(&self, _pr_id: &str) -> Result<String, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_body(&self, _pr_id: &str) -> Result<Option<String>, GitHubError> {
                unimplemented!("test-only")
            }

            fn get_pull_request_status(
                &self,
                _pr_id: &str,
            ) -> Result<(String, bool, Option<String>), GitHubError> {
                Err(GitHubError::Other("status failed".to_string()))
            }

            fn update_pr_base(&self, _pr_id: &str, _new_base: &str) -> Result<(), GitHubError> {
                unimplemented!("test-only")
            }

            fn get_current_branch_pull_request(
                &self,
                _current_branch: &str,
            ) -> Result<Option<String>, GitHubError> {
                unimplemented!("test-only")
            }
        }

        let github_repo = Arc::new(FailingStatusGitHubRepository);
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            build_llm_repo(),
            Some("main".to_string()),
        );

        let err = service.get_pr_status(Some("123")).unwrap_err();
        assert!(matches!(err, ServiceError::Other(_)));
        assert!(err.to_string().contains("GitHub API error"));
    }

    #[test]
    fn test_resolve_pr_id_returns_not_found_when_no_current_branch_pr() {
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_current_branch_pr(None);

        struct FailingCurrentBranchGitRepository {
            repo_info: RepoInfo,
        }

        impl GitRepository for FailingCurrentBranchGitRepository {
            fn get_repo_info(&self) -> RepoInfo {
                self.repo_info.clone()
            }

            fn get_ignore_directory_patterns(&self) -> Vec<String> {
                Vec::new()
            }

            fn get_working_tree_diff(
                &self,
                _base_branch: &str,
            ) -> Result<Option<String>, GitError> {
                unimplemented!("test-only")
            }

            fn create_branch(&self, _name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn delete_local_branch(&self, _name: &str, _force: bool) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn delete_remote_branch(&self, _name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn rename_branch(
                &self,
                _old_name: Option<&str>,
                _new_name: &str,
            ) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn list_branches(
                &self,
                _remove_prefix: bool,
                _all: bool,
            ) -> Result<Vec<domain::BranchInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn checkout_branch(&self, _name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn get_current_branch(&self) -> Result<String, GitError> {
                Ok("feature/test".to_string())
            }

            fn has_branch(&self, _name: &str) -> Result<(bool, bool), GitError> {
                unimplemented!("test-only")
            }

            fn get_default_branch(&self) -> Result<String, GitError> {
                Ok("main".to_string())
            }

            fn infer_target_branch(
                &self,
                _current_branch: &str,
            ) -> Result<Option<String>, GitError> {
                unimplemented!("test-only")
            }

            fn get_commit_info(&self, _ref_or_sha: &str) -> Result<domain::CommitInfo, GitError> {
                unimplemented!("test-only")
            }

            fn get_commit_changed_files(
                &self,
                _ref_or_sha: &str,
            ) -> Result<Vec<domain::CommitFileChange>, GitError> {
                unimplemented!("test-only")
            }

            fn get_commit_diff(&self, _ref_or_sha: &str) -> Result<Option<String>, GitError> {
                unimplemented!("test-only")
            }

            fn get_working_tree_status(&self) -> Result<domain::WorkingTreeStatus, GitError> {
                unimplemented!("test-only")
            }

            fn commit(&self, _message: &str, _all: bool) -> Result<String, GitError> {
                unimplemented!("test-only")
            }

            fn merge_branch(
                &self,
                _source_branch: &str,
                _strategy: domain::MergeStrategy,
            ) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn has_merge_conflicts(&self) -> Result<bool, GitError> {
                unimplemented!("test-only")
            }

            fn is_branch_merged(
                &self,
                _branch: &str,
                _base_branch: &str,
            ) -> Result<bool, GitError> {
                unimplemented!("test-only")
            }

            fn merge_base(&self, _branch1: &str, _branch2: &str) -> Result<String, GitError> {
                unimplemented!("test-only")
            }

            fn commits_to_merge(
                &self,
                _source_branch: &str,
                _target_branch: &str,
            ) -> Result<Vec<String>, GitError> {
                unimplemented!("test-only")
            }

            fn rebase_onto(&self, _target_branch: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn rebase_onto_with_upstream(
                &self,
                _newbase: &str,
                _upstream: &str,
                _branch: &str,
            ) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn push(&self, _branch_name: &str, _set_upstream: bool) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn pull(&self, _branch_name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn is_commit_in_remote_branch(
                &self,
                _branch: &str,
                _commit_sha: &str,
            ) -> Result<bool, GitError> {
                unimplemented!("test-only")
            }

            fn stash_push(&self, _message: Option<&str>) -> Result<usize, GitError> {
                unimplemented!("test-only")
            }

            fn stash_pop(&self, _index: usize) -> Result<StashPopResult, GitError> {
                unimplemented!("test-only")
            }

            fn stash_apply(&self, _index: usize) -> Result<StashApplyResult, GitError> {
                unimplemented!("test-only")
            }

            fn stash_list(&self) -> Result<Vec<StashEntry>, GitError> {
                unimplemented!("test-only")
            }

            fn stash_drop(&self, _index: usize) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn create_tag(
                &self,
                _name: &str,
                _target: Option<&str>,
                _message: Option<&str>,
                _scope: domain::TagCreateScope,
                _force: bool,
            ) -> Result<domain::TagCreateInfo, GitError> {
                unimplemented!("test-only")
            }

            fn delete_tag(
                &self,
                _name: &str,
                _scope: domain::TagDeleteScope,
                _force: bool,
            ) -> Result<domain::TagDeleteInfo, GitError> {
                unimplemented!("test-only")
            }

            fn delete_tags_by_pattern(
                &self,
                _pattern: &str,
                _scope: domain::TagDeleteScope,
                _force: bool,
            ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn list_tags(&self, _include_remote: bool) -> Result<Vec<String>, GitError> {
                unimplemented!("test-only")
            }

            fn has_tag(&self, _name: &str) -> Result<(bool, bool), GitError> {
                unimplemented!("test-only")
            }

            fn preview_delete(
                &self,
                _name: Option<&str>,
                _pattern: Option<&str>,
                _scope: domain::TagDeleteScope,
            ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn get_file_blame(
                &self,
                _file_path: &str,
                _revision: Option<&str>,
            ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn get_file_blame_range(
                &self,
                _file_path: &str,
                _start_line: usize,
                _end_line: usize,
                _revision: Option<&str>,
            ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
                unimplemented!("test-only")
            }
        }

        let git_repo = Arc::new(FailingCurrentBranchGitRepository {
            repo_info: build_repo_info(CodePlatform::GitHub),
        });
        let service = PullRequestServiceImpl::new(git_repo, github_repo, build_llm_repo());

        let err = service.get_pr_status(None).unwrap_err();
        assert!(matches!(err, ServiceError::NotFound(_)));
        assert!(err.to_string().contains("No PR found for current branch"));
    }

    #[test]
    fn test_create_pull_request_propagates_git_error() {
        struct FailingGitBranchRepository {
            repo_info: RepoInfo,
        }

        impl GitRepository for FailingGitBranchRepository {
            fn get_repo_info(&self) -> RepoInfo {
                self.repo_info.clone()
            }

            fn get_ignore_directory_patterns(&self) -> Vec<String> {
                Vec::new()
            }

            fn get_working_tree_diff(
                &self,
                _base_branch: &str,
            ) -> Result<Option<String>, GitError> {
                unimplemented!("test-only")
            }

            fn create_branch(&self, _name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn delete_local_branch(&self, _name: &str, _force: bool) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn delete_remote_branch(&self, _name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn rename_branch(
                &self,
                _old_name: Option<&str>,
                _new_name: &str,
            ) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn list_branches(
                &self,
                _remove_prefix: bool,
                _all: bool,
            ) -> Result<Vec<domain::BranchInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn checkout_branch(&self, _name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn get_current_branch(&self) -> Result<String, GitError> {
                Err(GitError::Other("Failed to get current branch".to_string()))
            }

            fn has_branch(&self, _name: &str) -> Result<(bool, bool), GitError> {
                unimplemented!("test-only")
            }

            fn get_default_branch(&self) -> Result<String, GitError> {
                unimplemented!("test-only")
            }

            fn infer_target_branch(
                &self,
                _current_branch: &str,
            ) -> Result<Option<String>, GitError> {
                unimplemented!("test-only")
            }

            fn get_commit_info(&self, _ref_or_sha: &str) -> Result<domain::CommitInfo, GitError> {
                unimplemented!("test-only")
            }

            fn get_commit_changed_files(
                &self,
                _ref_or_sha: &str,
            ) -> Result<Vec<domain::CommitFileChange>, GitError> {
                unimplemented!("test-only")
            }

            fn get_commit_diff(&self, _ref_or_sha: &str) -> Result<Option<String>, GitError> {
                unimplemented!("test-only")
            }

            fn get_working_tree_status(&self) -> Result<domain::WorkingTreeStatus, GitError> {
                unimplemented!("test-only")
            }

            fn commit(&self, _message: &str, _all: bool) -> Result<String, GitError> {
                unimplemented!("test-only")
            }

            fn merge_branch(
                &self,
                _source_branch: &str,
                _strategy: domain::MergeStrategy,
            ) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn has_merge_conflicts(&self) -> Result<bool, GitError> {
                unimplemented!("test-only")
            }

            fn is_branch_merged(
                &self,
                _branch: &str,
                _base_branch: &str,
            ) -> Result<bool, GitError> {
                unimplemented!("test-only")
            }

            fn merge_base(&self, _branch1: &str, _branch2: &str) -> Result<String, GitError> {
                unimplemented!("test-only")
            }

            fn commits_to_merge(
                &self,
                _source_branch: &str,
                _target_branch: &str,
            ) -> Result<Vec<String>, GitError> {
                unimplemented!("test-only")
            }

            fn rebase_onto(&self, _target_branch: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn rebase_onto_with_upstream(
                &self,
                _newbase: &str,
                _upstream: &str,
                _branch: &str,
            ) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn push(&self, _branch_name: &str, _set_upstream: bool) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn pull(&self, _branch_name: &str) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn is_commit_in_remote_branch(
                &self,
                _branch: &str,
                _commit_sha: &str,
            ) -> Result<bool, GitError> {
                unimplemented!("test-only")
            }

            fn stash_push(&self, _message: Option<&str>) -> Result<usize, GitError> {
                unimplemented!("test-only")
            }

            fn stash_pop(&self, _index: usize) -> Result<StashPopResult, GitError> {
                unimplemented!("test-only")
            }

            fn stash_apply(&self, _index: usize) -> Result<StashApplyResult, GitError> {
                unimplemented!("test-only")
            }

            fn stash_list(&self) -> Result<Vec<StashEntry>, GitError> {
                unimplemented!("test-only")
            }

            fn stash_drop(&self, _index: usize) -> Result<(), GitError> {
                unimplemented!("test-only")
            }

            fn create_tag(
                &self,
                _name: &str,
                _target: Option<&str>,
                _message: Option<&str>,
                _scope: domain::TagCreateScope,
                _force: bool,
            ) -> Result<domain::TagCreateInfo, GitError> {
                unimplemented!("test-only")
            }

            fn delete_tag(
                &self,
                _name: &str,
                _scope: domain::TagDeleteScope,
                _force: bool,
            ) -> Result<domain::TagDeleteInfo, GitError> {
                unimplemented!("test-only")
            }

            fn delete_tags_by_pattern(
                &self,
                _pattern: &str,
                _scope: domain::TagDeleteScope,
                _force: bool,
            ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn list_tags(&self, _include_remote: bool) -> Result<Vec<String>, GitError> {
                unimplemented!("test-only")
            }

            fn has_tag(&self, _name: &str) -> Result<(bool, bool), GitError> {
                unimplemented!("test-only")
            }

            fn preview_delete(
                &self,
                _name: Option<&str>,
                _pattern: Option<&str>,
                _scope: domain::TagDeleteScope,
            ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn get_file_blame(
                &self,
                _file_path: &str,
                _revision: Option<&str>,
            ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
                unimplemented!("test-only")
            }

            fn get_file_blame_range(
                &self,
                _file_path: &str,
                _start_line: usize,
                _end_line: usize,
                _revision: Option<&str>,
            ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
                unimplemented!("test-only")
            }
        }

        let git_repo = Arc::new(FailingGitBranchRepository {
            repo_info: build_repo_info(CodePlatform::GitHub),
        });
        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let service = PullRequestServiceImpl::new(git_repo, github_repo, build_llm_repo());

        let err = service
            .create_pull_request(None, Some("title"), Some("body"), None)
            .unwrap_err();
        assert!(matches!(err, ServiceError::Git(_)));
    }

    #[test]
    fn test_create_pull_request_propagates_llm_error() {
        struct FailingLLMRepository;

        impl LLMRepository for FailingLLMRepository {
            fn verify_config(&self) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_branch_name(
                &self,
                _title: Option<&str>,
                _exists_branches: Option<Vec<String>>,
            ) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_pr_content(
                &self,
                _branch_name: &str,
                _commits: &[String],
            ) -> Result<domain::pr::entity::PrContent, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_commit_message(&self, _changes: &str) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn translate_to_english(&self, _text: &str) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn create_pr_content(
                &self,
                _commit_title: &str,
                _exists_branches: Option<Vec<String>>,
                _git_diff: Option<String>,
            ) -> Result<PullRequestContent, LLMError> {
                Err(LLMError::Other("LLM failed".to_string()))
            }

            fn reword_pr(
                &self,
                _pr_diff: &str,
                _current_title: Option<&str>,
            ) -> Result<PullRequestReword, LLMError> {
                unimplemented!("test-only")
            }

            fn summarize_pr(
                &self,
                _pr_title: &str,
                _pr_diff: &str,
            ) -> Result<PullRequestSummary, LLMError> {
                unimplemented!("test-only")
            }

            fn summarize_file_change(
                &self,
                _file_path: &str,
                _file_diff: &str,
            ) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }
        }

        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        let llm_repo = Arc::new(FailingLLMRepository);
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let err = service.create_pull_request(None, None, None, None).unwrap_err();
        assert!(matches!(err, ServiceError::LLM(_)));
    }

    #[test]
    fn test_summarize_pull_request_propagates_llm_error() {
        struct FailingSummarizeLLMRepository;

        impl LLMRepository for FailingSummarizeLLMRepository {
            fn verify_config(&self) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_branch_name(
                &self,
                _title: Option<&str>,
                _exists_branches: Option<Vec<String>>,
            ) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_pr_content(
                &self,
                _branch_name: &str,
                _commits: &[String],
            ) -> Result<domain::pr::entity::PrContent, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_commit_message(&self, _changes: &str) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn translate_to_english(&self, _text: &str) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn create_pr_content(
                &self,
                _commit_title: &str,
                _exists_branches: Option<Vec<String>>,
                _git_diff: Option<String>,
            ) -> Result<PullRequestContent, LLMError> {
                unimplemented!("test-only")
            }

            fn reword_pr(
                &self,
                _pr_diff: &str,
                _current_title: Option<&str>,
            ) -> Result<PullRequestReword, LLMError> {
                unimplemented!("test-only")
            }

            fn summarize_pr(
                &self,
                _pr_title: &str,
                _pr_diff: &str,
            ) -> Result<PullRequestSummary, LLMError> {
                Err(LLMError::Other("summarize failed".to_string()))
            }

            fn summarize_file_change(
                &self,
                _file_path: &str,
                _file_diff: &str,
            ) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }
        }

        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_pull_request(
            PullRequestInfo {
                id: "88".to_string(),
                title: "PR title".to_string(),
                body: "PR body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/test".to_string(),
                target_branch: "main".to_string(),
            },
            ("open".to_string(), false, None),
        );

        let llm_repo = Arc::new(FailingSummarizeLLMRepository);
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let err = service.summarize_pull_request(Some("88")).unwrap_err();
        assert!(matches!(err, ServiceError::LLM(_)));
    }

    #[test]
    fn test_reword_pull_request_propagates_llm_error() {
        struct FailingRewordLLMRepository;

        impl LLMRepository for FailingRewordLLMRepository {
            fn verify_config(&self) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_branch_name(
                &self,
                _title: Option<&str>,
                _exists_branches: Option<Vec<String>>,
            ) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_pr_content(
                &self,
                _branch_name: &str,
                _commits: &[String],
            ) -> Result<domain::pr::entity::PrContent, LLMError> {
                unimplemented!("test-only")
            }

            fn generate_commit_message(&self, _changes: &str) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn translate_to_english(&self, _text: &str) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }

            fn create_pr_content(
                &self,
                _commit_title: &str,
                _exists_branches: Option<Vec<String>>,
                _git_diff: Option<String>,
            ) -> Result<PullRequestContent, LLMError> {
                unimplemented!("test-only")
            }

            fn reword_pr(
                &self,
                _pr_diff: &str,
                _current_title: Option<&str>,
            ) -> Result<PullRequestReword, LLMError> {
                Err(LLMError::Other("reword failed".to_string()))
            }

            fn summarize_pr(
                &self,
                _pr_title: &str,
                _pr_diff: &str,
            ) -> Result<PullRequestSummary, LLMError> {
                unimplemented!("test-only")
            }

            fn summarize_file_change(
                &self,
                _file_path: &str,
                _file_diff: &str,
            ) -> Result<String, LLMError> {
                unimplemented!("test-only")
            }
        }

        let github_repo = Arc::new(MockGitHubRepository::new("001"));
        github_repo.set_pull_request(
            PullRequestInfo {
                id: "99".to_string(),
                title: "PR title".to_string(),
                body: "PR body".to_string(),
                status: PullRequestStatus {
                    state: "open".to_string(),
                    merged: false,
                    merged_at: None,
                },
                source_branch: "feature/test".to_string(),
                target_branch: "main".to_string(),
            },
            ("open".to_string(), false, None),
        );

        let llm_repo = Arc::new(FailingRewordLLMRepository);
        let service = build_service(
            build_repo_info(CodePlatform::GitHub),
            github_repo,
            llm_repo,
            Some("main".to_string()),
        );

        let err = service.reword_pull_request(Some("99")).unwrap_err();
        assert!(matches!(err, ServiceError::LLM(_)));
    }
}
