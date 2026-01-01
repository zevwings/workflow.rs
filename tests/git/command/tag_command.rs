//! GitTagCommand 测试
//!
//! 测试 Tag 命令包装层的功能。

use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::GitTagCommand;

use crate::common::environments::GitTestEnv;
use crate::common::helpers::CurrentDirGuard;

/// 测试创建 tag
///
/// ## 测试目的
/// 验证 GitTagCommand::create_tag() 能够创建 tag。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建 tag
/// 3. 验证 tag 存在
///
/// ## 预期结果
/// - Tag 创建成功并存在
#[test]
#[serial]
fn test_create_tag_creates_tag() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let tag_name = "v1.0.0";

    // Act: 创建 tag
    GitTagCommand::create_tag(tag_name, None, None)?;

    // Assert: 验证 tag 存在
    let exists = GitTagCommand::tag_exists_local(tag_name, None)?;
    assert!(exists, "Tag should exist after creation");

    Ok(())
}

/// 测试列出本地 tag
///
/// ## 测试目的
/// 验证 GitTagCommand::list_local_tags() 能够列出所有本地 tag。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建多个 tag
/// 3. 列出所有 tag
/// 4. 验证包含所有 tag
///
/// ## 预期结果
/// - 返回所有 tag 的列表
#[test]
#[serial]
fn test_list_local_tags_returns_all_tags() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;

    // Act: 创建多个 tag
    GitTagCommand::create_tag("v1.0.0", None, None)?;
    GitTagCommand::create_tag("v1.1.0", None, None)?;

    // Act: 列出所有 tag
    let tags = GitTagCommand::list_local_tags(None)?;

    // Assert: 验证包含所有 tag
    assert!(
        tags.contains(&"v1.0.0".to_string()),
        "Should contain v1.0.0"
    );
    assert!(
        tags.contains(&"v1.1.0".to_string()),
        "Should contain v1.1.0"
    );

    Ok(())
}

/// 测试检查 tag 存在性
///
/// ## 测试目的
/// 验证 GitTagCommand::tag_exists_local() 能够检查 tag 是否存在。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建 tag
/// 3. 检查存在的 tag
/// 4. 检查不存在的 tag
///
/// ## 预期结果
/// - 存在的 tag 返回 true，不存在的返回 false
#[test]
#[serial]
fn test_tag_exists_checks_correctly() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let existing_tag = "v1.0.0";
    let nonexistent_tag = "v999.999.999";

    // Act: 创建 tag
    GitTagCommand::create_tag(existing_tag, None, None)?;

    // Act: 检查 tag 存在性
    let exists = GitTagCommand::tag_exists_local(existing_tag, None)?;
    let not_exists = !GitTagCommand::tag_exists_local(nonexistent_tag, None)?;

    // Assert: 验证结果
    assert!(exists, "Existing tag should return true");
    assert!(not_exists, "Nonexistent tag should return false");

    Ok(())
}

/// 测试删除本地 tag
///
/// ## 测试目的
/// 验证 GitTagCommand::delete_local() 能够删除本地 tag。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建 tag
/// 3. 删除 tag
/// 4. 验证 tag 被删除
///
/// ## 预期结果
/// - Tag 删除成功
#[test]
#[serial]
fn test_delete_local_removes_tag() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let tag_name = "v1.0.0";

    // Act: 创建 tag
    GitTagCommand::create_tag(tag_name, None, None)?;

    // Act: 删除 tag
    GitTagCommand::delete_local(tag_name, None)?;

    // Assert: 验证 tag 被删除
    let exists = GitTagCommand::tag_exists_local(tag_name, None)?;
    assert!(!exists, "Tag should not exist after deletion");

    Ok(())
}

/// 测试获取 tag 指向的 commit
///
/// ## 测试目的
/// 验证 GitTagCommand::get_tag_commit() 能够获取 tag 指向的 commit。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建 tag
/// 3. 获取 tag 指向的 commit
/// 4. 验证返回有效的 SHA
///
/// ## 预期结果
/// - 返回有效的 commit SHA
#[test]
#[serial]
fn test_get_tag_commit_returns_valid_sha() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let _dir_guard = CurrentDirGuard::new(env.path())?;
    let tag_name = "v1.0.0";

    // Act: 创建 tag
    GitTagCommand::create_tag(tag_name, None, None)?;

    // Act: 获取 tag 指向的 commit
    let commit_sha = GitTagCommand::get_tag_commit(tag_name, None)?;

    // Assert: 验证返回有效的 SHA（40 个字符的十六进制字符串）
    assert_eq!(commit_sha.len(), 40, "SHA should be 40 characters long");
    assert!(
        commit_sha.chars().all(|c| c.is_ascii_hexdigit()),
        "SHA should contain only hex digits"
    );

    Ok(())
}
