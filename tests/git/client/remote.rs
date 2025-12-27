//! GitRemote 封装测试
//!
//! 测试 GitRemote 的基础功能，包括：
//! - 获取远程 URL
//! - 逃生舱方法

use color_eyre::Result;
use tempfile::TempDir;
use workflow::git::GitRepository;

/// 测试查找 origin 远程仓库
#[test]
fn test_find_origin_remote() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库（使用 git2）
    let mut repo = GitRepository::init(repo_path, None)?;

    // 添加 origin 远程（使用 git2 API）
    repo.as_inner_mut()
        .remote("origin", "https://github.com/test/repo.git")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

    // 打开仓库并查找 origin 远程
    let mut repo = GitRepository::open_at(repo_path)?;
    let remote = repo.find_origin_remote()?;

    // 验证能够获取 URL
    let url = remote.url();
    assert_eq!(url, Some("https://github.com/test/repo.git"));

    Ok(())
}

/// 测试查找不存在的远程仓库
#[test]
fn test_find_remote_not_found() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库（使用 git2）
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

    // 初始化 Git 仓库（使用 git2）
    let mut repo = GitRepository::init(repo_path, None)?;

    // 添加 origin 远程（使用 git2 API）
    repo.as_inner_mut()
        .remote("origin", "git@github.com:test/repo.git")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

    // 打开仓库并查找 origin 远程
    let mut repo = GitRepository::open_at(repo_path)?;
    let remote = repo.find_origin_remote()?;

    // 验证能够获取 URL
    let url = remote.url();
    assert_eq!(url, Some("git@github.com:test/repo.git"));

    Ok(())
}

/// 测试逃生舱方法
#[test]
fn test_as_inner() -> Result<()> {
    // 创建一个临时目录并初始化 Git 仓库
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库（使用 git2）
    let mut repo = GitRepository::init(repo_path, None)?;

    // 添加 origin 远程（使用 git2 API）
    repo.as_inner_mut()
        .remote("origin", "https://github.com/test/repo.git")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to add remote: {}", e))?;

    // 打开仓库并查找 origin 远程
    let mut repo = GitRepository::open_at(repo_path)?;
    let mut remote = repo.find_origin_remote()?;

    // 测试逃生舱方法
    let _inner_ref = remote.as_inner();
    let _inner_mut_ref = remote.as_inner_mut();

    // 验证能够访问底层 Remote（不应该 panic）
    // 如果上面的调用没有 panic，测试就通过了

    Ok(())
}
