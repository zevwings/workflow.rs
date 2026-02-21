//! Service Mock 实现
//!
//! 提供 BranchService、CommitMessageService、PullRequestService 的 Mock 实现，
//! 用于 app 层集成测试，不执行真实的 LLM 或 Git 操作。

use std::sync::{Arc, Mutex};

use domain::{
    BranchService, BranchServiceError, CommitMessageError, CommitMessageService,
    CommitSummaryAnalysis, PrStatus, PullRequestError, PullRequestInfo, PullRequestService,
    PullRequestStatus,
};

// ==================== MockBranchService ====================

/// Mock 分支服务
///
/// 用于测试，不调用 LLM，返回预配置的分支名或固定名称。
pub struct MockBranchService {
    /// 预定义的返回值队列，按调用顺序依次返回；若为空则使用 default_name
    responses: Arc<Mutex<Vec<String>>>,
    /// 当 responses 为空时使用的默认分支名
    default_name: Arc<Mutex<String>>,
}

impl MockBranchService {
    /// 创建新的 Mock 服务，默认返回 "test-branch"
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            default_name: Arc::new(Mutex::new("test-branch".to_string())),
        }
    }

    /// 设置默认返回的分支名（当队列为空时使用）
    pub fn set_default_name(&self, name: impl Into<String>) {
        *self.default_name.lock().unwrap() = name.into();
    }

    /// 添加一次调用的返回值（按调用顺序返回）
    pub fn add_response(&self, name: impl Into<String>) {
        self.responses.lock().unwrap().push(name.into());
    }

    /// 清空返回值队列
    pub fn clear_responses(&self) {
        self.responses.lock().unwrap().clear();
    }
}

impl BranchService for MockBranchService {
    fn generate_branch_name(
        &self,
        _title: Option<&str>,
        _exists_branches: &[String],
    ) -> Result<String, BranchServiceError> {
        let mut responses = self.responses.lock().unwrap();
        if let Some(name) = responses.first() {
            let name = name.clone();
            responses.remove(0);
            Ok(name)
        } else {
            Ok(self.default_name.lock().unwrap().clone())
        }
    }
}

impl Default for MockBranchService {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== MockCommitMessageService ====================

/// 最小化的 CommitSummaryAnalysis JSON，用于反序列化
const MINIMAL_ANALYSIS_JSON: &str = r#"{
  "commit_message": {"title": "test: minimal commit", "body": "", "footer": ""},
  "structured_summary": {"type": "test", "scope": "", "subject": "minimal commit", "main_purpose": "", "key_changes": [], "details_by_category": {}, "changes_by_domain": []},
  "impact_analysis": {"breaking_changes": {"has_breaking": false, "description": "", "migration_guide": ""}, "affected_modules": [], "risk_assessment": {"overall_risk": "", "risk_factors": [], "mitigation": []}, "testing_suggestions": []},
  "statistics": {"total_files": 0, "additions": 0, "deletions": 0, "net_change": 0, "file_breakdown": {"added": 0, "modified": 0, "deleted": 0, "renamed": 0}},
  "metadata": {"complexity": "", "review_priority": "", "estimated_review_time": "", "tags": []}
}"#;

fn default_analysis() -> CommitSummaryAnalysis {
    serde_json::from_str(MINIMAL_ANALYSIS_JSON)
        .expect("minimal CommitSummaryAnalysis JSON must be valid")
}

/// Mock Commit Message 服务
///
/// 用于测试，不调用 LLM，返回预配置的 CommitSummaryAnalysis 或默认分析。
pub struct MockCommitMessageService {
    /// 预定义的成功返回值队列：generate_for_staged 与 generate_for_commit 共用
    responses: Arc<Mutex<Vec<CommitSummaryAnalysis>>>,
    /// 预定义的错误消息队列，按顺序返回 CommitMessageError::LLMError(msg)
    errors: Arc<Mutex<Vec<String>>>,
}

impl MockCommitMessageService {
    /// 创建新的 Mock 服务
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 添加一次成功返回值
    pub fn add_response(&self, analysis: CommitSummaryAnalysis) {
        self.responses.lock().unwrap().push(analysis);
    }

    /// 添加一次失败返回值（返回 CommitMessageError::LLMError(msg)）
    pub fn add_error(&self, msg: impl Into<String>) {
        self.errors.lock().unwrap().push(msg.into());
    }

    /// 清空返回值队列
    pub fn clear_responses(&self) {
        self.responses.lock().unwrap().clear();
        self.errors.lock().unwrap().clear();
    }

    fn next_result(&self) -> Result<CommitSummaryAnalysis, CommitMessageError> {
        let mut errors = self.errors.lock().unwrap();
        if let Some(msg) = errors.first() {
            let msg = msg.clone();
            errors.remove(0);
            return Err(CommitMessageError::LLMError(msg));
        }
        drop(errors);

        let mut responses = self.responses.lock().unwrap();
        if let Some(a) = responses.first() {
            let a = a.clone();
            responses.remove(0);
            Ok(a)
        } else {
            Ok(default_analysis())
        }
    }
}

impl CommitMessageService for MockCommitMessageService {
    fn generate_for_staged(&self) -> Result<CommitSummaryAnalysis, CommitMessageError> {
        self.next_result()
    }

    fn generate_for_commit(
        &self,
        _commit_ref: &str,
    ) -> Result<CommitSummaryAnalysis, CommitMessageError> {
        self.next_result()
    }
}

impl Default for MockCommitMessageService {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== MockPullRequestService ====================

/// 内存中的 PR 记录，用于 Mock
#[derive(Clone)]
struct MockPrRecord {
    id: String,
    title: String,
    body: String,
    state: String,
    merged: bool,
    source_branch: String,
    target_branch: String,
}

/// Mock Pull Request 服务
///
/// 用于测试，不调用 GitHub API，在内存中维护 PR 列表。
pub struct MockPullRequestService {
    prs: Arc<Mutex<Vec<MockPrRecord>>>,
    /// 当前分支 -> PR ID 映射
    branch_to_pr: Arc<Mutex<std::collections::HashMap<String, String>>>,
    /// 是否在 create_pull_request 等写操作时返回错误
    fail_next_write: Arc<Mutex<bool>>,
}

impl MockPullRequestService {
    /// 创建新的 Mock 服务
    pub fn new() -> Self {
        Self {
            prs: Arc::new(Mutex::new(Vec::new())),
            branch_to_pr: Arc::new(Mutex::new(std::collections::HashMap::new())),
            fail_next_write: Arc::new(Mutex::new(false)),
        }
    }

    /// 添加一个 PR 到 Mock 数据（便于测试预置数据）
    pub fn add_pr(
        &self,
        id: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
        state: impl Into<String>,
        merged: bool,
        source_branch: impl Into<String>,
        target_branch: impl Into<String>,
    ) {
        let id = id.into();
        let source_branch = source_branch.into();
        let record = MockPrRecord {
            id: id.clone(),
            title: title.into(),
            body: body.into(),
            state: state.into(),
            merged,
            source_branch: source_branch.clone(),
            target_branch: target_branch.into(),
        };
        self.prs.lock().unwrap().push(record);
        self.branch_to_pr.lock().unwrap().insert(source_branch, id);
    }

    /// 设置下一次写操作是否失败
    pub fn set_fail_next_write(&self, fail: bool) {
        *self.fail_next_write.lock().unwrap() = fail;
    }

    fn get_pr(&self, pr_id: &str) -> Result<MockPrRecord, PullRequestError> {
        self.prs
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.id == pr_id)
            .cloned()
            .ok_or_else(|| PullRequestError::NotFound(format!("PR not found: {}", pr_id)))
    }
}

impl PullRequestService for MockPullRequestService {
    fn create_pull_request(
        &self,
        _jira_id: Option<&str>,
        title: Option<&str>,
        description: Option<&str>,
        target_branch: Option<&str>,
    ) -> Result<String, PullRequestError> {
        if *self.fail_next_write.lock().unwrap() {
            *self.fail_next_write.lock().unwrap() = false;
            return Err(PullRequestError::Other("mock write failure".to_string()));
        }
        let id = format!("pr-{}", self.prs.lock().unwrap().len() + 1);
        let title = title.unwrap_or("Untitled").to_string();
        let body = description.unwrap_or("").to_string();
        let target = target_branch.unwrap_or("main").to_string();
        let source = "current-branch".to_string();
        self.add_pr(&id, &title, &body, "open", false, &source, &target);
        Ok(id)
    }

    fn merge_pull_request(&self, pr_id: &str, _force: bool) -> Result<(), PullRequestError> {
        if *self.fail_next_write.lock().unwrap() {
            *self.fail_next_write.lock().unwrap() = false;
            return Err(PullRequestError::Other("mock write failure".to_string()));
        }
        let mut prs = self.prs.lock().unwrap();
        if let Some(p) = prs.iter_mut().find(|p| p.id == pr_id) {
            p.state = "closed".to_string();
            p.merged = true;
            Ok(())
        } else {
            Err(PullRequestError::NotFound(format!(
                "PR not found: {}",
                pr_id
            )))
        }
    }

    fn get_pr_status(&self) -> Result<PrStatus, PullRequestError> {
        let current = "current-branch".to_string();
        let pr_id =
            self.branch_to_pr.lock().unwrap().get(&current).cloned().ok_or_else(|| {
                PullRequestError::NotFound(format!("No PR for branch {}", current))
            })?;
        let pr = self.get_pr(&pr_id)?;
        Ok(PrStatus {
            id: pr.id,
            title: pr.title,
            state: pr.state,
            merged: pr.merged,
        })
    }

    fn close_pull_request(&self, pr_id: &str) -> Result<(), PullRequestError> {
        if *self.fail_next_write.lock().unwrap() {
            *self.fail_next_write.lock().unwrap() = false;
            return Err(PullRequestError::Other("mock write failure".to_string()));
        }
        let mut prs = self.prs.lock().unwrap();
        if let Some(p) = prs.iter_mut().find(|p| p.id == pr_id) {
            p.state = "closed".to_string();
            Ok(())
        } else {
            Err(PullRequestError::NotFound(format!(
                "PR not found: {}",
                pr_id
            )))
        }
    }

    fn list_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PrStatus>, PullRequestError> {
        let prs = self.prs.lock().unwrap();
        let mut list: Vec<PrStatus> = prs
            .iter()
            .filter(|p| state.is_none_or(|s| p.state == s))
            .map(|p| PrStatus {
                id: p.id.clone(),
                title: p.title.clone(),
                state: p.state.clone(),
                merged: p.merged,
            })
            .collect();
        if let Some(n) = limit {
            list.truncate(n);
        }
        Ok(list)
    }

    fn update_pull_request(
        &self,
        pr_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<(), PullRequestError> {
        if *self.fail_next_write.lock().unwrap() {
            *self.fail_next_write.lock().unwrap() = false;
            return Err(PullRequestError::Other("mock write failure".to_string()));
        }
        let mut prs = self.prs.lock().unwrap();
        if let Some(p) = prs.iter_mut().find(|p| p.id == pr_id) {
            if let Some(t) = title {
                p.title = t.to_string();
            }
            if let Some(b) = body {
                p.body = b.to_string();
            }
            Ok(())
        } else {
            Err(PullRequestError::NotFound(format!(
                "PR not found: {}",
                pr_id
            )))
        }
    }

    fn add_comment(&self, pr_id: &str, comment: &str) -> Result<(), PullRequestError> {
        if comment.is_empty() {
            return Err(PullRequestError::InvalidInput(
                "Comment cannot be empty".to_string(),
            ));
        }
        if *self.fail_next_write.lock().unwrap() {
            *self.fail_next_write.lock().unwrap() = false;
            return Err(PullRequestError::Other("mock write failure".to_string()));
        }
        let _ = self.get_pr(pr_id)?;
        Ok(())
    }

    fn approve_pull_request(&self, pr_id: &str) -> Result<(), PullRequestError> {
        if *self.fail_next_write.lock().unwrap() {
            *self.fail_next_write.lock().unwrap() = false;
            return Err(PullRequestError::Other("mock write failure".to_string()));
        }
        let _ = self.get_pr(pr_id)?;
        Ok(())
    }

    fn get_pr_diff(&self, pr_id: &str) -> Result<String, PullRequestError> {
        let _ = self.get_pr(pr_id)?;
        Ok("mock diff".to_string())
    }

    fn get_pull_request(&self, pr_id: &str) -> Result<PullRequestInfo, PullRequestError> {
        let pr = self.get_pr(pr_id)?;
        Ok(PullRequestInfo {
            id: pr.id,
            title: pr.title,
            body: pr.body,
            status: PullRequestStatus {
                state: pr.state,
                merged: pr.merged,
                merged_at: None,
            },
            source_branch: pr.source_branch,
            target_branch: pr.target_branch,
        })
    }

    fn get_current_branch_pull_request(
        &self,
        current_branch: &str,
    ) -> Result<Option<String>, PullRequestError> {
        Ok(self.branch_to_pr.lock().unwrap().get(current_branch).cloned())
    }
}

impl Default for MockPullRequestService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use domain::BranchService;

    use super::*;

    #[test]
    fn mock_branch_service_returns_default() {
        let service = MockBranchService::new();
        let name = service.generate_branch_name(None, &[]).unwrap();
        assert_eq!(name, "test-branch");
    }

    #[test]
    fn mock_branch_service_returns_queued_responses() {
        let service = MockBranchService::new();
        service.add_response("feature/foo");
        service.add_response("fix/bar");
        assert_eq!(
            service.generate_branch_name(None, &[]).unwrap(),
            "feature/foo"
        );
        assert_eq!(service.generate_branch_name(None, &[]).unwrap(), "fix/bar");
        assert_eq!(
            service.generate_branch_name(None, &[]).unwrap(),
            "test-branch"
        );
    }

    #[test]
    fn mock_commit_message_service_returns_default_analysis() {
        let service = MockCommitMessageService::new();
        let r = service.generate_for_staged().unwrap();
        assert!(!r.commit_message.title.is_empty());
        let r2 = service.generate_for_commit("HEAD").unwrap();
        assert!(!r2.commit_message.title.is_empty());
    }

    #[test]
    fn mock_pull_request_service_add_and_get() {
        let service = MockPullRequestService::new();
        // get_pr_status() 使用 "current-branch" 作为当前分支，需添加对应 PR
        service.add_pr(
            "1",
            "Title",
            "Body",
            "open",
            false,
            "current-branch",
            "main",
        );
        let status = service.get_pr_status().unwrap();
        assert_eq!(status.id, "1");
        assert_eq!(status.title, "Title");
        assert_eq!(status.state, "open");
        assert!(!status.merged);
        let branch_pr = service.get_current_branch_pull_request("current-branch").unwrap();
        assert_eq!(branch_pr, Some("1".to_string()));
    }
}
