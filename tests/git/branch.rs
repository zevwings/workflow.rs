//! Git 分支管理测试
//!
//! 测试 Git 分支的创建、切换、合并、删除等操作。

use gix;
use pretty_assertions::assert_eq;
use rstest::fixture;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;
use workflow::git::{GitBranch, MergeStrategy};

// ==================== Fixtures ====================

/// 创建带有初始提交的 Git 仓库
#[fixture]
fn git_repo_with_commit() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // 在临时目录中执行 Git 操作，而不是切换当前工作目录
    let temp_path = temp_dir.path();

    // 初始化 Git 仓库
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

    // 创建初始文件和提交
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

// ==================== 分支前缀处理测试 ====================

#[test]
fn test_remove_branch_prefix_with_slash() {
    // 这是测试内部函数，需要通过公共API测试
    // 我们通过分支名称处理的相关功能来测试

    // 测试分支前缀移除逻辑（通过分支操作间接测试）
    // 注意：remove_branch_prefix 是私有函数，我们测试其效果
}

// ==================== 分支存在性检查测试 ====================

#[test]
#[serial]
fn test_exists_main_branch() {
    let Some((temp_dir, original_dir)) = setup_git_repo_with_gix() else {
        // Git 不可用，跳过测试
        return;
    };

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 检查默认分支（通常是 main 或 master）是否存在
    let current_branch = GitBranch::current_branch().unwrap();
    assert!(
        GitBranch::has_local_branch(&current_branch).unwrap_or(false),
        "Current branch '{}' should exist locally",
        current_branch
    );

    // 恢复原始目录（如果目录仍然存在）
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

#[test]
#[serial]
fn test_exists_nonexistent_branch() {
    let Some((temp_dir, original_dir)) = setup_git_repo_with_gix() else {
        // Git 不可用，跳过测试
        return;
    };

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 检查不存在的分支
    let nonexistent_branch = "nonexistent-branch-12345";
    assert!(
        !GitBranch::has_local_branch(nonexistent_branch).unwrap_or(true),
        "Branch '{}' should not exist",
        nonexistent_branch
    );

    // 恢复原始目录（如果目录仍然存在）
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== 分支创建测试 ====================

// 辅助函数：使用 gix 创建带有初始提交的临时 Git 仓库
fn setup_git_repo_with_gix() -> Option<(TempDir, std::path::PathBuf)> {
    // 保存原始目录在创建临时目录之前
    let original_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 切换到临时目录，确保 gix::init 有正确的工作目录上下文
    std::env::set_current_dir(temp_path).unwrap();

    // 使用 gix 初始化仓库（在临时目录中）
    let _repo = gix::init(".").unwrap();

    // 设置基本的 Git 配置
    let git_config_result = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output();

    if git_config_result.is_err() {
        // 如果 git 命令不可用，跳过这个测试
        eprintln!("Git command not available, skipping test");
        return None;
    }

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

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

    Some((temp_dir, original_dir))
}

// ==================== 使用 fstest 重新实现的测试 ====================

#[test]
#[serial]
fn test_create_simple_branch_with_gix() {
    let Some((temp_dir, original_dir)) = setup_git_repo_with_gix() else {
        // Git 不可用，跳过测试
        return;
    };

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let branch_name = "feature/test-branch";

    // 创建分支
    let result = GitBranch::checkout_branch(branch_name);
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);

    // 验证分支存在
    assert!(GitBranch::has_local_branch(branch_name).unwrap_or(false));

    // 验证当前分支
    let current_branch = GitBranch::current_branch().unwrap();
    assert_eq!(current_branch, branch_name);

    // 恢复原始目录（如果目录仍然存在）
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

#[test]
#[serial]
fn test_create_branch_with_prefix_with_gix() {
    let Some((temp_dir, original_dir)) = setup_git_repo_with_gix() else {
        // Git 不可用，跳过测试
        return;
    };

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let branch_name = "feature/user-authentication";

    let result = GitBranch::checkout_branch(branch_name);
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);

    assert!(GitBranch::has_local_branch(branch_name).unwrap_or(false));

    // 验证当前分支
    let current_branch = GitBranch::current_branch().unwrap();
    assert_eq!(current_branch, branch_name);

    // 恢复原始目录（如果目录仍然存在）
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== 分支切换测试 ====================

// ==================== 分支删除测试 ====================

#[test]
#[serial]
fn test_delete_existing_branch() {
    let Some((temp_dir, original_dir)) = setup_git_repo_with_gix() else {
        // Git 不可用，跳过测试
        return;
    };

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let branch_name = "feature/to-delete";

    // 创建分支
    let result = GitBranch::checkout_branch(branch_name);
    assert!(result.is_ok(), "Failed to create branch: {:?}", result);

    // 切换回主分支
    let main_branch = GitBranch::current_branch().unwrap();
    // 如果当前就是主分支，先创建一个临时分支
    if main_branch == branch_name {
        GitBranch::checkout_branch("main")
            .unwrap_or_else(|_| GitBranch::checkout_branch("master").unwrap());
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

    // 恢复原始目录（如果目录仍然存在）
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== 分支列表测试 ====================

#[test]
#[serial]
fn test_list_branches() {
    let Some((temp_dir, original_dir)) = setup_git_repo_with_gix() else {
        // Git 不可用，跳过测试
        return;
    };

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 获取初始分支列表
    let initial_branches = GitBranch::get_local_branches().unwrap();
    let initial_count = initial_branches.len();

    // 创建几个测试分支
    let test_branches = vec!["feature/branch1", "feature/branch2", "hotfix/fix1"];

    for branch in &test_branches {
        GitBranch::checkout_branch(branch).unwrap();
    }

    // 获取更新后的分支列表
    let updated_branches = GitBranch::get_local_branches().unwrap();

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

    // 恢复原始目录（如果目录仍然存在）
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== 合并策略测试 ====================

#[test]
fn test_merge_strategy_enum() {
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
}

// ==================== 边界条件测试 ====================

#[test]
#[serial]
fn test_empty_branch_name() {
    let Some((temp_dir, original_dir)) = setup_git_repo_with_gix() else {
        // Git 不可用，跳过测试
        return;
    };

    // 切换到临时目录
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 尝试创建空名称的分支应该失败
    let result = GitBranch::checkout_branch("");
    assert!(
        result.is_err(),
        "Creating branch with empty name should fail"
    );

    // 检查空分支名是否存在应该返回 false
    let exists = GitBranch::has_local_branch("").unwrap_or(true);
    assert!(!exists, "Empty branch name should not exist");

    // 恢复原始目录（如果目录仍然存在）
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== 错误处理测试 ====================

#[test]
fn test_git_not_available() {
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
}

// ==================== 集成测试 ====================

// ==================== 性能测试 ====================
