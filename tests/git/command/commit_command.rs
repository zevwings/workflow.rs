//! GitCommitCommand 测试
//!
//! 测试提交命令包装层的功能。

use color_eyre::Result;
use serial_test::serial;
use workflow::git::commands::GitCommitCommand;

use crate::common::environments::GitTestEnv;

/// 测试检查 Git 状态
///
/// ## 测试目的
/// 验证 GitCommitCommand::status() 能够获取 Git 状态。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 获取 Git 状态（使用明确的路径避免全局配置问题）
/// 3. 验证返回状态信息
///
/// ## 预期结果
/// - 返回状态信息（可能是空字符串如果没有更改）
///
/// ## 注意事项
/// - 不使用 CurrentDirGuard，直接指定路径以避免全局工作目录切换导致的竞态条件
/// - 使用明确的路径参数确保 Git 命令在正确的目录中执行
#[test]
#[serial]
fn test_status_returns_status() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 获取 Git 状态（使用明确的路径，避免依赖全局工作目录）
    // 不使用 CurrentDirGuard，直接指定路径以避免可能的竞态条件
    let status = GitCommitCommand::status(Some(repo_path.as_path()))?;

    // Assert: 验证返回状态（初始状态应该为空）
    // 注意：GitTestEnv 创建了初始提交，所以工作目录应该是干净的
    assert!(
        status.trim().is_empty(),
        "Initial status should be empty, got: {}",
        status
    );

    Ok(())
}

/// 测试检查是否有更改
///
/// ## 测试目的
/// 验证 GitCommitCommand::has_changes() 能够检查是否有未提交的更改。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建新文件
/// 3. 检查是否有更改
///
/// ## 预期结果
/// - 创建文件后有更改，返回 true
#[test]
#[serial]
fn test_has_changes_detects_changes() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 创建新文件
    std::fs::write(repo_path.join("test.txt"), "test content")?;

    // Act: 检查是否有更改（使用明确的路径）
    let has_changes = GitCommitCommand::has_changes(Some(repo_path.as_path()))?;

    // Assert: 验证检测到更改
    assert!(has_changes, "Should detect changes after creating file");

    Ok(())
}

/// 测试暂存文件
///
/// ## 测试目的
/// 验证 GitCommitCommand::add() 能够暂存文件。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建新文件
/// 3. 暂存文件
/// 4. 验证文件已暂存
///
/// ## 预期结果
/// - 文件成功暂存
#[test]
#[serial]
fn test_add_stages_file() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "test-add.txt";

    // Act: 创建并暂存文件（使用明确的路径）
    std::fs::write(repo_path.join(test_file), "test content")?;
    GitCommitCommand::add(test_file, Some(repo_path.as_path()))?;

    // Assert: 验证文件已暂存（通过检查暂存区状态）
    let status = GitCommitCommand::status(Some(repo_path.as_path()))?;
    assert!(
        status.contains(test_file),
        "File should be staged, status: {}",
        status
    );

    Ok(())
}

/// 测试创建提交
///
/// ## 测试目的
/// 验证 GitCommitCommand::commit() 能够创建提交。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建并暂存文件
/// 3. 创建提交
/// 4. 验证提交成功
///
/// ## 预期结果
/// - 提交创建成功
#[test]
#[serial]
fn test_commit_creates_commit() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "test-commit.txt";

    // Act: 创建文件、暂存并提交（使用明确的路径）
    std::fs::write(repo_path.join(test_file), "test content")?;
    GitCommitCommand::add(test_file, Some(repo_path.as_path()))?;
    GitCommitCommand::commit("Test commit", false, Some(repo_path.as_path()))?;

    // Assert: 验证工作目录干净（没有未提交的更改）
    let has_changes = GitCommitCommand::has_changes(Some(repo_path.as_path()))?;
    assert!(
        !has_changes,
        "Working directory should be clean after commit"
    );

    Ok(())
}

/// 测试获取 HEAD SHA
///
/// ## 测试目的
/// 验证 GitCommitCommand::get_head_sha() 能够获取 HEAD 的 SHA。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 获取 HEAD SHA
/// 3. 验证返回有效的 SHA
///
/// ## 预期结果
/// - 返回有效的 commit SHA
#[test]
#[serial]
fn test_get_head_sha_returns_valid_sha() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();

    // Act: 获取 HEAD SHA（使用明确的路径）
    let sha = GitCommitCommand::get_head_sha(Some(repo_path.as_path()))?;

    // Assert: 验证返回有效的 SHA（40 个字符的十六进制字符串）
    assert_eq!(sha.len(), 40, "SHA should be 40 characters long");
    assert!(
        sha.chars().all(|c| c.is_ascii_hexdigit()),
        "SHA should contain only hex digits"
    );

    Ok(())
}

/// 测试修改最后一次提交
///
/// ## 测试目的
/// 验证 GitCommitCommand::amend() 能够修改最后一次提交。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建提交
/// 3. 修改文件并暂存
/// 4. 修改提交
/// 5. 验证提交被修改
///
/// ## 预期结果
/// - 提交修改成功
#[test]
#[serial]
fn test_amend_modifies_last_commit() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "amend-test.txt";

    // Act: 创建初始提交（使用明确的路径）
    std::fs::write(repo_path.join(test_file), "initial content")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitCommitCommand::commit("Initial commit", false, Some(repo_path.as_path()))?;

    let original_sha = GitCommitCommand::get_head_sha(Some(repo_path.as_path()))?;

    // Act: 修改文件并暂存
    std::fs::write(repo_path.join(test_file), "amended content")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;

    // Act: 修改提交
    GitCommitCommand::amend(Some("Amended commit"), Some(repo_path.as_path()))?;

    // Assert: 验证提交 SHA 已改变（因为提交被修改）
    let new_sha = GitCommitCommand::get_head_sha(Some(repo_path.as_path()))?;
    assert_ne!(
        original_sha, new_sha,
        "Commit SHA should change after amend"
    );

    // 验证文件内容已更新
    let content = std::fs::read_to_string(repo_path.join(test_file))?;
    assert_eq!(
        content.trim(),
        "amended content",
        "File should contain amended content"
    );

    Ok(())
}

/// 测试获取提交信息
///
/// ## 测试目的
/// 验证 GitCommitCommand::get_commit_info() 能够获取提交信息。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建提交
/// 3. 获取提交信息
/// 4. 验证返回的信息
///
/// ## 预期结果
/// - 返回有效的提交信息（消息、作者、日期）
#[test]
#[serial]
fn test_get_commit_info_returns_commit_details() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let commit_message = "Test commit for info";

    // Act: 创建提交（使用明确的路径）
    std::fs::write(repo_path.join("info-test.txt"), "content")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitCommitCommand::commit(commit_message, false, Some(repo_path.as_path()))?;

    let commit_sha = GitCommitCommand::get_head_sha(Some(repo_path.as_path()))?;

    // Act: 获取提交信息
    let (message, author, date) =
        GitCommitCommand::get_commit_info(&commit_sha, Some(repo_path.as_path()))?;

    // Assert: 验证返回的信息
    assert_eq!(message, commit_message, "Commit message should match");
    assert!(!author.is_empty(), "Author should not be empty");
    assert!(!date.is_empty(), "Date should not be empty");

    Ok(())
}

/// 测试获取差异内容
///
/// ## 测试目的
/// 验证 GitCommitCommand::get_diff() 能够获取差异内容。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 创建文件并暂存
/// 3. 获取暂存区差异
/// 4. 验证差异内容
///
/// ## 预期结果
/// - 返回有效的差异内容
#[test]
#[serial]
fn test_get_diff_returns_diff_content() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "diff-test.txt";
    let test_content = "diff content\nline 2";

    // Act: 创建文件并暂存（使用明确的路径）
    std::fs::write(repo_path.join(test_file), test_content)?;
    GitCommitCommand::add(test_file, Some(repo_path.as_path()))?;

    // Act: 获取暂存区差异
    let diff = GitCommitCommand::get_diff(true, Some(repo_path.as_path()))?;

    // Assert: 验证差异内容包含文件信息
    assert!(!diff.is_empty(), "Diff should not be empty");
    assert!(
        diff.contains(test_file) || diff.contains("diff-test"),
        "Diff should contain file name"
    );

    Ok(())
}

/// 测试获取工作区差异
///
/// ## 测试目的
/// 验证 GitCommitCommand::get_diff() 能够获取工作区差异。
///
/// ## 测试场景
/// 1. 准备 Git 测试环境
/// 2. 修改文件（不暂存）
/// 3. 获取工作区差异
/// 4. 验证差异内容
///
/// ## 预期结果
/// - 返回有效的差异内容
#[test]
#[serial]
fn test_get_diff_returns_working_directory_diff() -> Result<()> {
    // Arrange: 准备 Git 测试环境
    let env = GitTestEnv::new()?;
    let repo_path = env.path();
    let test_file = "working-diff.txt";

    // Act: 创建并提交文件（使用明确的路径）
    std::fs::write(repo_path.join(test_file), "original content")?;
    GitCommitCommand::add_all(Some(repo_path.as_path()))?;
    GitCommitCommand::commit("Initial commit", false, Some(repo_path.as_path()))?;

    // Act: 修改文件（不暂存）
    std::fs::write(repo_path.join(test_file), "modified content")?;

    // Act: 获取工作区差异
    let diff = GitCommitCommand::get_diff(false, Some(repo_path.as_path()))?;

    // Assert: 验证差异内容
    assert!(!diff.is_empty(), "Diff should not be empty");
    assert!(
        diff.contains("modified") || diff.contains("original"),
        "Diff should contain content changes"
    );

    Ok(())
}
