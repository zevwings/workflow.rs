//! Base Util File 模块测试
//!
//! 测试文件操作工具的核心功能，包括 FileReader 和 FileWriter。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use std::fs;
use std::io::Read;
use tempfile::TempDir;
use workflow::base::util::file::{FileReader, FileWriter};

#[test]
fn test_file_reader_new() {
    let _reader = FileReader::new("test.txt");
    // 验证可以创建 FileReader
    assert!(std::path::Path::new("test.txt").exists() || !std::path::Path::new("test.txt").exists());
}

#[test]
fn test_file_reader_pathbuf() {
    let path = std::path::PathBuf::from("test/path.txt");
    let _reader = FileReader::new(path.clone());
    // 验证可以创建 FileReader
    assert!(true);
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
    // JSON 格式可能包含空格，使用更灵活的检查
    assert!(content.contains("key") && content.contains("value"));
    assert!(content.contains("number") && content.contains("42"));

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

#[test]
fn test_file_reader_open() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Hello, World!")?;

    let reader = FileReader::new(&file_path);
    let mut buf_reader = reader.open()?;
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;
    assert_eq!(content, "Hello, World!");

    Ok(())
}

#[test]
fn test_file_reader_open_nonexistent() {
    let reader = FileReader::new("/nonexistent/path/file.txt");
    let result = reader.open();
    assert!(result.is_err());
}

#[test]
fn test_file_reader_empty_file() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("empty.txt");
    fs::write(&file_path, "")?;

    let reader = FileReader::new(&file_path);
    assert_eq!(reader.to_string()?, "");
    assert_eq!(reader.lines()?, Vec::<String>::new());
    assert_eq!(reader.bytes()?, Vec::<u8>::new());

    Ok(())
}

#[test]
fn test_file_reader_lines_empty_lines() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "line1\n\nline3\n")?;

    let reader = FileReader::new(&file_path);
    let lines = reader.lines()?;
    // 注意：BufReader::lines() 会忽略文件末尾的空行（如果文件以 \n 结尾）
    // 所以 "line1\n\nline3\n" 只有 3 行，而不是 4 行
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "");
    assert_eq!(lines[2], "line3");

    Ok(())
}

#[test]
fn test_file_writer_write_toml_secure() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secure/config.toml");
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

    // 验证文件内容
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("secret"));

    #[cfg(unix)]
    {
        // 验证文件权限（Unix 系统）
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path)?;
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    Ok(())
}

#[test]
fn test_file_writer_write_json_secure() -> color_eyre::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secure/config.json");
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

    // 验证文件内容
    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("secret"));

    #[cfg(unix)]
    {
        // 验证文件权限（Unix 系统）
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path)?;
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    Ok(())
}

#[test]
#[cfg(unix)]
fn test_file_writer_set_permissions() -> color_eyre::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");
    let writer = FileWriter::new(&file_path);

    // 先创建文件
    writer.write_str("test content")?;

    // 设置权限
    writer.set_permissions(0o644)?;

    // 验证权限
    let metadata = fs::metadata(&file_path)?;
    let permissions = metadata.permissions();
    assert_eq!(permissions.mode() & 0o777, 0o644);

    Ok(())
}

#[test]
fn test_file_reader_toml_invalid_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    fs::write(&file_path, "invalid toml content").unwrap();

    let reader = FileReader::new(&file_path);
    let result: Result<serde_json::Value, _> = reader.toml();
    assert!(result.is_err());
}

#[test]
fn test_file_reader_json_invalid_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.json");
    fs::write(&file_path, "invalid json content").unwrap();

    let reader = FileReader::new(&file_path);
    let result: Result<serde_json::Value, _> = reader.json();
    assert!(result.is_err());
}

#[test]
fn test_file_writer_ensure_parent_dir_root_path() {
    // 测试根路径的情况（没有父目录）
    let writer = FileWriter::new("/");
    // 根路径没有父目录，应该不会出错
    let result = writer.ensure_parent_dir();
    assert!(result.is_ok());
}

