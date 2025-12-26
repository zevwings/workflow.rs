//! 分支工作流场景
//!
//! 提供分支创建、切换、管理等场景的测试辅助函数。

use color_eyre::Result;
use crate::common::environments::CliTestEnv;

/// 分支创建场景构建器
pub struct BranchCreationScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// 分支名称
    pub branch_name: String,
}

impl BranchCreationScenario {
    /// 创建新的分支创建场景
    pub fn new() -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        Ok(Self {
            env,
            branch_name: "feature/test".to_string(),
        })
    }

    /// 设置分支名称
    pub fn with_branch_name(mut self, name: &str) -> Self {
        self.branch_name = name.to_string();
        self
    }

    /// 设置并执行场景
    ///
    /// 执行以下步骤：
    /// 1. 创建初始提交
    /// 2. 创建分支
    /// 3. 切换到新分支
    pub fn setup(self) -> Result<CliTestEnv> {
        // 1. 创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 2. 创建并切换分支
        self.env
            .create_branch(&self.branch_name)?
            .checkout(&self.branch_name)?;

        Ok(self.env)
    }
}

/// 多分支场景构建器
pub struct MultiBranchScenario {
    /// CLI测试环境
    pub env: CliTestEnv,
    /// 分支列表
    pub branches: Vec<String>,
}

impl MultiBranchScenario {
    /// 创建新的多分支场景
    pub fn new() -> Result<Self> {
        let env = CliTestEnv::new()?;
        env.init_git_repo()?;

        Ok(Self {
            env,
            branches: vec!["feature/one".to_string(), "feature/two".to_string()],
        })
    }

    /// 添加分支
    pub fn add_branch(mut self, branch_name: &str) -> Self {
        self.branches.push(branch_name.to_string());
        self
    }

    /// 设置并执行场景
    ///
    /// 执行以下步骤：
    /// 1. 创建初始提交
    /// 2. 创建所有分支
    /// 3. 切换到第一个分支
    pub fn setup(self) -> Result<CliTestEnv> {
        // 1. 创建初始提交
        self.env
            .create_file("README.md", "# Test Project")?
            .create_commit("Initial commit")?;

        // 2. 创建所有分支
        for branch in &self.branches {
            self.env.create_branch(branch)?;
        }

        // 3. 切换到第一个分支
        if let Some(first_branch) = self.branches.first() {
            self.env.checkout(first_branch)?;
        }

        Ok(self.env)
    }
}

