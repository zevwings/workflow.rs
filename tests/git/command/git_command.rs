//! GitCommand 基础测试
//!
//! 测试 GitCommand 的命令执行功能和 GitError 的错误处理功能。

use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::command::GitCommand;
use workflow::git::commands::GitError;

use crate::common::environments::GitTestEnv;
use crate::common::helpers::CurrentDirGuard;

/// 测试执行简单 Git 命令
///
/// ## 测试目的
/// 验证 GitCommand::run() 能够正确执行 Git 命令并返回输出。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 执行 git rev-parse --git-dir 命令
/// 3. 验证输出包含 .git
///
/// ## 预期结果
/// - 命令执行成功，输出包含 .git
#[test]
#[serial]
fn test_run_simple_command_returns_output() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 执行 git rev-parse --git-dir 命令
    let output = GitCommand::run(&["rev-parse", "--git-dir"], None)?;

    // Assert: 验证输出包含 .git
    assert!(
        output.contains(".git"),
        "Output should contain '.git', got: {}",
        output
    );

    Ok(())
}

/// 测试执行命令失败时返回错误
///
/// ## 测试目的
/// 验证 GitCommand::run() 在命令失败时正确返回错误。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 执行不存在的 Git 命令
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 命令执行失败，返回错误
#[test]
#[serial]
fn test_run_invalid_command_returns_error() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 执行不存在的命令
    let result = GitCommand::run(&["invalid-command-that-does-not-exist"], None);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should return error for invalid command");

    Ok(())
}

/// 测试静默执行命令
///
/// ## 测试目的
/// 验证 GitCommand::execute() 能够静默执行命令。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 执行 git status 命令
/// 3. 验证命令成功执行（无输出）
///
/// ## 预期结果
/// - 命令执行成功
#[test]
#[serial]
fn test_execute_command_returns_ok() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 执行 git status 命令
    let result = GitCommand::execute(&["status"], None);

    // Assert: 验证命令成功执行
    assert!(result.is_ok(), "Command should execute successfully");

    Ok(())
}

/// 测试检查命令是否成功
///
/// ## 测试目的
/// 验证 GitCommand::check() 能够检查命令是否成功。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 检查 git rev-parse --git-dir 是否成功
/// 3. 验证返回 true
///
/// ## 预期结果
/// - 命令检查成功，返回 true
#[test]
#[serial]
fn test_check_valid_command_returns_true() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 检查 git rev-parse --git-dir 是否成功
    let success = GitCommand::check(&["rev-parse", "--git-dir"], None);

    // Assert: 验证返回 true
    assert!(success, "Valid command should return true");

    Ok(())
}

/// 测试检查无效命令返回 false
///
/// ## 测试目的
/// 验证 GitCommand::check() 对无效命令返回 false。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 检查不存在的命令
/// 3. 验证返回 false
///
/// ## 预期结果
/// - 无效命令返回 false
#[test]
#[serial]
fn test_check_invalid_command_returns_false() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 检查不存在的命令
    let success = GitCommand::check(&["invalid-command-that-does-not-exist"], None);

    // Assert: 验证返回 false
    assert!(!success, "Invalid command should return false");

    Ok(())
}

/// 测试指定工作目录执行命令
///
/// ## 测试目的
/// 验证 GitCommand 能够指定工作目录执行命令。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 使用指定路径执行命令
/// 3. 验证命令在正确目录执行
///
/// ## 预期结果
/// - 命令在指定目录执行成功
#[test]
#[serial]
fn test_run_with_cwd_executes_in_correct_directory() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 使用指定路径执行命令
    let output = GitCommand::run(&["rev-parse", "--show-toplevel"], Some(repo_path.as_path()))?;

    // Assert: 验证输出指向正确的目录
    assert!(
        output.trim().ends_with(repo_path.canonicalize()?.to_str().unwrap_or_default()),
        "Output should point to correct directory"
    );

    Ok(())
}

// ==================== GitError 错误类型测试 ====================

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
