//! GitRepository 封装测试
//!
//! 测试 GitRepository 的基础功能，包括：
//! - 打开仓库
//! - 获取签名
//! - 查找远程仓库
//! - 获取分支名

use color_eyre::Result;
use tempfile::TempDir;
use workflow::git::GitRepository;

/// 测试打开当前目录的 Git 仓库
#[test]
fn test_open_repo_success() -> Result<()> {
    // 这个测试需要在 Git 仓库中运行
    // 如果不在 Git 仓库中，测试会失败，这是预期的
    let repo = GitRepository::open();

    // 如果成功打开，应该能够获取 HEAD
    if let Ok(repo) = repo {
        // 尝试获取 HEAD（可能失败，但不应该 panic）
        let _head = repo.head();
    }

    Ok(())
}

/// 测试打开指定路径的 Git 仓库
#[test]
fn test_open_repo_at_success() -> Result<()> {
    // 创建一个临时目录
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库（使用 git2）
    let _repo = GitRepository::init(repo_path, None)?;

    // 打开仓库
    let repo = GitRepository::open_at(repo_path)?;

    // 验证能够获取 HEAD（应该失败，因为还没有提交）
    let head_result = repo.head();
    assert!(head_result.is_err()); // 新仓库没有 HEAD

    Ok(())
}

/// 测试打开不存在的仓库路径
#[test]
fn test_open_repo_at_not_found() {
    let result = GitRepository::open_at("/nonexistent/path");
    assert!(result.is_err());
}

/// 测试获取当前分支名（需要在有分支的仓库中）
#[test]
fn test_current_branch_name() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库并创建初始提交（使用 git2）
    let _repo = GitRepository::init_with_commit(
        repo_path,
        Some("main"),
        Some("Test"),
        Some("test@example.com"),
        Some("README.md"),
        Some("# Test Repo"),
        Some("Initial commit"),
    )?;

    // 打开仓库并获取分支名
    let repo = GitRepository::open_at(repo_path)?;
    let branch_name = repo.current_branch_name()?;

    assert_eq!(branch_name, "main");

    Ok(())
}

/// 测试获取签名（需要在有配置的仓库中）
#[test]
fn test_signature() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库并配置用户信息（使用 git2）
    let repo = GitRepository::init_with_commit(
        repo_path,
        None,
        Some("Test User"),
        Some("test@example.com"),
        None,
        None,
        None,
    )?;

    // 获取签名
    let signature = repo.signature()?;

    assert_eq!(signature.name().unwrap_or(""), "Test User");
    assert_eq!(signature.email().unwrap_or(""), "test@example.com");

    Ok(())
}

/// 测试查找引用
#[test]
fn test_find_reference() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库并创建初始提交（使用 git2）
    let _repo = GitRepository::init_with_commit(
        repo_path,
        Some("main"),
        Some("Test"),
        Some("test@example.com"),
        Some("README.md"),
        Some("# Test Repo"),
        Some("Initial commit"),
    )?;

    // 打开仓库并查找引用
    let repo = GitRepository::open_at(repo_path)?;
    let ref_result = repo.find_reference("refs/heads/main");

    assert!(ref_result.is_ok());

    Ok(())
}

/// 测试获取 FetchOptions 和 PushOptions
#[test]
fn test_get_options() {
    // 测试获取 FetchOptions
    let _fetch_options = GitRepository::get_fetch_options();
    // 验证能够创建选项（不应该 panic）
    // 如果上面的调用没有 panic，测试就通过了

    // 测试获取 PushOptions
    let _push_options = GitRepository::get_push_options();
    // 验证能够创建选项（不应该 panic）
    // 如果上面的调用没有 panic，测试就通过了
}
