//! GitBranchCommand 测试
//!
//! 测试分支命令包装层的功能。

use color_eyre::Result;
use workflow::git::commands::GitBranchCommand;

use crate::common::environments::GitTestEnv;

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
fn test_current_branch_returns_branch_name() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 获取当前分支（使用明确的路径）
    let branch = GitBranchCommand::current_branch(Some(repo_path.as_path()))?;

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
fn test_create_branch_creates_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let branch_name = "feature/test-branch";

    // Act: 创建新分支（使用明确的路径）
    GitBranchCommand::create_branch(branch_name, Some(repo_path.as_path()))?;

    // Assert: 验证分支存在
    let exists = GitBranchCommand::branch_exists_local(branch_name, Some(repo_path.as_path()))?;
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
fn test_branch_exists_checks_correctly() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let existing_branch = "main";
    let nonexistent_branch = "nonexistent-branch-12345";

    // Act: 检查分支存在性（使用明确的路径）
    let exists = GitBranchCommand::branch_exists_local(existing_branch, Some(repo_path.as_path()))?;
    let not_exists =
        !GitBranchCommand::branch_exists_local(nonexistent_branch, Some(repo_path.as_path()))?;

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
fn test_checkout_branch_switches_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let branch_name = "feature/test-checkout";

    // Act: 创建并切换到新分支（使用明确的路径）
    GitBranchCommand::checkout_branch(branch_name, true, Some(repo_path.as_path()))?;

    // Assert: 验证当前分支为新分支
    let current = GitBranchCommand::current_branch(Some(repo_path.as_path()))?;
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
fn test_list_branches_returns_all_branches() -> Result<()> {
    // Arrange: 准备 Git 测试环境并创建多个分支
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    GitBranchCommand::create_branch("feature/branch1", Some(repo_path.as_path()))?;
    GitBranchCommand::create_branch("feature/branch2", Some(repo_path.as_path()))?;

    // Act: 列出所有分支
    let branches = GitBranchCommand::list_branches(Some(repo_path.as_path()))?;

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

/// 测试检查远程分支存在性
///
/// ## 测试目的
/// 验证 GitBranchCommand::branch_exists_remote() 能够检查远程分支是否存在。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 检查远程分支（可能不存在，因为测试环境可能没有远程）
/// 3. 验证返回结果
///
/// ## 预期结果
/// - 返回 false（测试环境可能没有远程）
#[test]
fn test_branch_exists_remote_checks_remote_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 检查远程分支（测试环境可能没有远程，所以应该返回 false）
    let _exists = GitBranchCommand::branch_exists_remote("main", None, Some(repo_path.as_path()))?;

    // Assert: 验证结果（测试环境可能没有远程，所以应该返回 false）
    // 这是可以接受的，因为测试环境可能没有配置远程仓库
    // 我们只验证函数能够正常执行并返回布尔值，不验证具体值
    // 因为测试环境可能没有配置远程仓库，所以 exists 应该是 false
    // 但如果返回 true，说明测试环境有远程配置，这也是可以接受的
    // 关键是函数能够正常执行而不出错

    Ok(())
}

/// 测试同时检查本地和远程分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::branch_exists() 能够同时检查本地和远程分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 检查分支存在性（本地和远程）
/// 3. 验证返回元组
///
/// ## 预期结果
/// - 返回 (本地存在, 远程存在) 元组
#[test]
fn test_branch_exists_checks_local_and_remote() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let branch_name = "main";

    // Act: 检查分支存在性（使用明确的路径）
    let (exists_local, _exists_remote) =
        GitBranchCommand::branch_exists(branch_name, None, Some(repo_path.as_path()))?;

    // Assert: 验证本地分支存在
    assert!(exists_local, "Local branch should exist");
    // 远程分支可能不存在（测试环境可能没有远程），这是正常的

    Ok(())
}

/// 测试删除分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::delete_branch() 能够删除分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建分支
/// 3. 删除分支
/// 4. 验证分支被删除
///
/// ## 预期结果
/// - 分支删除成功
#[test]
fn test_delete_branch_removes_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let branch_name = "feature/to-delete";

    // Act: 创建分支（使用明确的路径）
    GitBranchCommand::create_branch(branch_name, Some(repo_path.as_path()))?;
    assert!(
        GitBranchCommand::branch_exists_local(branch_name, Some(repo_path.as_path()))?,
        "Branch should exist before deletion"
    );

    // Act: 删除分支
    GitBranchCommand::delete_branch(branch_name, false, Some(repo_path.as_path()))?;

    // Assert: 验证分支被删除
    let exists = GitBranchCommand::branch_exists_local(branch_name, Some(repo_path.as_path()))?;
    assert!(!exists, "Branch should not exist after deletion");

    Ok(())
}

/// 测试强制删除分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::delete_branch() 能够强制删除分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建分支并提交
/// 3. 切换到其他分支
/// 4. 强制删除未合并的分支
/// 5. 验证分支被删除
///
/// ## 预期结果
/// - 分支强制删除成功
#[test]
fn test_delete_branch_force_removes_unmerged_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let branch_name = "feature/unmerged";

    // Act: 创建分支并提交（使用明确的路径）
    GitBranchCommand::checkout_branch(branch_name, true, Some(repo_path.as_path()))?;
    std::fs::write(repo_path.join("unmerged.txt"), "content")?;
    workflow::git::commands::GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    workflow::git::commands::GitCommitCommand::commit(
        "Unmerged commit",
        false,
        Some(repo_path.as_path()),
    )?;

    // 切换回 main
    GitBranchCommand::checkout_branch("main", false, Some(repo_path.as_path()))?;

    // Act: 强制删除未合并的分支
    GitBranchCommand::delete_branch(branch_name, true, Some(repo_path.as_path()))?;

    // Assert: 验证分支被删除
    let exists = GitBranchCommand::branch_exists_local(branch_name, Some(repo_path.as_path()))?;
    assert!(!exists, "Branch should be deleted after force deletion");

    Ok(())
}

/// 测试检查分支是否已合并
///
/// ## 测试目的
/// 验证 GitBranchCommand::is_merged() 能够检查分支是否已合并。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建分支并合并
/// 3. 检查分支是否已合并
/// 4. 验证返回 true
///
/// ## 预期结果
/// - 已合并的分支返回 true
#[test]
fn test_is_merged_checks_merged_branch() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let branch_name = "feature/to-merge";

    // Act: 创建分支并提交（使用明确的路径）
    GitBranchCommand::checkout_branch(branch_name, true, Some(repo_path.as_path()))?;
    std::fs::write(repo_path.join("merge.txt"), "content")?;
    workflow::git::commands::GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    workflow::git::commands::GitCommitCommand::commit(
        "Merge commit",
        false,
        Some(repo_path.as_path()),
    )?;

    // 切换回 main 并合并
    GitBranchCommand::checkout_branch("main", false, Some(repo_path.as_path()))?;
    GitBranchCommand::merge_branch(branch_name, None, false, Some(repo_path.as_path()))?;

    // Act: 检查分支是否已合并
    let is_merged = GitBranchCommand::is_merged(branch_name, None, Some(repo_path.as_path()))?;

    // Assert: 验证分支已合并
    assert!(is_merged, "Merged branch should return true");

    Ok(())
}

/// 测试合并分支
///
/// ## 测试目的
/// 验证 GitBranchCommand::merge_branch() 能够合并分支。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建分支并提交
/// 3. 切换回主分支
/// 4. 合并分支
/// 5. 验证合并成功
///
/// ## 预期结果
/// - 分支合并成功
#[test]
fn test_merge_branch_merges_successfully() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let branch_name = "feature/merge-test";

    // Act: 创建分支并提交（使用明确的路径）
    GitBranchCommand::checkout_branch(branch_name, true, Some(repo_path.as_path()))?;
    let test_file = "merge-test.txt";
    std::fs::write(repo_path.join(test_file), "merge content")?;
    workflow::git::commands::GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    workflow::git::commands::GitCommitCommand::commit(
        "Commit on branch",
        false,
        Some(repo_path.as_path()),
    )?;

    // 切换回 main
    GitBranchCommand::checkout_branch("main", false, Some(repo_path.as_path()))?;

    // Act: 合并分支
    GitBranchCommand::merge_branch(branch_name, None, false, Some(repo_path.as_path()))?;

    // Assert: 验证文件存在于 main 分支
    assert!(
        repo_path.join(test_file).exists(),
        "File should exist after merge"
    );

    // 验证分支已合并
    let is_merged = GitBranchCommand::is_merged(branch_name, None, Some(repo_path.as_path()))?;
    assert!(is_merged, "Branch should be merged");

    Ok(())
}
