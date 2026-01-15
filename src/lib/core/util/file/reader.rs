//! 文件读取器
//!
//! 提供基于路径的文件读取操作。

use color_eyre::{eyre::WrapErr, Result};
use serde::de::DeserializeOwned;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

/// 文件读取器，基于路径提供常用读取操作。
pub struct FileReader {
    path: PathBuf,
}

impl FileReader {
    /// 创建一个新的文件读取器。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 打开文件并返回 `BufReader<File>`。
    pub fn open(&self) -> Result<BufReader<File>> {
        let file = File::open(&self.path)
            .wrap_err_with(|| format!("Failed to open file: {:?}", self.path))?;
        Ok(BufReader::new(file))
    }

    /// 读取文件内容为字符串。
    pub fn to_string(&self) -> Result<String> {
        fs::read_to_string(&self.path)
            .wrap_err_with(|| format!("Failed to read file: {:?}", self.path))
    }

    /// 读取文件的所有行。
    pub fn lines(&self) -> Result<Vec<String>> {
        let file = File::open(&self.path)
            .wrap_err_with(|| format!("Failed to open file: {:?}", self.path))?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line =
                line.wrap_err_with(|| format!("Failed to read line from file: {:?}", self.path))?;
            lines.push(line);
        }
        Ok(lines)
    }

    /// 读取文件内容为字节向量。
    pub fn bytes(&self) -> Result<Vec<u8>> {
        let mut file = File::open(&self.path)
            .wrap_err_with(|| format!("Failed to open file: {:?}", self.path))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .wrap_err_with(|| format!("Failed to read file: {:?}", self.path))?;
        Ok(buffer)
    }

    /// 读取 TOML 文件并解析为类型 `T`。
    pub fn toml<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let content = fs::read_to_string(&self.path)
            .wrap_err_with(|| format!("Failed to read config file: {:?}", self.path))?;
        toml::from_str(&content)
            .wrap_err_with(|| format!("Failed to parse TOML config: {:?}", self.path))
    }

    /// 读取 JSON 文件并解析为类型 `T`。
    pub fn json<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let content = fs::read_to_string(&self.path)
            .wrap_err_with(|| format!("Failed to read JSON file: {:?}", self.path))?;
        serde_json::from_str(&content)
            .wrap_err_with(|| format!("Failed to parse JSON file: {:?}", self.path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use serde::Deserialize;
    use std::fs;
    use std::io::Read;
    use tempfile::tempdir;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        version: String,
        enabled: bool,
    }

    // ==================== FileReader::new 测试 ====================

    #[test]
    fn test_file_reader_new() {
        let reader = FileReader::new("/tmp/test.txt");
        // 验证可以创建实例（通过尝试读取来验证，即使文件不存在）
        let _ = reader.to_string(); // 这会失败，但验证 reader 可用
    }

    #[test]
    fn test_file_reader_new_with_pathbuf() {
        use std::path::PathBuf;
        let path = PathBuf::from("/tmp/test.txt");
        let reader = FileReader::new(path.clone());
        // 验证可以创建实例
        let _ = reader.to_string();
    }

    // ==================== 文件打开测试 ====================

    #[test]
    fn test_open() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path)?;

        let reader = FileReader::new(&file_path);
        let mut buf_reader = reader.open()?;

        // 验证可以读取
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;
        assert_eq!(content, "");

        Ok(())
    }

    #[test]
    fn test_open_nonexistent_file() {
        let reader = FileReader::new("/nonexistent/file.txt");
        let result = reader.open();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to open file"));
    }

    // ==================== 字符串读取测试 ====================

    #[test]
    fn test_to_string() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!\nThis is a test file.";

        std::fs::write(&file_path, content)?;

        let reader = FileReader::new(&file_path);
        let read_content = reader.to_string()?;

        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_to_string_empty_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path)?;

        let reader = FileReader::new(&file_path);
        let content = reader.to_string()?;

        assert_eq!(content, "");

        Ok(())
    }

    #[test]
    fn test_to_string_unicode() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, 世界!\n测试文件\némoji🚀";

        std::fs::write(&file_path, content)?;

        let reader = FileReader::new(&file_path);
        let read_content = reader.to_string()?;

        assert_eq!(read_content, content);

        Ok(())
    }

    // ==================== 行读取测试 ====================

    #[test]
    fn test_lines() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "line1\nline2\nline3";

        std::fs::write(&file_path, content)?;

        let reader = FileReader::new(&file_path);
        let lines = reader.lines()?;

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");

        Ok(())
    }

    #[test]
    fn test_lines_empty_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path)?;

        let reader = FileReader::new(&file_path);
        let lines = reader.lines()?;

        assert_eq!(lines.len(), 0);

        Ok(())
    }

    #[test]
    fn test_lines_with_empty_lines() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "line1\n\nline2\n\n\nline3";

        std::fs::write(&file_path, content)?;

        let reader = FileReader::new(&file_path);
        let lines = reader.lines()?;

        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "line2");
        assert_eq!(lines[3], "");
        assert_eq!(lines[4], "");
        assert_eq!(lines[5], "line3");

        Ok(())
    }

    #[test]
    fn test_lines_without_trailing_newline() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "line1\nline2";

        std::fs::write(&file_path, content)?;

        let reader = FileReader::new(&file_path);
        let lines = reader.lines()?;

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");

        Ok(())
    }

    // ==================== 字节读取测试 ====================

    #[test]
    fn test_bytes() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.bin");
        let content = b"Hello, World!";

        std::fs::write(&file_path, content)?;

        let reader = FileReader::new(&file_path);
        let bytes = reader.bytes()?;

        assert_eq!(bytes, content);

        Ok(())
    }

    #[test]
    fn test_bytes_empty_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.bin");
        fs::File::create(&file_path)?;

        let reader = FileReader::new(&file_path);
        let bytes = reader.bytes()?;

        assert_eq!(bytes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_bytes_binary_data() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.bin");
        let content: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];

        std::fs::write(&file_path, &content)?;

        let reader = FileReader::new(&file_path);
        let bytes = reader.bytes()?;

        assert_eq!(bytes, content);

        Ok(())
    }

    // ==================== TOML 解析测试 ====================

    #[test]
    fn test_toml() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("config.toml");
        let toml_content = r#"
name = "test"
version = "1.0.0"
enabled = true
"#;

        std::fs::write(&file_path, toml_content)?;

        let reader = FileReader::new(&file_path);
        let config: TestConfig = reader.toml()?;

        assert_eq!(config.name, "test");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.enabled, true);

        Ok(())
    }

    #[test]
    fn test_toml_invalid_format() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("config.toml");
        let invalid_toml = "name = test\nversion = 1.0.0"; // 缺少引号

        std::fs::write(&file_path, invalid_toml).unwrap();

        let reader = FileReader::new(&file_path);
        let result: Result<TestConfig, _> = reader.toml();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse TOML") || error_msg.contains("TOML"));
    }

    #[test]
    fn test_toml_missing_fields() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("config.toml");
        let incomplete_toml = r#"
name = "test"
# version 和 enabled 缺失
"#;

        std::fs::write(&file_path, incomplete_toml).unwrap();

        let reader = FileReader::new(&file_path);
        let result: Result<TestConfig, _> = reader.toml();

        assert!(result.is_err());
    }

    // ==================== JSON 解析测试 ====================

    #[test]
    fn test_json() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("config.json");
        let json_content = r#"
{
    "name": "test",
    "version": "1.0.0",
    "enabled": true
}
"#;

        std::fs::write(&file_path, json_content)?;

        let reader = FileReader::new(&file_path);
        let config: TestConfig = reader.json()?;

        assert_eq!(config.name, "test");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.enabled, true);

        Ok(())
    }

    #[test]
    fn test_json_invalid_format() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("config.json");
        let invalid_json = r#"{ "name": "test", "version": }"#; // 语法错误

        std::fs::write(&file_path, invalid_json).unwrap();

        let reader = FileReader::new(&file_path);
        let result: Result<TestConfig, _> = reader.json();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse JSON") || error_msg.contains("JSON"));
    }

    #[test]
    fn test_json_missing_fields() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("config.json");
        let incomplete_json = r#"{ "name": "test" }"#; // 缺少 version 和 enabled

        std::fs::write(&file_path, incomplete_json).unwrap();

        let reader = FileReader::new(&file_path);
        let result: Result<TestConfig, _> = reader.json();

        assert!(result.is_err());
    }

    // ==================== 错误处理测试 ====================

    #[test]
    fn test_to_string_nonexistent_file() {
        let reader = FileReader::new("/nonexistent/file.txt");
        let result = reader.to_string();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to read file"));
    }

    #[test]
    fn test_lines_nonexistent_file() {
        let reader = FileReader::new("/nonexistent/file.txt");
        let result = reader.lines();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to open file"));
    }

    #[test]
    fn test_bytes_nonexistent_file() {
        let reader = FileReader::new("/nonexistent/file.txt");
        let result = reader.bytes();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to open file"));
    }

    #[test]
    fn test_toml_nonexistent_file() {
        let reader = FileReader::new("/nonexistent/config.toml");
        let result: Result<TestConfig, _> = reader.toml();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to read config file"));
    }

    #[test]
    fn test_json_nonexistent_file() {
        let reader = FileReader::new("/nonexistent/config.json");
        let result: Result<TestConfig, _> = reader.json();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to read JSON file"));
    }
}
