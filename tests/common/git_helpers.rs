//! Git测试辅助函数
//!
//! 提供用于测试的Git仓库设置和管理功能。

use color_eyre::Result;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Git测试环境
pub struct GitTestEnv {
    temp_dir: TempDir,
}

impl GitTestEnv {
    /// 创建新的Git测试环境
    ///
    /// 自动初始化Git仓库并配置测试用户
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // 初始化Git仓库，设置默认分支为main
        Self::run_git_command(repo_path, &["init", "-b", "main"])?;

        // 配置测试用户
        Self::run_git_command(repo_path, &["config", "user.name", "Test User"])?;
        Self::run_git_command(repo_path, &["config", "user.email", "test@example.com"])?;

        // 创建初始提交
        std::fs::write(repo_path.join("README.md"), "# Test Repository\n")?;
        Self::run_git_command(repo_path, &["add", "."])?;
        Self::run_git_command(repo_path, &["commit", "-m", "Initial commit"])?;

        Ok(Self { temp_dir })
    }

    /// 获取仓库路径
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// 创建新分支
    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        Self::run_git_command(self.path(), &["branch", branch_name])
    }

    /// 切换分支
    pub fn checkout(&self, branch_name: &str) -> Result<()> {
        Self::run_git_command(self.path(), &["checkout", branch_name])
    }

    /// 创建并切换到新分支
    pub fn checkout_new_branch(&self, branch_name: &str) -> Result<()> {
        Self::run_git_command(self.path(), &["checkout", "-b", branch_name])
    }

    /// 创建测试文件
    pub fn create_file(&self, filename: &str, content: &str) -> Result<()> {
        let file_path = self.path().join(filename);
        std::fs::write(file_path, content)?;
        Ok(())
    }

    /// 添加并提交更改
    pub fn add_and_commit(&self, message: &str) -> Result<()> {
        Self::run_git_command(self.path(), &["add", "."])?;
        Self::run_git_command(self.path(), &["commit", "-m", message])
    }

    /// 创建测试提交
    pub fn make_test_commit(&self, filename: &str, content: &str, message: &str) -> Result<()> {
        self.create_file(filename, content)?;
        self.add_and_commit(message)
    }

    /// 获取当前分支名
    pub fn current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(self.path())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to get current branch: {}",
                error
            ));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// 获取最后一次提交的SHA
    pub fn last_commit_sha(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(self.path())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Failed to get commit SHA: {}",
                error
            ));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// 运行Git命令
    fn run_git_command(repo_path: &Path, args: &[&str]) -> Result<()> {
        let output = Command::new("git").args(args).current_dir(repo_path).output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(color_eyre::eyre::eyre!(
                "Git command failed: git {}\nError: {}",
                args.join(" "),
                error
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_test_env_creation() -> Result<()> {
        let env = GitTestEnv::new()?;
        assert!(env.path().exists());
        assert!(env.path().join(".git").exists());
        Ok(())
    }

    #[test]
    fn test_create_and_checkout_branch() -> Result<()> {
        let env = GitTestEnv::new()?;

        env.create_branch("test-branch")?;
        env.checkout("test-branch")?;

        let current = env.current_branch()?;
        assert_eq!(current, "test-branch");

        Ok(())
    }

    #[test]
    fn test_make_test_commit() -> Result<()> {
        let env = GitTestEnv::new()?;

        let sha_before = env.last_commit_sha()?;

        env.make_test_commit("test.txt", "test content", "test commit")?;

        let sha_after = env.last_commit_sha()?;
        assert_ne!(sha_before, sha_after);

        Ok(())
    }
}
