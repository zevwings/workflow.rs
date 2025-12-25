//! 共享测试工具函数
//!
//! 提供测试中常用的辅助函数和工具。

#![allow(dead_code)] // 这些函数是为测试准备的公共 API

use std::fs;
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
/// let test_dir = create_temp_test_dir("my_test");
/// // 使用 test_dir 进行测试
/// ```
pub fn create_temp_test_dir(prefix: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let timestamp = workflow::base::util::date::get_unix_timestamp_nanos();
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
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    test_dir
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
pub fn create_test_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
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
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

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
    pub fn new(new_dir: impl AsRef<Path>) -> color_eyre::Result<Self> {
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(new_dir)?;
        Ok(Self { original_dir })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        // 忽略恢复失败，避免 panic during panic
        let _ = std::env::set_current_dir(&self.original_dir);
    }
}
