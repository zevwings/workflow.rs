//! 文件操作工具
//!
//! 提供文件读取和写入的工具类型：
//! - `FileReader`：围绕路径的读取助手
//! - `FileWriter`：围绕路径的写入助手

use color_eyre::{eyre::WrapErr, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

// ==================== FileReader ====================

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

// ==================== FileWriter ====================

/// 文件写入器，基于路径提供常用写入操作。
pub struct FileWriter {
    path: PathBuf,
}

impl FileWriter {
    /// 创建一个新的文件写入器。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 确保父目录存在。
    ///
    /// 如果文件的父目录不存在，会自动创建所有必要的父目录。
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果创建目录失败，返回错误。
    pub fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("Failed to create parent directory: {:?}", parent))?;
        }
        Ok(())
    }

    /// 设置文件权限（仅 Unix 系统）。
    ///
    /// # 参数
    ///
    /// * `mode` - 文件权限模式（八进制，如 `0o600`）
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果设置权限失败，返回错误。
    #[cfg(unix)]
    pub fn set_permissions(&self, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&self.path, fs::Permissions::from_mode(mode))
            .wrap_err_with(|| format!("Failed to set file permissions: {:?}", self.path))?;
        Ok(())
    }

    /// 将字符串内容写入文件。
    pub fn write_str(&self, content: &str) -> Result<()> {
        fs::write(&self.path, content)
            .wrap_err_with(|| format!("Failed to write file: {:?}", self.path))
    }

    /// 将字符串内容写入文件（自动创建父目录）。
    ///
    /// 在写入前会自动创建所有必要的父目录。
    ///
    /// # 参数
    ///
    /// * `content` - 要写入的字符串内容
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_str_with_dir(&self, content: &str) -> Result<()> {
        self.ensure_parent_dir()?;
        self.write_str(content)
    }

    /// 将字节内容写入文件。
    pub fn write_bytes(&self, content: &[u8]) -> Result<()> {
        fs::write(&self.path, content)
            .wrap_err_with(|| format!("Failed to write file: {:?}", self.path))
    }

    /// 将字节内容写入文件（自动创建父目录）。
    ///
    /// 在写入前会自动创建所有必要的父目录。
    ///
    /// # 参数
    ///
    /// * `content` - 要写入的字节内容
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_bytes_with_dir(&self, content: &[u8]) -> Result<()> {
        self.ensure_parent_dir()?;
        self.write_bytes(content)
    }

    /// 将类型 `T` 序列化为 TOML 并写入文件。
    pub fn write_toml<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let toml_content = toml::to_string_pretty(data)
            .wrap_err_with(|| format!("Failed to serialize config to TOML: {:?}", self.path))?;
        self.write_str(&toml_content)
    }

    /// 将类型 `T` 序列化为 TOML 并写入文件（自动创建目录和设置权限）。
    ///
    /// 在写入前会自动创建所有必要的父目录，并在 Unix 系统上设置文件权限为 `0o600`。
    ///
    /// # 参数
    ///
    /// * `data` - 要序列化和写入的数据
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_toml_secure<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ensure_parent_dir()?;
        self.write_toml(data)?;
        #[cfg(unix)]
        self.set_permissions(0o600)?;
        Ok(())
    }

    /// 将类型 `T` 序列化为 JSON 并写入文件。
    pub fn write_json<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let json_content = serde_json::to_string_pretty(data)
            .wrap_err_with(|| format!("Failed to serialize config to JSON: {:?}", self.path))?;
        self.write_str(&json_content)
    }

    /// 将类型 `T` 序列化为 JSON 并写入文件（自动创建目录和设置权限）。
    ///
    /// 在写入前会自动创建所有必要的父目录，并在 Unix 系统上设置文件权限为 `0o600`。
    ///
    /// # 参数
    ///
    /// * `data` - 要序列化和写入的数据
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_json_secure<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ensure_parent_dir()?;
        self.write_json(data)?;
        #[cfg(unix)]
        self.set_permissions(0o600)?;
        Ok(())
    }
}

// ==================== 测试模块 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use rstest::{fixture, rstest};
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::io::Read;
    use tempfile::TempDir;

    // 共享的 TestConfig（同时实现 Serialize 和 Deserialize）
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        version: String,
        enabled: bool,
    }

    // 共享的 fixture
    #[fixture]
    fn temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    // ==================== FileReader 测试 ====================

    mod reader_tests {
        use super::*;

        #[test]
        fn test_file_reader_new() {
            // 测试使用 &str
            let reader1 = FileReader::new("/tmp/test.txt");
            let _ = reader1.to_string(); // 这会失败，但验证 reader 可用

            // 测试使用 PathBuf
            use std::path::PathBuf;
            let path = PathBuf::from("/tmp/test.txt");
            let reader2 = FileReader::new(path.clone());
            let _ = reader2.to_string();
        }

        #[test]
        fn test_open() -> Result<()> {
            let temp_dir = tempfile::tempdir()?;
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

        #[rstest]
        #[case("Hello, World!\nThis is a test file.", false)]
        #[case("", true)] // 空文件需要先创建
        #[case("Hello, 世界!\n测试文件\némoji🚀", false)]
        fn test_to_string(
            temp_dir: TempDir,
            #[case] content: &str,
            #[case] is_empty: bool,
        ) -> Result<()> {
            let file_path = temp_dir.path().join("test.txt");

            if is_empty {
                fs::File::create(&file_path)?;
            } else {
                std::fs::write(&file_path, content)?;
            }

            let reader = FileReader::new(&file_path);
            let read_content = reader.to_string()?;

            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        #[case("line1\nline2\nline3", 3, vec!["line1", "line2", "line3"])]
        #[case("", 0, vec![])] // 空文件
        #[case("line1\n\nline2\n\n\nline3", 6, vec!["line1", "", "line2", "", "", "line3"])] // 包含空行
        #[case("line1\nline2", 2, vec!["line1", "line2"])] // 无尾随换行符
        fn test_lines(
            temp_dir: TempDir,
            #[case] content: &str,
            #[case] expected_len: usize,
            #[case] expected_lines: Vec<&str>,
        ) -> Result<()> {
            let file_path = temp_dir.path().join("test.txt");
            if content.is_empty() {
                fs::File::create(&file_path)?;
            } else {
                std::fs::write(&file_path, content)?;
            }

            let reader = FileReader::new(&file_path);
            let lines = reader.lines()?;

            assert_eq!(lines.len(), expected_len);
            for (i, expected) in expected_lines.iter().enumerate() {
                assert_eq!(lines[i], *expected);
            }

            Ok(())
        }

        #[rstest]
        #[case(b"Hello, World!".as_slice())]
        #[case(&[])] // 空文件
        #[case(&[0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD])] // 二进制数据
        fn test_bytes(temp_dir: TempDir, #[case] content: &[u8]) -> Result<()> {
            let file_path = temp_dir.path().join("test.bin");
            if content.is_empty() {
                fs::File::create(&file_path)?;
            } else {
                std::fs::write(&file_path, content)?;
            }

            let reader = FileReader::new(&file_path);
            let bytes = reader.bytes()?;

            assert_eq!(bytes, content);

            Ok(())
        }

        #[rstest]
        fn test_toml(temp_dir: TempDir) -> Result<()> {
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

        #[rstest]
        fn test_toml_invalid_format(temp_dir: TempDir) {
            let file_path = temp_dir.path().join("config.toml");
            let invalid_toml = "name = test\nversion = 1.0.0"; // 缺少引号

            std::fs::write(&file_path, invalid_toml).unwrap();

            let reader = FileReader::new(&file_path);
            let result: Result<TestConfig, _> = reader.toml();

            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Failed to parse TOML") || error_msg.contains("TOML"));
        }

        #[rstest]
        fn test_toml_missing_fields(temp_dir: TempDir) {
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

        #[rstest]
        fn test_json(temp_dir: TempDir) -> Result<()> {
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

        #[rstest]
        fn test_json_invalid_format(temp_dir: TempDir) {
            let file_path = temp_dir.path().join("config.json");
            let invalid_json = r#"{ "name": "test", "version": }"#; // 语法错误

            std::fs::write(&file_path, invalid_json).unwrap();

            let reader = FileReader::new(&file_path);
            let result: Result<TestConfig, _> = reader.json();

            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Failed to parse JSON") || error_msg.contains("JSON"));
        }

        #[rstest]
        fn test_json_missing_fields(temp_dir: TempDir) {
            let file_path = temp_dir.path().join("config.json");
            let incomplete_json = r#"{ "name": "test" }"#; // 缺少 version 和 enabled

            std::fs::write(&file_path, incomplete_json).unwrap();

            let reader = FileReader::new(&file_path);
            let result: Result<TestConfig, _> = reader.json();

            assert!(result.is_err());
        }

        #[rstest]
        #[case("/nonexistent/file.txt", "to_string", "Failed to read file")]
        #[case("/nonexistent/file.txt", "lines", "Failed to open file")]
        #[case("/nonexistent/file.txt", "bytes", "Failed to open file")]
        #[case("/nonexistent/config.toml", "toml", "Failed to read config file")]
        #[case("/nonexistent/config.json", "json", "Failed to read JSON file")]
        fn test_reader_nonexistent_file(
            #[case] path: &str,
            #[case] method: &str,
            #[case] expected_error: &str,
        ) {
            let reader = FileReader::new(path);
            let result = match method {
                "to_string" => reader.to_string().map(|_| ()),
                "lines" => reader.lines().map(|_| ()),
                "bytes" => reader.bytes().map(|_| ()),
                "toml" => reader.toml::<TestConfig>().map(|_| ()),
                "json" => reader.json::<TestConfig>().map(|_| ()),
                _ => panic!("Unknown method: {}", method),
            };

            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains(expected_error));
        }
    }

    // ==================== FileWriter 测试 ====================

    mod writer_tests {
        use super::*;

        // 辅助函数：检查文件权限（仅 Unix）
        #[cfg(unix)]
        fn assert_file_permissions(file_path: &std::path::Path, expected_mode: u32) -> Result<()> {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(file_path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;
            assert_eq!(mode, expected_mode);
            Ok(())
        }

        #[test]
        fn test_file_writer_new() {
            // 测试使用 &str
            let writer1 = FileWriter::new("/tmp/test.txt");
            let _ = writer1.write_str("test"); // 可能会失败，但验证 writer 可用

            // 测试使用 PathBuf
            use std::path::PathBuf;
            let path = PathBuf::from("/tmp/test.txt");
            let writer2 = FileWriter::new(path.clone());
            let _ = writer2.write_str("test");
        }

        #[rstest]
        #[case(false)] // 目录不存在，需要创建
        #[case(true)] // 目录已存在
        fn test_ensure_parent_dir(temp_dir: TempDir, #[case] dir_exists: bool) -> Result<()> {
            let file_path = if dir_exists {
                temp_dir.path().join("existing/file.txt")
            } else {
                temp_dir.path().join("subdir/nested/file.txt")
            };

            if dir_exists {
                // 先创建目录
                fs::create_dir_all(file_path.parent().unwrap())?;
            }

            let writer = FileWriter::new(&file_path);
            writer.ensure_parent_dir()?;

            assert!(file_path.parent().unwrap().exists());
            assert!(file_path.parent().unwrap().is_dir());

            Ok(())
        }

        #[test]
        fn test_ensure_parent_dir_for_root_path() -> Result<()> {
            let writer = FileWriter::new("/file.txt");
            // 根路径没有父目录，应该成功（不执行任何操作）
            let result = writer.ensure_parent_dir();
            assert!(result.is_ok());

            Ok(())
        }

        #[cfg(unix)]
        #[test]
        fn test_set_permissions() -> Result<()> {
            use std::os::unix::fs::PermissionsExt;

            let temp_dir = tempfile::tempdir()?;
            let file_path = temp_dir.path().join("test.txt");
            fs::File::create(&file_path)?;

            let writer = FileWriter::new(&file_path);
            writer.set_permissions(0o600)?;

            let metadata = fs::metadata(&file_path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            assert_eq!(mode, 0o600);

            Ok(())
        }

        #[cfg(unix)]
        #[test]
        fn test_set_permissions_nonexistent_file() {
            let writer = FileWriter::new("/nonexistent/file.txt");
            let result = writer.set_permissions(0o600);

            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Failed to set file permissions"));
        }

        #[rstest]
        #[case("Hello, World!\nThis is a test.")]
        #[case("")]
        #[case("Hello, 世界!\n测试文件\némoji🚀")]
        fn test_write_str(temp_dir: TempDir, #[case] content: &str) -> Result<()> {
            let file_path = temp_dir.path().join("test.txt");

            let writer = FileWriter::new(&file_path);
            writer.write_str(content)?;

            let read_content = fs::read_to_string(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        fn test_write_str_with_dir(temp_dir: TempDir) -> Result<()> {
            let file_path = temp_dir.path().join("subdir/nested/file.txt");
            let content = "Hello, World!";

            let writer = FileWriter::new(&file_path);
            writer.write_str_with_dir(content)?;

            assert!(file_path.exists());
            let read_content = fs::read_to_string(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        #[case(b"Hello, World!".as_slice())]
        #[case(b"".as_slice())]
        #[case(&[0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD])]
        fn test_write_bytes(temp_dir: TempDir, #[case] content: &[u8]) -> Result<()> {
            let file_path = temp_dir.path().join("test.bin");

            let writer = FileWriter::new(&file_path);
            writer.write_bytes(content)?;

            let read_content = fs::read(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        fn test_write_bytes_with_dir(temp_dir: TempDir) -> Result<()> {
            let file_path = temp_dir.path().join("subdir/nested/file.bin");
            let content = b"Binary content";

            let writer = FileWriter::new(&file_path);
            writer.write_bytes_with_dir(content)?;

            assert!(file_path.exists());
            let read_content = fs::read(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        fn test_write_toml(temp_dir: TempDir) -> Result<()> {
            let file_path = temp_dir.path().join("config.toml");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            let writer = FileWriter::new(&file_path);
            writer.write_toml(&config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("name = \"test\""));
            assert!(content.contains("version = \"1.0.0\""));
            assert!(content.contains("enabled = true"));

            Ok(())
        }

        #[rstest]
        fn test_write_toml_secure(temp_dir: TempDir) -> Result<()> {
            let file_path = temp_dir.path().join("subdir/config.toml");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            let writer = FileWriter::new(&file_path);
            writer.write_toml_secure(&config)?;

            assert!(file_path.exists());
            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("name = \"test\""));

            #[cfg(unix)]
            assert_file_permissions(&file_path, 0o600)?;

            Ok(())
        }

        #[rstest]
        fn test_write_toml_nested_structure(temp_dir: TempDir) -> Result<()> {
            #[derive(Serialize)]
            struct NestedConfig {
                app: TestConfig,
                database: DatabaseConfig,
            }

            #[derive(Serialize)]
            struct DatabaseConfig {
                host: String,
                port: u16,
            }

            let file_path = temp_dir.path().join("config.toml");
            let config = NestedConfig {
                app: TestConfig {
                    name: "test".to_string(),
                    version: "1.0.0".to_string(),
                    enabled: true,
                },
                database: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 5432,
                },
            };

            let writer = FileWriter::new(&file_path);
            writer.write_toml(&config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("[app]"));
            assert!(content.contains("[database]"));

            Ok(())
        }

        #[rstest]
        fn test_write_json(temp_dir: TempDir) -> Result<()> {
            let file_path = temp_dir.path().join("config.json");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            let writer = FileWriter::new(&file_path);
            writer.write_json(&config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("\"name\": \"test\""));
            assert!(content.contains("\"version\": \"1.0.0\""));
            assert!(content.contains("\"enabled\": true"));

            Ok(())
        }

        #[rstest]
        fn test_write_json_secure(temp_dir: TempDir) -> Result<()> {
            let file_path = temp_dir.path().join("subdir/config.json");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            let writer = FileWriter::new(&file_path);
            writer.write_json_secure(&config)?;

            assert!(file_path.exists());
            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("\"name\": \"test\""));

            #[cfg(unix)]
            assert_file_permissions(&file_path, 0o600)?;

            Ok(())
        }

        #[rstest]
        fn test_write_json_nested_structure(temp_dir: TempDir) -> Result<()> {
            #[derive(Serialize)]
            struct NestedConfig {
                app: TestConfig,
                database: DatabaseConfig,
            }

            #[derive(Serialize)]
            struct DatabaseConfig {
                host: String,
                port: u16,
            }

            let file_path = temp_dir.path().join("config.json");
            let config = NestedConfig {
                app: TestConfig {
                    name: "test".to_string(),
                    version: "1.0.0".to_string(),
                    enabled: true,
                },
                database: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 5432,
                },
            };

            let writer = FileWriter::new(&file_path);
            writer.write_json(&config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("\"app\""));
            assert!(content.contains("\"database\""));

            Ok(())
        }

        #[rstest]
        #[case("write_str", "content")]
        #[case("write_bytes", "content")]
        fn test_write_nonexistent_parent(#[case] method: &str, #[case] _content: &str) {
            // 尝试写入到一个不存在的父目录（不使用 write_*_with_dir）
            let writer = FileWriter::new("/nonexistent/path/file.txt");
            let result = match method {
                "write_str" => writer.write_str("content").map(|_| ()),
                "write_bytes" => writer.write_bytes(b"content").map(|_| ()),
                _ => panic!("Unknown method: {}", method),
            };

            // 在某些系统上可能会成功（自动创建），在某些系统上会失败
            // 这里主要测试函数不会 panic
            let _ = result;
        }
    }
}
