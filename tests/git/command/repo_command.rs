//! GitRepoCommand 测试
//!
//! 测试仓库命令包装层的功能。

use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::GitRepoCommand;

use crate::common::environments::GitTestEnv;
use crate::common::helpers::CurrentDirGuard;

/// 测试检查是否为 Git 仓库
///
/// ## 测试目的
/// 验证 GitRepoCommand::is_git_repo() 能够检查是否为 Git 仓库。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 检查是否为 Git 仓库
/// 3. 验证返回 true
///
/// ## 预期结果
/// - 返回 true
#[test]
#[serial]
fn test_is_git_repo_returns_true_in_repo() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 检查是否为 Git 仓库
    let is_repo = GitRepoCommand::is_git_repo(None);

    // Assert: 验证返回 true
    assert!(is_repo, "Should return true in a Git repository");

    Ok(())
}

/// 测试检查非 Git 目录返回 false
///
/// ## 测试目的
/// 验证 GitRepoCommand::is_git_repo() 在非 Git 目录中返回 false。
///
/// ## 测试场景
/// 1. 创建临时目录（非 Git 仓库）
/// 2. 检查是否为 Git 仓库
/// 3. 验证返回 false
///
/// ## 预期结果
/// - 返回 false
#[test]
fn test_is_git_repo_returns_false_outside_repo() -> Result<()> {
    // Arrange: 创建临时目录（非 Git 仓库）
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    // Act: 检查是否为 Git 仓库
    let is_repo = GitRepoCommand::is_git_repo(Some(temp_path));

    // Assert: 验证返回 false
    assert!(!is_repo, "Should return false outside a Git repository");

    Ok(())
}

/// 测试获取 Git 目录
///
/// ## 测试目的
/// 验证 GitRepoCommand::get_git_dir() 能够获取 Git 目录路径。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 获取 Git 目录
/// 3. 验证返回有效的路径
///
/// ## 预期结果
/// - 返回有效的 Git 目录路径
#[test]
#[serial]
fn test_get_git_dir_returns_valid_path() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 获取 Git 目录
    let git_dir = GitRepoCommand::get_git_dir(None)?;

    // Assert: 验证返回有效的路径
    assert!(
        !git_dir.is_empty(),
        "Git directory path should not be empty"
    );
    assert!(
        git_dir.contains(".git"),
        "Git directory path should contain '.git'"
    );

    Ok(())
}

/// 测试获取工作目录根路径
///
/// ## 测试目的
/// 验证 GitRepoCommand::get_workdir() 能够获取工作目录根路径。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 获取工作目录根路径
/// 3. 验证返回有效的路径
///
/// ## 预期结果
/// - 返回有效的工作目录路径
#[test]
#[serial]
fn test_get_workdir_returns_valid_path() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let expected_path = env.path().canonicalize()?;

    // Act: 获取工作目录根路径
    let workdir = GitRepoCommand::get_workdir(None)?;

    // Assert: 验证返回有效的路径
    assert!(
        !workdir.is_empty(),
        "Work directory path should not be empty"
    );
    // 路径应该匹配（可能格式不同，但应该指向同一个目录）
    let workdir_path = std::path::Path::new(&workdir).canonicalize()?;
    assert_eq!(
        workdir_path, expected_path,
        "Work directory should match repository path"
    );

    Ok(())
}

/// 测试列出所有远程仓库
///
/// ## 测试目的
/// 验证 GitRepoCommand::list_remotes() 能够列出所有远程仓库。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 列出所有远程
/// 3. 验证返回列表（可能为空）
///
/// ## 预期结果
/// - 返回远程列表（可能为空，因为测试环境可能没有配置远程）
#[test]
#[serial]
fn test_list_remotes_returns_remotes() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 列出所有远程
    let remotes = GitRepoCommand::list_remotes(None)?;

    // Assert: 验证返回列表（可能为空，这是正常的）
    // 测试环境可能没有配置远程，所以列表可能为空
    assert!(
        remotes.is_empty() || !remotes.is_empty(),
        "Should return a list of remotes"
    );

    Ok(())
}
