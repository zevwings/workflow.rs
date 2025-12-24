//! Base Util Path 模块测试
//!
//! 测试路径操作工具的核心功能，包括 PathAccess 结构体。

use std::fs;
use tempfile::TempDir;
use workflow::base::fs::path::PathAccess;

#[test]
fn test_path_access_new() {
    let _path_access = PathAccess::new("test/path");
    // 验证可以创建 PathAccess
}

#[test]
fn test_path_access_new_pathbuf() {
    let path = std::path::PathBuf::from("test/path");
    let _path_access = PathAccess::new(path);
    // 验证可以创建 PathAccess
}

#[test]
fn test_path_access_exists() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let existing_path = temp_dir.path().join("existing.txt");
    fs::write(&existing_path, "test")?;

    let path_access = PathAccess::new(&existing_path);
    assert!(path_access.exists());

    let non_existing_path = temp_dir.path().join("non_existing.txt");
    let path_access = PathAccess::new(&non_existing_path);
    assert!(!path_access.exists());

    Ok(())
}

#[test]
fn test_path_access_is_file() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test")?;

    let path_access = PathAccess::new(&file_path);
    assert!(path_access.is_file());

    let dir_path = temp_dir.path().join("subdir");
    fs::create_dir(&dir_path)?;
    let path_access = PathAccess::new(&dir_path);
    assert!(!path_access.is_file());

    Ok(())
}

#[test]
fn test_path_access_is_dir() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("subdir");
    fs::create_dir(&dir_path)?;

    let path_access = PathAccess::new(&dir_path);
    assert!(path_access.is_dir());

    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test")?;
    let path_access = PathAccess::new(&file_path);
    assert!(!path_access.is_dir());

    Ok(())
}

#[test]
fn test_path_access_ensure_dir_exists() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let new_dir = temp_dir.path().join("new/dir/path");

    let path_access = PathAccess::new(&new_dir);
    path_access.ensure_dir_exists()?;
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());

    Ok(())
}

#[test]
fn test_path_access_ensure_dir_exists_existing() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let existing_dir = temp_dir.path().join("existing");
    fs::create_dir_all(&existing_dir)?;

    let path_access = PathAccess::new(&existing_dir);
    // 应该不会失败，即使目录已存在
    path_access.ensure_dir_exists()?;
    assert!(existing_dir.exists());

    Ok(())
}

#[test]
fn test_path_access_ensure_parent_exists() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("parent/dir/file.txt");

    let path_access = PathAccess::new(&file_path);
    path_access.ensure_parent_exists()?;
    assert!(file_path.parent().unwrap().exists());
    assert!(file_path.parent().unwrap().is_dir());

    Ok(())
}

#[test]
fn test_path_access_ensure_parent_exists_no_parent() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    // 根路径没有父目录
    let root_path = temp_dir.path();

    let path_access = PathAccess::new(root_path);
    // 应该不会失败，即使没有父目录
    path_access.ensure_parent_exists()?;

    Ok(())
}

#[test]
fn test_path_access_read_dir_safe() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir(&dir_path)?;

    // 创建一些文件
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    let path_access = PathAccess::new(&dir_path);
    let entries = path_access.read_dir_safe()?;

    // 应该包含至少3个条目（2个文件 + 1个目录）
    assert!(entries.len() >= 3);

    Ok(())
}

#[test]
fn test_path_access_read_dir_safe_nonexistent() {
    let path_access = PathAccess::new("/nonexistent/path/that/does/not/exist");
    let result = path_access.read_dir_safe();
    assert!(result.is_err());
}

#[test]
fn test_path_access_read_dir_safe_file() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test")?;

    let path_access = PathAccess::new(&file_path);
    let result = path_access.read_dir_safe();
    // 尝试读取文件作为目录应该失败
    assert!(result.is_err());

    Ok(())
}
