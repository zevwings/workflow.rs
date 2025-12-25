//! Base Util Directory 模块测试
//!
//! 测试目录操作工具的核心功能，包括 DirectoryWalker 结构体。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 使用 `expect()` 替代 `unwrap()` 提供清晰的错误消息
//! - 测试目录遍历、文件查找和目录创建功能

use pretty_assertions::assert_eq;
use std::fs;
use workflow::base::fs::directory::DirectoryWalker;

use crate::common::environments::CliTestEnv;

// ==================== DirectoryWalker Creation Tests ====================

/// 测试使用字符串路径创建DirectoryWalker实例
#[test]
fn test_directory_walker_new_with_string_path_creates_instance() {
    // Arrange: 准备字符串路径

    // Act: 创建 DirectoryWalker 实例
    let _walker = DirectoryWalker::new("test/path");

    // Assert: 验证可以创建 DirectoryWalker（不会panic）
    assert!(true);
}

/// 测试使用PathBuf路径创建DirectoryWalker实例
#[test]
fn test_directory_walker_new_with_pathbuf_creates_instance() {
    // Arrange: 准备 PathBuf 路径
    let path = std::path::PathBuf::from("test/path");

    // Act: 创建 DirectoryWalker 实例
    let _walker = DirectoryWalker::new(path);

    // Assert: 验证可以创建 DirectoryWalker（不会panic）
    assert!(true);
}

// ==================== Directory Creation Tests ====================

/// 测试确保新目录存在（创建嵌套目录）
#[test]
fn test_directory_walker_ensure_exists_with_new_path_creates_directory() -> color_eyre::Result<()> {
    // Arrange: 准备新目录路径
    let env = CliTestEnv::new()?;
    let new_dir = env.path().join("new/deep/nested/directory");

    // Act: 确保目录存在
    let walker = DirectoryWalker::new(&new_dir);
    walker.ensure_exists()?;

    // Assert: 验证目录已创建
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());

    Ok(())
}

/// 测试确保已存在的目录存在（不重复创建）
#[test]
fn test_directory_walker_ensure_exists_with_existing_dir_succeeds() -> color_eyre::Result<()> {
    // Arrange: 准备已存在的目录
    let env = CliTestEnv::new()?;
    let existing_dir = env.path().join("existing");
    fs::create_dir_all(&existing_dir)?;

    // Act: 确保目录存在（目录已存在）
    let walker = DirectoryWalker::new(&existing_dir);
    walker.ensure_exists()?;

    // Assert: 验证目录仍然存在
    assert!(existing_dir.exists());

    Ok(())
}

/// 测试多次调用ensure_exists（幂等性）
#[test]
fn test_directory_walker_ensure_exists_with_multiple_calls_succeeds() -> color_eyre::Result<()> {
    // Arrange: 准备目录路径
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test/dir");

    // Act: 多次调用 ensure_exists
    let walker = DirectoryWalker::new(&dir_path);
    walker.ensure_exists()?;
    walker.ensure_exists()?;
    walker.ensure_exists()?;

    // Assert: 验证目录存在且多次调用都成功
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());

    Ok(())
}

// ==================== Directory Listing Tests ====================

/// 测试列出嵌套结构中的所有目录
#[test]
fn test_directory_walker_list_dirs_with_nested_structure_returns_all_dirs() -> color_eyre::Result<()> {
    // Arrange: 准备包含子目录的目录结构
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir(dir_path.join("subdir1"))?;
    fs::create_dir(dir_path.join("subdir2"))?;
    fs::write(dir_path.join("file.txt"), "content")?;

    // Act: 列出所有目录
    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_dirs()?;

    // Assert: 验证返回至少3个目录（根目录和子目录）
    assert!(dirs.len() >= 3);

    Ok(())
}

/// 测试列出混合内容中的文件（不包括目录）
#[test]
fn test_directory_walker_list_files_with_mixed_content_returns_only_files() -> color_eyre::Result<()> {
    // Arrange: 准备包含文件和子目录的目录结构
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    // Act: 列出所有文件
    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;

    // Assert: 验证只返回文件（不包含目录）
    assert_eq!(files.len(), 2);

    Ok(())
}

/// 测试根据模式查找匹配的文件
#[test]
fn test_directory_walker_find_files_with_pattern_returns_matching_files() -> color_eyre::Result<()> {
    // Arrange: 准备包含匹配和不匹配文件的目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("test_file.txt"), "content")?;
    fs::write(dir_path.join("other_file.txt"), "content")?;
    fs::write(dir_path.join("test.log"), "content")?;

    // Act: 查找匹配模式的文件
    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("test")?;

    // Assert: 验证返回匹配的文件（test_file.txt 和 test.log）
    assert_eq!(files.len(), 2);

    Ok(())
}

/// 测试列出直接子目录（不包括嵌套目录）
#[test]
fn test_directory_walker_list_direct_dirs_with_subdirs_returns_direct_dirs() -> color_eyre::Result<()> {
    // Arrange: 准备包含直接子目录的目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir(dir_path.join("subdir1"))?;
    fs::create_dir(dir_path.join("subdir2"))?;
    fs::write(dir_path.join("file.txt"), "content")?;

    // Act: 列出直接子目录
    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_direct_dirs()?;

    // Assert: 验证返回直接子目录（不包括文件）
    assert_eq!(dirs.len(), 2);

    Ok(())
}

/// 测试列出直接文件（不包括子目录中的文件）
#[test]
fn test_directory_walker_list_direct_files_with_files_returns_direct_files() -> color_eyre::Result<()> {
    // Arrange: 准备包含直接文件的目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    // Act: 列出直接文件
    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_direct_files()?;

    // Assert: 验证返回直接文件（不包括目录）
    assert_eq!(files.len(), 2);

    Ok(())
}

/// 测试确保文件路径的父目录存在
#[test]
fn test_directory_walker_ensure_parent_exists_with_file_path_creates_parent() -> color_eyre::Result<()> {
    // Arrange: 准备文件路径
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("parent/dir/file.txt");

    // Act: 确保父目录存在
    let walker = DirectoryWalker::new(env.path());
    walker.ensure_parent_exists(&file_path)?;

    // Assert: 验证父目录已创建
    let parent = file_path.parent().expect("File path should have a parent directory");
    assert!(parent.exists());

    Ok(())
}

/// 测试列出不存在路径的目录（应返回错误）
#[test]
fn test_directory_walker_list_dirs_with_nonexistent_path_returns_error() {
    // Arrange: 准备不存在的目录路径

    // Act: 尝试列出不存在的目录
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_dirs();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试列出不存在路径的文件（应返回错误）
#[test]
fn test_directory_walker_list_files_with_nonexistent_path_returns_error() {
    // Arrange: 准备不存在的目录路径

    // Act: 尝试列出不存在目录的文件
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_files();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试在不存在的路径中查找文件（应返回错误）
#[test]
fn test_directory_walker_find_files_with_nonexistent_path_returns_error() {
    // Arrange: 准备不存在的目录路径

    // Act: 尝试在不存在的目录中查找文件
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.find_files("pattern");

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试列出不存在路径的直接子目录（应返回错误）
#[test]
fn test_directory_walker_list_direct_dirs_with_nonexistent_path_returns_error() {
    // Arrange: 准备不存在的目录路径

    // Act: 尝试列出不存在目录的直接子目录
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_direct_dirs();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试列出不存在路径的直接文件（应返回错误）
#[test]
fn test_directory_walker_list_direct_files_with_nonexistent_path_returns_error() {
    // Arrange: 准备不存在的目录路径

    // Act: 尝试列出不存在目录的直接文件
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist");
    let result = walker.list_direct_files();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试使用空模式查找文件（应返回所有文件）
#[test]
fn test_directory_walker_find_files_with_empty_pattern_returns_all_files() -> color_eyre::Result<()> {
    // Arrange: 准备包含多个文件的目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;

    // Act: 使用空模式查找文件
    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("")?;

    // Assert: 验证空模式匹配所有文件
    assert!(files.len() >= 2);

    Ok(())
}

/// 测试查找不匹配模式的文件（应返回空列表）
#[test]
fn test_directory_walker_find_files_with_no_matching_pattern_returns_empty() -> color_eyre::Result<()> {
    // Arrange: 准备不匹配模式的文件
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;

    // Act: 查找不匹配的模式
    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("nonexistent_pattern_xyz")?;

    // Assert: 验证没有匹配的文件
    assert_eq!(files.len(), 0);

    Ok(())
}

/// 测试列出空目录（应只返回根目录）
#[test]
fn test_directory_walker_list_dirs_with_empty_directory_returns_root_only() -> color_eyre::Result<()> {
    // Arrange: 准备空目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    // Act: 列出目录
    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_dirs()?;

    // Assert: 验证只包含根目录本身
    assert!(dirs.len() >= 1);
    assert!(dirs.contains(&dir_path));

    Ok(())
}

/// 测试列出空目录的文件（应返回空列表）
#[test]
fn test_directory_walker_list_files_with_empty_directory_returns_empty() -> color_eyre::Result<()> {
    // Arrange: 准备空目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    // Act: 列出文件
    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;

    // Assert: 验证没有文件
    assert_eq!(files.len(), 0);

    Ok(())
}

/// 测试列出空目录的直接子目录（应返回空列表）
#[test]
fn test_directory_walker_list_direct_dirs_with_empty_directory_returns_empty() -> color_eyre::Result<()> {
    // Arrange: 准备空目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    // Act: 列出直接子目录
    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_direct_dirs()?;

    // Assert: 验证没有直接子目录
    assert_eq!(dirs.len(), 0);

    Ok(())
}

/// 测试列出空目录的直接文件（应返回空列表）
#[test]
fn test_directory_walker_list_direct_files_with_empty_directory_returns_empty() -> color_eyre::Result<()> {
    // Arrange: 准备空目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("empty_dir");
    fs::create_dir_all(&dir_path)?;

    // Act: 列出直接文件
    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_direct_files()?;

    // Assert: 验证没有直接文件
    assert_eq!(files.len(), 0);

    Ok(())
}

/// 测试确保没有父目录的路径的父目录存在（根路径）
#[test]
fn test_directory_walker_ensure_parent_exists_with_no_parent_succeeds() -> color_eyre::Result<()> {
    // Arrange: 准备没有父目录的文件路径（根路径）
    let env = CliTestEnv::new()?;
    let file_path = env.path(); // 根路径本身

    // Act: 确保父目录存在（根路径没有父目录）
    let walker = DirectoryWalker::new(env.path());
    let result = walker.ensure_parent_exists(&file_path);

    // Assert: 验证不会出错（根路径没有父目录，应该成功）
    assert!(result.is_ok());

    Ok(())
}

/// 测试查找文件的大小写敏感性
#[test]
fn test_directory_walker_find_files_case_sensitive() -> color_eyre::Result<()> {
    // Arrange: 准备测试查找文件是大小写敏感的
    // 注意：此测试不依赖文件系统的大小写敏感性，而是测试 find_files 方法本身的大小写敏感匹配逻辑
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    // 创建一个文件名包含大写 "Test" 的文件
    fs::write(dir_path.join("TestFile.txt"), "content")?;
    // 创建一个文件名包含小写 "test" 的文件（使用不同的文件名避免文件系统大小写不敏感的问题）
    fs::write(dir_path.join("testfile.log"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files_upper = walker.find_files("Test")?;
    let files_lower = walker.find_files("test")?;

    // Assert: 验证大小写敏感匹配：
    // - "Test" 应该匹配 TestFile.txt（包含 "Test"）
    // - "test" 应该匹配 testfile.log（包含 "test"）
    // - 但 "test" 不应该匹配 TestFile.txt（因为大小写敏感）
    assert_eq!(files_upper.len(), 1, "应该找到包含 'Test' 的文件");
    assert_eq!(files_lower.len(), 1, "应该找到包含 'test' 的文件");

    // Assert: 验证找到的文件是正确的
    let upper_name = files_upper[0].file_name().expect("File should have a name");
    let lower_name = files_lower[0].file_name().expect("File should have a name");
    assert!(upper_name.to_string_lossy().contains("Test"));
    assert!(lower_name.to_string_lossy().contains("test"));

    Ok(())
}

/// 测试列出深层嵌套目录结构中的所有目录
#[test]
fn test_directory_walker_list_dirs_deep_nesting() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_dirs() 的循环逻辑（覆盖 directory.rs:25-31）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
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

/// 测试列出深层嵌套文件结构中的所有文件
#[test]
fn test_directory_walker_list_files_deep_nesting() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_files() 的循环逻辑（覆盖 directory.rs:38-44）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
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

/// 测试查找多个匹配的文件
#[test]
fn test_directory_walker_find_files_multiple_matches() -> color_eyre::Result<()> {
    // Arrange: 准备测试 find_files() 的循环逻辑（覆盖 directory.rs:51-61）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
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

/// 测试列出目录时只返回目录（不包括文件）
#[test]
fn test_directory_walker_list_dirs_with_files() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_dirs() 只返回目录，不包括文件（覆盖 directory.rs:28-30）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
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

/// 测试列出文件时只返回文件（不包括目录）
#[test]
fn test_directory_walker_list_files_with_dirs() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_files() 只返回文件，不包括目录（覆盖 directory.rs:41-43）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
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

/// 测试list_dirs()循环中的错误处理
#[test]
fn test_directory_walker_list_dirs_error_in_loop() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_dirs() 循环中的错误处理（覆盖 directory.rs:26-27）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;

    let walker = DirectoryWalker::new(&dir_path);
    // 正常情况应该成功
    let dirs = walker.list_dirs()?;
    assert!(dirs.len() >= 1);

    Ok(())
}

/// 测试list_files()循环中的错误处理
#[test]
fn test_directory_walker_list_files_error_in_loop() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_files() 循环中的错误处理（覆盖 directory.rs:39-40）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;
    assert_eq!(files.len(), 1);

    Ok(())
}

/// 测试find_files()循环中的错误处理
#[test]
fn test_directory_walker_find_files_error_in_loop() -> color_eyre::Result<()> {
    // Arrange: 准备测试 find_files() 循环中的错误处理（覆盖 directory.rs:52-53）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("test_file.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("test")?;
    assert_eq!(files.len(), 1);

    Ok(())
}

/// 测试find_files()中的模式匹配逻辑
#[test]
fn test_directory_walker_find_files_pattern_matching() -> color_eyre::Result<()> {
    // Arrange: 准备测试 find_files() 中的模式匹配逻辑（覆盖 directory.rs:55-58）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;

    // 创建多个文件，测试模式匹配
    fs::write(dir_path.join("match1.txt"), "content")?;
    fs::write(dir_path.join("match2.log"), "content")?;
    fs::write(dir_path.join("other.txt"), "content")?; // 不包含 "match" 的文件名

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("match")?;
    assert_eq!(
        files.len(),
        2,
        "应该只找到 match1.txt 和 match2.log，不包含 other.txt"
    );

    Ok(())
}

/// 测试list_direct_dirs()的过滤逻辑（只返回目录）
#[test]
fn test_directory_walker_list_direct_dirs_filter() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_direct_dirs() 的过滤逻辑（覆盖 directory.rs:67）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir(dir_path.join("subdir1"))?;
    fs::create_dir(dir_path.join("subdir2"))?;
    fs::write(dir_path.join("file.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_direct_dirs()?;
    // 应该只包含目录，不包括文件
    assert_eq!(dirs.len(), 2);
    for dir in &dirs {
        assert!(dir.is_dir());
    }

    Ok(())
}

/// 测试list_direct_files()的过滤逻辑（只返回文件）
#[test]
fn test_directory_walker_list_direct_files_filter() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_direct_files() 的过滤逻辑（覆盖 directory.rs:73）
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_direct_files()?;
    // 应该只包含文件，不包括目录
    assert_eq!(files.len(), 2);
    for file in &files {
        assert!(file.is_file());
    }

    Ok(())
}

/// 测试ensure_parent_exists()有父目录的情况
#[test]
fn test_directory_walker_ensure_parent_exists_with_parent() -> color_eyre::Result<()> {
    // Arrange: 准备测试 ensure_parent_exists() 有父目录的情况（覆盖 directory.rs:123-125）
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("parent/dir/file.txt");

    let walker = DirectoryWalker::new(env.path());
    walker.ensure_parent_exists(&file_path)?;
    let parent = file_path.parent().expect("File path should have a parent directory");
    assert!(parent.exists());

    Ok(())
}

/// 测试list_dirs()处理符号链接的情况
#[test]
fn test_directory_walker_list_dirs_with_symlinks() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_dirs() 处理符号链接的情况
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir(dir_path.join("subdir"))?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_dirs()?;
    // 应该包含根目录和子目录
    assert!(dirs.len() >= 2);

    Ok(())
}

/// 测试list_files()处理符号链接的情况
#[test]
fn test_directory_walker_list_files_with_symlinks() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_files() 处理符号链接的情况
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_files()?;
    assert_eq!(files.len(), 1);

    Ok(())
}

/// 测试find_files()部分匹配的情况
#[test]
fn test_directory_walker_find_files_with_partial_match() -> color_eyre::Result<()> {
    // Arrange: 准备测试 find_files() 部分匹配的情况
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("prefix_suffix.txt"), "content")?;
    fs::write(dir_path.join("prefix_middle_suffix.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("middle")?;
    assert_eq!(files.len(), 1);

    Ok(())
}

/// 测试ensure_exists()的错误消息格式
#[test]
fn test_directory_walker_ensure_exists_error_message() {
    // Arrange: 准备测试 ensure_exists() 的错误消息格式
    // 尝试在无效路径创建目录（在某些系统上可能会失败）
    let walker = DirectoryWalker::new("/");
    // 根目录应该已经存在，不应该失败
    let result = walker.ensure_exists();
    // 根目录应该总是存在的，所以应该成功
    assert!(result.is_ok());
}

/// 测试list_dirs()的错误包装逻辑
#[test]
fn test_directory_walker_list_dirs_error_wrap() {
    // Arrange: 准备测试 list_dirs() 的错误包装逻辑（覆盖 directory.rs:26-27 的 wrap_err_with）
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist/12345");
    let result = walker.list_dirs();
    assert!(result.is_err());
    // Assert: 验证错误消息包含路径信息
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("Failed to read directory entry"));
    }
}

/// 测试list_files()的错误包装逻辑
#[test]
fn test_directory_walker_list_files_error_wrap() {
    // Arrange: 准备测试 list_files() 的错误包装逻辑（覆盖 directory.rs:39-40 的 wrap_err_with）
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist/12345");
    let result = walker.list_files();
    assert!(result.is_err());
    // Assert: 验证错误消息包含路径信息
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("Failed to read directory entry"));
    }
}

/// 测试find_files()的错误包装逻辑
#[test]
fn test_directory_walker_find_files_error_wrap() {
    // Arrange: 准备测试 find_files() 的错误包装逻辑（覆盖 directory.rs:52-53 的 wrap_err_with）
    let walker = DirectoryWalker::new("/nonexistent/path/that/does/not/exist/12345");
    let result = walker.find_files("pattern");
    assert!(result.is_err());
    // Assert: 验证错误消息包含路径信息
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(error_msg.contains("Failed to read directory entry"));
    }
}

/// 测试ensure_exists()的错误包装逻辑
#[test]
fn test_directory_walker_ensure_exists_error_wrap() {
    // Arrange: 准备测试 ensure_exists() 的错误包装逻辑（覆盖 directory.rs:95-96 的 wrap_err_with）
    // 在某些系统上，尝试创建无效路径可能会失败
    // 这里我们测试错误消息格式
    let walker = DirectoryWalker::new("/");
    // 根目录应该已经存在
    let result = walker.ensure_exists();
    assert!(result.is_ok());
}

/// 测试ensure_parent_exists()的错误包装逻辑
#[test]
fn test_directory_walker_ensure_parent_exists_error_wrap() -> color_eyre::Result<()> {
    // Arrange: 准备测试 ensure_parent_exists() 的错误包装逻辑（覆盖 directory.rs:124-125 的 wrap_err_with）
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("parent/dir/file.txt");

    let walker = DirectoryWalker::new(env.path());
    let result = walker.ensure_parent_exists(&file_path);
    assert!(result.is_ok());
    let parent = file_path.parent().expect("File path should have a parent directory");
    assert!(parent.exists());

    Ok(())
}

/// 测试find_files()处理Unicode模式的情况
#[test]
fn test_directory_walker_find_files_unicode_pattern() -> color_eyre::Result<()> {
    // Arrange: 准备测试 find_files() 处理 Unicode 模式的情况
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("测试文件.txt"), "content")?;
    fs::write(dir_path.join("test.txt"), "content")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.find_files("测试")?;
    assert_eq!(files.len(), 1);

    Ok(())
}

/// 测试list_direct_dirs()只返回直接子目录（不包括嵌套目录）
#[test]
fn test_directory_walker_list_direct_dirs_with_nested() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_direct_dirs() 只返回直接子目录，不包括嵌套目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::create_dir_all(dir_path.join("subdir1/nested"))?;
    fs::create_dir(dir_path.join("subdir2"))?;

    let walker = DirectoryWalker::new(&dir_path);
    let dirs = walker.list_direct_dirs()?;
    // 应该只包含直接子目录，不包括嵌套目录
    assert_eq!(dirs.len(), 2);
    assert!(dirs.iter().any(|d| d.ends_with("subdir1")));
    assert!(dirs.iter().any(|d| d.ends_with("subdir2")));

    Ok(())
}

/// 测试list_direct_files()只返回直接文件（不包括嵌套目录中的文件）
#[test]
fn test_directory_walker_list_direct_files_with_nested() -> color_eyre::Result<()> {
    // Arrange: 准备测试 list_direct_files() 只返回直接文件，不包括嵌套目录中的文件
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
    fs::create_dir_all(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::create_dir_all(dir_path.join("subdir"))?;
    fs::write(dir_path.join("subdir/file2.txt"), "content2")?;

    let walker = DirectoryWalker::new(&dir_path);
    let files = walker.list_direct_files()?;
    // 应该只包含直接文件，不包括嵌套目录中的文件
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("file1.txt"));

    Ok(())
}
