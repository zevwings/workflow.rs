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
use workflow::base::fs::path::PathAccess;

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::cli_env;
use rstest::rstest;

// ==================== PathAccess Creation Tests ====================

/// 测试使用字符串路径创建 PathAccess
///
/// ## 测试目的
/// 验证 PathAccess::new() 能够使用字符串路径创建实例。
///
/// ## 测试场景
/// 1. 使用字符串路径创建 PathAccess 实例
/// 2. 验证创建成功
///
/// ## 预期结果
/// - PathAccess 实例创建成功
#[test]
fn test_path_access_new_with_string_path_creates_instance() {
    // Arrange: 准备字符串路径

    // Act: 创建 PathAccess 实例
    let _path_access = PathAccess::new("test/path");

    // Assert: 验证可以创建 PathAccess（不会panic）
    assert!(true);
}

/// 测试使用 PathBuf 创建 PathAccess
///
/// ## 测试目的
/// 验证 PathAccess::new() 能够使用 PathBuf 创建实例。
///
/// ## 测试场景
/// 1. 使用 PathBuf 路径创建 PathAccess 实例
/// 2. 验证创建成功
///
/// ## 预期结果
/// - PathAccess 实例创建成功
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

/// 测试检查路径是否存在
///
/// ## 测试目的
/// 验证 PathAccess::exists() 能够正确检查路径是否存在。
///
/// ## 测试场景
/// 1. 准备存在的和不存在的路径
/// 2. 检查路径是否存在
/// 3. 验证存在性检查正确
///
/// ## 预期结果
/// - 存在的路径返回 true，不存在的路径返回 false
#[rstest]
fn test_path_access_exists_with_existing_and_nonexisting_paths_return_result(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // Arrange: 准备存在的和不存在的路径
    let existing_path = cli_env.path().join("existing.txt");
    fs::write(&existing_path, "test")?;
    let non_existing_path = cli_env.path().join("non_existing.txt");

    // Act: 检查路径是否存在
    let existing_access = PathAccess::new(&existing_path);
    let non_existing_access = PathAccess::new(&non_existing_path);

    // Assert: 验证存在性检查正确
    assert!(existing_access.exists());
    assert!(!non_existing_access.exists());

    Ok(())
}

/// 测试检查是否为文件
///
/// ## 测试目的
/// 验证 PathAccess::is_file() 能够正确检查路径是否为文件。
///
/// ## 测试场景
/// 1. 准备文件和目录路径
/// 2. 检查是否为文件
/// 3. 验证文件检查正确
///
/// ## 预期结果
/// - 文件路径返回 true，目录路径返回 false
#[rstest]
fn test_path_access_is_file_with_file_and_dir_return_result(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // Arrange: 准备文件和目录路径
    let file_path = cli_env.path().join("test.txt");
    fs::write(&file_path, "test")?;
    let dir_path = cli_env.path().join("subdir");
    fs::create_dir(&dir_path)?;

    // Act: 检查是否为文件
    let file_access = PathAccess::new(&file_path);
    let dir_access = PathAccess::new(&dir_path);

    // Assert: 验证文件检查正确
    assert!(file_access.is_file());
    assert!(!dir_access.is_file());

    Ok(())
}

/// 测试检查是否为目录
///
/// ## 测试目的
/// 验证 PathAccess::is_dir() 能够正确检查路径是否为目录。
///
/// ## 测试场景
/// 1. 准备目录和文件路径
/// 2. 检查是否为目录
/// 3. 验证目录检查正确
///
/// ## 预期结果
/// - 目录路径返回 true，文件路径返回 false
#[rstest]
fn test_path_access_is_dir_with_dir_and_file_return_result(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // Arrange: 准备目录和文件路径
    let dir_path = cli_env.path().join("subdir");
    fs::create_dir(&dir_path)?;
    let file_path = cli_env.path().join("test.txt");
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

/// 测试确保目录存在（新路径）
///
/// ## 测试目的
/// 验证 PathAccess::ensure_dir_exists() 能够为新路径创建目录。
///
/// ## 测试场景
/// 1. 准备新目录路径
/// 2. 调用 ensure_dir_exists() 确保目录存在
/// 3. 验证目录已创建
///
/// ## 预期结果
/// - 目录被创建且存在
#[test]
fn test_path_access_ensure_dir_exists_with_new_path_creates_directory() -> color_eyre::Result<()> {
    // Arrange: 准备新目录路径
    let env = CliTestEnv::new()?;
    let new_dir = env.path().join("new/dir/path");

    // Act: 确保目录存在
    let path_access = PathAccess::new(&new_dir);
    path_access.ensure_dir_exists()?;

    // Assert: 验证目录已创建
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());

    Ok(())
}

/// 测试确保目录存在（已存在的目录）
///
/// ## 测试目的
/// 验证 PathAccess::ensure_dir_exists() 对已存在的目录不会失败。
///
/// ## 测试场景
/// 1. 准备已存在的目录
/// 2. 调用 ensure_dir_exists() 确保目录存在
/// 3. 验证目录仍然存在
///
/// ## 预期结果
/// - 目录仍然存在，操作成功
#[test]
fn test_path_access_ensure_dir_exists_with_existing_dir_succeeds() -> color_eyre::Result<()> {
    // Arrange: 准备已存在的目录
    let env = CliTestEnv::new()?;
    let existing_dir = env.path().join("existing");
    fs::create_dir_all(&existing_dir)?;

    // Act: 确保目录存在（目录已存在）
    let path_access = PathAccess::new(&existing_dir);
    path_access.ensure_dir_exists()?;

    // Assert: 验证目录仍然存在
    assert!(existing_dir.exists());

    Ok(())
}

/// 测试确保父目录存在（文件路径）
///
/// ## 测试目的
/// 验证 PathAccess::ensure_parent_exists() 能够为文件路径创建父目录。
///
/// ## 测试场景
/// 1. 准备文件路径（父目录不存在）
/// 2. 调用 ensure_parent_exists() 确保父目录存在
/// 3. 验证父目录已创建
///
/// ## 预期结果
/// - 父目录被创建且存在
#[test]
fn test_path_access_ensure_parent_exists_with_file_path_creates_parent_directory() -> color_eyre::Result<()> {
    // Arrange: 准备文件路径（父目录不存在）
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("parent/dir/file.txt");

    // Act: 确保父目录存在
    let path_access = PathAccess::new(&file_path);
    path_access.ensure_parent_exists()?;

    // Assert: 验证父目录已创建
    let parent = file_path.parent().expect("File path should have a parent directory");
    assert!(parent.exists());
    assert!(parent.is_dir());

    Ok(())
}

/// 测试确保父目录存在（根路径）
///
/// ## 测试目的
/// 验证 PathAccess::ensure_parent_exists() 对根路径（没有父目录）不会失败。
///
/// ## 测试场景
/// 1. 准备根路径（没有父目录）
/// 2. 调用 ensure_parent_exists() 确保父目录存在
/// 3. 验证不会失败
///
/// ## 预期结果
/// - 操作成功，即使没有父目录也不会失败
#[test]
fn test_path_access_ensure_parent_exists_with_root_path_succeeds() -> color_eyre::Result<()> {
    // Arrange: 准备根路径（没有父目录）
    let env = CliTestEnv::new()?;
    let root_path = env.path();

    // Act: 确保父目录存在（根路径没有父目录）
    let path_access = PathAccess::new(root_path);
    path_access.ensure_parent_exists()?;

    // Assert: 验证不会失败（即使没有父目录）
    Ok(())
}

// ==================== Directory Reading Tests ====================

/// 测试安全读取目录（有效目录）
///
/// ## 测试目的
/// 验证 PathAccess::read_dir_safe() 能够安全读取有效目录并返回条目。
///
/// ## 测试场景
/// 1. 准备包含文件和子目录的目录
/// 2. 调用 read_dir_safe() 安全读取目录
/// 3. 验证返回至少3个条目（2个文件 + 1个目录）
///
/// ## 预期结果
/// - 返回目录条目列表，至少包含3个条目
#[test]
fn test_path_access_read_dir_safe_with_valid_directory_return_result() -> color_eyre::Result<()> {
    // Arrange: 准备包含文件和子目录的目录
    let env = CliTestEnv::new()?;
    let dir_path = env.path().join("test_dir");
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

/// 测试安全读取目录（不存在的路径）
///
/// ## 测试目的
/// 验证 PathAccess::read_dir_safe() 对不存在的路径返回错误。
///
/// ## 测试场景
/// 1. 准备不存在的路径
/// 2. 尝试读取目录
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回错误
#[test]
fn test_path_access_read_dir_safe_with_nonexistent_path_returns_error() {
    // Arrange: 准备不存在的路径
    let path_access = PathAccess::new("/nonexistent/path/that/does/not/exist");

    // Act: 尝试读取目录
    let result = path_access.read_dir_safe();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试安全读取目录（文件路径）
///
/// ## 测试目的
/// 验证 PathAccess::read_dir_safe() 对文件路径（不是目录）返回错误。
///
/// ## 测试场景
/// 1. 准备文件路径（不是目录）
/// 2. 尝试读取文件作为目录
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回错误
#[test]
fn test_path_access_read_dir_safe_with_file_path_return_result() -> color_eyre::Result<()> {
    // Arrange: 准备文件路径（不是目录）
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("test.txt");
    fs::write(&file_path, "test")?;

    // Act: 尝试读取文件作为目录
    let path_access = PathAccess::new(&file_path);
    let result = path_access.read_dir_safe();

    // Assert: 验证返回错误
    assert!(result.is_err());

    Ok(())
}
