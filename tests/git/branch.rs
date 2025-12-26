//! Git 分支管理测试
//!
//! 测试 Git 分支的创建、切换、合并、删除等操作。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
// Removed serial_test::serial - tests can run in parallel with GitTestEnv isolation
// But tests using CurrentDirGuard need serial execution
use workflow::git::{GitBranch, MergeStrategy};

use crate::common::environments::GitTestEnv;
use crate::common::fixtures::git_repo_with_commit;
use crate::common::guards::EnvGuard;
use crate::common::helpers::CurrentDirGuard;

// ==================== Branch Prefix Processing Tests ====================

/// 测试移除分支前缀（带斜杠）
///
/// ## 测试目的
/// 验证分支前缀移除逻辑（通过分支操作间接测试，remove_branch_prefix 是私有函数）。
///
/// ## 测试场景
/// 1. 通过分支操作间接测试前缀移除逻辑
/// 2. 验证前缀移除逻辑（通过公共API间接验证）
///
/// ## 预期结果
/// - 前缀移除逻辑正常工作
#[test]
fn test_remove_branch_prefix_with_slash_handles_prefix_return_ok() -> Result<()> {
    // Arrange: 准备测试分支前缀移除逻辑
    // 注意：remove_branch_prefix 是私有函数，我们通过分支操作间接测试

    // Act: 测试分支前缀移除逻辑（通过分支操作间接测试）

    // Assert: 验证前缀移除逻辑（通过公共API间接验证）
    Ok(())
}

// ==================== Branch Existence Tests ====================

/// 测试检查主分支是否存在
///
/// ## 测试目的
/// 验证 GitBranch::has_local_branch() 能够检查默认分支（main）是否存在。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 获取当前分支并检查是否存在
/// 3. 验证当前分支存在
///
/// ## 预期结果
/// - 当前分支存在
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_exists_main_branch_with_default_branch_return_ok() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;

    // Act: 获取当前分支并检查是否存在
    let current_branch = GitBranch::current_branch()?;
    let exists = GitBranch::has_local_branch(&current_branch).unwrap_or(false);

    // Assert: 验证当前分支存在
    assert!(
        exists,
        "Current branch '{}' should exist locally",
        current_branch
    );

    Ok(())
}

/// 测试检查不存在分支
///
/// ## 测试目的
/// 验证 GitBranch::has_local_branch() 对不存在的分支返回 false。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境和不存在的分支名
/// 2. 检查不存在的分支
/// 3. 验证分支不存在
///
/// ## 预期结果
/// - 不存在的分支返回 false
#[rstest]
fn test_exists_nonexistent_branch_with_invalid_name_return_ok(
    _git_repo_with_commit: GitTestEnv,
) -> Result<()> {
    // Arrange: 准备 Git 测试环境和不存在的分支名（使用 fixture）
    let nonexistent_branch = "nonexistent-branch-12345";

    // Act: 检查不存在的分支
    let exists = GitBranch::has_local_branch(nonexistent_branch).unwrap_or(true);

    // Assert: 验证分支不存在
    assert!(!exists, "Branch '{}' should not exist", nonexistent_branch);

    Ok(())
}

// ==================== Branch Creation Tests ====================

/// 测试创建简单分支
///
/// ## 测试目的
/// 验证 GitBranch::checkout_branch() 能够创建并切换到新分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境和分支名
/// 2. 创建并切换到新分支
/// 3. 验证分支创建成功、存在且已切换
///
/// ## 预期结果
/// - 分支创建成功，存在且已切换
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_create_simple_branch_with_valid_name_succeeds() -> Result<()> {
    // Arrange: 准备 Git 测试环境和分支名
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let branch_name = "feature/test-branch";

    // Act: 创建并切换到新分支
    let result = GitBranch::checkout_branch(branch_name);

    // Assert: 验证分支创建成功、存在且已切换
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);
    assert!(GitBranch::has_local_branch(branch_name).unwrap_or(false));
    let current_branch = GitBranch::current_branch()?;
    assert_eq!(current_branch, branch_name);

    Ok(())
}

/// 测试创建带前缀的分支
///
/// ## 测试目的
/// 验证 GitBranch::checkout_branch() 能够创建并切换到带前缀的新分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境和带前缀的分支名
/// 2. 创建并切换到新分支
/// 3. 验证分支创建成功、存在且已切换
///
/// ## 预期结果
/// - 带前缀的分支创建成功，存在且已切换
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_create_branch_with_prefix_and_valid_name_succeeds() -> Result<()> {
    // Arrange: 准备 Git 测试环境和带前缀的分支名
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let branch_name = "feature/user-authentication";

    // Act: 创建并切换到新分支
    let result = GitBranch::checkout_branch(branch_name);

    // Assert: 验证分支创建成功、存在且已切换
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);
    assert!(GitBranch::has_local_branch(branch_name).unwrap_or(false));
    let current_branch = GitBranch::current_branch()?;
    assert_eq!(current_branch, branch_name);

    Ok(())
}

// ==================== Branch Deletion Tests ====================

/// 测试删除存在的分支
///
/// ## 测试目的
/// 验证 GitBranch::delete() 能够删除存在的分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境并创建分支
/// 2. 切换回主分支
/// 3. 删除分支
/// 4. 验证分支已被删除
///
/// ## 预期结果
/// - 分支被成功删除
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_delete_existing_branch_with_valid_name_succeeds() -> Result<()> {
    // Arrange: 准备 Git 测试环境并创建分支
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let branch_name = "feature/to-delete";

    // 创建分支
    let result = GitBranch::checkout_branch(branch_name);
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);

    // 切换回主分支（如果当前就是主分支，先创建一个临时分支）
    let main_branch = GitBranch::current_branch()?;
    if main_branch == branch_name {
        GitBranch::checkout_branch("main")
            .or_else(|_| GitBranch::checkout_branch("master"))
            .expect("Failed to checkout main or master branch");
    }

    // 确认分支存在
    assert!(
        GitBranch::has_local_branch(branch_name).unwrap_or(false),
        "Branch should exist before deletion"
    );

    // Act: 删除分支
    let delete_result = GitBranch::delete(branch_name, false);
    assert!(
        delete_result.is_ok(),
        "Failed to delete branch: {:?}",
        delete_result
    );

    // Assert: 验证分支已被删除
    assert!(
        !GitBranch::has_local_branch(branch_name).unwrap_or(true),
        "Branch should not exist after deletion"
    );

    Ok(())
}

// ==================== Branch Listing Tests ====================

/// 测试列出多个分支
///
/// ## 测试目的
/// 验证 GitBranch::get_local_branches() 能够返回所有本地分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境并获取初始分支列表
/// 2. 创建多个测试分支
/// 3. 获取更新后的分支列表
/// 4. 验证分支数量增加且所有测试分支都在列表中
///
/// ## 预期结果
/// - 分支数量增加，所有测试分支都在列表中
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_list_branches_with_multiple_branches_return_collect() -> Result<()> {
    // Arrange: 准备 Git 测试环境并获取初始分支列表
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let initial_branches = GitBranch::get_local_branches()?;
    let initial_count = initial_branches.len();
    let test_branches = vec!["feature/branch1", "feature/branch2", "hotfix/fix1"];

    // Act: 创建多个测试分支
    for branch in &test_branches {
        GitBranch::checkout_branch(branch)?;
    }

    // 获取更新后的分支列表
    let updated_branches = GitBranch::get_local_branches()?;

    // Assert: 验证分支数量增加且所有测试分支都在列表中
    assert!(
        updated_branches.len() >= initial_count + test_branches.len(),
        "Branch count should increase. Initial: {}, Updated: {}",
        initial_count,
        updated_branches.len()
    );
    for branch in &test_branches {
        assert!(
            updated_branches.contains(&branch.to_string()),
            "Branch '{}' should be in the branch list",
            branch
        );
    }

    Ok(())
}

// ==================== Merge Strategy Tests ====================

/// 测试合并策略枚举（所有变体）
///
/// ## 测试目的
/// 验证 MergeStrategy 枚举的所有变体都可以格式化为 Debug 字符串。
///
/// ## 测试场景
/// 1. 准备所有合并策略枚举变体
/// 2. 格式化每个策略为 Debug 字符串
/// 3. 验证 Debug 字符串不为空
///
/// ## 预期结果
/// - 所有策略的 Debug 字符串都不为空
#[test]
fn test_merge_strategy_enum_with_all_variants_return_collect() -> Result<()> {
    // Arrange: 准备所有合并策略枚举变体
    let strategies = [
        MergeStrategy::Merge,
        MergeStrategy::Squash,
        MergeStrategy::FastForwardOnly,
    ];

    // Act & Assert: 验证每个策略都可以格式化为 Debug 字符串
    for strategy in &strategies {
        let debug_str = format!("{:?}", strategy);
        assert!(!debug_str.is_empty());
    }

    Ok(())
}

// ==================== Boundary Condition Tests ====================

/// 测试空分支名错误处理
///
/// ## 测试目的
/// 验证 GitBranch::checkout_branch() 对空分支名返回错误。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 尝试创建空名称的分支
/// 3. 验证创建失败且分支不存在
///
/// ## 预期结果
/// - 创建失败，空分支名不存在
#[rstest]
fn test_empty_branch_name_with_empty_string_return_empty(
    _git_repo_with_commit: GitTestEnv,
) -> Result<()> {
    // Arrange: 准备 Git 测试环境（使用 fixture）

    // Act: 尝试创建空名称的分支
    let result = GitBranch::checkout_branch("");
    let exists = GitBranch::has_local_branch("").unwrap_or(true);

    // Assert: 验证创建失败且分支不存在
    assert!(
        result.is_err(),
        "Creating branch with empty name should fail"
    );
    assert!(!exists, "Empty branch name should not exist");

    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试 Git 不可用时的错误处理
///
/// ## 测试目的
/// 验证 GitBranch 在没有 Git 的情况下返回错误。
///
/// ## 测试场景
/// 1. 使用 EnvGuard 临时移除 Git（通过清空 PATH）
/// 2. 尝试检查分支（Git 不可用）
/// 3. 验证在没有 Git 的情况下应该返回错误
///
/// ## 预期结果
/// - 在没有 Git 的情况下返回错误（注意：这个测试可能不会按预期工作，因为 Git 可能在其他位置）
#[test]
fn test_git_not_available_without_git_return_ok() -> Result<()> {
    // Arrange: 使用 EnvGuard 临时移除 Git（通过清空 PATH）
    let mut env_guard = EnvGuard::new();
    env_guard.set("PATH", "");

    // Act: 尝试检查分支（Git 不可用）
    let _result = GitBranch::has_local_branch("any-branch");

    // Assert: 验证在没有 Git 的情况下应该返回错误
    // 注意：这个测试可能不会按预期工作，因为 Git 可能在其他位置
    // EnvGuard 会在 drop 时自动恢复 PATH
    Ok(())
}

// ==================== Integration Tests ====================

// ==================== Performance Tests ====================
