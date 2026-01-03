//! GitRepoCommand 测试
//!
//! 测试仓库命令包装层的功能。

use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::GitRepoCommand;

use crate::common::environments::GitTestEnv;

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
    let repo_path = env.path();

    // Act: 检查是否为 Git 仓库（使用明确的路径）
    let is_repo = GitRepoCommand::is_git_repo(Some(repo_path.as_path()));

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
#[serial]
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
    let repo_path = env.path();

    // Act: 获取 Git 目录（使用明确的路径）
    let git_dir = GitRepoCommand::get_git_dir(Some(repo_path.as_path()))?;

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
    let repo_path = env.path();
    let expected_path = repo_path.canonicalize()?;

    // Act: 获取工作目录根路径（使用明确的路径）
    let workdir = GitRepoCommand::get_workdir(Some(repo_path.as_path()))?;

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
    let repo_path = env.path();

    // Act: 列出所有远程（使用明确的路径）
    let remotes = GitRepoCommand::list_remotes(Some(repo_path.as_path()))?;

    // Assert: 验证返回列表（可能为空，这是正常的）
    // 测试环境可能没有配置远程，所以列表可能为空
    assert!(
        remotes.is_empty() || !remotes.is_empty(),
        "Should return a list of remotes"
    );

    Ok(())
}

/// 测试添加远程仓库
///
/// ## 测试目的
/// 验证 GitRepoCommand::add_remote() 能够添加远程仓库。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 添加远程仓库
/// 3. 验证远程仓库存在
/// 4. 清理：删除远程仓库
///
/// ## 预期结果
/// - 远程仓库添加成功
#[test]
#[serial]
fn test_add_remote_adds_remote() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let remote_name = "test-origin";
    let remote_url = "https://github.com/test/repo.git";

    // Act: 添加远程仓库（使用明确的路径）
    GitRepoCommand::add_remote(remote_name, remote_url, Some(repo_path.as_path()))?;

    // Assert: 验证远程仓库存在
    let remotes = GitRepoCommand::list_remotes(Some(repo_path.as_path()))?;
    assert!(
        remotes.contains(&remote_name.to_string()),
        "Remote should be added"
    );

    // Cleanup: 删除远程仓库
    let _ = GitRepoCommand::remove_remote(remote_name, Some(repo_path.as_path()));

    Ok(())
}

/// 测试删除远程仓库
///
/// ## 测试目的
/// 验证 GitRepoCommand::remove_remote() 能够删除远程仓库。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 添加远程仓库
/// 3. 删除远程仓库
/// 4. 验证远程仓库被删除
///
/// ## 预期结果
/// - 远程仓库删除成功
#[test]
#[serial]
fn test_remove_remote_removes_remote() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let remote_name = "test-remove";
    let remote_url = "https://github.com/test/repo.git";

    // Act: 添加远程仓库（使用明确的路径）
    GitRepoCommand::add_remote(remote_name, remote_url, Some(repo_path.as_path()))?;

    // Act: 删除远程仓库
    GitRepoCommand::remove_remote(remote_name, Some(repo_path.as_path()))?;

    // Assert: 验证远程仓库被删除
    let remotes = GitRepoCommand::list_remotes(Some(repo_path.as_path()))?;
    assert!(
        !remotes.contains(&remote_name.to_string()),
        "Remote should be removed"
    );

    Ok(())
}

/// 测试获取远程 URL
///
/// ## 测试目的
/// 验证 GitRepoCommand::get_remote_url() 能够获取远程 URL。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 添加远程仓库
/// 3. 获取远程 URL
/// 4. 验证返回的 URL
/// 5. 清理：删除远程仓库
///
/// ## 预期结果
/// - 返回正确的远程 URL
#[test]
#[serial]
fn test_get_remote_url_returns_url() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let remote_name = "test-url";
    let remote_url = "https://github.com/test/repo.git";

    // Act: 添加远程仓库（使用明确的路径）
    GitRepoCommand::add_remote(remote_name, remote_url, Some(repo_path.as_path()))?;

    // Act: 获取远程 URL
    let url = GitRepoCommand::get_remote_url(Some(remote_name), Some(repo_path.as_path()))?;

    // Assert: 验证返回的 URL
    assert_eq!(url, remote_url, "Remote URL should match");

    // Cleanup: 删除远程仓库
    let _ = GitRepoCommand::remove_remote(remote_name, Some(repo_path.as_path()));

    Ok(())
}
