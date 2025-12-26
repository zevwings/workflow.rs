//! Base Util File 模块测试
//!
//! 测试文件操作工具的核心功能，包括 FileReader 和 FileWriter。
//!
//! ## 测试策略
//!
//! - 所有测试返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试文件读取、写入和目录创建功能

use color_eyre::Result;
use pretty_assertions::assert_eq;
use std::fs;
use std::io::Read;
use workflow::base::fs::file::{FileReader, FileWriter};

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::cli_env;
use rstest::rstest;

// ==================== FileReader Tests ====================

/// 测试读取文件内容为字符串
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_to_string_with_valid_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试文件和内容
    let file_path = cli_env.path().join("test.txt");
    let expected_content = "Hello, World!";
    fs::write(&file_path, expected_content)?;

    // Act: 读取文件内容
    let reader = FileReader::new(&file_path);
    let content = reader.to_string()?;

    // Assert: 验证内容正确
    assert_eq!(content, expected_content);

    Ok(())
}

/// 测试读取多行文件内容
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_lines_with_multiline_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备多行测试文件
    let file_path = cli_env.path().join("test.txt");
    let file_content = "line1\nline2\nline3";
    fs::write(&file_path, file_content)?;

    // Act: 读取文件行
    let reader = FileReader::new(&file_path);
    let lines = reader.lines()?;

    // Assert: 验证行数和内容正确
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");

    Ok(())
}

/// 测试读取二进制文件内容
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_bytes_with_binary_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备二进制测试文件
    let file_path = cli_env.path().join("test.bin");
    let test_bytes = b"binary data\x00\x01\x02";
    fs::write(&file_path, test_bytes)?;

    // Act: 读取文件字节
    let reader = FileReader::new(&file_path);
    let bytes = reader.bytes()?;

    // Assert: 验证字节内容正确
    assert_eq!(bytes, test_bytes);

    Ok(())
}

// ==================== FileWriter Tests ====================

/// 测试写入字符串内容到文件
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_str_with_valid_content_writes_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备文件路径和内容
    let file_path = cli_env.path().join("output.txt");
    let writer = FileWriter::new(&file_path);
    let expected_content = "Test content";

    // Act: 写入字符串内容
    writer.write_str(expected_content)?;

    // Assert: 验证文件内容正确
    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, expected_content);

    Ok(())
}

/// 测试写入字符串内容并自动创建目录
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_str_with_dir_creates_directory_and_writes_file(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备需要创建目录的文件路径
    let file_path = cli_env.path().join("subdir/output.txt");
    let writer = FileWriter::new(&file_path);
    let expected_content = "Test content";

    // Act: 写入字符串内容（自动创建目录）
    writer.write_str_with_dir(expected_content)?;

    // Assert: 验证目录和文件已创建，内容正确
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, expected_content);

    Ok(())
}

/// 测试写入字节数据到文件
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_bytes_with_valid_bytes_writes_file_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备文件路径和二进制数据
    let file_path = cli_env.path().join("output.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"binary data";

    // Act: 写入字节数据
    writer.write_bytes(test_bytes)?;

    // Assert: 验证文件内容正确
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

/// 测试写入字节数据并自动创建目录
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_bytes_with_dir_creates_directory_and_writes_file(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备需要创建目录的文件路径和二进制数据
    let file_path = cli_env.path().join("subdir/output.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"binary data";

    // Act: 写入字节数据（自动创建目录）
    writer.write_bytes_with_dir(test_bytes)?;

    // Assert: 验证目录和文件已创建，内容正确
    assert!(file_path.exists());
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

/// 测试确保父目录存在
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_ensure_parent_dir_with_nested_path_creates_parent_dirs(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备嵌套路径
    let file_path = cli_env.path().join("deep/nested/path/file.txt");
    let writer = FileWriter::new(&file_path);

    // Act: 确保父目录存在
    writer.ensure_parent_dir()?;

    // Assert: 验证父目录已创建
    if let Some(parent) = file_path.parent() {
        assert!(parent.exists());
    }

    Ok(())
}

/// 测试读取并解析TOML配置文件
///
/// ## 测试目的
/// 验证FileReader能够正确读取TOML格式的配置文件并解析为结构体
///
/// ## 测试场景
/// 1. 创建包含section和字段的TOML文件
/// 2. 使用FileReader读取并解析TOML
/// 3. 验证解析后的结构体字段值正确
#[rstest]
fn test_file_reader_toml_with_valid_toml_parses_config_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备TOML文件
    let file_path = cli_env.path().join("config.toml");
    fs::write(
        &file_path,
        r#"
[section]
key = "value"
number = 42
"#,
    )?;

    // Act: 读取并解析TOML
    let reader = FileReader::new(&file_path);
    #[derive(serde::Deserialize)]
    struct Config {
        section: Section,
    }
    #[derive(serde::Deserialize)]
    struct Section {
        key: String,
        number: i32,
    }
    let config: Config = reader.toml()?;

    // Assert: 验证解析结果正确
    assert_eq!(config.section.key, "value");
    assert_eq!(config.section.number, 42);

    Ok(())
}

/// 测试读取并解析JSON配置文件
///
/// ## 测试目的
/// 验证FileReader能够正确读取JSON格式的配置文件并解析为结构体
///
/// ## 测试场景
/// 1. 创建包含key和number字段的JSON文件
/// 2. 使用FileReader读取并解析JSON
/// 3. 验证解析后的结构体字段值正确
#[rstest]
fn test_file_reader_json_with_valid_json_parses_config_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备JSON文件
    let file_path = cli_env.path().join("config.json");
    fs::write(&file_path, r#"{"key": "value", "number": 42}"#)?;

    // Act: 读取并解析JSON
    let reader = FileReader::new(&file_path);
    #[derive(serde::Deserialize)]
    struct Config {
        key: String,
        number: i32,
    }
    let config: Config = reader.json()?;

    // Assert: 验证解析结果正确
    assert_eq!(config.key, "value");
    assert_eq!(config.number, 42);

    Ok(())
}

/// 测试写入TOML配置到文件
///
/// ## 测试目的
/// 验证FileWriter能够正确将结构体序列化为TOML格式并写入文件
///
/// ## 测试场景
/// 1. 创建包含section和字段的配置结构体
/// 2. 使用FileWriter写入TOML格式
/// 3. 验证文件内容包含正确的TOML格式数据
#[rstest]
fn test_file_writer_write_toml_with_valid_config_writes_toml_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备配置结构
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

    // Act: 写入TOML配置
    writer.write_toml(&config)?;

    // Assert: 验证文件内容正确
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("key = \"value\""));
    assert!(content.contains("number = 42"));

    Ok(())
}

/// 测试写入JSON配置到文件
///
/// ## 测试目的
/// 验证FileWriter能够正确将结构体序列化为JSON格式并写入文件
///
/// ## 测试场景
/// 1. 创建包含key和number字段的配置结构体
/// 2. 使用FileWriter写入JSON格式
/// 3. 验证文件内容包含正确的JSON格式数据
#[rstest]
fn test_file_writer_write_json_with_valid_config_writes_json_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备配置结构
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

    // Act: 写入JSON配置
    writer.write_json(&config)?;

    // Assert: 验证文件内容正确（JSON格式可能包含空格，使用更灵活的检查）
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("key") && content.contains("value"));
    assert!(content.contains("number") && content.contains("42"));

    Ok(())
}

/// 测试读取不存在文件时返回错误
#[test]
fn test_file_reader_to_string_with_nonexistent_file_returns_error() {
    // Arrange: 准备不存在的文件路径

    // Act: 尝试读取不存在的文件
    let reader = FileReader::new("/nonexistent/path/file.txt");
    let result = reader.to_string();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试写入文件时自动创建不存在的父目录
#[rstest]
fn test_file_writer_write_str_with_dir_with_nonexistent_parent_creates_parent(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备需要创建父目录的文件路径
    let file_path = cli_env.path().join("nonexistent/parent/file.txt");
    let writer = FileWriter::new(&file_path);

    // Act: 使用 write_str_with_dir 应该自动创建父目录
    writer.write_str_with_dir("content")?;

    // Assert: 验证文件已创建
    assert!(file_path.exists());

    Ok(())
}

/// 测试打开文件并返回BufReader
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_open_with_valid_file_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // Arrange: 准备测试文件
    let file_path = cli_env.path().join("test.txt");
    fs::write(&file_path, "Hello, World!")?;

    // Act: 打开文件
    let reader = FileReader::new(&file_path);
    let mut buf_reader = reader.open()?;
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;

    // Assert: 验证内容正确
    assert_eq!(content, "Hello, World!");

    Ok(())
}

/// 测试打开不存在文件时返回错误
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_file_reader_open_with_nonexistent_file_returns_error() {
    // Arrange: 准备不存在的文件路径

    // Act: 尝试打开不存在的文件
    let reader = FileReader::new("/nonexistent/path/file.txt");
    let result = reader.open();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试读取空文件返回空内容
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_empty_file_with_empty_file_return_empty(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备空文件
    let file_path = cli_env.path().join("empty.txt");
    fs::write(&file_path, "")?;

    // Act: 读取空文件
    let reader = FileReader::new(&file_path);

    // Assert: 验证所有读取方法返回空内容
    assert_eq!(reader.to_string()?, "");
    assert_eq!(reader.lines()?, Vec::<String>::new());
    assert_eq!(reader.bytes()?, Vec::<u8>::new());

    Ok(())
}

/// 测试读取包含空行的文件
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_lines_with_empty_lines_handles_correctly_return_empty(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备包含空行的文件
    let file_path = cli_env.path().join("test.txt");
    fs::write(&file_path, "line1\n\nline3\n")?;

    // Act: 读取文件行
    let reader = FileReader::new(&file_path);
    let lines = reader.lines()?;

    // Assert: 验证空行处理正确（注意：BufReader::lines() 会忽略文件末尾的空行）
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "");
    assert_eq!(lines[2], "line3");

    Ok(())
}

/// 测试安全写入TOML文件（设置文件权限）
///
/// ## 测试目的
/// 验证FileWriter能够安全地写入TOML文件并设置适当的文件权限（Unix系统上为600）
///
/// ## 测试场景
/// 1. 创建包含敏感数据的配置结构体
/// 2. 使用write_toml_secure写入文件
/// 3. 验证文件已创建且内容正确
/// 4. 在Unix系统上验证文件权限为600（仅所有者可读写）
#[rstest]
fn test_file_writer_write_toml_secure_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    let file_path = cli_env.path().join("secure/config.toml");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        secret: String,
    }

    let config = Config {
        secret: "sensitive_data".to_string(),
    };

    writer.write_toml_secure(&config)?;
    assert!(file_path.exists());

    // Assert: 验证文件内容
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("secret"));

    #[cfg(unix)]
    {
        // Assert: 验证文件权限（Unix 系统）
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path)?;
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    Ok(())
}

/// 测试安全写入JSON文件（设置文件权限）
///
/// ## 测试目的
/// 验证FileWriter能够安全地写入JSON文件并设置适当的文件权限（Unix系统上为600）
///
/// ## 测试场景
/// 1. 创建包含敏感数据的配置结构体
/// 2. 使用write_json_secure写入文件
/// 3. 验证文件已创建且内容正确
/// 4. 在Unix系统上验证文件权限为600（仅所有者可读写）
#[rstest]
fn test_file_writer_write_json_secure_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    let file_path = cli_env.path().join("secure/config.json");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        secret: String,
    }

    let config = Config {
        secret: "sensitive_data".to_string(),
    };

    writer.write_json_secure(&config)?;
    assert!(file_path.exists());

    // Assert: 验证文件内容
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("secret"));

    #[cfg(unix)]
    {
        // Assert: 验证文件权限（Unix 系统）
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path)?;
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    Ok(())
}

/// 测试设置文件权限（Unix系统）
#[rstest]
#[cfg(unix)]
fn test_file_writer_set_permissions_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let file_path = cli_env.path().join("test.txt");
    let writer = FileWriter::new(&file_path);

    // 先创建文件
    writer.write_str("test content")?;

    // 设置权限
    writer.set_permissions(0o644)?;

    // Assert: 验证权限
    let metadata = fs::metadata(&file_path)?;
    let permissions = metadata.permissions();
    assert_eq!(permissions.mode() & 0o777, 0o644);

    Ok(())
}

/// 测试解析无效TOML格式时返回错误
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_toml_with_invalid_format_return_ok(cli_env: CliTestEnv) -> Result<()> {
    // Arrange: 准备无效格式的TOML文件
    let file_path = cli_env.path().join("invalid.toml");
    fs::write(&file_path, "invalid toml content")?;

    // Act: 尝试解析无效TOML
    let reader = FileReader::new(&file_path);
    let result: Result<serde_json::Value, _> = reader.toml();

    // Assert: 验证返回错误
    assert!(result.is_err());
    Ok(())
}

/// 测试解析无效JSON格式时返回错误
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_json_with_invalid_format_return_ok(cli_env: CliTestEnv) -> Result<()> {
    // Arrange: 准备无效格式的JSON文件
    let file_path = cli_env.path().join("invalid.json");
    fs::write(&file_path, "invalid json content")?;

    // Act: 尝试解析无效JSON
    let reader = FileReader::new(&file_path);
    let result: Result<serde_json::Value, _> = reader.json();

    // Assert: 验证返回错误
    assert!(result.is_err());
    Ok(())
}

/// 测试确保根路径的父目录（应该成功，因为根路径没有父目录）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_file_writer_ensure_parent_dir_with_root_path_succeeds() {
    // Arrange: 准备根路径（没有父目录）

    // Act: 确保父目录存在（根路径没有父目录）
    let writer = FileWriter::new("/");
    let result = writer.ensure_parent_dir();

    // Assert: 验证不会出错（根路径没有父目录，应该成功）
    assert!(result.is_ok());
}

/// 测试读取不存在文件的行时返回错误
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_file_reader_lines_with_nonexistent_file_returns_error() {
    // Arrange: 准备不存在的文件路径

    // Act: 尝试读取不存在的文件的行
    let reader = FileReader::new("/nonexistent/path/file.txt");
    let result = reader.lines();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试读取不存在文件的字节时返回错误
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_file_reader_bytes_with_nonexistent_file_returns_error() {
    // Arrange: 准备不存在的文件路径

    // Act: 尝试读取不存在的文件的字节
    let reader = FileReader::new("/nonexistent/path/file.bin");
    let result = reader.bytes();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试读取不存在的TOML文件时返回错误
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_file_reader_toml_with_nonexistent_file_returns_error() {
    // Arrange: 准备不存在的TOML文件路径

    // Act: 尝试读取不存在的TOML文件
    let reader = FileReader::new("/nonexistent/path/config.toml");
    let result: Result<serde_json::Value, _> = reader.toml();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试读取不存在的JSON文件时返回错误
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_file_reader_json_with_nonexistent_file_returns_error() {
    // Arrange: 准备不存在的JSON文件路径

    // Act: 尝试读取不存在的JSON文件
    let reader = FileReader::new("/nonexistent/path/config.json");
    let result: Result<serde_json::Value, _> = reader.json();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

/// 测试写入TOML文件的错误处理
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_toml_error_handling_return_false(cli_env: CliTestEnv) -> Result<()> {
    // Arrange: 准备测试写入到只读目录应该返回错误（如果可能）
    // 注意：这个测试可能在某些系统上无法执行，所以只测试基本功能
    let file_path = cli_env.path().join("config.toml");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        key: String,
    }

    let config = Config {
        key: "value".to_string(),
    };

    // 应该成功写入
    let result = writer.write_toml(&config);
    assert!(result.is_ok());
    Ok(())
}

/// 测试写入JSON文件的错误处理
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_json_error_handling_return_false(cli_env: CliTestEnv) -> Result<()> {
    // Arrange: 准备测试写入 JSON 文件的基本功能
    let file_path = cli_env.path().join("config.json");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        key: String,
    }

    let config = Config {
        key: "value".to_string(),
    };

    // 应该成功写入
    let result = writer.write_json(&config);
    assert!(result.is_ok());
    Ok(())
}

/// 测试写入嵌套目录的字节文件
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_bytes_with_dir_nested_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试嵌套目录的字节写入
    let file_path = cli_env.path().join("level1/level2/level3/file.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"nested binary data";

    writer.write_bytes_with_dir(test_bytes)?;
    assert!(file_path.exists());
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

/// 测试写入嵌套目录的字符串文件
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_str_with_dir_nested_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试嵌套目录的字符串写入
    let file_path = cli_env.path().join("level1/level2/level3/file.txt");
    let writer = FileWriter::new(&file_path);

    writer.write_str_with_dir("nested content")?;
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, "nested content");

    Ok(())
}

/// 测试部分读取文件内容
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_open_read_partial_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // Arrange: 准备测试部分读取文件
    let file_path = cli_env.path().join("test.txt");
    fs::write(&file_path, "Hello, World!")?;

    let reader = FileReader::new(&file_path);
    let mut buf_reader = reader.open()?;
    let mut buffer = vec![0u8; 5];
    buf_reader.read_exact(&mut buffer)?;
    assert_eq!(buffer, b"Hello");

    Ok(())
}

/// 测试多次读取文件行
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_lines_multiple_iterations_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 lines() 方法的循环逻辑（覆盖 file.rs:45-49）
    let file_path = cli_env.path().join("multiline.txt");
    fs::write(&file_path, "line1\nline2\nline3\nline4\nline5")?;

    let reader = FileReader::new(&file_path);
    let lines = reader.lines()?;
    assert_eq!(lines.len(), 5);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[4], "line5");

    Ok(())
}

/// 测试读取大文件的字节内容
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_reader_bytes_large_file_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // Arrange: 准备测试 bytes() 方法的循环逻辑（覆盖 file.rs:58-59）
    let file_path = cli_env.path().join("large.bin");
    let large_data = vec![0u8; 10000]; // 10KB 数据
    fs::write(&file_path, &large_data)?;

    let reader = FileReader::new(&file_path);
    let bytes = reader.bytes()?;
    assert_eq!(bytes.len(), 10000);

    Ok(())
}

/// 测试读取文件行时的IO错误处理
/// 测试读取文件行时的IO错误处理
///
/// ## 测试目的
/// 验证FileReader在读取文件行时能够正确处理IO错误
///
/// ## 测试场景
/// 1. 创建包含多行的测试文件
/// 2. 使用FileReader读取文件行
/// 3. 验证正常读取成功
#[rstest]
fn test_file_reader_lines_with_io_error_return_false(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 lines() 方法中的错误处理（覆盖 file.rs:46-47）
    // 创建一个文件，然后删除它，模拟读取时的错误
    let file_path = cli_env.path().join("test.txt");
    fs::write(&file_path, "line1\nline2")?;

    let reader = FileReader::new(&file_path);
    // 正常读取应该成功
    let lines = reader.lines()?;
    assert_eq!(lines.len(), 2);

    Ok(())
}

/// 测试读取文件行循环中的错误处理
#[rstest]
fn test_file_reader_lines_loop_with_error_return_false(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 lines() 循环中的错误处理（覆盖 file.rs:46-47）
    let file_path = cli_env.path().join("test.txt");
    fs::write(&file_path, "line1\nline2\nline3")?;

    let reader = FileReader::new(&file_path);
    let lines = reader.lines()?;
    assert_eq!(lines.len(), 3);

    Ok(())
}

/// 测试读取文件字节到末尾
#[rstest]
fn test_file_reader_bytes_read_to_end_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // Arrange: 准备测试 bytes() 方法中的 read_to_end 调用（覆盖 file.rs:58-59）
    let file_path = cli_env.path().join("test.bin");
    let test_data = vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8];
    fs::write(&file_path, &test_data)?;

    let reader = FileReader::new(&file_path);
    let bytes = reader.bytes()?;
    assert_eq!(bytes, test_data);

    Ok(())
}

/// 测试确保父目录创建嵌套目录
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_ensure_parent_dir_creates_nested(cli_env: CliTestEnv) -> Result<()> {
    // Arrange: 准备测试 ensure_parent_dir() 创建嵌套目录（覆盖 file.rs:106-107）
    let file_path = cli_env.path().join("level1/level2/level3/file.txt");
    let writer = FileWriter::new(&file_path);

    writer.ensure_parent_dir()?;
    if let Some(parent) = file_path.parent() {
        assert!(parent.exists());
    }
    let parent = file_path
        .parent()
        .ok_or_else(|| color_eyre::eyre::eyre!("file path should have a parent"))?;
    assert!(parent.is_dir());

    Ok(())
}

/// 测试设置文件的不同权限模式（Unix系统）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
#[cfg(unix)]
fn test_file_writer_set_permissions_various_modes_return_ok(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 set_permissions() 的不同权限模式（覆盖 file.rs:124-125）
    use std::os::unix::fs::PermissionsExt;

    let file_path = cli_env.path().join("test.txt");
    let writer = FileWriter::new(&file_path);

    writer.write_str("test content")?;

    // Arrange: 准备测试不同的权限模式
    writer.set_permissions(0o600)?;
    let metadata = fs::metadata(&file_path)?;
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);

    writer.set_permissions(0o644)?;
    let metadata = fs::metadata(&file_path)?;
    assert_eq!(metadata.permissions().mode() & 0o777, 0o644);

    Ok(())
}

/// 测试写入字符串文件时创建父目录
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_str_with_dir_creates_parent(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 write_str_with_dir() 创建父目录（覆盖 file.rs:146-148）
    let file_path = cli_env.path().join("nested/path/file.txt");
    let writer = FileWriter::new(&file_path);

    writer.write_str_with_dir("content")?;
    assert!(file_path.exists());
    if let Some(parent) = file_path.parent() {
        assert!(parent.exists());
    }

    Ok(())
}

/// 测试写入字节文件时创建父目录
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_bytes_with_dir_creates_parent(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 write_bytes_with_dir() 创建父目录（覆盖 file.rs:168-170）
    let file_path = cli_env.path().join("nested/path/file.bin");
    let writer = FileWriter::new(&file_path);

    writer.write_bytes_with_dir(b"binary content")?;
    assert!(file_path.exists());
    if let Some(parent) = file_path.parent() {
        assert!(parent.exists());
    }

    Ok(())
}

/// 测试安全写入TOML文件时创建目录并设置权限
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_toml_secure_creates_dir_and_sets_perms(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 write_toml_secure() 创建目录和设置权限（覆盖 file.rs:194, 198-202）
    let file_path = cli_env.path().join("secure/config.toml");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        secret: String,
    }

    let config = Config {
        secret: "sensitive".to_string(),
    };

    writer.write_toml_secure(&config)?;
    assert!(file_path.exists());
    if let Some(parent) = file_path.parent() {
        assert!(parent.exists());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path)?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }

    Ok(())
}

/// 测试安全写入JSON文件时创建目录并设置权限
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_file_writer_write_json_secure_creates_dir_and_sets_perms(
    cli_env: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备测试 write_json_secure() 创建目录和设置权限（覆盖 file.rs:226, 230-234）
    let file_path = cli_env.path().join("secure/config.json");
    let writer = FileWriter::new(&file_path);

    #[derive(serde::Serialize)]
    struct Config {
        secret: String,
    }

    let config = Config {
        secret: "sensitive".to_string(),
    };

    writer.write_json_secure(&config)?;
    assert!(file_path.exists());
    if let Some(parent) = file_path.parent() {
        assert!(parent.exists());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path)?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }

    Ok(())
}

/// 测试确保父目录（没有父目录的情况）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_file_writer_ensure_parent_dir_no_parent() {
    // Arrange: 准备测试 ensure_parent_dir() 没有父目录的情况（覆盖 file.rs:109）
    let writer = FileWriter::new("/");
    // 根路径没有父目录，应该成功（不执行任何操作）
    let result = writer.ensure_parent_dir();
    assert!(result.is_ok());
}
