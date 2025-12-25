//! Base Util Path 模块测试
//!
//! 测试路径操作工具的核心功能，包括 PathAccess 结构体。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 使用 `expect()` 替代 `unwrap()` 提供清晰的错误消息
//! - 测试路径访问、目录创建和安全读取功能

use std::fs;
use tempfile::TempDir;
use workflow::base::fs::path::PathAccess;

// ==================== PathAccess Creation Tests ====================

#[test]
fn test_path_access_new_with_string_path_creates_instance() {
    // Arrange: 准备字符串路径

    // Act: 创建 PathAccess 实例
    let _path_access = PathAccess::new("test/path");

    // Assert: 验证可以创建 PathAccess（不会panic）
    assert!(true);
}

#[test]
fn test_path_access_new_with_pathbuf_creates_instance() {
    // Arrange: 准备 PathBuf 路径
    let path = std::path::PathBuf::from("test/path");

    // Act: 创建 PathAccess 实例
    let _path_access = PathAccess::new(path);

    // Assert: 验证可以创建 PathAccess（不会panic）
    assert!(true);
}

// ==================== Path Existence Tests ====================

#[test]
fn test_path_access_exists_with_existing_and_nonexisting_paths_returns_correct() -> color_eyre::Result<()> {
    // Arrange: 准备存在的和不存在的路径
    let temp_dir = TempDir::new()?;
    let existing_path = temp_dir.path().join("existing.txt");
    fs::write(&existing_path, "test")?;
    let non_existing_path = temp_dir.path().join("non_existing.txt");

    // Act: 检查路径是否存在
    let existing_access = PathAccess::new(&existing_path);
    let non_existing_access = PathAccess::new(&non_existing_path);

    // Assert: 验证存在性检查正确
    assert!(existing_access.exists());
    assert!(!non_existing_access.exists());

    Ok(())
}

#[test]
fn test_path_access_is_file_with_file_and_dir_returns_correct() -> color_eyre::Result<()> {
    // Arrange: 准备文件和目录路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test")?;
    let dir_path = temp_dir.path().join("subdir");
    fs::create_dir(&dir_path)?;

    // Act: 检查是否为文件
    let file_access = PathAccess::new(&file_path);
    let dir_access = PathAccess::new(&dir_path);

    // Assert: 验证文件检查正确
    assert!(file_access.is_file());
    assert!(!dir_access.is_file());

    Ok(())
}

#[test]
fn test_path_access_is_dir_with_dir_and_file_returns_correct() -> color_eyre::Result<()> {
    // Arrange: 准备目录和文件路径
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("subdir");
    fs::create_dir(&dir_path)?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test")?;

    // Act: 检查是否为目录
    let dir_access = PathAccess::new(&dir_path);
    let file_access = PathAccess::new(&file_path);

    // Assert: 验证目录检查正确
    assert!(dir_access.is_dir());
    assert!(!file_access.is_dir());

    Ok(())
}

// ==================== Directory Creation Tests ====================

#[test]
fn test_path_access_ensure_dir_exists_with_new_path_creates_directory() -> color_eyre::Result<()> {
    // Arrange: 准备新目录路径
    let temp_dir = TempDir::new()?;
    let new_dir = temp_dir.path().join("new/dir/path");

    // Act: 确保目录存在
    let path_access = PathAccess::new(&new_dir);
    path_access.ensure_dir_exists()?;

    // Assert: 验证目录已创建
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());

    Ok(())
}

#[test]
fn test_path_access_ensure_dir_exists_with_existing_dir_succeeds() -> color_eyre::Result<()> {
    // Arrange: 准备已存在的目录
    let temp_dir = TempDir::new()?;
    let existing_dir = temp_dir.path().join("existing");
    fs::create_dir_all(&existing_dir)?;

    // Act: 确保目录存在（目录已存在）
    let path_access = PathAccess::new(&existing_dir);
    path_access.ensure_dir_exists()?;

    // Assert: 验证目录仍然存在
    assert!(existing_dir.exists());

    Ok(())
}

#[test]
fn test_path_access_ensure_parent_exists_with_file_path_creates_parent_directory() -> color_eyre::Result<()> {
    // Arrange: 准备文件路径（父目录不存在）
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("parent/dir/file.txt");

    // Act: 确保父目录存在
    let path_access = PathAccess::new(&file_path);
    path_access.ensure_parent_exists()?;

    // Assert: 验证父目录已创建
    let parent = file_path.parent().expect("File path should have a parent directory");
    assert!(parent.exists());
    assert!(parent.is_dir());

    Ok(())
}

#[test]
fn test_path_access_ensure_parent_exists_with_root_path_succeeds() -> color_eyre::Result<()> {
    // Arrange: 准备根路径（没有父目录）
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // Act: 确保父目录存在（根路径没有父目录）
    let path_access = PathAccess::new(root_path);
    path_access.ensure_parent_exists()?;

    // Assert: 验证不会失败（即使没有父目录）
    Ok(())
}

// ==================== Directory Reading Tests ====================

#[test]
fn test_path_access_read_dir_safe_with_valid_directory_returns_entries() -> color_eyre::Result<()> {
    // Arrange: 准备包含文件和子目录的目录
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir(&dir_path)?;
    fs::write(dir_path.join("file1.txt"), "content1")?;
    fs::write(dir_path.join("file2.txt"), "content2")?;
    fs::create_dir(dir_path.join("subdir"))?;

    // Act: 安全读取目录
    let path_access = PathAccess::new(&dir_path);
    let entries = path_access.read_dir_safe()?;

    // Assert: 验证返回至少3个条目（2个文件 + 1个目录）
    assert!(entries.len() >= 3);

    Ok(())
}

#[test]
fn test_path_access_read_dir_safe_with_nonexistent_path_returns_error() {
    // Arrange: 准备不存在的路径
    let path_access = PathAccess::new("/nonexistent/path/that/does/not/exist");

    // Act: 尝试读取目录
    let result = path_access.read_dir_safe();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

#[test]
fn test_path_access_read_dir_safe_with_file_path_returns_error() -> color_eyre::Result<()> {
    // Arrange: 准备文件路径（不是目录）
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test")?;

    // Act: 尝试读取文件作为目录
    let path_access = PathAccess::new(&file_path);
    let result = path_access.read_dir_safe();

    // Assert: 验证返回错误
    assert!(result.is_err());

    Ok(())
}
