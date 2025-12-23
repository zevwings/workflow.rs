//! Base Util File 模块测试
//!
//! 测试文件操作工具的核心功能，包括 FileReader 和 FileWriter。

use pretty_assertions::assert_eq;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use workflow::base::util::file::{FileReader, FileWriter};

#[test]
fn test_file_reader_new() {
    let reader = FileReader::new("test.txt");
    assert_eq!(reader.path, std::path::PathBuf::from("test.txt"));
}

#[test]
fn test_file_reader_pathbuf() {
    let path = std::path::PathBuf::from("test/path.txt");
    let reader = FileReader::new(path.clone());
    assert_eq!(reader.path, path);
}

#[test]
fn test_file_writer_new() {
    let writer = FileWriter::new("test.txt");
    assert_eq!(writer.path, std::path::PathBuf::from("test.txt"));
}

#[test]
fn test_file_writer_pathbuf() {
    let path = std::path::PathBuf::from("test/path.txt");
    let writer = FileWriter::new(path.clone());
    assert_eq!(writer.path, path);
}

#[test]
fn test_file_reader_to_string() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Hello, World!")?;

    let reader = FileReader::new(&file_path);
    let content = reader.to_string()?;
    assert_eq!(content, "Hello, World!");

    Ok(())
}

#[test]
fn test_file_reader_lines() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "line1\nline2\nline3")?;

    let reader = FileReader::new(&file_path);
    let lines = reader.lines()?;
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");

    Ok(())
}

#[test]
fn test_file_reader_bytes() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.bin");
    let test_bytes = b"binary data\x00\x01\x02";
    fs::write(&file_path, test_bytes)?;

    let reader = FileReader::new(&file_path);
    let bytes = reader.bytes()?;
    assert_eq!(bytes, test_bytes);

    Ok(())
}

#[test]
fn test_file_writer_write_str() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("output.txt");
    let writer = FileWriter::new(&file_path);

    writer.write_str("Test content")?;
    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, "Test content");

    Ok(())
}

#[test]
fn test_file_writer_write_str_with_dir() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("subdir/output.txt");
    let writer = FileWriter::new(&file_path);

    writer.write_str_with_dir("Test content")?;
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, "Test content");

    Ok(())
}

#[test]
fn test_file_writer_write_bytes() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("output.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"binary data";

    writer.write_bytes(test_bytes)?;
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

#[test]
fn test_file_writer_write_bytes_with_dir() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("subdir/output.bin");
    let writer = FileWriter::new(&file_path);
    let test_bytes = b"binary data";

    writer.write_bytes_with_dir(test_bytes)?;
    assert!(file_path.exists());
    let content = fs::read(&file_path)?;
    assert_eq!(content, test_bytes);

    Ok(())
}

#[test]
fn test_file_writer_ensure_parent_dir() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("deep/nested/path/file.txt");
    let writer = FileWriter::new(&file_path);

    writer.ensure_parent_dir()?;
    assert!(file_path.parent().unwrap().exists());

    Ok(())
}

#[test]
fn test_file_reader_toml() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("config.toml");
    fs::write(&file_path, r#"
[section]
key = "value"
number = 42
"#)?;

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
    assert_eq!(config.section.key, "value");
    assert_eq!(config.section.number, 42);

    Ok(())
}

#[test]
fn test_file_reader_json() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("config.json");
    fs::write(&file_path, r#"{"key": "value", "number": 42}"#)?;

    let reader = FileReader::new(&file_path);
    #[derive(serde::Deserialize)]
    struct Config {
        key: String,
        number: i32,
    }

    let config: Config = reader.json()?;
    assert_eq!(config.key, "value");
    assert_eq!(config.number, 42);

    Ok(())
}

#[test]
fn test_file_writer_write_toml() -> color_eyre::Result<()> {
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

    writer.write_toml(&config)?;
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("key = \"value\""));
    assert!(content.contains("number = 42"));

    Ok(())
}

#[test]
fn test_file_writer_write_json() -> color_eyre::Result<()> {
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

    writer.write_json(&config)?;
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("\"key\":\"value\""));
    assert!(content.contains("\"number\":42"));

    Ok(())
}

#[test]
fn test_file_reader_nonexistent_file() {
    let reader = FileReader::new("/nonexistent/path/file.txt");
    let result = reader.to_string();
    assert!(result.is_err());
}

#[test]
fn test_file_writer_nonexistent_parent() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("nonexistent/parent/file.txt");
    let writer = FileWriter::new(&file_path);

    // 使用 write_str_with_dir 应该自动创建父目录
    writer.write_str_with_dir("content")?;
    assert!(file_path.exists());

    Ok(())
}

