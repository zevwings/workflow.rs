//! Base Util Directory 模块测试
//!
//! 测试目录操作工具的核心功能，包括 DirectoryWalker 结构体。

use pretty_assertions::assert_eq;
use std::fs;
use tempfile::TempDir;
use workflow::base::util::directory::DirectoryWalker;

#[test]
fn test_directory_walker_new() {
    let _walker = DirectoryWalker::new("test/path");
    // 验证可以创建 DirectoryWalker
}

#[test]
fn test_directory_walker_new_pathbuf() {
    let path = std::path::PathBuf::from("test/path");
    let _walker = DirectoryWalker::new(path);
    // 验证可以创建 DirectoryWalker
}

#[test]
fn test_directory_walker_ensure_exists() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let new_dir = temp_dir.path().join("new/deep/nested/directory");

    let walker = DirectoryWalker::new(&new_dir);
    walker.ensure_exists()?;
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());

    Ok(())
}

#[test]
fn test_directory_walker_ensure_exists_existing() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let existing_dir = temp_dir.path().join("existing");
    fs::create_dir_all(&existing_dir)?;

    let walker = DirectoryWalker::new(&existing_dir);
    // 应该不会失败，即使目录已存在
    walker.ensure_exists()?;
    assert!(existing_dir.exists());

    Ok(())
}

#[test]
fn test_directory_walker_ensure_exists_multiple_times() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test/dir");

    let walker = DirectoryWalker::new(&dir_path);
    // 多次调用应该都成功
    walker.ensure_exists()?;
    walker.ensure_exists()?;
    walker.ensure_exists()?;

    assert!(dir_path.exists());
    assert!(dir_path.is_dir());

    Ok(())
}

#[test]
fn test_directory_walker_list_dirs() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir(dir_path.join("subdir1"))?;
    fs::create_dir(dir_path.join("subdir2"))?;
    fs::write(dir_path.join("file.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_dirs()?;
    // 应该包含根目录和子目录
    assert!(dirs.len() >= 3);

    Ok(())
}

#[test]
fn test_directory_walker_list_files() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;
    assert_eq!(files.len(), 2);

    Ok(())
}

#[test]
fn test_directory_walker_find_files() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("test_file.txt"), "content")?;
    fs::write(dir_path.join("other_file.txt"), "content")?;
    fs::write(dir_path.join("test.log"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("test")?;
    assert_eq!(files.len(), 2); // test_file.txt 和 test.log

    Ok(())
}

#[test]
fn test_directory_walker_list_direct_dirs() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir(dir_path.join("subdir1"))?;
    fs::create_dir(dir_path.join("subdir2"))?;
    fs::write(dir_path.join("file.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_direct_dirs()?;
    assert_eq!(dirs.len(), 2);

    Ok(())
}

#[test]
fn test_directory_walker_list_direct_files() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_direct_files()?;
    assert_eq!(files.len(), 2);

    Ok(())
}

#[test]
fn test_directory_walker_ensure_parent_exists() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("parent/dir/file.txt");

    let walker = DirectoryWalker::new(temp_dir.path());
    walker.ensure_parent_exists(&file_path)?;
    assert!(file_path.parent().unwrap().exists());

    Ok(())
}

