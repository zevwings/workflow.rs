//! GitError 错误类型测试
//!
//! 测试 GitError 的错误处理功能。

use color_eyre::Result;
use workflow::git::commands::{GitCommand, GitError};

/// 测试 NotGitRepo 错误
///
/// ## 测试目的
/// 验证在非 Git 仓库中执行命令时返回合适的错误。
///
/// ## 测试场景
/// 1. 创建非 Git 目录
/// 2. 执行 Git 命令
/// 3. 验证返回错误（可能是 NotGitRepo 或 CommandFailed）
///
/// ## 预期结果
/// - 返回错误（NotGitRepo 或 CommandFailed，取决于 Git 版本和错误消息）
#[test]
fn test_not_git_repo_error() -> Result<()> {
    // Arrange: 创建临时目录（非 Git 仓库）
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    // Act: 执行 Git 命令
    let result = GitCommand::run(&["status"], Some(temp_path));

    // Assert: 验证返回错误（根据 Git 版本可能返回不同的错误类型）
    match result {
        Err(GitError::NotGitRepo) => Ok(()),
        Err(GitError::CommandFailed { .. }) => {
            // Git 可能返回 CommandFailed，这也表示不是 Git 仓库
            Ok(())
        }
        Err(e) => Err(color_eyre::eyre::eyre!(
            "Expected NotGitRepo or CommandFailed error, got: {:?}",
            e
        )),
        Ok(_) => Err(color_eyre::eyre::eyre!("Expected error, got success")),
    }
}

/// 测试错误格式化
///
/// ## 测试目的
/// 验证 GitError 能够正确格式化错误消息。
///
/// ## 测试场景
/// 1. 创建各种错误类型
/// 2. 格式化错误消息
/// 3. 验证错误消息格式正确
///
/// ## 预期结果
/// - 错误消息格式正确
#[test]
fn test_error_formatting() -> Result<()> {
    // Arrange: 创建各种错误类型
    let not_git_repo = GitError::NotGitRepo;
    let branch_not_found = GitError::BranchNotFound {
        branch: "test-branch".to_string(),
    };
    let command_failed = GitError::CommandFailed {
        command: "git test".to_string(),
        stderr: "error message".to_string(),
        stdout: "output".to_string(),
    };

    // Act: 格式化错误消息
    let not_git_repo_msg = format!("{}", not_git_repo);
    let branch_not_found_msg = format!("{}", branch_not_found);
    let command_failed_msg = format!("{}", command_failed);

    // Assert: 验证错误消息格式正确
    assert!(
        not_git_repo_msg.contains("Git repository"),
        "NotGitRepo error message should contain 'Git repository'"
    );
    assert!(
        branch_not_found_msg.contains("test-branch"),
        "BranchNotFound error message should contain branch name"
    );
    assert!(
        command_failed_msg.contains("git test"),
        "CommandFailed error message should contain command"
    );

    Ok(())
}
