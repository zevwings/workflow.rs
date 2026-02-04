//! Services Registry
//!
//! 定义 services crate 的服务注册和依赖注入。

use std::sync::Arc;

use registry::{bind, try_bind, Container, Scope};

/// Services 模块标记类型
///
/// 用于标识 services 层服务的注册状态
#[derive(Debug, Clone, Copy)]
pub struct ServicesModule;

/// 构建 Services 模块
///
/// 注册所有 services 层的服务，包括：
/// - AliasService
/// - PullRequestService
/// - CompletionService
///
/// # 错误
///
/// 如果服务注册失败，返回 `registry::Error`。
pub fn register_services() -> registry::Result<ServicesModule> {
    // AliasService - 依赖 GlobalConfigRepository
    try_bind!(dyn domain::AliasService, |c: &Container| {
        let config_repo = c.get::<dyn domain::GlobalConfigRepository>()?;
        Ok(Arc::new(crate::AliasServiceImpl::new(config_repo)))
    })
    .in_scope(Scope::Singleton)?;

    // PullRequestService - 依赖 GitRepository、GitHubRepository 和 LLMRepository
    try_bind!(dyn domain::PullRequestService, |c: &Container| {
        let git_repo = c.get::<dyn domain::GitRepository>()?;
        let github_repo = c.get::<dyn domain::GitHubRepository>()?;
        let llm_repo = c.get::<dyn domain::LLMRepository>()?;

        Ok(Arc::new(crate::PullRequestServiceImpl::new(
            git_repo,
            github_repo,
            llm_repo,
        )))
    })
    .in_scope(Scope::Singleton)?;

    // CompletionService - 无外部依赖
    bind!(dyn domain::CompletionService, |_c: &Container| {
        Arc::new(crate::CompletionServiceImpl::new())
    })
    .in_scope(Scope::Singleton)?;

    Ok(ServicesModule)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use domain::{
        alias::AliasService, completion::CompletionService, errors::ServiceError,
        pr::PullRequestService, GlobalConfigRepository,
    };
    use domain::{
        config::global::config::GlobalConfig, git::error::GitError, git::repository::GitRepository,
        github::error::GitHubError, github::repository::GitHubRepository, llm::error::LLMError,
        llm::repository::LLMRepository, CodePlatform, RepoInfo,
    };

    struct MockGlobalConfigRepository {
        config: GlobalConfig,
    }

    impl GlobalConfigRepository for MockGlobalConfigRepository {
        fn load(&self) -> Result<GlobalConfig, ServiceError> {
            Ok(self.config.clone())
        }

        fn save(&self, _settings: &GlobalConfig) -> Result<(), ServiceError> {
            Ok(())
        }

        fn check_permissions(&self) -> Option<String> {
            None
        }
    }

    struct MockGitRepository;

    impl GitRepository for MockGitRepository {
        fn get_repo_info(&self) -> RepoInfo {
            RepoInfo {
                is_valid: true,
                kind: Some(CodePlatform::GitHub),
                origin_url: Some("https://github.com/test/repo.git".to_string()),
                directory: Some("/tmp/repo/.git".to_string()),
                name: Some("test/repo".to_string()),
                owner: Some("test".to_string()),
            }
        }

        fn get_ignore_directory_patterns(&self) -> Vec<String> {
            Vec::new()
        }

        fn get_working_tree_diff(&self, _base_branch: &str) -> Result<Option<String>, GitError> {
            Ok(None)
        }

        fn create_branch(&self, _name: &str) -> Result<(), GitError> {
            Ok(())
        }

        fn delete_local_branch(&self, _name: &str, _force: bool) -> Result<(), GitError> {
            Ok(())
        }

        fn delete_remote_branch(&self, _name: &str) -> Result<(), GitError> {
            Ok(())
        }

        fn rename_branch(&self, _old_name: Option<&str>, _new_name: &str) -> Result<(), GitError> {
            Ok(())
        }

        fn list_branches(
            &self,
            _remove_prefix: bool,
            _all: bool,
        ) -> Result<Vec<domain::BranchInfo>, GitError> {
            Ok(Vec::new())
        }

        fn checkout_branch(&self, _name: &str) -> Result<(), GitError> {
            Ok(())
        }

        fn get_current_branch(&self) -> Result<String, GitError> {
            Ok("main".to_string())
        }

        fn has_branch(&self, _name: &str) -> Result<(bool, bool), GitError> {
            Ok((false, false))
        }

        fn get_default_branch(&self) -> Result<String, GitError> {
            Ok("main".to_string())
        }

        fn infer_target_branch(&self, _current_branch: &str) -> Result<Option<String>, GitError> {
            Ok(None)
        }

        fn get_commit_info(&self, _ref_or_sha: &str) -> Result<domain::CommitInfo, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn get_working_tree_status(&self) -> Result<domain::WorkingTreeStatus, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn commit(&self, _message: &str, _all: bool) -> Result<String, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn merge_branch(
            &self,
            _source_branch: &str,
            _strategy: domain::MergeStrategy,
        ) -> Result<(), GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn has_merge_conflicts(&self) -> Result<bool, GitError> {
            Ok(false)
        }

        fn is_branch_merged(&self, _branch: &str, _base_branch: &str) -> Result<bool, GitError> {
            Ok(false)
        }

        fn merge_base(&self, _branch1: &str, _branch2: &str) -> Result<String, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn rebase_onto(&self, _target_branch: &str) -> Result<(), GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn rebase_onto_with_upstream(
            &self,
            _newbase: &str,
            _upstream: &str,
            _branch: &str,
        ) -> Result<(), GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn push(&self, _branch_name: &str, _set_upstream: bool) -> Result<(), GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn pull(&self, _branch_name: &str) -> Result<(), GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn is_commit_in_remote_branch(
            &self,
            _branch: &str,
            _commit_sha: &str,
        ) -> Result<bool, GitError> {
            Ok(false)
        }

        fn stash_push(&self, _message: Option<&str>) -> Result<usize, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn stash_pop(&self, _index: usize) -> Result<domain::git::StashPopResult, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn stash_apply(&self, _index: usize) -> Result<domain::git::StashApplyResult, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn stash_list(&self) -> Result<Vec<domain::git::StashEntry>, GitError> {
            Ok(Vec::new())
        }

        fn stash_drop(&self, _index: usize) -> Result<(), GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn create_tag(
            &self,
            _name: &str,
            _target: Option<&str>,
            _message: Option<&str>,
            _scope: domain::TagCreateScope,
            _force: bool,
        ) -> Result<domain::TagCreateInfo, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn delete_tag(
            &self,
            _name: &str,
            _scope: domain::TagDeleteScope,
            _force: bool,
        ) -> Result<domain::TagDeleteInfo, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn delete_tags_by_pattern(
            &self,
            _pattern: &str,
            _scope: domain::TagDeleteScope,
            _force: bool,
        ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn list_tags(&self, _include_remote: bool) -> Result<Vec<String>, GitError> {
            Ok(Vec::new())
        }

        fn has_tag(&self, _name: &str) -> Result<(bool, bool), GitError> {
            Ok((false, false))
        }

        fn preview_delete(
            &self,
            _name: Option<&str>,
            _pattern: Option<&str>,
            _scope: domain::TagDeleteScope,
        ) -> Result<Vec<domain::TagDeleteInfo>, GitError> {
            Ok(Vec::new())
        }

        fn get_file_blame(
            &self,
            _file_path: &str,
            _revision: Option<&str>,
        ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }

        fn get_file_blame_range(
            &self,
            _file_path: &str,
            _start_line: usize,
            _end_line: usize,
            _revision: Option<&str>,
        ) -> Result<Vec<domain::BlameLineInfo>, GitError> {
            Err(GitError::Other("test-only".to_string()))
        }
    }

    struct MockGitHubRepository;

    impl GitHubRepository for MockGitHubRepository {
        fn create_pull_request(
            &self,
            _title: &str,
            _body: &str,
            _source_branch: &str,
            _target_branch: &str,
        ) -> Result<String, GitHubError> {
            Ok("123".to_string())
        }

        fn get_pull_request(
            &self,
            _pr_id: &str,
        ) -> Result<domain::pr::entity::PullRequestInfo, GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn merge_pull_request(&self, _pr_id: &str, _force: bool) -> Result<(), GitHubError> {
            Ok(())
        }

        fn get_user_info(&self) -> Result<domain::github::entity::GitHubUser, GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn close_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
            Ok(())
        }

        fn list_pull_requests(
            &self,
            _state: Option<&str>,
            _limit: Option<usize>,
        ) -> Result<Vec<domain::pr::entity::PullRequestInfo>, GitHubError> {
            Ok(Vec::new())
        }

        fn update_pull_request(
            &self,
            _pr_id: &str,
            _title: Option<&str>,
            _body: Option<&str>,
        ) -> Result<(), GitHubError> {
            Ok(())
        }

        fn add_comment(&self, _pr_id: &str, _comment: &str) -> Result<(), GitHubError> {
            Ok(())
        }

        fn approve_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
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
            _pr_id: &str,
        ) -> Result<(String, bool, Option<String>), GitHubError> {
            Err(GitHubError::Other("test-only".to_string()))
        }

        fn update_pr_base(&self, _pr_id: &str, _new_base: &str) -> Result<(), GitHubError> {
            Ok(())
        }

        fn get_current_branch_pull_request(
            &self,
            _current_branch: &str,
        ) -> Result<Option<String>, GitHubError> {
            Ok(None)
        }
    }

    struct MockLLMRepository;

    impl LLMRepository for MockLLMRepository {
        fn verify_config(&self) -> Result<String, LLMError> {
            Ok("ok".to_string())
        }

        fn generate_branch_name(
            &self,
            _title: Option<&str>,
            _exists_branches: Option<Vec<String>>,
        ) -> Result<String, LLMError> {
            Ok("feature/test".to_string())
        }

        fn generate_pr_content(
            &self,
            _branch_name: &str,
            _commits: &[String],
        ) -> Result<domain::pr::entity::PrContent, LLMError> {
            Err(LLMError::Other("test-only".to_string()))
        }

        fn generate_commit_message(&self, _changes: &str) -> Result<String, LLMError> {
            Ok("test commit".to_string())
        }

        fn translate_to_english(&self, _text: &str) -> Result<String, LLMError> {
            Ok(_text.to_string())
        }

        fn create_pr_content(
            &self,
            _commit_title: &str,
            _exists_branches: Option<Vec<String>>,
            _git_diff: Option<String>,
        ) -> Result<domain::llm::entity::PullRequestContent, LLMError> {
            Ok(domain::llm::entity::PullRequestContent {
                branch_name: "feature/test".to_string(),
                pr_title: "Test PR".to_string(),
                description: Some("Test description".to_string()),
                scope: None,
                summary: None,
            })
        }

        fn reword_pr(
            &self,
            _pr_diff: &str,
            _current_title: Option<&str>,
        ) -> Result<domain::llm::entity::PullRequestReword, LLMError> {
            Ok(domain::llm::entity::PullRequestReword {
                pr_title: "Reworded".to_string(),
                description: Some("Reworded body".to_string()),
            })
        }

        fn summarize_pr(
            &self,
            _pr_title: &str,
            _pr_diff: &str,
        ) -> Result<domain::llm::entity::PullRequestSummary, LLMError> {
            Ok(domain::llm::entity::PullRequestSummary {
                summary: "Summary".to_string(),
                filename: "summary.md".to_string(),
            })
        }

        fn summarize_file_change(
            &self,
            _file_path: &str,
            _file_diff: &str,
        ) -> Result<String, LLMError> {
            Ok("file summary".to_string())
        }
    }

    #[test]
    fn test_register_services_succeeds() {
        let container = registry::Container::global();
        container.unbind_all();

        // 先注册依赖
        container
            .bind::<dyn GlobalConfigRepository>(Arc::new(MockGlobalConfigRepository {
                config: GlobalConfig::default(),
            }) as Arc<dyn GlobalConfigRepository>)
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        container
            .bind::<dyn GitRepository>(Arc::new(MockGitRepository) as Arc<dyn GitRepository>)
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        container
            .bind::<dyn GitHubRepository>(
                Arc::new(MockGitHubRepository) as Arc<dyn GitHubRepository>
            )
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        container
            .bind::<dyn LLMRepository>(Arc::new(MockLLMRepository) as Arc<dyn LLMRepository>)
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        // 注册 services
        let result = register_services();
        assert!(result.is_ok(), "register_services should succeed");
    }

    #[test]
    fn test_register_services_resolves_dependencies() {
        let container = registry::Container::global();
        container.unbind_all();

        // 先注册依赖
        container
            .bind::<dyn GlobalConfigRepository>(Arc::new(MockGlobalConfigRepository {
                config: GlobalConfig::default(),
            }) as Arc<dyn GlobalConfigRepository>)
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        container
            .bind::<dyn GitRepository>(Arc::new(MockGitRepository) as Arc<dyn GitRepository>)
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        container
            .bind::<dyn GitHubRepository>(
                Arc::new(MockGitHubRepository) as Arc<dyn GitHubRepository>
            )
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        container
            .bind::<dyn LLMRepository>(Arc::new(MockLLMRepository) as Arc<dyn LLMRepository>)
            .in_scope(registry::Scope::Singleton)
            .unwrap();

        // 注册 services
        register_services().unwrap();

        // 验证服务可以正确解析
        let alias_service: Arc<dyn AliasService> = container.get().unwrap();
        assert!(alias_service.list().is_ok());

        let completion_service: Arc<dyn CompletionService> = container.get().unwrap();
        assert!(completion_service.check_status().is_ok());

        let pr_service: Arc<dyn PullRequestService> = container.get().unwrap();
        assert!(pr_service.list_pull_requests(None, None).is_ok());
    }

    #[test]
    fn test_register_services_fails_when_dependency_missing() {
        let container = registry::Container::global();
        container.unbind_all();

        // 不注册 GlobalConfigRepository，直接注册 services
        // 注意：绑定是延迟的，所以注册本身不会失败
        let result = register_services();
        assert!(
            result.is_ok(),
            "register_services should succeed even without dependencies"
        );

        // 但获取服务时会失败
        let err: Result<Arc<dyn AliasService>, _> = container.get();
        assert!(
            err.is_err(),
            "getting service should fail when dependency is missing"
        );
    }
}
