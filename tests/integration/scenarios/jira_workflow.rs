//! Jira工作流场景
//!
//! 提供Jira ticket创建、更新、状态同步等场景的测试辅助函数。

use crate::common::environments::CliTestEnv;
use color_eyre::Result;

/// Jira集成场景构建器
pub struct JiraIntegrationScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// Jira ticket ID
    pub ticket_id: String,
    /// 分支名称（基于ticket ID）
    pub branch_name: String,
}

impl JiraIntegrationScenario {
    /// 创建新的Jira集成场景
    ///
    /// # 参数
    ///
    /// * `ticket_id` - Jira ticket ID（如 "PROJ-123"）
    pub fn new(ticket_id: &str) -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        let branch_name = format!("feature/{}", ticket_id);

        Ok(Self {
            env,
            ticket_id: ticket_id.to_string(),
            branch_name,
        })
    }

    /// 设置Jira配置
    ///
    /// # 参数
    ///
    /// * `url` - Jira URL
    /// * `username` - Jira用户名
    pub fn with_jira_config(self, url: &str, username: &str) -> Result<Self> {
        let config = format!(
            r#"
[jira]
url = "{}"
username = "{}"
"#,
            url, username
        );
        self.env.create_config(&config)?;
        Ok(self)
    }

    /// 设置并执行场景
    ///
    /// 执行以下步骤：
    /// 1. 创建初始提交
    /// 2. 创建关联Jira ticket的分支
    /// 3. 创建文件并提交（包含ticket ID）
    pub fn setup(self) -> Result<CliTestEnv> {
        // 1. 创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 2. 创建关联Jira ticket的分支
        self.env.create_branch(&self.branch_name)?.checkout(&self.branch_name)?;

        // 3. 创建文件并提交（包含ticket ID）
        let commit_message = format!("feat({}): add feature", self.ticket_id);
        self.env
            .create_file("feature.txt", "new feature")?
            .create_commit(&commit_message)?;

        Ok(self.env)
    }
}

/// Jira状态同步场景构建器
pub struct JiraStatusSyncScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// Jira ticket ID
    pub ticket_id: String,
}

impl JiraStatusSyncScenario {
    /// 创建新的Jira状态同步场景
    pub fn new(ticket_id: &str) -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        Ok(Self {
            env,
            ticket_id: ticket_id.to_string(),
        })
    }

    /// 设置Jira配置
    pub fn with_jira_config(self, url: &str, username: &str) -> Result<Self> {
        let config = format!(
            r#"
[jira]
url = "{}"
username = "{}"
"#,
            url, username
        );
        self.env.create_config(&config)?;
        Ok(self)
    }

    /// 设置并执行场景
    ///
    /// 创建包含Jira ticket的PR场景，用于测试状态同步。
    pub fn setup(self) -> Result<CliTestEnv> {
        // 创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 创建关联ticket的分支
        let branch_name = format!("feature/{}", self.ticket_id);
        self.env.create_branch(&branch_name)?.checkout(&branch_name)?;

        // 创建提交
        let commit_message = format!("feat({}): implement feature", self.ticket_id);
        self.env
            .create_file("feature.txt", "new feature")?
            .create_commit(&commit_message)?;

        Ok(self.env)
    }
}
