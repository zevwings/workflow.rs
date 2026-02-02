//! Pull Request Diff 服务
//!
//! 提供 Pull Request diff 相关的业务逻辑实现

use std::sync::Arc;

use domain::CNBError;
use toolkit::log_debug;

use crate::cnb::client::CNBClient;
use crate::cnb::services::ServiceContext;

/// Pull Request Diff 服务接口
pub trait PullRequestDiffService: Send + Sync {
    /// 获取 PR 的 diff 内容
    fn get_pull_request_diff(&self, pull_request_id: &str) -> Result<String, CNBError>;
}

/// Pull Request Diff 服务实现
pub struct PullRequestDiffServiceImpl {
    client: Arc<dyn CNBClient>,
    context: Arc<dyn ServiceContext>,
}

impl PullRequestDiffServiceImpl {
    pub fn new(client: Arc<dyn CNBClient>, context: Arc<dyn ServiceContext>) -> Self {
        Self { client, context }
    }
}

impl PullRequestDiffService for PullRequestDiffServiceImpl {
    fn get_pull_request_diff(&self, pull_request_id: &str) -> Result<String, CNBError> {
        let project_path = self.context.project_path()?;
        let encoded_path = urlencoding::encode(&project_path);
        let pr_number = self.context.parse_pr_number(pull_request_id)?;

        // CNB API 使用 .diff 端点获取 diff 内容
        let url = format!("/repos/{}/pulls/{}.diff", encoded_path, pr_number);

        log_debug!("Fetching PR diff from: {}", url);

        let response = self.client.get(&url)?;
        let diff = response.text()?;

        Ok(diff)
    }
}
