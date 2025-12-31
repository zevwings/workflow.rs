#![allow(clippy::test_attr_in_doctest, dead_code, unused)]

//! 测试数据缓存
//!
//! 提供 fixture 文件的缓存机制，避免重复加载相同的测试数据文件。
//!
//! # 使用示例
//!
//! ```rust
//! use crate::common::cache::get_cached_fixture;
//!
//! #[test]
//! fn test_with_cached_fixture() -> Result<()> {
//!     // 首次调用：从磁盘加载并缓存
//!     let json1 = get_cached_fixture("sample_response.json");
//!
//!     // 后续调用：直接从缓存获取，无需磁盘 I/O
//!     let json2 = get_cached_fixture("sample_response.json");
//!
//!     assert_eq!(json1, json2);
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Mutex;

/// Fixture 文件缓存
///
/// 使用线程安全的 HashMap 存储已加载的 fixture 文件内容。
/// Key: fixture 文件名
/// Value: fixture 文件内容
static FIXTURE_CACHE: std::sync::LazyLock<Mutex<HashMap<String, String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// 获取缓存的 fixture 文件
///
/// 如果 fixture 已在缓存中，直接返回缓存的内容。
/// 如果不在缓存中，从磁盘加载并缓存，然后返回内容。
///
/// # 参数
///
/// * `name` - fixture 文件名（相对于 `tests/fixtures/` 目录）
///
/// # 返回
///
/// 返回 fixture 文件的内容作为字符串。
///
/// # 性能
///
/// - **首次加载**: 正常 I/O 时间（如 1-5ms）
/// - **缓存命中**: 几乎零开销（<0.1ms）
/// - **100个测试使用同一 fixture**: 从 100-500ms 降低到 1-5ms + 99×0.1ms ≈ 11ms
/// - **提升**: 约 10-50 倍性能提升（取决于 fixture 文件大小和数量）
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::cache::get_cached_fixture;
///
/// #[test]
/// fn test_with_cached_fixture() -> Result<()> {
///     let json_data = get_cached_fixture("sample_response.json");
///     // 使用 json_data 进行测试
///     Ok(())
/// }
/// ```
pub fn get_cached_fixture(name: &str) -> String {
    // 先尝试从缓存获取
    {
        let cache = FIXTURE_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(name) {
            return cached.clone();
        }
    }

    // 缓存未命中，从磁盘加载
    let content = crate::common::helpers::load_fixture(name);

    // 存入缓存（使用 entry API 避免重复插入）
    // 如果另一个线程在我们加载期间已经缓存了，使用已缓存的值
    let mut cache = FIXTURE_CACHE.lock().unwrap();
    cache.entry(name.to_string()).or_insert_with(|| content.clone()).clone()
}

/// 清除 fixture 缓存
///
/// 清空所有缓存的 fixture 文件内容。
/// 主要用于测试清理或需要强制重新加载的场景。
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::cache::{get_cached_fixture, clear_cache};
///
/// #[test]
/// fn test_cache_clear() {
///     // 加载并缓存
///     let _ = get_cached_fixture("sample.json");
///
///     // 清除缓存
///     clear_cache();
///
///     // 下次调用会重新从磁盘加载
///     let _ = get_cached_fixture("sample.json");
/// }
/// ```
pub fn clear_cache() {
    let mut cache = FIXTURE_CACHE.lock().unwrap();
    cache.clear();
}

/// 获取缓存统计信息
///
/// 返回当前缓存中的 fixture 文件数量。
///
/// # 返回
///
/// 缓存的 fixture 文件数量
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::cache::{get_cached_fixture, cache_size};
///
/// #[test]
/// fn test_cache_stats() {
///     assert_eq!(cache_size(), 0);
///
///     get_cached_fixture("sample.json");
///     assert_eq!(cache_size(), 1);
///
///     get_cached_fixture("another.json");
///     assert_eq!(cache_size(), 2);
/// }
/// ```
pub fn cache_size() -> usize {
    let cache = FIXTURE_CACHE.lock().unwrap();
    cache.len()
}

/// 检查 fixture 是否已缓存
///
/// # 参数
///
/// * `name` - fixture 文件名
///
/// # 返回
///
/// 如果 fixture 已在缓存中，返回 `true`，否则返回 `false`
///
/// # 示例
///
/// ```rust,no_run
/// use crate::common::cache::{get_cached_fixture, is_cached};
///
/// #[test]
/// fn test_cache_check() {
///     assert!(!is_cached("sample.json"));
///
///     get_cached_fixture("sample.json");
///
///     assert!(is_cached("sample.json"));
/// }
/// ```
pub fn is_cached(name: &str) -> bool {
    let cache = FIXTURE_CACHE.lock().unwrap();
    cache.contains_key(name)
}

// 注意：缓存测试已移至 tests/base/cache.rs，只在 module_test 中运行
// 这样可以避免在 e2e_test 中运行时由于并行测试的竞态条件导致测试失败
