//! `workflow version` 子命令实现
//!
//! 迁移自 `.go/internal/commands/version.go`：
//! - 打印 CLI 名称
//! - 显示版本号、构建时间、Git 提交哈希

use prompt::{br, print};

/// Version 命令
pub struct VersionCommand {
    version: String,
    build_date: String,
    git_commit: String,
}

impl VersionCommand {
    /// 创建新的 VersionCommand
    pub fn new(version: &str, build_date: &str, git_commit: &str) -> Self {
        Self {
            version: version.to_string(),
            build_date: build_date.to_string(),
            git_commit: git_commit.to_string(),
        }
    }

    /// 运行 `version` 子命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("Workflow CLI");
        br!();
        print!("Version:    {}", self.version);
        print!("Build Date: {}", self.build_date);
        print!("Git Commit: {}", self.git_commit);

        Ok(())
    }
}
