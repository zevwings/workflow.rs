//! Git 提交管理测试
//!
//! 测试 Git 提交状态检查、暂存操作和提交创建功能。

use gix;
use pretty_assertions::assert_eq;
use rstest::fixture;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;
use workflow::git::GitCommit;

// 辅助函数：创建带有初始提交的临时 Git 仓库
fn setup_git_repo() -> (TempDir, std::path::PathBuf) {
    // 保存原始目录在创建临时目录之前
    let original_dir = std::env::current_dir().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 使用 gix 初始化仓库
    let _repo = gix::init(temp_path).unwrap();

    // 设置基本的 Git 配置
    let config_path = temp_path.join(".git").join("config");
    std::fs::write(
        &config_path,
        "[user]\n\tname = Test User\n\temail = test@example.com\n",
    )
    .unwrap();

    // 创建初始文件
    let readme_path = temp_path.join("README.md");
    std::fs::write(&readme_path, "# Test Repository\n").unwrap();

    // 使用命令行 git 来创建初始提交（在临时目录中执行）
    let add_output = std::process::Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add file");

    if !add_output.status.success() {
        panic!(
            "Git add failed: {}",
            String::from_utf8_lossy(&add_output.stderr)
        );
    }

    let commit_output = std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to create commit");

    if !commit_output.status.success() {
        panic!(
            "Git commit failed: {}",
            String::from_utf8_lossy(&commit_output.stderr)
        );
    }

    (temp_dir, original_dir)
}

// ==================== Fixtures ====================

/// 创建带有初始提交的 Git 仓库
#[fixture]
fn git_repo_with_commit() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // 在临时目录中执行 Git 操作，而不是切换当前工作目录
    let init_output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to init git repo");

    if !init_output.status.success() {
        panic!(
            "Git init failed: {}",
            String::from_utf8_lossy(&init_output.stderr)
        );
    }

    // 配置 Git 用户
    let name_output = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user name");

    if !name_output.status.success() {
        panic!(
            "Git config user.name failed: {}",
            String::from_utf8_lossy(&name_output.stderr)
        );
    }

    let email_output = std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

    if !email_output.status.success() {
        panic!(
            "Git config user.email failed: {}",
            String::from_utf8_lossy(&email_output.stderr)
        );
    }

    // 创建初始提交
    let readme_path = temp_path.join("README.md");
    fs::write(&readme_path, "# Test Repository").expect("Failed to write file");

    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to commit");

    temp_dir
}

/// 创建干净的 Git 仓库（无提交）
#[fixture]
fn clean_git_repo() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // 在临时目录中执行 Git 操作，而不是切换当前工作目录
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to init git repo");

    // 配置 Git 用户
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user name");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

    temp_dir
}

// ==================== 工作树状态检查测试 ====================

// ==================== 使用 gix 重新实现的工作树状态测试 ====================

#[test]
#[serial]
fn test_worktree_status_clean_with_gix() {
    let (temp_dir, original_dir) = setup_git_repo();

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 测试干净的工作树状态
    let result = GitCommit::get_worktree_status();
    assert!(
        result.is_ok(),
        "Failed to get worktree status: {:?}",
        result
    );

    let status = result.unwrap();

    // 干净的工作树应该没有未跟踪或修改的文件
    assert_eq!(status.untracked_count, 0, "Expected no untracked files");
    assert_eq!(status.modified_count, 0, "Expected no modified files");
    assert_eq!(status.staged_count, 0, "Expected no staged files");

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
}

// ==================== 更改检查测试 ====================

// ==================== 使用 gix 重新实现的更改检查测试 ====================

#[test]
#[serial]
fn test_has_changes_clean_repo_with_gix() {
    let (temp_dir, original_dir) = setup_git_repo();

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 干净的仓库应该没有更改
    // 使用 get_worktree_status 检查是否有变更
    let status = GitCommit::get_worktree_status();
    let result =
        status.map(|s| s.modified_count > 0 || s.staged_count > 0 || s.untracked_count > 0);
    assert!(result.is_ok());
    assert!(!result.unwrap(), "Clean repo should have no changes");

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_has_changes_with_untracked_files_with_gix() {
    let (temp_dir, original_dir) = setup_git_repo();

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 创建未跟踪文件
    std::fs::write(temp_dir.path().join("new_file.txt"), "New content").unwrap();

    // 使用 get_worktree_status 检查是否有变更
    let status = GitCommit::get_worktree_status();
    let result =
        status.map(|s| s.modified_count > 0 || s.staged_count > 0 || s.untracked_count > 0);
    assert!(result.is_ok());
    assert!(
        result.unwrap(),
        "Repo with untracked files should have changes"
    );

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_has_changes_with_modified_files_with_gix() {
    let (temp_dir, original_dir) = setup_git_repo();

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 修改现有文件（README.md 在 setup 中已经创建）
    std::fs::write(temp_dir.path().join("README.md"), "# Updated README").unwrap();

    // 使用 get_worktree_status 检查是否有变更
    let status = GitCommit::get_worktree_status();
    let result =
        status.map(|s| s.modified_count > 0 || s.staged_count > 0 || s.untracked_count > 0);
    assert!(result.is_ok());
    assert!(
        result.unwrap(),
        "Repo with modified files should have changes"
    );

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
}

// ==================== 暂存操作测试 ====================

#[test]
#[serial]
fn test_stage_all_changes() {
    let (temp_dir, original_dir) = setup_git_repo();

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 创建一些新文件
    std::fs::write(temp_dir.path().join("new_file1.txt"), "Content 1").unwrap();
    std::fs::write(temp_dir.path().join("new_file2.txt"), "Content 2").unwrap();

    // 修改现有文件
    std::fs::write(temp_dir.path().join("README.md"), "# Updated README").unwrap();

    // 暂存所有更改
    let result = GitCommit::add_all();
    assert!(result.is_ok(), "Failed to stage all changes: {:?}", result);

    // 验证文件已暂存
    let status = GitCommit::get_worktree_status().unwrap();
    assert!(status.staged_count > 0, "Should have staged files");

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_stage_specific_file() {
    let (temp_dir, original_dir) = setup_git_repo();

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 创建测试文件
    let test_file = "specific_file.txt";
    std::fs::write(temp_dir.path().join(test_file), "Specific content").unwrap();

    // 暂存特定文件
    let result = GitCommit::add_files(&[test_file.to_string()]);
    assert!(
        result.is_ok(),
        "Failed to stage specific file: {:?}",
        result
    );

    // 验证文件已暂存
    let status = GitCommit::get_worktree_status().unwrap();
    assert!(
        status.staged_count > 0,
        "Should have staged the specific file"
    );

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
}

// ==================== 提交创建测试 ====================

// ==================== 提交信息获取测试 ====================

#[test]
#[serial]
fn test_get_latest_commit_info() {
    let (temp_dir, original_dir) = setup_git_repo();

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 获取最新提交信息
    let commit_info = GitCommit::get_last_commit_info();
    assert!(
        commit_info.is_ok(),
        "Failed to get latest commit info: {:?}",
        commit_info
    );

    let info = commit_info.unwrap();

    // 验证提交信息的基本字段
    assert!(!info.sha.is_empty(), "Commit SHA should not be empty");
    assert!(
        !info.message.is_empty(),
        "Commit message should not be empty"
    );
    assert!(!info.author.is_empty(), "Commit author should not be empty");
    assert!(!info.date.is_empty(), "Commit date should not be empty");

    // 验证 SHA 格式（应该是 40 个字符的十六进制）
    assert_eq!(
        info.sha.len(),
        40,
        "Commit SHA should be 40 characters long"
    );
    assert!(
        info.sha.chars().all(|c| c.is_ascii_hexdigit()),
        "Commit SHA should be hexadecimal"
    );

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
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

// ==================== 边界条件测试 ====================

// ==================== 错误处理测试 ====================

// ==================== 使用 gix 重新实现的错误处理测试 ====================

#[test]
#[serial] // 确保这个测试串行运行，避免并发问题
fn test_operations_outside_git_repo_with_container() {
    use tempfile::tempdir;

    // 在非 Git 目录中测试操作
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().expect("Failed to get current dir");

    // 切换到非Git目录
    std::env::set_current_dir(&temp_dir).expect("Failed to change dir");

    // 确保这不是一个 Git 仓库，并且父目录也不是
    assert!(
        !temp_dir.path().join(".git").exists(),
        "Temp directory should not be a git repository"
    );

    // 验证当前工作目录确实是临时目录（处理 macOS 路径规范化）
    let current_dir = std::env::current_dir().expect("Failed to get current dir");
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

    // 测试 add_all 是否正确返回错误
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

    // 测试 get_last_commit_info 是否正确返回错误
    let commit_info_result = GitCommit::get_last_commit_info();
    assert!(
        commit_info_result.is_err(),
        "get_last_commit_info should fail in non-git directory, but got: {:?}",
        commit_info_result
    );

    // 恢复原始目录
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
}

// ==================== 集成测试 ====================

// ==================== 性能测试 ====================
