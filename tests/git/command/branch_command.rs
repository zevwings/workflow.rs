//! GitBranchCommand 测试
//!
//! 测试分支命令包装层的功能。

use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::GitBranchCommand;

use crate::common::environments::GitTestEnv;
use crate::common::helpers::CurrentDirGuard;

/// 测试获取当前分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::current_branch() 能够获取当前分支名。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 获取当前分支
/// 3. 验证返回分支名
///
/// ## 预期结果
/// - 返回当前分支名（main）
#[test]
#[serial]
fn test_current_branch_returns_branch_name() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 获取当前分支
    let branch = GitBranchCommand::current_branch(None)?;

    // Assert: 验证返回分支名
    assert!(!branch.is_empty(), "Branch name should not be empty");
    assert_eq!(branch, "main", "Default branch should be 'main'");

    Ok(())
}

/// 测试创建分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::create_branch() 能够创建新分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建新分支
/// 3. 验证分支存在
///
/// ## 预期结果
/// - 分支创建成功并存在
#[test]
#[serial]
fn test_create_branch_creates_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let branch_name = "feature/test-branch";

    // Act: 创建新分支
    GitBranchCommand::create_branch(branch_name, None)?;

    // Assert: 验证分支存在
    let exists = GitBranchCommand::branch_exists_local(branch_name, None)?;
    assert!(exists, "Branch should exist after creation");

    Ok(())
}

/// 测试检查分支存在性
///
/// ## 测试目的
/// 验证 GitBranchCommand::branch_exists_local() 能够检查分支是否存在。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 检查存在的分支
/// 3. 检查不存在的分支
///
/// ## 预期结果
/// - 存在的分支返回 true，不存在的返回 false
#[test]
#[serial]
fn test_branch_exists_checks_correctly() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let existing_branch = "main";
    let nonexistent_branch = "nonexistent-branch-12345";

    // Act: 检查分支存在性
    let exists = GitBranchCommand::branch_exists_local(existing_branch, None)?;
    let not_exists = !GitBranchCommand::branch_exists_local(nonexistent_branch, None)?;

    // Assert: 验证结果
    assert!(exists, "Existing branch should return true");
    assert!(not_exists, "Nonexistent branch should return false");

    Ok(())
}

/// 测试切换分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::checkout_branch() 能够切换分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建新分支
/// 3. 切换到新分支
/// 4. 验证当前分支为新分支
///
/// ## 预期结果
/// - 成功切换到新分支
#[test]
#[serial]
fn test_checkout_branch_switches_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let branch_name = "feature/test-checkout";

    // Act: 创建并切换到新分支
    GitBranchCommand::checkout_branch(branch_name, true, None)?;

    // Assert: 验证当前分支为新分支
    let current = GitBranchCommand::current_branch(None)?;
    assert_eq!(
        current, branch_name,
        "Current branch should be the new branch"
    );

    Ok(())
}

/// 测试列出所有分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::list_branches() 能够列出所有本地分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建多个分支
/// 3. 列出所有分支
/// 4. 验证包含所有分支
///
/// ## 预期结果
/// - 返回所有分支的列表
#[test]
#[serial]
fn test_list_branches_returns_all_branches() -> Result<()> {
    // Arrange: 准备 Git 测试环境并创建多个分支
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    GitBranchCommand::create_branch("feature/branch1", None)?;
    GitBranchCommand::create_branch("feature/branch2", None)?;

    // Act: 列出所有分支
    let branches = GitBranchCommand::list_branches(None)?;

    // Assert: 验证包含所有分支
    assert!(
        branches.contains(&"feature/branch1".to_string()),
        "Should contain branch1"
    );
    assert!(
        branches.contains(&"feature/branch2".to_string()),
        "Should contain branch2"
    );
    assert!(
        branches.contains(&"main".to_string()),
        "Should contain main branch"
    );

    Ok(())
}
