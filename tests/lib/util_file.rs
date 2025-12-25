//! Base Util File 模块测试
//!
//! 测试文件操作工具的核心功能，包括 FileReader 和 FileWriter。

use pretty_assertions::assert_eq;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use workflow::base::util::file::{FileReader, FileWriter};

// ==================== FileReader Initialization Tests ====================

#[test]
fn test_file_reader_new_with_str_path_creates_reader() {
    // Arrange: 准备字符串路径
    let path_str = "test.txt";

    // Act: 使用字符串路径创建 FileReader
    let reader = FileReader::new(path_str);

    // Assert: 验证路径正确设置
    assert_eq!(reader.path, std::path::PathBuf::from(path_str));
}

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

#[test]
fn test_file_writer_new_with_str_path_creates_writer() {
    // Arrange: 准备字符串路径
    let path_str = "test.txt";

    // Act: 使用字符串路径创建 FileWriter
    let writer = FileWriter::new(path_str);

    // Assert: 验证路径正确设置
    assert_eq!(writer.path, std::path::PathBuf::from(path_str));
}

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

#[test]
fn test_file_reader_to_string_with_text_file_reads_content() -> color_eyre::Result<()> {
    // Arrange: 准备临时文件和内容
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    let expected_content = "Hello, World!";
    fs::write(&file_path, expected_content)?;

    // Act: 读取文件内容为字符串
    let reader = FileReader::new(&file_path);
    let content = reader.to_string()?;

    // Assert: 验证读取的内容正确
    assert_eq!(content, expected_content);

    Ok(())
}

#[test]
fn test_file_reader_lines_with_multiline_file_reads_lines() -> color_eyre::Result<()> {
    // Arrange: 准备多行临时文件
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
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

#[test]
fn test_file_reader_bytes_with_binary_file_reads_bytes() -> color_eyre::Result<()> {
    // Arrange: 准备二进制临时文件
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.bin");
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

#[test]
fn test_file_writer_write_str_with_text_content_writes_file() -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和文件路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("output.txt");
    let writer = FileWriter::new(&file_path);
    let content = "Test content";

    // Act: 写入字符串内容
    writer.write_str(content)?;

    // Assert: 验证文件内容正确
    let read_content = fs::read_to_string(&file_path)?;
    assert_eq!(read_content, content);

    Ok(())
}

#[test]
fn test_file_writer_write_str_with_dir_creates_dir_and_writes_file() -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和子目录文件路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("subdir/output.txt");
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

#[test]
fn test_file_writer_write_bytes_with_binary_content_writes_file() -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和文件路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("output.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"binary data";

    // Act: 写入字节内容
    writer.write_bytes(test_bytes)?;

    // Assert: 验证文件内容正确
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

#[test]
fn test_file_writer_write_bytes_with_dir_creates_dir_and_writes_file() -> color_eyre::Result<()> {
    // Arrange: 准备临时目录和子目录文件路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("subdir/output.bin");
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

#[test]
fn test_file_writer_ensure_parent_dir_with_nested_path_creates_dirs() -> color_eyre::Result<()> {
    // Arrange: 准备嵌套路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("deep/nested/path/file.txt");
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

#[test]
fn test_file_reader_toml_with_valid_toml_parses_config() -> color_eyre::Result<()> {
    // Arrange: 准备 TOML 文件
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("config.toml");
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

#[test]
fn test_file_reader_json_with_valid_json_parses_config() -> color_eyre::Result<()> {
    // Arrange: 准备 JSON 文件
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("config.json");
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

#[test]
fn test_file_writer_write_toml_with_valid_config_writes_toml() -> color_eyre::Result<()> {
    // Arrange: 准备配置结构和文件路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("config.toml");
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

#[test]
fn test_file_writer_write_json_with_valid_config_writes_json() -> color_eyre::Result<()> {
    // Arrange: 准备配置结构和文件路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("config.json");
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

#[test]
fn test_file_reader_nonexistent_file_with_invalid_path_returns_error() {
    // Arrange: 准备不存在的文件路径
    let reader = FileReader::new("/nonexistent/path/file.txt");

    // Act: 尝试读取文件
    let result = reader.to_string();

    // Assert: 验证返回错误
    assert!(result.is_err());
}

#[test]
fn test_file_writer_nonexistent_parent_with_missing_dir_creates_dir() -> color_eyre::Result<()> {
    // Arrange: 准备不存在的父目录路径
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("nonexistent/parent/file.txt");
    let writer = FileWriter::new(&file_path);

    // Act: 使用 write_str_with_dir 自动创建父目录
    writer.write_str_with_dir("content")?;

    // Assert: 验证文件已创建
    assert!(file_path.exists());

    Ok(())
}

