//! Zip 模块测试
//!
//! 测试解压工具的核心功能，包括 tar.gz 和 zip 文件解压。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 辅助函数中的 `unwrap()` 保留（测试辅助函数失败时 panic 是合理的）
//! - 测试 tar.gz 和 zip 文件的解压功能

use color_eyre::Result;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use workflow::base::zip::Unzip;

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::cli_env;
use rstest::rstest;

// 辅助函数：创建测试用的 tar.gz 文件
fn create_test_tar_gz(env: &CliTestEnv) -> PathBuf {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let tar_gz_path = env.path().join("test.tar.gz");

    // 创建临时文件用于打包
    let file1_path = env.path().join("file1.txt");
    fs::write(&file1_path, "content1").expect("should write file1");

    let file2_path = env.path().join("file2.txt");
    fs::write(&file2_path, "content2").expect("should write file2");

    let subdir = env.path().join("subdir");
    fs::create_dir_all(&subdir).expect("should create subdir");
    let file3_path = subdir.join("file3.txt");
    fs::write(&file3_path, "content3").expect("should write file3");

    // 创建 tar.gz 文件
    let tar_gz_file = fs::File::create(&tar_gz_path).expect("should create tar.gz file");
    let enc = GzEncoder::new(tar_gz_file, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_path_with_name(&file1_path, "file1.txt")
        .expect("should append file1 to tar");
    tar.append_path_with_name(&file2_path, "file2.txt")
        .expect("should append file2 to tar");
    tar.append_path_with_name(&file3_path, "subdir/file3.txt")
        .expect("should append file3 to tar");

    tar.finish().expect("should finish tar archive");

    tar_gz_path
}

// 辅助函数：创建测试用的 zip 文件
fn create_test_zip(env: &CliTestEnv) -> PathBuf {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};
    use zip::CompressionMethod;

    let zip_path = env.path().join("test.zip");
    let zip_file = fs::File::create(&zip_path).expect("should create zip file");
    let mut zip = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 添加文件
    zip.start_file("file1.txt", options).expect("should start file1 in zip");
    zip.write_all(b"content1").expect("should write content1 to zip");

    zip.start_file("file2.txt", options).expect("should start file2 in zip");
    zip.write_all(b"content2").expect("should write content2 to zip");

    // 添加目录
    zip.add_directory("subdir/", options).expect("should add subdir to zip");

    // 添加子目录中的文件
    zip.start_file("subdir/file3.txt", options).expect("should start file3 in zip");
    zip.write_all(b"content3").expect("should write content3 to zip");

    zip.finish().expect("should finish zip archive");

    zip_path
}

// ==================== Unzip Extraction Tests ====================

/// 测试解压 tar.gz 文件（有效文件）
///
/// ## 测试目的
/// 验证 Unzip::extract_tar_gz() 能够解压有效的 tar.gz 文件。
///
/// ## 测试场景
/// 1. 准备临时目录和 tar.gz 文件
/// 2. 解压 tar.gz 文件
/// 3. 验证文件已解压且内容正确
///
/// ## 预期结果
/// - 所有文件都被正确解压，文件内容正确
#[rstest]
fn test_unzip_extract_tar_gz_with_valid_file_extracts_files_return_collect(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和tar.gz文件
    let tar_gz_path = create_test_tar_gz(&cli_env);
    let output_dir = cli_env.path().join("output");

    // Act: 解压tar.gz文件
    Unzip::extract_tar_gz(&tar_gz_path, &output_dir)?;

    // Assert: 验证文件已解压
    assert!(output_dir.join("file1.txt").exists());
    assert!(output_dir.join("file2.txt").exists());
    assert!(output_dir.join("subdir/file3.txt").exists());

    // Assert: 验证文件内容
    assert_eq!(
        fs::read_to_string(output_dir.join("file1.txt"))?,
        "content1"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("file2.txt"))?,
        "content2"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("subdir/file3.txt"))?,
        "content3"
    );

    Ok(())
}

/// 测试解压 tar.gz 文件（不存在的文件）
///
/// ## 测试目的
/// 验证 Unzip::extract_tar_gz() 对不存在的文件返回错误。
///
/// ## 测试场景
/// 1. 准备不存在的文件路径
/// 2. 尝试解压不存在的文件
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回文件不存在错误
#[rstest]
fn test_unzip_extract_tar_gz_nonexistent_file_with_missing_file_return_ok(
    cli_env: CliTestEnv,
) -> Result<()> {
    // Arrange: 准备不存在的文件路径
    let nonexistent_path = cli_env.path().join("nonexistent.tar.gz");
    let output_dir = cli_env.path().join("output");

    // Act: 尝试解压不存在的文件
    let result = Unzip::extract_tar_gz(&nonexistent_path, &output_dir);

    // Assert: 验证返回错误
    assert!(result.is_err());
    Ok(())
}

/// 测试解压 tar.gz 文件（无效格式）
///
/// ## 测试目的
/// 验证 Unzip::extract_tar_gz() 对无效格式的文件返回错误。
///
/// ## 测试场景
/// 1. 准备无效格式的文件
/// 2. 尝试解压无效格式的文件
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回格式错误
#[rstest]
fn test_unzip_extract_tar_gz_invalid_format_with_invalid_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备无效格式的文件
    let invalid_file = cli_env.path().join("invalid.tar.gz");
    fs::write(&invalid_file, "not a valid tar.gz file")?;
    let output_dir = cli_env.path().join("output");

    // Act: 尝试解压无效格式的文件
    let result = Unzip::extract_tar_gz(&invalid_file, &output_dir);

    // Assert: 验证返回错误
    assert!(result.is_err());

    Ok(())
}

/// 测试解压 tar.gz 文件时自动创建输出目录
///
/// ## 测试目的
/// 验证 Unzip::extract_tar_gz() 在输出目录不存在时能够自动创建。
///
/// ## 测试场景
/// 1. 准备 tar.gz 文件和不存在的输出目录
/// 2. 解压文件（输出目录不存在，应该自动创建）
/// 3. 验证目录已创建
///
/// ## 预期结果
/// - 输出目录被自动创建
#[rstest]
fn test_unzip_extract_tar_gz_output_dir_created_with_missing_dir_creates_dir(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备tar.gz文件和不存在的输出目录
    let tar_gz_path = create_test_tar_gz(&cli_env);
    let output_dir = cli_env.path().join("new/output/dir");
    assert!(!output_dir.exists());

    // Act: 解压文件（输出目录不存在，应该自动创建）
    Unzip::extract_tar_gz(&tar_gz_path, &output_dir)?;

    // Assert: 验证目录已创建
    assert!(output_dir.exists());
    assert!(output_dir.is_dir());

    Ok(())
}

/// 测试解压 zip 文件（有效文件）
///
/// ## 测试目的
/// 验证 Unzip::extract_zip() 能够解压有效的 zip 文件。
///
/// ## 测试场景
/// 1. 准备临时目录和 zip 文件
/// 2. 解压 zip 文件
/// 3. 验证文件已解压且内容正确
///
/// ## 预期结果
/// - 所有文件都被正确解压，文件内容正确
#[rstest]
fn test_unzip_extract_zip_with_valid_file_extracts_files_return_collect(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和zip文件
    let zip_path = create_test_zip(&cli_env);
    let output_dir = cli_env.path().join("output");

    // Act: 解压zip文件
    Unzip::extract_zip(&zip_path, &output_dir)?;

    // Assert: 验证文件已解压且内容正确
    assert!(output_dir.join("file1.txt").exists());
    assert!(output_dir.join("file2.txt").exists());
    assert!(output_dir.join("subdir/file3.txt").exists());
    assert_eq!(
        fs::read_to_string(output_dir.join("file1.txt"))?,
        "content1"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("file2.txt"))?,
        "content2"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("subdir/file3.txt"))?,
        "content3"
    );

    Ok(())
}

/// 测试解压 zip 文件（不存在的文件）
///
/// ## 测试目的
/// 验证 Unzip::extract_zip() 对不存在的文件返回错误。
///
/// ## 测试场景
/// 1. 准备不存在的文件路径
/// 2. 尝试解压不存在的文件
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回文件不存在错误
#[rstest]
fn test_unzip_extract_zip_nonexistent_file_with_missing_file_return_ok(
    cli_env: CliTestEnv,
) -> Result<()> {
    // Arrange: 准备不存在的文件路径
    let nonexistent_path = cli_env.path().join("nonexistent.zip");
    let output_dir = cli_env.path().join("output");

    // Act: 尝试解压不存在的文件
    let result = Unzip::extract_zip(&nonexistent_path, &output_dir);

    // Assert: 验证返回错误
    assert!(result.is_err());
    Ok(())
}

/// 测试解压 zip 文件（无效格式）
///
/// ## 测试目的
/// 验证 Unzip::extract_zip() 对无效格式的文件返回错误。
///
/// ## 测试场景
/// 1. 准备无效格式的文件
/// 2. 尝试解压无效格式的文件
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回格式错误
#[rstest]
fn test_unzip_extract_zip_invalid_format_with_invalid_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备无效格式的文件
    let invalid_file = cli_env.path().join("invalid.zip");
    fs::write(&invalid_file, "not a valid zip file")?;
    let output_dir = cli_env.path().join("output");

    // Act: 尝试解压无效格式的文件
    let result = Unzip::extract_zip(&invalid_file, &output_dir);

    // Assert: 验证返回错误
    assert!(result.is_err());

    Ok(())
}

/// 测试解压 zip 文件时自动创建输出目录
///
/// ## 测试目的
/// 验证 Unzip::extract_zip() 在输出目录不存在时能够自动创建。
///
/// ## 测试场景
/// 1. 准备 zip 文件和不存在的输出目录
/// 2. 解压文件（输出目录不存在，应该自动创建）
/// 3. 验证目录已创建
///
/// ## 预期结果
/// - 输出目录被自动创建
#[rstest]
fn test_unzip_extract_zip_output_dir_created_with_missing_dir_creates_dir(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    let zip_path = create_test_zip(&cli_env);
    let output_dir = cli_env.path().join("new/output/dir");

    // 输出目录不存在，应该自动创建
    assert!(!output_dir.exists());

    Unzip::extract_zip(&zip_path, &output_dir)?;

    // Assert: 验证目录已创建
    assert!(output_dir.exists());
    assert!(output_dir.is_dir());

    Ok(())
}

/// 测试解压包含目录结构的 zip 文件
///
/// ## 测试目的
/// 验证 Unzip::extract_zip() 能够解压包含嵌套目录结构的 zip 文件。
///
/// ## 测试场景
/// 1. 创建包含嵌套目录结构的 zip 文件
/// 2. 解压 zip 文件
/// 3. 验证嵌套目录结构已创建
///
/// ## 预期结果
/// - 嵌套目录结构被正确创建，文件内容正确
#[rstest]
fn test_unzip_extract_zip_with_directories_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};
    use zip::CompressionMethod;

    let zip_path = cli_env.path().join("test.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 添加嵌套目录结构
    zip.add_directory("level1/", options)?;
    zip.add_directory("level1/level2/", options)?;
    zip.start_file("level1/level2/file.txt", options)?;
    zip.write_all(b"nested content")?;

    zip.finish()?;

    let output_dir = cli_env.path().join("output");
    Unzip::extract_zip(&zip_path, &output_dir)?;

    // Assert: 验证嵌套目录结构已创建
    assert!(output_dir.join("level1/level2/file.txt").exists());
    assert_eq!(
        fs::read_to_string(output_dir.join("level1/level2/file.txt"))?,
        "nested content"
    );

    Ok(())
}

/// 测试解压只包含单个文件的 tar.gz 文件
///
/// ## 测试目的
/// 验证 Unzip::extract_tar_gz() 能够解压只包含单个文件的 tar.gz 归档。
///
/// ## 测试场景
/// 1. 创建只包含一个文件的 tar.gz 归档
/// 2. 解压 tar.gz 文件
/// 3. 验证文件已解压
///
/// ## 预期结果
/// - 文件被正确解压，文件内容正确
#[rstest]
fn test_unzip_extract_tar_gz_single_file_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let tar_gz_path = cli_env.path().join("single.tar.gz");

    // 创建一个只包含一个文件的 tar.gz 归档
    let tar_gz_file = fs::File::create(&tar_gz_path)?;
    let enc = GzEncoder::new(tar_gz_file, Compression::default());
    let mut tar = Builder::new(enc);

    // 添加一个文件条目
    let file_path = cli_env.path().join("single.txt");
    fs::write(&file_path, "single file content")?;
    tar.append_path_with_name(&file_path, "single.txt")?;
    drop(tar); // 确保 tar builder 被 drop，刷新所有数据

    let output_dir = cli_env.path().join("output");
    Unzip::extract_tar_gz(&tar_gz_path, &output_dir)?;

    // Assert: 验证文件已解压
    assert!(output_dir.exists());
    assert!(output_dir.join("single.txt").exists());
    assert_eq!(
        fs::read_to_string(output_dir.join("single.txt"))?,
        "single file content"
    );

    Ok(())
}

/// 测试解压空 zip 归档
///
/// ## 测试目的
/// 验证 Unzip::extract_zip() 对空 zip 归档的处理。
///
/// ## 测试场景
/// 1. 创建空的 zip 文件
/// 2. 解压空 zip 文件
/// 3. 验证解压成功（只是没有文件）
///
/// ## 预期结果
/// - 解压成功，输出目录存在
#[rstest]
fn test_unzip_extract_zip_empty_archive_return_empty(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    use zip::write::ZipWriter;

    let zip_path = cli_env.path().join("empty.zip");

    // 创建空的 zip 文件
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    zip.finish()?;

    let output_dir = cli_env.path().join("output");
    let result = Unzip::extract_zip(&zip_path, &output_dir);
    // 空归档应该成功解压（只是没有文件）
    assert!(result.is_ok());
    assert!(output_dir.exists());

    Ok(())
}
