//! 提交工作流场景
//!
//! 提供提交创建、历史管理等场景的测试辅助函数。

use crate::common::environments::CliTestEnv;
use color_eyre::Result;

/// 提交创建场景构建器
pub struct CommitCreationScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// 提交列表（文件名，内容，提交消息）
    pub commits: Vec<(String, String, String)>,
}

impl CommitCreationScenario {
    /// 创建新的提交创建场景
    pub fn new() -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        Ok(Self {
            env,
            commits: Vec::new(),
        })
    }

    /// 添加提交
    ///
    /// # 参数
    ///
    /// * `file_name` - 文件名
    /// * `file_content` - 文件内容
    /// * `commit_message` - 提交消息
    pub fn add_commit(mut self, file_name: &str, file_content: &str, commit_message: &str) -> Self {
        self.commits.push((
            file_name.to_string(),
            file_content.to_string(),
            commit_message.to_string(),
        ));
        self
    }

    /// 设置并执行场景
    ///
    /// 执行以下步骤：
    /// 1. 创建初始提交
    /// 2. 创建所有指定的提交
    pub fn setup(self) -> Result<CliTestEnv> {
        // 1. 创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 2. 创建所有指定的提交
        for (file_name, file_content, commit_message) in &self.commits {
            self.env.create_file(file_name, file_content)?.create_commit(commit_message)?;
        }

        Ok(self.env)
    }
}

/// 提交历史场景构建器
pub struct CommitHistoryScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// 提交数量
    pub commit_count: usize,
}

impl CommitHistoryScenario {
    /// 创建新的提交历史场景
    pub fn new() -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        Ok(Self {
            env,
            commit_count: 5,
        })
    }

    /// 设置提交数量
    pub fn with_commit_count(mut self, count: usize) -> Self {
        self.commit_count = count;
        self
    }

    /// 设置并执行场景
    ///
    /// 创建指定数量的提交，用于测试提交历史相关功能。
    pub fn setup(self) -> Result<CliTestEnv> {
        // 创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 创建多个提交
        for i in 1..=self.commit_count {
            let file_name = format!("file{}.txt", i);
            let file_content = format!("Content {}", i);
            let commit_message = format!("feat: add file {}", i);

            self.env
                .create_file(&file_name, &file_content)?
                .create_commit(&commit_message)?;
        }

        Ok(self.env)
    }
}
