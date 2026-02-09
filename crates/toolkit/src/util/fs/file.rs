//! 文件操作工具
//!
//! 提供文件读取和写入的工具函数。

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::util::fs::FileError;

// ============================================================================
// 文件读取函数
// ============================================================================

/// 打开文件并返回 `BufReader<File>`。
///
/// # 参数
///
/// * `path` - 文件路径
///
/// # 返回
///
/// 返回文件的缓冲读取器。
pub fn open(path: &Path) -> Result<BufReader<File>, FileError> {
    let file = File::open(path).map_err(FileError::Io)?;
    Ok(BufReader::new(file))
}

/// 读取文件内容为字符串。
///
/// # 参数
///
/// * `path` - 文件路径
///
/// # 返回
///
/// 返回文件内容字符串。
pub fn read_string(path: impl AsRef<Path>) -> Result<String, FileError> {
    fs::read_to_string(path).map_err(FileError::Io)
}

/// 读取文件的所有行。
///
/// # 参数
///
/// * `path` - 文件路径
///
/// # 返回
///
/// 返回文件各行的字符串向量。
pub fn read_lines(path: &Path) -> Result<Vec<String>, FileError> {
    let file = File::open(path).map_err(FileError::Io)?;
    let reader = BufReader::new(file);
    reader.lines().collect::<std::io::Result<Vec<String>>>().map_err(FileError::Io)
}

/// 读取文件内容为字节向量。
///
/// # 参数
///
/// * `path` - 文件路径
///
/// # 返回
///
/// 返回文件内容的字节向量。
pub fn read_bytes(path: &Path) -> Result<Vec<u8>, FileError> {
    let mut file = File::open(path).map_err(FileError::Io)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(FileError::Io)?;
    Ok(buffer)
}

/// 读取 TOML 文件并解析为类型 `T`。
///
/// # 参数
///
/// * `path` - TOML 文件路径
///
/// # 返回
///
/// 返回解析后的类型 `T`。
pub fn read_toml<T>(path: &Path) -> Result<T, FileError>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path).map_err(FileError::Io)?;
    toml::from_str(&content).map_err(FileError::Toml)
}

/// 读取 JSON 文件并解析为类型 `T`。
///
/// # 参数
///
/// * `path` - JSON 文件路径
///
/// # 返回
///
/// 返回解析后的类型 `T`。
pub fn read_json<T>(path: &Path) -> Result<T, FileError>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path).map_err(FileError::Io)?;
    serde_json::from_str(&content).map_err(FileError::Json)
}

// ============================================================================
// 文件写入函数
// ============================================================================

/// 确保文件父目录存在。
///
/// 如果文件的父目录不存在，会自动创建所有必要的父目录。
///
/// # 参数
///
/// * `path` - 文件路径
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn ensure_parent_dir(path: impl AsRef<Path>) -> Result<(), FileError> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).map_err(FileError::Io)?;
    }
    Ok(())
}

/// 设置文件权限（仅 Unix 系统）。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `mode` - 文件权限模式（八进制，如 `0o600`）
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
#[cfg(unix)]
pub fn set_permissions(path: impl AsRef<Path>, mode: u32) -> Result<(), FileError> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(FileError::Io)
}

/// 将字符串内容写入文件。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `content` - 要写入的字符串内容
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_string(path: impl AsRef<Path>, content: &str) -> Result<(), FileError> {
    fs::write(path, content).map_err(FileError::Io)
}

/// 将字符串内容写入文件（自动创建父目录）。
///
/// 在写入前会自动创建所有必要的父目录。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `content` - 要写入的字符串内容
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_string_with_dir(path: impl AsRef<Path>, content: &str) -> Result<(), FileError> {
    ensure_parent_dir(path.as_ref())?;
    write_string(path, content)
}

/// 将字节内容写入文件。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `content` - 要写入的字节内容
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_bytes(path: impl AsRef<Path>, content: &[u8]) -> Result<(), FileError> {
    fs::write(path, content).map_err(FileError::Io)
}

/// 将字节内容写入文件（自动创建父目录）。
///
/// 在写入前会自动创建所有必要的父目录。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `content` - 要写入的字节内容
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_bytes_with_dir(path: impl AsRef<Path>, content: &[u8]) -> Result<(), FileError> {
    ensure_parent_dir(path.as_ref())?;
    write_bytes(path, content)
}

/// 将类型 `T` 序列化为 TOML 并写入文件。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `data` - 要序列化的数据
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_toml<T>(path: impl AsRef<Path>, data: &T) -> Result<(), FileError>
where
    T: Serialize,
{
    let toml_content = toml::to_string_pretty(data)
        .map_err(|e| FileError::Other(format!("Failed to serialize to TOML: {}", e)))?;
    write_string(path, &toml_content)
}

/// 将类型 `T` 序列化为 TOML 并写入文件（自动创建目录和设置权限）。
///
/// 在写入前会自动创建所有必要的父目录，并在 Unix 系统上设置文件权限为 `0o600`。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `data` - 要序列化和写入的数据
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_toml_secure<T>(path: impl AsRef<Path>, data: &T) -> Result<(), FileError>
where
    T: Serialize,
{
    let path = path.as_ref();
    ensure_parent_dir(path)?;
    write_toml(path, data)?;
    #[cfg(unix)]
    set_permissions(path, 0o600)?;
    Ok(())
}

/// 将类型 `T` 序列化为 JSON 并写入文件。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `data` - 要序列化的数据
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_json<T>(path: impl AsRef<Path>, data: &T) -> Result<(), FileError>
where
    T: Serialize,
{
    let json_content = serde_json::to_string_pretty(data).map_err(FileError::Json)?;
    write_string(path, &json_content)
}

/// 将类型 `T` 序列化为 JSON 并写入文件（自动创建目录和设置权限）。
///
/// 在写入前会自动创建所有必要的父目录，并在 Unix 系统上设置文件权限为 `0o600`。
///
/// # 参数
///
/// * `path` - 文件路径
/// * `data` - 要序列化和写入的数据
///
/// # 返回
///
/// 成功返回 `Ok(())`，失败返回错误。
pub fn write_json_secure<T>(path: impl AsRef<Path>, data: &T) -> Result<(), FileError>
where
    T: Serialize,
{
    let path = path.as_ref();
    ensure_parent_dir(path)?;
    write_json(path, data)?;
    #[cfg(unix)]
    set_permissions(path, 0o600)?;
    Ok(())
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use rstest::{fixture, rstest};
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    use super::*;

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

    // ==================== 读取函数测试 ====================

    mod reader_tests {
        use super::*;

        #[test]
        fn test_open() -> Result<(), FileError> {
            let temp_dir = tempfile::tempdir()?;
            let file_path = temp_dir.path().join("test.txt");
            fs::File::create(&file_path)?;

            let mut buf_reader = open(&file_path)?;

            // 验证可以读取
            let mut content = String::new();
            buf_reader.read_to_string(&mut content)?;
            assert_eq!(content, "");

            Ok(())
        }

        #[test]
        fn test_open_nonexistent_file() -> Result<(), FileError> {
            let result = open(std::path::Path::new("/nonexistent/file.txt"));

            assert!(matches!(result, Err(FileError::Io(_))));

            Ok(())
        }

        #[rstest]
        #[case("Hello, World!\nThis is a test file.", false)]
        #[case("", true)] // 空文件需要先创建
        #[case("Hello, 世界!\n测试文件\némoji🚀", false)]
        fn test_read_string(
            temp_dir: TempDir,
            #[case] content: &str,
            #[case] is_empty: bool,
        ) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("test.txt");

            if is_empty {
                fs::File::create(&file_path)?;
            } else {
                std::fs::write(&file_path, content)?;
            }

            let read_content = read_string(file_path)?;

            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        #[case("line1\nline2\nline3", 3, vec!["line1", "line2", "line3"])]
        #[case("", 0, vec![])] // 空文件
        #[case("line1\n\nline2\n\n\nline3", 6, vec!["line1", "", "line2", "", "", "line3"])] // 包含空行
        #[case("line1\nline2", 2, vec!["line1", "line2"])] // 无尾随换行符
        fn test_read_lines(
            temp_dir: TempDir,
            #[case] content: &str,
            #[case] expected_len: usize,
            #[case] expected_lines: Vec<&str>,
        ) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("test.txt");
            if content.is_empty() {
                fs::File::create(&file_path)?;
            } else {
                std::fs::write(&file_path, content)?;
            }

            let lines = read_lines(&file_path)?;

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
        fn test_read_bytes(temp_dir: TempDir, #[case] content: &[u8]) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("test.bin");
            if content.is_empty() {
                fs::File::create(&file_path)?;
            } else {
                std::fs::write(&file_path, content)?;
            }

            let bytes = read_bytes(&file_path)?;

            assert_eq!(bytes, content);

            Ok(())
        }

        #[rstest]
        fn test_read_toml(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.toml");
            let toml_content = r#"
name = "test"
version = "1.0.0"
enabled = true
"#;

            std::fs::write(&file_path, toml_content)?;

            let config: TestConfig = read_toml(&file_path)?;

            assert_eq!(config.name, "test");
            assert_eq!(config.version, "1.0.0");
            assert!(config.enabled);

            Ok(())
        }

        #[rstest]
        fn test_read_toml_invalid_format(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.toml");
            let invalid_toml = "name = test\nversion = 1.0.0"; // 缺少引号

            std::fs::write(&file_path, invalid_toml)?;

            let result: Result<TestConfig, _> = read_toml(&file_path);

            assert!(matches!(result, Err(FileError::Toml(_))));

            Ok(())
        }

        #[rstest]
        fn test_read_toml_missing_fields(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.toml");
            let incomplete_toml = r#"
name = "test"
# version 和 enabled 缺失
"#;

            std::fs::write(&file_path, incomplete_toml)?;

            let result: Result<TestConfig, _> = read_toml(&file_path);

            assert!(result.is_err());

            Ok(())
        }

        #[rstest]
        fn test_read_json(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.json");
            let json_content = r#"
{
    "name": "test",
    "version": "1.0.0",
    "enabled": true
}
"#;

            std::fs::write(&file_path, json_content)?;

            let config: TestConfig = read_json(&file_path)?;

            assert_eq!(config.name, "test");
            assert_eq!(config.version, "1.0.0");
            assert!(config.enabled);

            Ok(())
        }

        #[rstest]
        fn test_read_json_invalid_format(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.json");
            let invalid_json = r#"{ "name": "test", "version": }"#; // 语法错误

            std::fs::write(&file_path, invalid_json)?;

            let result: Result<TestConfig, _> = read_json(&file_path);

            assert!(matches!(result, Err(FileError::Json(_))));

            Ok(())
        }

        #[rstest]
        fn test_read_json_missing_fields(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.json");
            let incomplete_json = r#"{ "name": "test" }"#; // 缺少 version 和 enabled

            std::fs::write(&file_path, incomplete_json)?;

            let result: Result<TestConfig, _> = read_json(&file_path);

            assert!(result.is_err());

            Ok(())
        }
    }

    // ==================== 写入函数测试 ====================

    mod writer_tests {
        use super::*;

        // 辅助函数：检查文件权限（仅 Unix）
        #[cfg(unix)]
        fn assert_file_permissions(
            file_path: &std::path::Path,
            expected_mode: u32,
        ) -> Result<(), FileError> {
            let metadata = fs::metadata(file_path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;
            assert_eq!(mode, expected_mode);
            Ok(())
        }

        #[rstest]
        #[case(false)] // 目录不存在，需要创建
        #[case(true)] // 目录已存在
        fn test_ensure_parent_dir(
            temp_dir: TempDir,
            #[case] dir_exists: bool,
        ) -> Result<(), FileError> {
            let file_path = if dir_exists {
                temp_dir.path().join("existing/file.txt")
            } else {
                temp_dir.path().join("subdir/nested/file.txt")
            };

            if dir_exists {
                // 先创建目录
                fs::create_dir_all(file_path.parent().unwrap())?;
            }

            ensure_parent_dir(&file_path)?;

            assert!(file_path.parent().unwrap().exists());
            assert!(file_path.parent().unwrap().is_dir());

            Ok(())
        }

        #[test]
        fn test_ensure_parent_dir_for_root_path() -> Result<(), FileError> {
            // 根路径没有父目录，应该成功（不执行任何操作）
            let result = ensure_parent_dir(std::path::Path::new("/file.txt"));
            assert!(result.is_ok());

            Ok(())
        }

        #[cfg(unix)]
        #[test]
        fn test_set_permissions() -> Result<(), FileError> {
            let temp_dir = tempfile::tempdir()?;
            let file_path = temp_dir.path().join("test.txt");
            fs::File::create(&file_path)?;

            set_permissions(&file_path, 0o600)?;

            let metadata = fs::metadata(&file_path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            assert_eq!(mode, 0o600);

            Ok(())
        }

        #[cfg(unix)]
        #[test]
        fn test_set_permissions_nonexistent_file() -> Result<(), FileError> {
            let result = set_permissions(std::path::Path::new("/nonexistent/file.txt"), 0o600);

            assert!(matches!(result, Err(FileError::Io(_))));

            Ok(())
        }

        #[rstest]
        #[case("Hello, World!\nThis is a test.")]
        #[case("")]
        #[case("Hello, 世界!\n测试文件\némoji🚀")]
        fn test_write_string(temp_dir: TempDir, #[case] content: &str) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("test.txt");

            write_string(&file_path, content)?;

            let read_content = fs::read_to_string(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        fn test_write_string_with_dir(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("subdir/nested/file.txt");
            let content = "Hello, World!";

            write_string_with_dir(&file_path, content)?;

            assert!(file_path.exists());
            let read_content = fs::read_to_string(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        #[case(b"Hello, World!".as_slice())]
        #[case(b"".as_slice())]
        #[case(&[0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD])]
        fn test_write_bytes(temp_dir: TempDir, #[case] content: &[u8]) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("test.bin");

            write_bytes(&file_path, content)?;

            let read_content = fs::read(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        fn test_write_bytes_with_dir(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("subdir/nested/file.bin");
            let content = b"Binary content";

            write_bytes_with_dir(&file_path, content)?;

            assert!(file_path.exists());
            let read_content = fs::read(&file_path)?;
            assert_eq!(read_content, content);

            Ok(())
        }

        #[rstest]
        fn test_write_toml(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.toml");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            write_toml(&file_path, &config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("name = \"test\""));
            assert!(content.contains("version = \"1.0.0\""));
            assert!(content.contains("enabled = true"));

            Ok(())
        }

        #[rstest]
        fn test_write_toml_secure(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("subdir/config.toml");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            write_toml_secure(&file_path, &config)?;

            assert!(file_path.exists());
            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("name = \"test\""));

            #[cfg(unix)]
            assert_file_permissions(&file_path, 0o600)?;

            Ok(())
        }

        #[rstest]
        fn test_write_toml_nested_structure(temp_dir: TempDir) -> Result<(), FileError> {
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

            write_toml(&file_path, &config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("[app]"));
            assert!(content.contains("[database]"));

            Ok(())
        }

        #[rstest]
        fn test_write_json(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("config.json");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            write_json(&file_path, &config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("\"name\": \"test\""));
            assert!(content.contains("\"version\": \"1.0.0\""));
            assert!(content.contains("\"enabled\": true"));

            Ok(())
        }

        #[rstest]
        fn test_write_json_secure(temp_dir: TempDir) -> Result<(), FileError> {
            let file_path = temp_dir.path().join("subdir/config.json");
            let config = TestConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
            };

            write_json_secure(&file_path, &config)?;

            assert!(file_path.exists());
            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("\"name\": \"test\""));

            #[cfg(unix)]
            assert_file_permissions(&file_path, 0o600)?;

            Ok(())
        }

        #[rstest]
        fn test_write_json_nested_structure(temp_dir: TempDir) -> Result<(), FileError> {
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

            write_json(&file_path, &config)?;

            let content = fs::read_to_string(&file_path)?;
            assert!(content.contains("\"app\""));
            assert!(content.contains("\"database\""));

            Ok(())
        }
    }
}
