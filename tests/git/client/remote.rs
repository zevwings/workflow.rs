//! GitRemote 封装测试
//!
//! 测试 GitRemote 的基础功能，包括：
//! - 获取远程 URL
//! - 远程仓库操作

use color_eyre::Result;
use tempfile::TempDir;
use workflow::git::GitRepository;

/// 测试查找 origin 远程仓库
#[test]
fn test_find_origin_remote() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库
    let _repo = GitRepository::init(repo_path, None)?;

    // 添加 origin 远程（使用 GitRepoCommand）
    use workflow::git::commands::GitRepoCommand;
    GitRepoCommand::add_remote(
        "origin",
        "https://github.com/test/repo.git",
        Some(repo_path),
    )
    .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

    // 打开仓库并查找 origin 远程
    let mut repo = GitRepository::open_at(repo_path)?;
    let remote = repo.find_origin_remote()?;

    // 验证能够获取 URL
    let url = remote.url()?;
    assert_eq!(url, "https://github.com/test/repo.git");

    Ok(())
}

/// 测试查找不存在的远程仓库
#[test]
fn test_find_remote_not_found() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库
    let _repo = GitRepository::init(repo_path, None)?;

    // 打开仓库并尝试查找不存在的远程
    let mut repo = GitRepository::open_at(repo_path)?;
    let result = repo.find_remote("nonexistent");

    assert!(result.is_err());

    Ok(())
}

/// 测试获取远程 URL
#[test]
fn test_remote_url() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库
    let _repo = GitRepository::init(repo_path, None)?;

    // 添加 origin 远程（使用 GitRepoCommand）
    use workflow::git::commands::GitRepoCommand;
    GitRepoCommand::add_remote("origin", "git@github.com:test/repo.git", Some(repo_path))
        .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

    // 打开仓库并查找 origin 远程
    let mut repo = GitRepository::open_at(repo_path)?;
    let remote = repo.find_origin_remote()?;

    // 验证能够获取 URL
    // 注意：Git 命令本身支持简写 SSH URL，所以返回的可能是原始格式或规范化后的格式
    let url = remote.url()?;
    // URL 可能是原始格式或规范化后的格式，都接受
    assert!(
        url == "git@github.com:test/repo.git" || url == "ssh://git@github.com/test/repo.git",
        "URL should be in expected format, got: {}",
        url
    );

    Ok(())
}

/// 测试远程 URL 获取
#[test]
fn test_remote_url_access() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库
    let _repo = GitRepository::init(repo_path, None)?;

    // 添加 origin 远程（使用 GitRepoCommand）
    use workflow::git::commands::GitRepoCommand;
    GitRepoCommand::add_remote(
        "origin",
        "https://github.com/test/repo.git",
        Some(repo_path),
    )
    .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

    // 打开仓库并查找 origin 远程
    let mut repo = GitRepository::open_at(repo_path)?;
    let remote = repo.find_origin_remote()?;

    // 测试获取 URL（不应该 panic）
    let url = remote.url()?;
    assert_eq!(url, "https://github.com/test/repo.git");

    Ok(())
}
