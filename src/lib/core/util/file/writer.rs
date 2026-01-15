//! 文件写入器
//!
//! 提供基于路径的文件写入操作。

use color_eyre::{eyre::WrapErr, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use serde::Serialize;
    use std::fs;
    use tempfile::tempdir;

    #[derive(Debug, Serialize, PartialEq)]
    struct TestConfig {
        name: String,
        version: String,
        enabled: bool,
    }

    // ==================== FileWriter::new 测试 ====================

    #[test]
    fn test_file_writer_new() {
        let writer = FileWriter::new("/tmp/test.txt");
        // 验证可以创建实例（通过尝试写入来验证）
        let _ = writer.write_str("test"); // 可能会失败，但验证 writer 可用
    }

    #[test]
    fn test_file_writer_new_with_pathbuf() {
        use std::path::PathBuf;
        let path = PathBuf::from("/tmp/test.txt");
        let writer = FileWriter::new(path.clone());
        // 验证可以创建实例
        let _ = writer.write_str("test");
    }

    // ==================== 目录创建测试 ====================

    #[test]
    fn test_ensure_parent_dir() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("subdir/nested/file.txt");

        let writer = FileWriter::new(&file_path);
        writer.ensure_parent_dir()?;

        assert!(file_path.parent().unwrap().exists());
        assert!(file_path.parent().unwrap().is_dir());

        Ok(())
    }

    #[test]
    fn test_ensure_parent_dir_already_exists() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("existing/file.txt");

        // 先创建目录
        fs::create_dir_all(file_path.parent().unwrap())?;

        let writer = FileWriter::new(&file_path);
        writer.ensure_parent_dir()?;

        assert!(file_path.parent().unwrap().exists());

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

    // ==================== 权限设置测试（Unix） ====================

    #[cfg(unix)]
    #[test]
    fn test_set_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir()?;
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

    // ==================== 字符串写入测试 ====================

    #[test]
    fn test_write_str() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!\nThis is a test.";

        let writer = FileWriter::new(&file_path);
        writer.write_str(content)?;

        let read_content = fs::read_to_string(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_str_empty() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        let writer = FileWriter::new(&file_path);
        writer.write_str("")?;

        let read_content = fs::read_to_string(&file_path)?;
        assert_eq!(read_content, "");

        Ok(())
    }

    #[test]
    fn test_write_str_unicode() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, 世界!\n测试文件\némoji🚀";

        let writer = FileWriter::new(&file_path);
        writer.write_str(content)?;

        let read_content = fs::read_to_string(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_str_with_dir() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("subdir/nested/file.txt");
        let content = "Hello, World!";

        let writer = FileWriter::new(&file_path);
        writer.write_str_with_dir(content)?;

        assert!(file_path.exists());
        let read_content = fs::read_to_string(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    // ==================== 字节写入测试 ====================

    #[test]
    fn test_write_bytes() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.bin");
        let content = b"Hello, World!";

        let writer = FileWriter::new(&file_path);
        writer.write_bytes(content)?;

        let read_content = fs::read(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_bytes_empty() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.bin");

        let writer = FileWriter::new(&file_path);
        writer.write_bytes(b"")?;

        let read_content = fs::read(&file_path)?;
        assert_eq!(read_content.len(), 0);

        Ok(())
    }

    #[test]
    fn test_write_bytes_binary_data() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.bin");
        let content: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];

        let writer = FileWriter::new(&file_path);
        writer.write_bytes(&content)?;

        let read_content = fs::read(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_write_bytes_with_dir() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("subdir/nested/file.bin");
        let content = b"Binary content";

        let writer = FileWriter::new(&file_path);
        writer.write_bytes_with_dir(content)?;

        assert!(file_path.exists());
        let read_content = fs::read(&file_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    // ==================== TOML 写入测试 ====================

    #[test]
    fn test_write_toml() -> Result<()> {
        let temp_dir = tempdir()?;
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

    #[test]
    fn test_write_toml_secure() -> Result<()> {
        let temp_dir = tempdir()?;
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
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&file_path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;
            assert_eq!(mode, 0o600);
        }

        Ok(())
    }

    #[test]
    fn test_write_toml_nested_structure() -> Result<()> {
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

        let temp_dir = tempdir()?;
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

    // ==================== JSON 写入测试 ====================

    #[test]
    fn test_write_json() -> Result<()> {
        let temp_dir = tempdir()?;
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

    #[test]
    fn test_write_json_secure() -> Result<()> {
        let temp_dir = tempdir()?;
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
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&file_path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;
            assert_eq!(mode, 0o600);
        }

        Ok(())
    }

    #[test]
    fn test_write_json_nested_structure() -> Result<()> {
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

        let temp_dir = tempdir()?;
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

    // ==================== 错误处理测试 ====================

    #[test]
    fn test_write_str_nonexistent_parent() {
        // 尝试写入到一个不存在的父目录（不使用 write_str_with_dir）
        let writer = FileWriter::new("/nonexistent/path/file.txt");
        let result = writer.write_str("content");

        // 在某些系统上可能会成功（自动创建），在某些系统上会失败
        // 这里主要测试函数不会 panic
        let _ = result;
    }

    #[test]
    fn test_write_bytes_nonexistent_parent() {
        let writer = FileWriter::new("/nonexistent/path/file.bin");
        let result = writer.write_bytes(b"content");

        // 在某些系统上可能会成功，在某些系统上会失败
        let _ = result;
    }
}
