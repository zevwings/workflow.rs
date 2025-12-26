//! Base Util File 模块测试
//!
//! 测试文件操作工具的核心功能，包括 FileReader 和 FileWriter。

use pretty_assertions::assert_eq;
use std::fs;
use std::io::Write;
use workflow::base::util::file::{FileReader, FileWriter};

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::cli_env;
use rstest::rstest;

// ==================== FileReader Initialization Tests ====================

/// 测试使用字符串路径创建 FileReader
///
/// ## 测试目的
/// 验证 FileReader::new() 能够使用字符串路径创建文件读取器。
///
/// ## 测试场景
/// 1. 使用字符串路径创建 FileReader
/// 2. 验证路径正确设置
///
/// ## 预期结果
/// - FileReader 的路径字段正确设置
#[test]
fn test_file_reader_new_with_str_path_creates_reader() {
    // Arrange: 准备字符串路径
    let path_str = "test.txt";

    // Act: 使用字符串路径创建 FileReader
    let reader = FileReader::new(path_str);

    // Assert: 验证路径正确设置
    assert_eq!(reader.path, std::path::PathBuf::from(path_str));
}

/// 测试使用 PathBuf 创建 FileReader
///
/// ## 测试目的
/// 验证 FileReader::new() 能够使用 PathBuf 创建文件读取器。
///
/// ## 测试场景
/// 1. 使用 PathBuf 路径创建 FileReader
/// 2. 验证路径正确设置
///
/// ## 预期结果
/// - FileReader 的路径字段正确设置
#[test]
fn test_file_reader_pathbuf_with_pathbuf_creates_reader() {
    // Arrange: 准备 PathBuf 路径
    let path = std::path::PathBuf::from("test/path.txt");

    // Act: 使用 PathBuf 创建 FileReader
    let reader = FileReader::new(path.clone());

    // Assert: 验证路径正确设置
    assert_eq!(reader.path, path);
}

// ==================== FileWriter Initialization Tests ====================

/// 测试使用字符串路径创建 FileWriter
///
/// ## 测试目的
/// 验证 FileWriter::new() 能够使用字符串路径创建文件写入器。
///
/// ## 测试场景
/// 1. 使用字符串路径创建 FileWriter
/// 2. 验证路径正确设置
///
/// ## 预期结果
/// - FileWriter 的路径字段正确设置
#[test]
fn test_file_writer_new_with_str_path_creates_writer() {
    // Arrange: 准备字符串路径
    let path_str = "test.txt";

    // Act: 使用字符串路径创建 FileWriter
    let writer = FileWriter::new(path_str);

    // Assert: 验证路径正确设置
    assert_eq!(writer.path, std::path::PathBuf::from(path_str));
}

/// 测试使用 PathBuf 创建 FileWriter
///
/// ## 测试目的
/// 验证 FileWriter::new() 能够使用 PathBuf 创建文件写入器。
///
/// ## 测试场景
/// 1. 使用 PathBuf 路径创建 FileWriter
/// 2. 验证路径正确设置
///
/// ## 预期结果
/// - FileWriter 的路径字段正确设置
#[test]
fn test_file_writer_pathbuf_with_pathbuf_creates_writer() {
    // Arrange: 准备 PathBuf 路径
    let path = std::path::PathBuf::from("test/path.txt");

    // Act: 使用 PathBuf 创建 FileWriter
    let writer = FileWriter::new(path.clone());

    // Assert: 验证路径正确设置
    assert_eq!(writer.path, path);
}

// ==================== FileReader Reading Tests ====================

/// 测试读取文本文件内容为字符串
///
/// ## 测试目的
/// 验证 FileReader::to_string() 能够读取文本文件内容。
///
/// ## 测试场景
/// 1. 创建包含文本内容的临时文件
/// 2. 使用 FileReader 读取文件内容
/// 3. 验证读取的内容正确
///
/// ## 预期结果
/// - 文件内容被正确读取为字符串
#[rstest]
fn test_file_reader_to_string_with_text_file_reads_content_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时文件和内容
    let file_path = cli_env.path().join("test.txt");
    let expected_content = "Hello, World!";
    fs::write(&file_path, expected_content)?;

    // Act: 读取文件内容为字符串
    let reader = FileReader::new(&file_path);
    let content = reader.to_string()?;

    // Assert: 验证读取的内容正确
    assert_eq!(content, expected_content);

    Ok(())
}

/// 测试读取多行文件的所有行
///
/// ## 测试目的
/// 验证 FileReader::lines() 能够读取文件的所有行。
///
/// ## 测试场景
/// 1. 创建包含多行内容的临时文件
/// 2. 使用 FileReader 读取所有行
/// 3. 验证行数和内容正确
///
/// ## 预期结果
/// - 所有行被正确读取，行数和内容正确
#[rstest]
fn test_file_reader_lines_with_multiline_file_reads_lines_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备多行临时文件
    let file_path = cli_env.path().join("test.txt");
    let file_content = "line1\nline2\nline3";
    fs::write(&file_path, file_content)?;

    // Act: 读取文件的所有行
    let reader = FileReader::new(&file_path);
    let lines = reader.lines()?;

    // Assert: 验证行数和内容正确
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");

    Ok(())
}

/// 测试读取二进制文件的字节内容
///
/// ## 测试目的
/// 验证 FileReader::bytes() 能够读取二进制文件的字节内容。
///
/// ## 测试场景
/// 1. 创建包含二进制数据的临时文件
/// 2. 使用 FileReader 读取字节内容
/// 3. 验证读取的字节内容正确
///
/// ## 预期结果
/// - 二进制文件的字节内容被正确读取
#[rstest]
fn test_file_reader_bytes_with_binary_file_reads_bytes_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备二进制临时文件
    let file_path = cli_env.path().join("test.bin");
    let test_bytes = b"binary data\x00\x01\x02";
    fs::write(&file_path, test_bytes)?;

    // Act: 读取文件的字节内容
    let reader = FileReader::new(&file_path);
    let bytes = reader.bytes()?;

    // Assert: 验证读取的字节内容正确
    assert_eq!(bytes, test_bytes);

    Ok(())
}

// ==================== FileWriter Writing Tests ====================

/// 测试写入文本内容到文件
///
/// ## 测试目的
/// 验证 FileWriter::write_str() 能够将文本内容写入文件。
///
/// ## 测试场景
/// 1. 创建 FileWriter
/// 2. 写入字符串内容
/// 3. 验证文件内容正确
///
/// ## 预期结果
/// - 文件内容与写入的内容一致
#[rstest]
fn test_file_writer_write_str_with_text_content_writes_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和文件路径
    let file_path = cli_env.path().join("output.txt");
    let writer = FileWriter::new(&file_path);
    let content = "Test content";

    // Act: 写入字符串内容
    writer.write_str(content)?;

    // Assert: 验证文件内容正确
    let read_content = fs::read_to_string(&file_path)?;
    assert_eq!(read_content, content);

    Ok(())
}

/// 测试写入文本内容并自动创建目录
///
/// ## 测试目的
/// 验证 FileWriter::write_str_with_dir() 能够自动创建目录并写入文件。
///
/// ## 测试场景
/// 1. 创建指向子目录文件的 FileWriter
/// 2. 使用 write_str_with_dir 写入内容
/// 3. 验证目录和文件都已创建
///
/// ## 预期结果
/// - 目录被自动创建，文件内容正确
#[rstest]
fn test_file_writer_write_str_with_dir_creates_dir_and_writes_file(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和子目录文件路径
    let file_path = cli_env.path().join("subdir/output.txt");
    let writer = FileWriter::new(&file_path);
    let content = "Test content";

    // Act: 写入字符串内容（自动创建目录）
    writer.write_str_with_dir(content)?;

    // Assert: 验证文件存在且内容正确
    assert!(file_path.exists());
    let read_content = fs::read_to_string(&file_path)?;
    assert_eq!(read_content, content);

    Ok(())
}

/// 测试写入二进制内容到文件
///
/// ## 测试目的
/// 验证 FileWriter::write_bytes() 能够将二进制内容写入文件。
///
/// ## 测试场景
/// 1. 创建 FileWriter
/// 2. 写入字节内容
/// 3. 验证文件内容正确
///
/// ## 预期结果
/// - 文件内容与写入的字节内容一致
#[rstest]
fn test_file_writer_write_bytes_with_binary_content_writes_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和文件路径
    let file_path = cli_env.path().join("output.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"binary data";

    // Act: 写入字节内容
    writer.write_bytes(test_bytes)?;

    // Assert: 验证文件内容正确
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

/// 测试写入二进制内容并自动创建目录
///
/// ## 测试目的
/// 验证 FileWriter::write_bytes_with_dir() 能够自动创建目录并写入二进制文件。
///
/// ## 测试场景
/// 1. 创建指向子目录文件的 FileWriter
/// 2. 使用 write_bytes_with_dir 写入字节内容
/// 3. 验证目录和文件都已创建
///
/// ## 预期结果
/// - 目录被自动创建，文件内容正确
#[rstest]
fn test_file_writer_write_bytes_with_dir_creates_dir_and_writes_file(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和子目录文件路径
    let file_path = cli_env.path().join("subdir/output.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"binary data";

    // Act: 写入字节内容（自动创建目录）
    writer.write_bytes_with_dir(test_bytes)?;

    // Assert: 验证文件存在且内容正确
    assert!(file_path.exists());
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

/// 测试确保父目录存在
///
/// ## 测试目的
/// 验证 FileWriter::ensure_parent_dir() 能够创建嵌套路径的父目录。
///
/// ## 测试场景
/// 1. 创建指向嵌套路径的 FileWriter
/// 2. 调用 ensure_parent_dir()
/// 3. 验证父目录已创建
///
/// ## 预期结果
/// - 所有父目录都被创建
#[rstest]
fn test_file_writer_ensure_parent_dir_with_nested_path_creates_dirs(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备嵌套路径
    let file_path = cli_env.path().join("deep/nested/path/file.txt");
    let writer = FileWriter::new(&file_path);

    // Act: 确保父目录存在
    writer.ensure_parent_dir()?;

    // Assert: 验证父目录已创建
    assert!(file_path
        .parent()
        .expect("file path should have a parent")
        .exists());

    Ok(())
}

// ==================== FileReader Format Parsing Tests ====================

/// 测试读取并解析 TOML 文件
///
/// ## 测试目的
/// 验证 FileReader::toml() 能够读取并解析 TOML 配置文件。
///
/// ## 测试场景
/// 1. 创建包含有效 TOML 的临时文件
/// 2. 使用 FileReader 读取并解析 TOML
/// 3. 验证解析的配置正确
///
/// ## 预期结果
/// - TOML 文件被正确解析为配置结构
#[rstest]
fn test_file_reader_toml_with_valid_toml_parses_config_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备 TOML 文件
    let file_path = cli_env.path().join("config.toml");
    fs::write(&file_path, r#"
[section]
key = "value"
number = 42
"#)?;

    #[derive(serde::Deserialize)]
    struct Config {
        section: Section,
    }
    #[derive(serde::Deserialize)]
    struct Section {
        key: String,
        number: i32,
    }

    // Act: 读取并解析 TOML 文件
    let reader = FileReader::new(&file_path);
    let config: Config = reader.toml()?;

    // Assert: 验证解析的配置正确
    assert_eq!(config.section.key, "value");
    assert_eq!(config.section.number, 42);

    Ok(())
}

/// 测试读取并解析 JSON 文件
///
/// ## 测试目的
/// 验证 FileReader::json() 能够读取并解析 JSON 配置文件。
///
/// ## 测试场景
/// 1. 创建包含有效 JSON 的临时文件
/// 2. 使用 FileReader 读取并解析 JSON
/// 3. 验证解析的配置正确
///
/// ## 预期结果
/// - JSON 文件被正确解析为配置结构
#[rstest]
fn test_file_reader_json_with_valid_json_parses_config_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备 JSON 文件
    let file_path = cli_env.path().join("config.json");
    fs::write(&file_path, r#"{"key": "value", "number": 42}"#)?;

    #[derive(serde::Deserialize)]
    struct Config {
        key: String,
        number: i32,
    }

    // Act: 读取并解析 JSON 文件
    let reader = FileReader::new(&file_path);
    let config: Config = reader.json()?;

    // Assert: 验证解析的配置正确
    assert_eq!(config.key, "value");
    assert_eq!(config.number, 42);

    Ok(())
}

// ==================== FileWriter Format Writing Tests ====================

/// 测试写入 TOML 配置到文件
///
/// ## 测试目的
/// 验证 FileWriter::write_toml() 能够将配置结构序列化为 TOML 并写入文件。
///
/// ## 测试场景
/// 1. 创建配置结构
/// 2. 使用 FileWriter 写入 TOML 配置
/// 3. 验证文件内容包含预期字段
///
/// ## 预期结果
/// - 配置被正确序列化为 TOML 格式并写入文件
#[rstest]
fn test_file_writer_write_toml_with_valid_config_writes_toml_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备配置结构和文件路径
    let file_path = cli_env.path().join("config.toml");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        section: Section,
    }
    #[derive(serde::Serialize)]
    struct Section {
        key: String,
        number: i32,
    }

    let config = Config {
        section: Section {
            key: "value".to_string(),
            number: 42,
        },
    };

    // Act: 写入 TOML 配置
    writer.write_toml(&config)?;

    // Assert: 验证文件内容包含预期字段
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("key = \"value\""));
    assert!(content.contains("number = 42"));

    Ok(())
}

/// 测试写入 JSON 配置到文件
///
/// ## 测试目的
/// 验证 FileWriter::write_json() 能够将配置结构序列化为 JSON 并写入文件。
///
/// ## 测试场景
/// 1. 创建配置结构
/// 2. 使用 FileWriter 写入 JSON 配置
/// 3. 验证文件内容包含预期字段
///
/// ## 预期结果
/// - 配置被正确序列化为 JSON 格式并写入文件
#[rstest]
fn test_file_writer_write_json_with_valid_config_writes_json_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备配置结构和文件路径
    let file_path = cli_env.path().join("config.json");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        key: String,
        number: i32,
    }

    let config = Config {
        key: "value".to_string(),
        number: 42,
    };

    // Act: 写入 JSON 配置
    writer.write_json(&config)?;

    // Assert: 验证文件内容包含预期字段
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("\"key\":\"value\""));
    assert!(content.contains("\"number\":42"));

    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试读取不存在文件时的错误处理
///
/// ## 测试目的
/// 验证 FileReader 在读取不存在的文件时返回错误。
///
/// ## 测试场景
/// 1. 创建指向不存在文件的 FileReader
/// 2. 尝试读取文件
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 返回文件不存在错误
#[test]
fn test_file_reader_nonexistent_file_with_invalid_path_returns_error() {
    // Arrange: 准备不存在的文件路径
    let reader = FileReader::new("/nonexistent/path/file.txt");

    // Act: 尝试读取文件
    let result = reader.to_string();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试写入文件时自动创建不存在的父目录
///
/// ## 测试目的
/// 验证 FileWriter::write_str_with_dir() 在父目录不存在时能够自动创建。
///
/// ## 测试场景
/// 1. 创建指向不存在父目录的 FileWriter
/// 2. 使用 write_str_with_dir 写入内容
/// 3. 验证父目录和文件都已创建
///
/// ## 预期结果
/// - 父目录被自动创建，文件写入成功
#[rstest]
fn test_file_writer_nonexistent_parent_with_missing_dir_creates_dir(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备不存在的父目录路径
    let file_path = cli_env.path().join("nonexistent/parent/file.txt");
    let writer = FileWriter::new(&file_path);

    // Act: 使用 write_str_with_dir 自动创建父目录
    writer.write_str_with_dir("content")?;

    // Assert: 验证文件已创建
    assert!(file_path.exists());

    Ok(())
}

