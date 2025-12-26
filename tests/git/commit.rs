//! Git 提交管理测试
//!
//! 测试 Git 提交状态检查、暂存操作和提交创建功能。
//!
//! ## 测试策略
//!
//! - 测试函数返回 `Result<()>`，使用 `?` 运算符处理错误
//! - Fixture 函数中的 `expect()` 保留（fixture 失败应该panic）
//! - 使用 `GitTestEnv` 确保测试隔离和自动清理（支持并行执行）

use color_eyre::Result;
use pretty_assertions::assert_eq;
use serial_test::serial;
// Removed serial_test::serial - tests can run in parallel with GitTestEnv isolation
// But tests using CurrentDirGuard need serial execution
use workflow::git::GitCommit;

use crate::common::fixtures::git_repo_with_commit;
use crate::common::helpers::CurrentDirGuard;

// ==================== Worktree Status Check Tests ====================

// ==================== Worktree Status Tests (Reimplemented with gix) ====================

/// 测试干净的Git工作树状态检测
///
/// ## 测试目的
/// 验证 `GitCommit::get_worktree_status()` 能够正确检测干净的工作树状态
/// （没有未跟踪、修改或暂存的文件）。
///
/// ## 测试场景
/// 1. 创建临时Git仓库（包含初始提交）
/// 2. 切换到临时仓库目录
/// 3. 获取工作树状态
/// 4. 验证所有计数器为0
/// 5. 恢复原始工作目录
///
/// ## 技术细节
/// - 使用 `GitTestEnv` 确保测试隔离（支持并行执行）
/// - 使用临时目录进行隔离测试
/// - 使用 `gix` 库（纯Rust的Git实现）而非git2
/// - 自动恢复原始工作目录（即使测试失败）
///
/// ## 预期结果
/// - `untracked_count == 0`：无未跟踪文件
/// - `modified_count == 0`：无修改文件
/// - `staged_count == 0`：无暂存文件
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_worktree_status_clean_with_gix_return_ok() -> Result<()> {
    // 切换到测试仓库目录
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;

    // Arrange: 准备测试干净的工作树状态
    let status = GitCommit::get_worktree_status()?;

    // 干净的工作树应该没有未跟踪或修改的文件
    assert_eq!(status.untracked_count, 0, "Expected no untracked files");
    assert_eq!(status.modified_count, 0, "Expected no modified files");
    assert_eq!(status.staged_count, 0, "Expected no staged files");

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== Change Detection Tests ====================

// ==================== Change Detection Tests (Reimplemented with gix) ====================

/// 测试检查干净仓库是否有更改（使用gix）
///
/// ## 测试目的
/// 验证 `GitCommit::get_worktree_status()` 在干净仓库（无修改、无暂存、无未跟踪文件）中返回正确的状态。
///
/// ## 测试场景
/// 1. 创建Git测试环境（GitTestEnv自动创建干净仓库）
/// 2. 调用 `get_worktree_status()` 检查工作树状态
/// 3. 验证返回的状态表示无更改
///
/// ## 预期结果
/// - 返回Ok状态
/// - modified_count、staged_count、untracked_count 都为0
/// - 表示仓库干净，无更改
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_has_changes_clean_repo_with_gix_return_ok() -> Result<()> {
    // 切换到测试仓库目录
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;

    // 干净的仓库应该没有更改
    // 使用 get_worktree_status 检查是否有变更
    let status = GitCommit::get_worktree_status();
    let result =
        status.map(|s| s.modified_count > 0 || s.staged_count > 0 || s.untracked_count > 0);
    assert!(result.is_ok());
    assert!(!result?, "Clean repo should have no changes");

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

/// 测试检查包含未跟踪文件的仓库是否有更改（使用gix）
///
/// ## 测试目的
/// 验证 `GitCommit::get_worktree_status()` 能够正确检测未跟踪文件的存在。
///
/// ## 测试场景
/// 1. 创建Git测试环境
/// 2. 创建未跟踪文件（new_file.txt）
/// 3. 调用 `get_worktree_status()` 检查工作树状态
/// 4. 验证返回的状态表示有更改
///
/// ## 预期结果
/// - 返回Ok状态
/// - untracked_count > 0 或 modified_count > 0 或 staged_count > 0
/// - 表示仓库有未跟踪文件，存在更改
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_has_changes_with_untracked_files_with_gix_return_collect() -> Result<()> {
    // 切换到测试仓库目录
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let env = &git_repo_with_commit;

    // 创建未跟踪文件
    std::fs::write(env.path().join("new_file.txt"), "New content")?;

    // 使用 get_worktree_status 检查是否有变更
    let status = GitCommit::get_worktree_status();
    let result =
        status.map(|s| s.modified_count > 0 || s.staged_count > 0 || s.untracked_count > 0);
    assert!(result.is_ok());
    assert!(result?, "Repo with untracked files should have changes");

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

/// 测试检查包含已修改文件的仓库是否有更改（使用gix）
///
/// ## 测试目的
/// 验证 `GitCommit::get_worktree_status()` 能够正确检测已修改文件的存在。
///
/// ## 测试场景
/// 1. 创建Git测试环境（GitTestEnv自动创建README.md）
/// 2. 修改现有文件（README.md）
/// 3. 调用 `get_worktree_status()` 检查工作树状态
/// 4. 验证返回的状态表示有更改
///
/// ## 预期结果
/// - 返回Ok状态
/// - modified_count > 0 或 staged_count > 0 或 untracked_count > 0
/// - 表示仓库有已修改文件，存在更改
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_has_changes_with_modified_files_with_gix_return_collect() -> Result<()> {
    // 切换到测试仓库目录
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let env = &git_repo_with_commit;

    // 修改现有文件（README.md 在 GitTestEnv::new() 中已经创建）
    std::fs::write(env.path().join("README.md"), "# Updated README")?;

    // 使用 get_worktree_status 检查是否有变更
    let status = GitCommit::get_worktree_status();
    let result =
        status.map(|s| s.modified_count > 0 || s.staged_count > 0 || s.untracked_count > 0);
    assert!(result.is_ok());
    assert!(result?, "Repo with modified files should have changes");

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== Staging Operations Tests ====================

/// 测试暂存所有更改（多个文件）
///
/// ## 测试目的
/// 验证 `GitCommit::add_all()` 方法能够正确暂存工作树中的所有更改（包括多个文件）。
///
/// ## 测试场景
/// 1. 创建Git测试环境
/// 2. 创建多个新文件（new_file1.txt, new_file2.txt）
/// 3. 修改现有文件（README.md）
/// 4. 调用 `add_all()` 暂存所有更改
/// 5. 验证所有文件都被暂存
///
/// ## 预期结果
/// - `add_all()` 返回Ok
/// - `get_worktree_status()` 显示 staged_count > 0
/// - 所有更改的文件都被暂存
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_stage_all_changes_with_multiple_files_stages_all_return_collect() -> Result<()> {
    // Arrange: 准备 Git 测试环境并创建多个文件
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let env = &git_repo_with_commit;
    std::fs::write(env.path().join("new_file1.txt"), "Content 1")?;
    std::fs::write(env.path().join("new_file2.txt"), "Content 2")?;

    // Act: 暂存所有变更
    std::fs::write(env.path().join("README.md"), "# Updated README")?;

    // 暂存所有更改
    let result = GitCommit::add_all();
    assert!(result.is_ok(), "Failed to stage all changes: {:?}", result);

    // Assert: 验证文件已暂存
    let status = GitCommit::get_worktree_status()?;
    assert!(status.staged_count > 0, "Should have staged files");

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

/// 测试暂存特定文件
///
/// ## 测试目的
/// 验证 `GitCommit::add_files()` 方法能够正确暂存指定的单个文件。
///
/// ## 测试场景
/// 1. 创建Git测试环境
/// 2. 创建测试文件（specific_file.txt）
/// 3. 调用 `add_files()` 暂存特定文件
/// 4. 验证该文件被暂存
///
/// ## 预期结果
/// - `add_files()` 返回Ok
/// - `get_worktree_status()` 显示 staged_count > 0
/// - 指定的文件被暂存
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_stage_specific_file_return_ok() -> Result<()> {
    // 切换到测试仓库目录
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;
    let env = &git_repo_with_commit;

    // 创建测试文件
    let test_file = "specific_file.txt";
    std::fs::write(env.path().join(test_file), "Specific content")?;

    // 暂存特定文件
    let result = GitCommit::add_files(&[test_file.to_string()]);
    assert!(
        result.is_ok(),
        "Failed to stage specific file: {:?}",
        result
    );

    // Assert: 验证文件已暂存
    let status = GitCommit::get_worktree_status()?;
    assert!(
        status.staged_count > 0,
        "Should have staged the specific file"
    );

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== Commit Creation Tests ====================

// ==================== Commit Info Retrieval Tests ====================

/// 测试获取最新提交信息
///
/// ## 测试目的
/// 验证 `GitCommit::get_last_commit_info()` 方法能够正确获取仓库中最新提交的详细信息。
///
/// ## 测试场景
/// 1. 创建Git测试环境（GitTestEnv自动创建初始提交）
/// 2. 调用 `get_last_commit_info()` 获取最新提交信息
/// 3. 验证返回的提交信息包含所有必需字段
///
/// ## 预期结果
/// - 返回Ok，包含CommitInfo结构
/// - SHA不为空，长度为40个字符，为十六进制格式
/// - message、author、date字段都不为空
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial]
fn test_get_latest_commit_info_return_ok() -> Result<()> {
    // 切换到测试仓库目录
    let git_repo_with_commit = git_repo_with_commit();
    let _dir_guard = CurrentDirGuard::new(git_repo_with_commit.path())?;

    // 获取最新提交信息
    let commit_info = GitCommit::get_last_commit_info();
    assert!(
        commit_info.is_ok(),
        "Failed to get latest commit info: {:?}",
        commit_info
    );

    let info = commit_info?;

    // Assert: 验证提交信息的基本字段
    assert!(!info.sha.is_empty(), "Commit SHA should not be empty");
    assert!(
        !info.message.is_empty(),
        "Commit message should not be empty"
    );
    assert!(!info.author.is_empty(), "Commit author should not be empty");
    assert!(!info.date.is_empty(), "Commit date should not be empty");

    // Assert: 验证 SHA 格式（应该是 40 个字符的十六进制）
    assert_eq!(
        info.sha.len(),
        40,
        "Commit SHA should be 40 characters long"
    );
    assert!(
        info.sha.chars().all(|c| c.is_ascii_hexdigit()),
        "Commit SHA should be hexadecimal"
    );

    // GitTestEnv 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== WorktreeStatus 结构体测试 ====================

// 注意：WorktreeStatus 没有 Default trait，跳过此测试
// #[test]
// fn test_worktree_status_default() {
//     // WorktreeStatus 不支持 Default trait
// }

// 注意：WorktreeStatus 没有 has_changes 方法，跳过此测试
// #[test]
// fn test_worktree_status_has_changes() {
//     // WorktreeStatus 不支持 has_changes 方法
// }

// ==================== Boundary Condition Tests ====================

// ==================== Error Handling Tests ====================

/// 测试在非Git仓库中执行操作（错误处理）
///
/// ## 测试目的
/// 验证Git操作在非Git仓库目录中能够正确返回错误，而不是panic或产生未定义行为。
///
/// ## 测试场景
/// 1. 创建临时目录（非Git仓库）
/// 2. 切换到该目录
/// 3. 尝试执行Git操作（如 `get_worktree_status()`）
/// 4. 验证返回适当的错误
///
/// ## 预期结果
/// - Git操作返回Err
/// - 错误消息明确指示当前目录不是Git仓库
/// - 不会panic或产生未定义行为
///
/// ## 注意事项
/// - 此测试使用 `CurrentDirGuard` 切换全局工作目录，需要串行执行以避免并行测试时的竞态条件
#[test]
#[serial_test::serial]
fn test_operations_outside_git_repo_with_container() -> Result<()> {
    // 注意：这里使用 tempfile::tempdir 而不是 GitTestEnv，因为我们需要测试非 Git 仓库的情况
    use tempfile::tempdir;

    // 在非 Git 目录中测试操作
    // 注意：这里不使用 GitTestEnv，因为我们需要测试非 Git 仓库的情况
    let temp_dir =
        tempdir().map_err(|e| color_eyre::eyre::eyre!("Failed to create temp dir: {}", e))?;

    // 切换到非Git目录（使用RAII确保恢复）
    let _dir_guard = crate::common::helpers::CurrentDirGuard::new(&temp_dir)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to change dir: {}", e))?;

    // 确保这不是一个 Git 仓库，并且父目录也不是
    assert!(
        !temp_dir.path().join(".git").exists(),
        "Temp directory should not be a git repository"
    );

    // Assert: 验证当前工作目录确实是临时目录（处理 macOS 路径规范化）
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));
    let current_canonical = current_dir.canonicalize().unwrap_or(current_dir);
    let temp_canonical =
        temp_dir.path().canonicalize().unwrap_or_else(|_| temp_dir.path().to_path_buf());
    assert_eq!(
        current_canonical, temp_canonical,
        "Current directory should be the temp directory"
    );

    // 所有 Git 操作都应该失败
    assert!(
        GitCommit::get_worktree_status().is_err(),
        "get_worktree_status should fail in non-git directory"
    );

    // Arrange: 准备测试 add_all 是否正确返回错误
    let add_result = GitCommit::add_all();
    assert!(
        add_result.is_err(),
        "add_all should fail in non-git directory, but got: {:?}",
        add_result
    );

    assert!(
        GitCommit::commit("test", false).is_err(),
        "commit should fail in non-git directory"
    );

    // Arrange: 准备测试 get_last_commit_info 是否正确返回错误
    let commit_info_result = GitCommit::get_last_commit_info();
    assert!(
        commit_info_result.is_err(),
        "get_last_commit_info should fail in non-git directory, but got: {:?}",
        commit_info_result
    );

    // 目录会在函数结束时自动恢复
    Ok(())
}

// ==================== Integration Tests ====================

// ==================== Performance Tests ====================
