//! 测试辅助函数
//!
//! 提供测试中使用的辅助函数。

use std::path::PathBuf;
use tempfile::TempDir;

/// 创建临时目录用于测试
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// 创建临时文件路径
pub fn create_temp_file_path(prefix: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let temp_dir = create_temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    temp_dir.path().join(format!("{}_{}", prefix, timestamp))
}

/// 设置测试环境变量
pub fn set_test_env(key: &str, value: &str) {
    std::env::set_var(key, value);
}

/// 清除测试环境变量
pub fn unset_test_env(key: &str) {
    std::env::remove_var(key);
}

/// 在测试中禁用交互式提示（用于非交互式测试）
pub fn disable_interactive() {
    std::env::set_var("WORKFLOW_NON_INTERACTIVE", "1");
}

/// 启用交互式提示
pub fn enable_interactive() {
    std::env::remove_var("WORKFLOW_NON_INTERACTIVE");
}
