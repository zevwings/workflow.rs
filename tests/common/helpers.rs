#![allow(dead_code, clippy::test_attr_in_doctest)] // 这些函数是为测试准备的公共 API

//! 共享测试工具函数
//!
//! 提供测试中常用的辅助函数和工具。
//!
//! # 路径获取函数
//!
//! 本模块提供了统一的路径获取函数，使用 `dirs` crate 并支持测试环境隔离：
//!
//! - [`test_home_dir()`] - 获取主目录（测试环境感知）
//! - [`test_config_dir()`] - 获取配置目录（测试环境感知）
//! - [`test_data_dir()`] - 获取数据目录（测试环境感知）
//! - [`test_cache_dir()`] - 获取缓存目录（测试环境感知）
//!
//! 这些函数优先使用环境变量（支持测试隔离），然后回退到 `dirs` crate 的标准目录。
//! 与源代码中的 `Paths::home_dir()` 行为一致，确保测试环境的行为与生产环境一致。
//!
//! ## 使用示例
//!
//! ```no_run
//! use tests::common::helpers::test_home_dir;
//! use tests::common::guards::EnvGuard;
//!
//! #[test]
//! fn test_example() -> color_eyre::Result<()> {
//!     let mut guard = EnvGuard::new();
//!     guard.set("HOME", "/test/isolated/home");
//!
//!     let home = test_home_dir()?;
//!     assert_eq!(home, PathBuf::from("/test/isolated/home"));
//!     Ok(())
//! }
//! ```
//!
//! ## 注意事项
//!
//! - **测试隔离**：使用 `EnvGuard` 设置环境变量后，这些函数会返回测试隔离的路径
//! - **临时目录**：临时目录应继续使用 `std::env::temp_dir()` 或 `tempfile::tempdir()`
//! - **当前目录**：当前目录应继续使用 `std::env::current_dir()`
//! - **测试基础设施**：`TestIsolation` 和 `CliTestEnv` 创建的路径不需要使用这些函数

use color_eyre::eyre::Context;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Once;

static INIT: Once = Once::new();

/// 初始化测试环境
///
/// 确保测试环境变量和配置已正确设置。
/// 这个函数只会执行一次，即使被多次调用。
pub fn setup_test_env() {
    INIT.call_once(|| {
        // 设置测试环境变量
        std::env::set_var("RUST_LOG", "debug");
        // 可以在这里添加其他环境变量设置
    });
}

/// 清理测试环境
///
/// 清理测试过程中创建的临时文件和目录。
pub fn cleanup_test_env() {
    // 如果需要，可以在这里添加清理逻辑
}

/// 创建临时测试目录
///
/// 在系统临时目录下创建一个唯一的测试目录。
///
/// # 返回
///
/// 返回创建的临时目录路径。
///
/// # 示例
///
/// ```no_run
/// use tests::common::helpers::create_temp_test_dir;
///
/// let test_dir = create_temp_test_dir("my_test")?;
/// // 使用 test_dir 进行测试
/// ```
pub fn create_temp_test_dir(prefix: &str) -> color_eyre::Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let timestamp = workflow::base::format::date::get_unix_timestamp_nanos();
    let random_suffix = random_string(8);
    let test_dir = temp_dir.join(format!(
        "workflow_test_{}_{}_{}",
        prefix, timestamp, random_suffix
    ));

    // 如果目录已存在，先删除
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir).ok();
    }

    // 创建目录
    fs::create_dir_all(&test_dir)
        .wrap_err_with(|| format!("Failed to create test directory: {}", test_dir.display()))?;
    Ok(test_dir)
}

/// 清理临时测试目录
///
/// 删除指定的临时测试目录及其所有内容。
///
/// # 参数
///
/// * `dir` - 要删除的目录路径
pub fn cleanup_temp_test_dir(dir: &Path) {
    if dir.exists() {
        fs::remove_dir_all(dir).ok();
    }
}

/// 加载测试 fixture 文件
///
/// 从 `tests/fixtures/` 目录加载测试数据文件。
///
/// # 参数
///
/// * `name` - fixture 文件名（相对于 fixtures 目录）
///
/// # 返回
///
/// 返回文件内容作为字符串。
///
/// # 示例
///
/// ```no_run
/// use tests::common::helpers::load_fixture;
///
/// let json_data = load_fixture("sample_response.json");
/// ```
pub fn load_fixture(name: &str) -> String {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);

    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

/// 获取 fixture 文件路径
///
/// 返回 fixture 文件的完整路径，但不读取内容。
///
/// # 参数
///
/// * `name` - fixture 文件名（相对于 fixtures 目录）
///
/// # 返回
///
/// 返回 fixture 文件的路径。
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// 创建测试文件
///
/// 在指定目录下创建测试文件并写入内容。
///
/// # 参数
///
/// * `dir` - 目标目录
/// * `filename` - 文件名
/// * `content` - 文件内容
///
/// # 返回
///
/// 返回创建的文件路径。
pub fn create_test_file(dir: &Path, filename: &str, content: &str) -> color_eyre::Result<PathBuf> {
    let file_path = dir.join(filename);
    fs::write(&file_path, content)
        .wrap_err_with(|| format!("Failed to write test file: {}", file_path.display()))?;
    Ok(file_path)
}

/// 断言文件存在
///
/// 检查指定路径的文件是否存在，如果不存在则测试失败。
///
/// # 参数
///
/// * `path` - 文件路径
pub fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "Expected file to exist: {}", path.display());
    assert!(
        path.is_file(),
        "Expected path to be a file: {}",
        path.display()
    );
}

/// 断言目录存在
///
/// 检查指定路径的目录是否存在，如果不存在则测试失败。
///
/// # 参数
///
/// * `path` - 目录路径
pub fn assert_dir_exists(path: &Path) {
    assert!(
        path.exists(),
        "Expected directory to exist: {}",
        path.display()
    );
    assert!(
        path.is_dir(),
        "Expected path to be a directory: {}",
        path.display()
    );
}

/// 读取文件内容
///
/// 读取文件内容并返回字符串，如果读取失败则测试失败。
///
/// # 参数
///
/// * `path` - 文件路径
///
/// # 返回
///
/// 返回文件内容。
pub fn read_file_content(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e))
}

/// 等待一小段时间
///
/// 在测试中用于等待异步操作完成。
///
/// # 参数
///
/// * `millis` - 等待的毫秒数
pub fn wait_millis(millis: u64) {
    std::thread::sleep(std::time::Duration::from_millis(millis));
}

/// 生成随机字符串
///
/// 生成指定长度的随机字符串，用于测试中的唯一标识符。
///
/// # 参数
///
/// * `length` - 字符串长度
///
/// # 返回
///
/// 返回随机字符串。
pub fn random_string(length: usize) -> String {
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    format!("{:x}", hasher.finish())[..length.min(16)].to_string()
}

/// 断言错误消息包含预期的关键词
///
/// 用于测试错误处理，验证错误消息是否包含预期的关键词。
///
/// # 参数
///
/// * `error_msg` - 错误消息
/// * `keywords` - 预期的关键词列表（至少包含一个）
///
/// # 示例
///
/// ```no_run
/// use tests::common::helpers::assert_error_contains;
///
/// let error_msg = "Log file not found";
/// assert_error_contains(&error_msg, &["not found", "Log file"]);
/// ```
pub fn assert_error_contains(error_msg: &str, keywords: &[&str]) {
    let found = keywords.iter().any(|keyword| error_msg.contains(keyword));
    assert!(
        found,
        "Error message should contain at least one of {:?}: {}",
        keywords, error_msg
    );
}

/// 当前目录守卫
///
/// 使用 RAII 模式确保当前目录在作用域结束时恢复到原始值。
/// 即使在测试失败（panic）时也能保证恢复，避免测试间的状态污染。
///
/// # 使用场景
///
/// - 需要临时切换到其他目录执行操作
/// - 确保测试间的目录隔离
/// - 避免全局状态污染
///
/// # 示例
///
/// ```no_run
/// use tests::common::helpers::CurrentDirGuard;
/// use std::path::Path;
///
/// #[test]
/// fn my_test() -> color_eyre::Result<()> {
///     // 自动恢复目录，即使测试失败
///     let _guard = CurrentDirGuard::new("/tmp/test")?;
///
///     // 在新目录中执行操作
///     assert_eq!(std::env::current_dir()?, Path::new("/tmp/test"));
///
///     // Drop 时自动恢复到原始目录
///     Ok(())
/// }
/// ```
///
/// # 注意事项
///
/// - 必须保持`_guard`变量在作用域内，通常命名为`_guard`以表明其用途
/// - 如果需要手动提前恢复，可以显式调用`drop(_guard)`
/// - Drop 时的恢复失败会被忽略（避免 panic during panic）
pub struct CurrentDirGuard {
    original_dir: PathBuf,
}

impl CurrentDirGuard {
    /// 创建目录守卫并切换到新目录
    ///
    /// # 参数
    ///
    /// * `new_dir` - 要切换到的目标目录
    ///
    /// # 返回
    ///
    /// 成功时返回守卫实例，失败时返回错误
    ///
    /// # 错误
    ///
    /// - 无法获取当前目录
    /// - 无法切换到目标目录
    ///
    /// # 注意事项
    ///
    /// - 使用绝对路径存储原始目录，避免相对路径问题
    /// - 在并发测试环境中，如果当前目录不存在，会尝试使用备用方案
    pub fn new(new_dir: impl AsRef<Path>) -> color_eyre::Result<Self> {
        // 获取当前目录，使用绝对路径避免相对路径问题
        // 在并发测试环境中，如果当前目录被删除，尝试使用备用方案
        let original_dir = std::env::current_dir()
            .and_then(|p| p.canonicalize())
            .or_else(|_| {
                // 如果当前目录不存在，尝试使用项目根目录或可执行文件目录作为备用
                std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).or_else(|_| {
                    std::env::current_exe().and_then(|exe| {
                        exe.parent()
                            .ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::NotFound,
                                    "Cannot determine original directory",
                                )
                            })
                            .map(PathBuf::from)
                    })
                })
            })
            .wrap_err("Failed to get current directory")?;

        // 确保目标目录存在且是绝对路径
        let new_dir = new_dir.as_ref();
        let new_dir_abs = if new_dir.is_absolute() {
            new_dir.to_path_buf()
        } else {
            // 如果是相对路径，基于原始目录解析
            original_dir.join(new_dir)
        };

        // 确保目标目录存在
        if !new_dir_abs.exists() {
            return Err(color_eyre::eyre::eyre!(
                "Target directory does not exist: {}",
                new_dir_abs.display()
            ));
        }

        std::env::set_current_dir(&new_dir_abs).wrap_err_with(|| {
            format!("Failed to change directory to: {}", new_dir_abs.display())
        })?;

        Ok(Self { original_dir })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        // 尝试恢复原始目录
        // 如果原始目录不存在，尝试使用备用方案
        if std::env::set_current_dir(&self.original_dir).is_err() {
            // 如果原始目录不存在，尝试使用项目根目录或可执行文件目录
            if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                let _ = std::env::set_current_dir(manifest_dir);
            } else if let Ok(exe_dir) = std::env::current_exe().and_then(|exe| {
                exe.parent()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "Cannot determine directory",
                        )
                    })
                    .map(PathBuf::from)
            }) {
                let _ = std::env::set_current_dir(exe_dir);
            }
            // 如果所有方案都失败，忽略错误，避免 panic during panic
        }
    }
}

/// 获取当前目录（带备用方案）
///
/// 在并发测试环境中，如果当前目录被删除，会尝试使用备用方案：
/// 1. 尝试获取并规范化当前目录
/// 2. 如果失败，尝试使用 `CARGO_MANIFEST_DIR` 环境变量
/// 3. 如果失败，尝试使用可执行文件的父目录
///
/// # 返回
///
/// 返回当前目录的绝对路径，失败时返回错误
///
/// # 示例
///
/// ```no_run
/// use tests::common::helpers::get_current_dir_with_fallback;
///
/// #[test]
/// fn test_example() -> color_eyre::Result<()> {
///     let current_dir = get_current_dir_with_fallback()?;
///     // 使用 current_dir
///     Ok(())
/// }
/// ```
pub fn get_current_dir_with_fallback() -> color_eyre::Result<PathBuf> {
    std::env::current_dir()
        .and_then(|p| p.canonicalize())
        .or_else(|_| {
            // 如果当前目录不存在，尝试使用项目根目录或可执行文件目录作为备用
            std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).or_else(|_| {
                std::env::current_exe().and_then(|exe| {
                    exe.parent()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "Cannot determine current directory",
                            )
                        })
                        .map(PathBuf::from)
                })
            })
        })
        .wrap_err("Failed to get current directory")
}

// ==================== 统一路径获取函数（使用 dirs crate）====================

/// 获取主目录（测试环境感知）
///
/// 优先使用环境变量（支持测试隔离），然后回退到 `dirs::home_dir()`。
/// 这与源代码中的 `Paths::home_dir()` 行为一致，确保测试环境的行为与生产环境一致。
///
/// # 返回
///
/// 返回用户主目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法确定主目录，返回错误信息。
///
/// # 示例
///
/// ```no_run
/// use tests::common::helpers::test_home_dir;
/// use crate::common::guards::EnvGuard;
///
/// #[test]
/// fn test_with_isolated_home() -> color_eyre::Result<()> {
///     let mut guard = EnvGuard::new();
///     guard.set("HOME", "/test/isolated/home");
///
///     let home = test_home_dir()?;
///     assert_eq!(home, PathBuf::from("/test/isolated/home"));
///     Ok(())
/// }
/// ```
pub fn test_home_dir() -> color_eyre::Result<PathBuf> {
    // 优先检查环境变量（确保测试环境中的 HOME 被正确使用）
    #[cfg(unix)]
    {
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);
            if home_path.is_absolute() {
                return Ok(home_path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            let home_path = PathBuf::from(home);
            if home_path.is_absolute() {
                return Ok(home_path);
            }
        }
    }

    // 回退到 dirs::home_dir()
    dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Cannot determine home directory"))
}

/// 获取配置目录（测试环境感知）
///
/// 返回测试环境中的配置目录路径。
/// 如果设置了 `WORKFLOW_CONFIG_DIR` 环境变量，使用该路径。
/// 否则使用标准配置目录路径（`~/.workflow/config`）。
///
/// # 返回
///
/// 返回配置目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法确定主目录，返回错误信息。
///
/// # 示例
///
/// ```no_run
/// use tests::common::helpers::test_config_dir;
/// use crate::common::guards::EnvGuard;
///
/// #[test]
/// fn test_with_isolated_config() -> color_eyre::Result<()> {
///     let mut guard = EnvGuard::new();
///     guard.set("HOME", "/test/home");
///
///     let config_dir = test_config_dir()?;
///     assert!(config_dir.to_string_lossy().contains(".workflow"));
///     assert!(config_dir.to_string_lossy().contains("config"));
///     Ok(())
/// }
/// ```
pub fn test_config_dir() -> color_eyre::Result<PathBuf> {
    // 优先使用测试环境变量
    if let Ok(config_dir) = std::env::var("WORKFLOW_CONFIG_DIR") {
        return Ok(PathBuf::from(config_dir));
    }

    // 使用标准配置目录
    let home = test_home_dir()?;
    Ok(home.join(".workflow").join("config"))
}

/// 获取数据目录（测试环境感知）
///
/// 返回测试环境中的数据目录路径。
/// 优先使用环境变量，然后回退到 `dirs` crate 的标准目录。
///
/// # 返回
///
/// 返回数据目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法确定主目录，返回错误信息。
///
/// # 平台差异
///
/// - **Windows**: `%LOCALAPPDATA%` 或 `dirs::data_local_dir()`
/// - **Unix**: `$XDG_DATA_HOME` 或 `~/.local/share`
pub fn test_data_dir() -> color_eyre::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(data_dir) = std::env::var("LOCALAPPDATA") {
            let data_path = PathBuf::from(data_dir);
            if data_path.is_absolute() {
                return Ok(data_path);
            }
        }
        // 回退到 dirs
        if let Some(data_dir) = dirs::data_local_dir() {
            return Ok(data_dir);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix: 使用 XDG_DATA_HOME 或默认 ~/.local/share
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            let data_path = PathBuf::from(data_home);
            if data_path.is_absolute() {
                return Ok(data_path);
            }
        }
        if let Some(data_dir) = dirs::data_dir() {
            return Ok(data_dir);
        }
    }

    // 回退到主目录下的标准位置
    let home = test_home_dir()?;
    Ok(home.join(".local").join("share"))
}

/// 获取缓存目录（测试环境感知）
///
/// 返回测试环境中的缓存目录路径。
/// 优先使用环境变量，然后回退到 `dirs` crate 的标准目录。
///
/// # 返回
///
/// 返回缓存目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法确定主目录，返回错误信息。
///
/// # 平台差异
///
/// - **Unix**: `$XDG_CACHE_HOME` 或 `~/.cache`
/// - **Windows**: `%LOCALAPPDATA%` 下的缓存目录
pub fn test_cache_dir() -> color_eyre::Result<PathBuf> {
    #[cfg(not(target_os = "windows"))]
    {
        // Unix: 使用 XDG_CACHE_HOME 或默认 ~/.cache
        if let Ok(cache_home) = std::env::var("XDG_CACHE_HOME") {
            let cache_path = PathBuf::from(cache_home);
            if cache_path.is_absolute() {
                return Ok(cache_path);
            }
        }
    }

    if let Some(cache_dir) = dirs::cache_dir() {
        return Ok(cache_dir);
    }

    // 回退到主目录下的标准位置
    let home = test_home_dir()?;
    Ok(home.join(".cache"))
}
