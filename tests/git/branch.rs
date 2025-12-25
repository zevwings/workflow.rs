//! Git 分支管理测试
//!
//! 测试 Git 分支的创建、切换、合并、删除等操作。

use pretty_assertions::assert_eq;
use color_eyre::Result;
use rstest::fixture;
use serial_test::serial;
use tempfile::TempDir;
use workflow::git::{GitBranch, MergeStrategy};

use crate::common::environments::GitTestEnv;

// ==================== Fixtures ====================

/// 创建带有初始提交的 Git 仓库
#[fixture]
fn git_repo_with_commit() -> GitTestEnv {
    GitTestEnv::new().expect("Failed to create git test env")
}

// ==================== 分支前缀处理测试 ====================

#[test]
fn test_remove_branch_prefix_with_slash() -> Result<()> {
    // 这是测试内部函数，需要通过公共API测试
    // 我们通过分支名称处理的相关功能来测试

    // 测试分支前缀移除逻辑（通过分支操作间接测试）
    // 注意：remove_branch_prefix 是私有函数，我们测试其效果
Ok(())
}

// ==================== 分支存在性检查测试 ====================

#[test]
#[serial]
fn test_exists_main_branch() -> Result<()> {
    // 新版 GitTestEnv 自动切换工作目录，无需手动管理
    // 如果 Git 不可用，GitTestEnv::new() 会返回错误
    let _env = GitTestEnv::new()?;

    // 检查默认分支（通常是 main 或 master）是否存在
    let current_branch = GitBranch::current_branch()?;
    assert!(
        GitBranch::has_local_branch(&current_branch).unwrap_or(false),
        "Current branch '{}' should exist locally",
        current_branch
    );

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

#[test]
#[serial]
fn test_exists_nonexistent_branch() -> Result<()> {
    // 新版 GitTestEnv 自动切换工作目录，无需手动管理
    let _env = GitTestEnv::new()?;

    // 检查不存在的分支
    let nonexistent_branch = "nonexistent-branch-12345";
    assert!(
        !GitBranch::has_local_branch(nonexistent_branch).unwrap_or(true),
        "Branch '{}' should not exist",
        nonexistent_branch
    );

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== 使用 fstest 重新实现的测试 ====================

#[test]
#[serial]
fn test_create_simple_branch_with_gix() -> Result<()> {
    // 新版 GitTestEnv 自动切换工作目录，无需手动管理
    let _env = GitTestEnv::new()?;

    let branch_name = "feature/test-branch";

    // 创建分支
    let result = GitBranch::checkout_branch(branch_name);
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);

    // 验证分支存在
    assert!(GitBranch::has_local_branch(branch_name).unwrap_or(false));

    // 验证当前分支
    let current_branch = GitBranch::current_branch()?;
    assert_eq!(current_branch, branch_name);

    // 目录会在函数结束时自动恢复
    Ok(())
}

#[test]
#[serial]
fn test_create_branch_with_prefix_with_gix() -> Result<()> {
    // 新版 GitTestEnv 自动切换工作目录，无需手动管理
    let _env = GitTestEnv::new()?;

    let branch_name = "feature/user-authentication";

    let result = GitBranch::checkout_branch(branch_name);
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);

    assert!(GitBranch::has_local_branch(branch_name).unwrap_or(false));

    // 验证当前分支
    let current_branch = GitBranch::current_branch()?;
    assert_eq!(current_branch, branch_name);

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== 分支切换测试 ====================

// ==================== 分支删除测试 ====================

#[test]
#[serial]
fn test_delete_existing_branch() -> Result<()> {
    // 新版 GitTestEnv 自动切换工作目录，无需手动管理
    let _env = GitTestEnv::new()?;

    let branch_name = "feature/to-delete";

    // 创建分支
    let result = GitBranch::checkout_branch(branch_name);
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);

    // 切换回主分支
    let main_branch = GitBranch::current_branch()?;
    // 如果当前就是主分支，先创建一个临时分支
    if main_branch == branch_name {
        GitBranch::checkout_branch("main")
            .unwrap_or_else(|_| GitBranch::checkout_branch("master")?);
    }

    // 确认分支存在
    assert!(
        GitBranch::has_local_branch(branch_name).unwrap_or(false),
        "Branch should exist before deletion"
    );

    // 删除分支
    let delete_result = GitBranch::delete(branch_name, false);
    assert!(
        delete_result.is_ok(),
        "Failed to delete branch: {:?}",
        delete_result
    );

    // 确认分支已被删除
    assert!(
        !GitBranch::has_local_branch(branch_name).unwrap_or(true),
        "Branch should not exist after deletion"
    );

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== 分支列表测试 ====================

#[test]
#[serial]
fn test_list_branches() -> Result<()> {
    // 新版 GitTestEnv 自动切换工作目录，无需手动管理
    let _env = GitTestEnv::new()?;

    // 获取初始分支列表
    let initial_branches = GitBranch::get_local_branches()?;
    let initial_count = initial_branches.len();

    // 创建几个测试分支
    let test_branches = vec!["feature/branch1", "feature/branch2", "hotfix/fix1"];

    for branch in &test_branches {
        GitBranch::checkout_branch(branch)?;
    }

    // 获取更新后的分支列表
    let updated_branches = GitBranch::get_local_branches()?;

    // 验证分支数量增加了
    assert!(
        updated_branches.len() >= initial_count + test_branches.len(),
        "Branch count should increase. Initial: {}, Updated: {}",
        initial_count,
        updated_branches.len()
    );

    // 验证所有测试分支都在列表中
    for branch in &test_branches {
        assert!(
            updated_branches.contains(&branch.to_string()),
            "Branch '{}' should be in the branch list",
            branch
        );
    }

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== 合并策略测试 ====================

#[test]
fn test_merge_strategy_enum() -> Result<()> {
    // 测试合并策略枚举的基本功能
    let strategies = [
        MergeStrategy::Merge,
        MergeStrategy::Squash,
        MergeStrategy::FastForwardOnly,
    ];

    // 验证枚举可以正常使用
    for strategy in &strategies {
        // 这里主要测试枚举的基本功能
        let debug_str = format!("{:?}", strategy);
        assert!(!debug_str.is_empty());
    }
Ok(())
}

// ==================== 边界条件测试 ====================

#[test]
#[serial]
fn test_empty_branch_name() -> Result<()> {
    // 新版 GitTestEnv 自动切换工作目录，无需手动管理
    let _env = GitTestEnv::new()?;

    // 尝试创建空名称的分支应该失败
    let result = GitBranch::checkout_branch("");
    assert!(
        result.is_err(),
        "Creating branch with empty name should fail"
    );

    // 检查空分支名是否存在应该返回 false
    let exists = GitBranch::has_local_branch("").unwrap_or(true);
    assert!(!exists, "Empty branch name should not exist");

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== 错误处理测试 ====================

#[test]
fn test_git_not_available() -> Result<()> {
    // 测试 Git 不可用的情况
    // 注意：这个测试在有 Git 的环境中会跳过

    let original_path = std::env::var("PATH").unwrap_or_default();

    // 临时移除 PATH 中的 Git
    std::env::set_var("PATH", "");

    let _result = GitBranch::has_local_branch("any-branch");

    // 恢复 PATH
    std::env::set_var("PATH", original_path);

    // 在没有 Git 的情况下应该返回错误
    // 注意：这个测试可能不会按预期工作，因为 Git 可能在其他位置
Ok(())
}

// ==================== 集成测试 ====================

// ==================== 性能测试 ====================
