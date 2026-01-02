//! GitStashCommand 测试
//!
//! 测试 Stash 命令包装层的功能。

use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::{GitCommitCommand, GitStashCommand};

use crate::common::environments::GitTestEnv;

/// 测试保存 stash
///
/// ## 测试目的
/// 验证 GitStashCommand::stash_push() 能够保存未提交的更改到 stash。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建未提交的更改
/// 3. 保存到 stash
/// 4. 验证 stash 创建成功
///
/// ## 预期结果
/// - Stash 保存成功
#[test]
#[serial]
fn test_stash_push_saves_changes() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "test-stash.txt";

    // Act: 创建未提交的更改并保存到 stash（使用明确的路径）
    std::fs::write(repo_path.join(test_file), "stash content")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?; // 需要先暂存才能 stash
    GitStashCommand::stash_push(Some("Test stash"), Some(repo_path.as_path()))?;

    // Assert: 验证文件不在工作目录中（已被 stash）
    let has_changes = GitCommitCommand::has_changes(Some(repo_path.as_path()))?;
    assert!(
        !has_changes,
        "Working directory should be clean after stash"
    );

    // 验证 stash 列表不为空
    let stashes = GitStashCommand::list_stash(Some(repo_path.as_path()))?;
    assert!(!stashes.is_empty(), "Should have at least one stash");

    Ok(())
}

/// 测试列出 stash
///
/// ## 测试目的
/// 验证 GitStashCommand::list_stash() 能够列出所有 stash。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建多个 stash
/// 3. 列出所有 stash
/// 4. 验证返回所有 stash
///
/// ## 预期结果
/// - 返回所有 stash 的列表
#[test]
#[serial]
fn test_list_stash_returns_all_stashes() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 创建多个 stash（使用明确的路径）
    std::fs::write(repo_path.join("file1.txt"), "content1")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitStashCommand::stash_push(Some("Stash 1"), Some(repo_path.as_path()))?;

    std::fs::write(repo_path.join("file2.txt"), "content2")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitStashCommand::stash_push(Some("Stash 2"), Some(repo_path.as_path()))?;

    // Act: 列出所有 stash
    let stashes = GitStashCommand::list_stash(Some(repo_path.as_path()))?;

    // Assert: 验证返回所有 stash
    assert!(
        stashes.len() >= 2,
        "Should have at least 2 stashes, got: {}",
        stashes.len()
    );

    Ok(())
}

/// 测试应用 stash
///
/// ## 测试目的
/// 验证 GitStashCommand::stash_apply() 能够应用 stash 而不删除它。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建并保存 stash
/// 3. 应用 stash
/// 4. 验证 stash 仍然存在
///
/// ## 预期结果
/// - Stash 应用成功，且 stash 仍然存在
#[test]
#[serial]
fn test_stash_apply_applies_stash() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "test-apply.txt";
    let test_content = "apply content";

    // Act: 创建并保存 stash（使用明确的路径）
    std::fs::write(repo_path.join(test_file), test_content)?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitStashCommand::stash_push(Some("Test apply"), Some(repo_path.as_path()))?;

    // 记录 stash 数量
    let stash_count_before = GitStashCommand::list_stash(Some(repo_path.as_path()))?.len();

    // Act: 应用 stash
    GitStashCommand::stash_apply(None, Some(repo_path.as_path()))?;

    // Assert: 验证文件被应用回来
    let file_content = std::fs::read_to_string(repo_path.join(test_file))?;
    assert_eq!(
        file_content.trim(),
        test_content,
        "File content should match stashed content"
    );

    // 验证 stash 仍然存在
    let stash_count_after = GitStashCommand::list_stash(Some(repo_path.as_path()))?.len();
    assert_eq!(
        stash_count_before, stash_count_after,
        "Stash should still exist after apply"
    );

    Ok(())
}

/// 测试恢复 stash（pop）
///
/// ## 测试目的
/// 验证 GitStashCommand::stash_pop() 能够恢复 stash 并删除它。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建并保存 stash
/// 3. 恢复 stash（pop）
/// 4. 验证 stash 被删除
///
/// ## 预期结果
/// - Stash 恢复成功，且 stash 被删除
#[test]
#[serial]
fn test_stash_pop_applies_and_deletes_stash() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "test-pop.txt";
    let test_content = "pop content";

    // Act: 创建并保存 stash（使用明确的路径）
    std::fs::write(repo_path.join(test_file), test_content)?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitStashCommand::stash_push(Some("Test pop"), Some(repo_path.as_path()))?;

    let stash_count_before = GitStashCommand::list_stash(Some(repo_path.as_path()))?.len();

    // Act: 恢复 stash（pop）
    GitStashCommand::stash_pop(None, Some(repo_path.as_path()))?;

    // Assert: 验证文件被恢复
    let file_content = std::fs::read_to_string(repo_path.join(test_file))?;
    assert_eq!(
        file_content.trim(),
        test_content,
        "File content should match stashed content"
    );

    // 验证 stash 被删除
    let stash_count_after = GitStashCommand::list_stash(Some(repo_path.as_path()))?.len();
    assert_eq!(
        stash_count_after,
        stash_count_before - 1,
        "Stash should be deleted after pop"
    );

    Ok(())
}

/// 测试删除 stash
///
/// ## 测试目的
/// 验证 GitStashCommand::drop_stash() 能够删除 stash。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建并保存 stash
/// 3. 删除 stash
/// 4. 验证 stash 被删除
///
/// ## 预期结果
/// - Stash 删除成功
#[test]
#[serial]
fn test_drop_stash_deletes_stash() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 创建并保存 stash（使用明确的路径）
    std::fs::write(repo_path.join("test.txt"), "content")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitStashCommand::stash_push(Some("Test drop"), Some(repo_path.as_path()))?;

    let stash_count_before = GitStashCommand::list_stash(Some(repo_path.as_path()))?.len();

    // Act: 删除 stash
    GitStashCommand::drop_stash(None, Some(repo_path.as_path()))?;

    // Assert: 验证 stash 被删除
    let stash_count_after = GitStashCommand::list_stash(Some(repo_path.as_path()))?.len();
    assert_eq!(
        stash_count_after,
        stash_count_before - 1,
        "Stash should be deleted"
    );

    Ok(())
}

/// 测试检查冲突
///
/// ## 测试目的
/// 验证 GitStashCommand::check_conflicts() 能够检查冲突。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建文件并提交
/// 3. 修改文件
/// 4. 检查冲突（应该没有冲突）
/// 5. 验证返回 false
///
/// ## 预期结果
/// - 没有冲突时返回 false
#[test]
#[serial]
fn test_check_conflicts_returns_false_when_no_conflicts() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 创建文件并提交（使用明确的路径）
    std::fs::write(repo_path.join("conflict-test.txt"), "content")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitCommitCommand::commit("Initial commit", false, Some(repo_path.as_path()))?;

    // Act: 修改文件
    std::fs::write(repo_path.join("conflict-test.txt"), "modified content")?;

    // Act: 检查冲突
    let has_conflicts = GitStashCommand::check_conflicts(Some(repo_path.as_path()))?;

    // Assert: 验证没有冲突
    assert!(!has_conflicts, "Should not have conflicts in normal state");

    Ok(())
}
