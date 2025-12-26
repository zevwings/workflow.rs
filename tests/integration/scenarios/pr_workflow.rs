//! PR工作流场景
//!
//! 提供PR创建、更新、合并等场景的测试辅助函数。

#![allow(dead_code)]

use crate::common::environments::CliTestEnv;
use color_eyre::Result;

/// PR创建场景构建器
///
/// 用于构建和设置PR创建测试场景。
#[allow(dead_code)]
pub struct PRCreationScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// 分支名称
    pub branch_name: String,
    /// 提交消息
    pub commit_message: String,
    /// 文件名
    pub file_name: String,
    /// 文件内容
    pub file_content: String,
}

#[allow(dead_code)]
impl PRCreationScenario {
    /// 创建新的PR创建场景
    ///
    /// # 返回
    ///
    /// 返回初始化的场景构建器
    pub fn new() -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        Ok(Self {
            env,
            branch_name: "feature/test".to_string(),
            commit_message: "feat: add test".to_string(),
            file_name: "test.txt".to_string(),
            file_content: "test content".to_string(),
        })
    }

    /// 设置分支名称
    pub fn with_branch(mut self, branch_name: &str) -> Self {
        self.branch_name = branch_name.to_string();
        self
    }

    /// 设置提交消息
    pub fn with_commit_message(mut self, message: &str) -> Self {
        self.commit_message = message.to_string();
        self
    }

    /// 设置文件名和内容
    pub fn with_file(mut self, name: &str, content: &str) -> Self {
        self.file_name = name.to_string();
        self.file_content = content.to_string();
        self
    }

    /// 设置并执行场景
    ///
    /// 执行以下步骤：
    /// 1. 创建初始提交
    /// 2. 创建并切换到新分支
    /// 3. 创建文件并提交
    ///
    /// # 返回
    ///
    /// 返回设置好的环境，可用于后续测试
    pub fn setup(self) -> Result<CliTestEnv> {
        // 1. 创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 2. 创建并切换到新分支
        self.env.create_branch(&self.branch_name)?.checkout(&self.branch_name)?;

        // 3. 创建文件并提交
        self.env
            .create_file(&self.file_name, &self.file_content)?
            .create_commit(&self.commit_message)?;

        Ok(self.env)
    }
}

/// PR合并场景构建器
#[allow(dead_code)]
pub struct PRMergeScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// 源分支名称
    pub source_branch: String,
    /// 目标分支名称
    pub target_branch: String,
}

impl PRMergeScenario {
    /// 创建新的PR合并场景
    pub fn new() -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        Ok(Self {
            env,
            source_branch: "feature/test".to_string(),
            target_branch: "main".to_string(),
        })
    }

    /// 设置源分支
    pub fn with_source_branch(mut self, branch: &str) -> Self {
        self.source_branch = branch.to_string();
        self
    }

    /// 设置目标分支
    pub fn with_target_branch(mut self, branch: &str) -> Self {
        self.target_branch = branch.to_string();
        self
    }

    /// 设置并执行场景
    ///
    /// 执行以下步骤：
    /// 1. 在目标分支创建初始提交
    /// 2. 创建并切换到源分支
    /// 3. 在源分支创建提交
    /// 4. 切换回目标分支
    pub fn setup(self) -> Result<CliTestEnv> {
        // 1. 在目标分支创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 2. 创建并切换到源分支
        self.env.create_branch(&self.source_branch)?.checkout(&self.source_branch)?;

        // 3. 在源分支创建提交
        self.env
            .create_file("feature.txt", "new feature")?
            .create_commit("feat: add feature")?;

        // 4. 切换回目标分支
        self.env.checkout(&self.target_branch)?;

        Ok(self.env)
    }
}
