//! 文件操作工具
//!
//! 提供文件读取相关的工具函数，包括：
//! - 文件读取器（`FileReader`）
//! - TOML 文件读写工具函数
//! - JSON 文件读写工具函数

use color_eyre::{eyre::WrapErr, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// 文件读取器
///
/// 提供文件打开和读取功能。
pub struct FileReader;

impl FileReader {
    /// 打开文件并返回 BufReader
    ///
    /// 打开指定路径的文件，并返回一个缓冲读取器。
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回 `BufReader<File>`，用于读取文件内容。
    ///
    /// # 错误
    ///
    /// 如果文件无法打开，返回相应的错误信息。
    pub fn open(file_path: &Path) -> Result<BufReader<File>> {
        let file = File::open(file_path)
            .wrap_err_with(|| format!("Failed to open file: {:?}", file_path))?;
        Ok(BufReader::new(file))
    }

    /// 读取文件内容为字符串
    ///
    /// 从指定路径读取文件内容并返回字符串。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回文件内容的字符串。
    ///
    /// # 错误
    ///
    /// 如果文件读取失败，返回相应的错误信息。
    pub fn read_to_string(path: &Path) -> Result<String> {
        fs::read_to_string(path).wrap_err_with(|| format!("Failed to read file: {:?}", path))
    }

    /// 读取文件的所有行
    ///
    /// 从指定路径读取文件内容，按行分割并返回字符串向量。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回文件所有行的向量。
    ///
    /// # 错误
    ///
    /// 如果文件读取失败，返回相应的错误信息。
    pub fn read_lines(path: &Path) -> Result<Vec<String>> {
        let file = File::open(path).wrap_err_with(|| format!("Failed to open file: {:?}", path))?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line =
                line.wrap_err_with(|| format!("Failed to read line from file: {:?}", path))?;
            lines.push(line);
        }
        Ok(lines)
    }

    /// 读取文件内容为字节向量
    ///
    /// 从指定路径读取文件内容并返回字节向量。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回文件内容的字节向量。
    ///
    /// # 错误
    ///
    /// 如果文件读取失败，返回相应的错误信息。
    pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
        let mut file =
            File::open(path).wrap_err_with(|| format!("Failed to open file: {:?}", path))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .wrap_err_with(|| format!("Failed to read file: {:?}", path))?;
        Ok(buffer)
    }
}

/// 读取 TOML 文件并解析为指定类型
///
/// 从指定路径读取 TOML 文件内容，并解析为类型 `T`。
///
/// # 类型参数
///
/// * `T` - 目标类型，必须实现 `DeserializeOwned` trait
///
/// # 参数
///
/// * `path` - TOML 文件路径
///
/// # 返回
///
/// 返回解析后的类型 `T` 实例。
///
/// # 错误
///
/// 如果文件读取失败或 TOML 解析失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::read_toml_file;
/// use serde::Deserialize;
/// use std::path::PathBuf;
///
/// #[derive(Debug, Deserialize)]
/// struct Config {
///     name: String,
/// }
///
/// # fn main() -> color_eyre::Result<()> {
/// let config: Config = read_toml_file(PathBuf::from("config.toml").as_path())?;
/// # Ok(())
/// # }
/// ```
pub fn read_toml_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read config file: {:?}", path))?;
    toml::from_str(&content).wrap_err_with(|| format!("Failed to parse TOML config: {:?}", path))
}

/// 读取 TOML 文件并解析为 `toml::Value`
///
/// 从指定路径读取 TOML 文件内容，并解析为 `toml::Value`。
/// 这对于需要部分读取或动态操作 TOML 内容的场景很有用。
///
/// # 参数
///
/// * `path` - TOML 文件路径
///
/// # 返回
///
/// 返回解析后的 `toml::Value` 实例。
///
/// # 错误
///
/// 如果文件读取失败或 TOML 解析失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::read_toml_value;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let value = read_toml_value(PathBuf::from("config.toml").as_path())?;
/// if let Some(section) = value.get("template") {
///     // 处理 template 部分
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_toml_value(path: &Path) -> Result<toml::Value> {
    read_toml_file(path)
}

/// 将数据序列化为 TOML 格式并写入文件
///
/// 将类型 `T` 的实例序列化为 TOML 格式（使用 `toml::to_string_pretty`），
/// 然后写入指定路径的文件。
///
/// # 类型参数
///
/// * `T` - 源类型，必须实现 `Serialize` trait
///
/// # 参数
///
/// * `path` - 目标文件路径
/// * `data` - 要序列化和写入的数据
///
/// # 错误
///
/// 如果序列化失败或文件写入失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::write_toml_file;
/// use serde::Serialize;
/// use std::path::PathBuf;
///
/// #[derive(Debug, Serialize)]
/// struct Config {
///     name: String,
/// }
///
/// # fn main() -> color_eyre::Result<()> {
/// let config = Config {
///     name: "example".to_string(),
/// };
/// write_toml_file(PathBuf::from("config.toml").as_path(), &config)?;
/// # Ok(())
/// # }
/// ```
pub fn write_toml_file<T>(path: &Path, data: &T) -> Result<()>
where
    T: Serialize,
{
    let toml_content = toml::to_string_pretty(data)
        .wrap_err_with(|| format!("Failed to serialize config to TOML: {:?}", path))?;
    fs::write(path, toml_content)
        .wrap_err_with(|| format!("Failed to write config file: {:?}", path))?;
    Ok(())
}

/// 将 `toml::Value` 写入文件
///
/// 将 `toml::Value` 序列化为 TOML 格式并写入指定路径的文件。
///
/// # 参数
///
/// * `path` - 目标文件路径
/// * `value` - 要写入的 `toml::Value` 实例
///
/// # 错误
///
/// 如果序列化失败或文件写入失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::write_toml_value;
/// use toml::{map::Map, Value};
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let mut table = Map::new();
/// table.insert("key".to_string(), Value::String("value".to_string()));
/// let value = Value::Table(table);
/// write_toml_value(PathBuf::from("config.toml").as_path(), &value)?;
/// # Ok(())
/// # }
/// ```
pub fn write_toml_value(path: &Path, value: &toml::Value) -> Result<()> {
    write_toml_file(path, value)
}

/// 读取 JSON 文件并解析为指定类型
///
/// 从指定路径读取 JSON 文件内容，并解析为类型 `T`。
///
/// # 类型参数
///
/// * `T` - 目标类型，必须实现 `DeserializeOwned` trait
///
/// # 参数
///
/// * `path` - JSON 文件路径
///
/// # 返回
///
/// 返回解析后的类型 `T` 实例。
///
/// # 错误
///
/// 如果文件读取失败或 JSON 解析失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::read_json_file;
/// use serde::Deserialize;
/// use std::path::PathBuf;
///
/// #[derive(Debug, Deserialize)]
/// struct Config {
///     name: String,
/// }
///
/// # fn main() -> color_eyre::Result<()> {
/// let config: Config = read_json_file(PathBuf::from("config.json").as_path())?;
/// # Ok(())
/// # }
/// ```
pub fn read_json_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read JSON file: {:?}", path))?;
    serde_json::from_str(&content)
        .wrap_err_with(|| format!("Failed to parse JSON file: {:?}", path))
}

/// 读取 JSON 文件并解析为 `serde_json::Value`
///
/// 从指定路径读取 JSON 文件内容，并解析为 `serde_json::Value`。
/// 这对于需要部分读取或动态操作 JSON 内容的场景很有用。
///
/// # 参数
///
/// * `path` - JSON 文件路径
///
/// # 返回
///
/// 返回解析后的 `serde_json::Value` 实例。
///
/// # 错误
///
/// 如果文件读取失败或 JSON 解析失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::read_json_value;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let value = read_json_value(PathBuf::from("config.json").as_path())?;
/// if let Some(field) = value.get("name") {
///     // 处理 name 字段
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_json_value(path: &Path) -> Result<serde_json::Value> {
    read_json_file(path)
}

/// 将数据序列化为 JSON 格式并写入文件
///
/// 将类型 `T` 的实例序列化为 JSON 格式（使用 `serde_json::to_string_pretty`），
/// 然后写入指定路径的文件。
///
/// # 类型参数
///
/// * `T` - 源类型，必须实现 `Serialize` trait
///
/// # 参数
///
/// * `path` - 目标文件路径
/// * `data` - 要序列化和写入的数据
///
/// # 错误
///
/// 如果序列化失败或文件写入失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::write_json_file;
/// use serde::Serialize;
/// use std::path::PathBuf;
///
/// #[derive(Debug, Serialize)]
/// struct Config {
///     name: String,
/// }
///
/// # fn main() -> color_eyre::Result<()> {
/// let config = Config {
///     name: "example".to_string(),
/// };
/// write_json_file(PathBuf::from("config.json").as_path(), &config)?;
/// # Ok(())
/// # }
/// ```
pub fn write_json_file<T>(path: &Path, data: &T) -> Result<()>
where
    T: Serialize,
{
    let json_content = serde_json::to_string_pretty(data)
        .wrap_err_with(|| format!("Failed to serialize config to JSON: {:?}", path))?;
    fs::write(path, json_content)
        .wrap_err_with(|| format!("Failed to write JSON file: {:?}", path))?;
    Ok(())
}

/// 将 `serde_json::Value` 写入文件
///
/// 将 `serde_json::Value` 序列化为 JSON 格式并写入指定路径的文件。
///
/// # 参数
///
/// * `path` - 目标文件路径
/// * `value` - 要写入的 `serde_json::Value` 实例
///
/// # 错误
///
/// 如果序列化失败或文件写入失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::write_json_value;
/// use serde_json::json;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let value = json!({
///     "name": "example"
/// });
/// write_json_value(PathBuf::from("config.json").as_path(), &value)?;
/// # Ok(())
/// # }
/// ```
pub fn write_json_value(path: &Path, value: &serde_json::Value) -> Result<()> {
    write_json_file(path, value)
}

/// 将字符串内容写入文件
///
/// 将字符串内容写入指定路径的文件，提供统一的错误处理。
///
/// # 参数
///
/// * `path` - 目标文件路径
/// * `content` - 要写入的字符串内容
///
/// # 错误
///
/// 如果文件写入失败，返回包含上下文信息的错误。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::write_file_with_context;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// write_file_with_context(PathBuf::from("output.txt").as_path(), "Hello, World!")?;
/// # Ok(())
/// # }
/// ```
pub fn write_file_with_context(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).wrap_err_with(|| format!("Failed to write file: {:?}", path))
}

/// 将字节内容写入文件
///
/// 将字节内容写入指定路径的文件，提供统一的错误处理。
///
/// # 参数
///
/// * `path` - 目标文件路径
/// * `content` - 要写入的字节内容
///
/// # 错误
///
/// 如果文件写入失败，返回包含上下文信息的错误。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::file::write_file_bytes_with_context;
/// use std::path::PathBuf;
///
/// # fn main() -> color_eyre::Result<()> {
/// let data = b"binary data";
/// write_file_bytes_with_context(PathBuf::from("output.bin").as_path(), data)?;
/// # Ok(())
/// # }
/// ```
pub fn write_file_bytes_with_context(path: &Path, content: &[u8]) -> Result<()> {
    fs::write(path, content).wrap_err_with(|| format!("Failed to write file: {:?}", path))
}
