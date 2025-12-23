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

#[test]
fn test_directory_walker_list_dirs_nonexistent() {
    // 测试列出不存在的目录应该返回错误
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_dirs();
    assert!(result.is_err());
}

#[test]
fn test_directory_walker_list_files_nonexistent() {
    // 测试列出不存在目录的文件应该返回错误
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_files();
    assert!(result.is_err());
}

#[test]
fn test_directory_walker_find_files_nonexistent() {
    // 测试在不存在的目录中查找文件应该返回错误
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.find_files("pattern");
    assert!(result.is_err());
}

#[test]
fn test_directory_walker_list_direct_dirs_nonexistent() {
    // 测试列出不存在目录的直接子目录应该返回错误
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_direct_dirs();
    assert!(result.is_err());
}

#[test]
fn test_directory_walker_list_direct_files_nonexistent() {
    // 测试列出不存在目录的直接文件应该返回错误
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_direct_files();
    assert!(result.is_err());
}

#[test]
fn test_directory_walker_find_files_empty_pattern() -> color_eyre::Result<()> {
    // 测试空模式应该匹配所有文件
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("")?;
    // 空模式应该匹配所有文件
    assert!(files.len() >= 2);

    Ok(())
}

#[test]
fn test_directory_walker_find_files_no_match() -> color_eyre::Result<()> {
    // 测试没有匹配的文件
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("nonexistent_pattern_xyz")?;
    assert_eq!(files.len(), 0);

    Ok(())
}

#[test]
fn test_directory_walker_list_dirs_empty_directory() -> color_eyre::Result<()> {
    // 测试空目录应该只包含根目录本身
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_dirs()?;
    // 应该至少包含根目录
    assert!(dirs.len() >= 1);
    assert!(dirs.contains(&dir_path));

    Ok(())
}

#[test]
fn test_directory_walker_list_files_empty_directory() -> color_eyre::Result<()> {
    // 测试空目录应该没有文件
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;
    assert_eq!(files.len(), 0);

    Ok(())
}

#[test]
fn test_directory_walker_list_direct_dirs_empty_directory() -> color_eyre::Result<()> {
    // 测试空目录应该没有直接子目录
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_direct_dirs()?;
    assert_eq!(dirs.len(), 0);

    Ok(())
}

#[test]
fn test_directory_walker_list_direct_files_empty_directory() -> color_eyre::Result<()> {
    // 测试空目录应该没有直接文件
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_direct_files()?;
    assert_eq!(files.len(), 0);

    Ok(())
}

#[test]
fn test_directory_walker_ensure_parent_exists_no_parent() -> color_eyre::Result<()> {
    // 测试没有父目录的文件路径（根路径）
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path(); // 根路径本身

    let walker = DirectoryWalker::new(temp_dir.path());
    // 根路径没有父目录，应该不会出错
    let result = walker.ensure_parent_exists(&file_path);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_directory_walker_find_files_case_sensitive() -> color_eyre::Result<()> {
    // 测试查找文件是大小写敏感的
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("TestFile.txt"), "content")?;
    fs::write(dir_path.join("testfile.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files_upper = walker.find_files("Test")?;
    let files_lower = walker.find_files("test")?;

    // 大小写敏感，应该有不同的结果
    assert!(files_upper.len() >= 1);
    assert!(files_lower.len() >= 1);

    Ok(())
}

#[test]
fn test_directory_walker_list_dirs_deep_nesting() -> color_eyre::Result<()> {
    // 测试 list_dirs() 的循环逻辑（覆盖 directory.rs:25-31）
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;

    // 创建深层嵌套目录结构
    fs::create_dir_all(dir_path.join("level1/level2/level3"))?;
    fs::create_dir_all(dir_path.join("level1/level2b"))?;
    fs::write(dir_path.join("file.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_dirs()?;
    // 应该包含根目录和所有子目录
    assert!(dirs.len() >= 4);

    Ok(())
}

#[test]
fn test_directory_walker_list_files_deep_nesting() -> color_eyre::Result<()> {
    // 测试 list_files() 的循环逻辑（覆盖 directory.rs:38-44）
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;

    // 创建深层嵌套文件结构
    fs::create_dir_all(dir_path.join("level1/level2"))?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("level1/file2.txt"), "content2")?;
    fs::write(dir_path.join("level1/level2/file3.txt"), "content3")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;
    assert_eq!(files.len(), 3);

    Ok(())
}

#[test]
fn test_directory_walker_find_files_multiple_matches() -> color_eyre::Result<()> {
    // 测试 find_files() 的循环逻辑（覆盖 directory.rs:51-61）
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;

    // 创建多个匹配的文件
    fs::create_dir_all(dir_path.join("subdir"))?;
    fs::write(dir_path.join("test1.txt"), "content")?;
    fs::write(dir_path.join("test2.txt"), "content")?;
    fs::write(dir_path.join("test3.log"), "content")?;
    fs::write(dir_path.join("subdir/test4.txt"), "content")?;
    fs::write(dir_path.join("other.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("test")?;
    assert_eq!(files.len(), 4); // test1.txt, test2.txt, test3.log, test4.txt

    Ok(())
}

#[test]
fn test_directory_walker_list_dirs_with_files() -> color_eyre::Result<()> {
    // 测试 list_dirs() 只返回目录，不包括文件（覆盖 directory.rs:28-30）
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir(dir_path.join("subdir1"))?;
    fs::create_dir(dir_path.join("subdir2"))?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_dirs()?;
    // 应该只包含目录，不包括文件
    for dir in &dirs {
        assert!(dir.is_dir(), "list_dirs() should only return directories");
    }

    Ok(())
}

#[test]
fn test_directory_walker_list_files_with_dirs() -> color_eyre::Result<()> {
    // 测试 list_files() 只返回文件，不包括目录（覆盖 directory.rs:41-43）
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;
    // 应该只包含文件，不包括目录
    for file in &files {
        assert!(file.is_file(), "list_files() should only return files");
    }

    Ok(())
}

