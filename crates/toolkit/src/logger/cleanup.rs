//! 日志文件清理模块
//!
//! 负责清理过期的日志文件，防止日志目录无限增长。
//!
//! ## 清理策略
//!
//! - **时间限制**：删除超过 30 天的日志文件
//! - **数量限制**：最多保留 100 个日志文件
//! - **触发时机**：每次 `logger::init()` 时自动执行

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// 日志文件最大保留天数
const MAX_LOG_AGE_DAYS: u64 = 7;

/// 日志文件最大保留数量
const MAX_LOG_FILES: usize = 100;

/// 清理过期的日志文件
///
/// 扫描日志目录下的 `~/.workflow/logs/` 子目录，根据文件数量和修改时间清理过期日志。
/// 清理过程中的错误会被静默忽略，不影响正常的日志初始化流程。
///
/// # 清理规则
///
/// 1. 按修改时间降序排列所有 `.log` 文件
/// 2. 超过 [`MAX_LOG_FILES`] 数量限制的文件会被删除
/// 3. 修改时间超过 [`MAX_LOG_AGE_DAYS`] 天的文件会被删除
/// 4. 两个条件满足任一即删除
///
/// # 参数
///
/// * `logs_dir` - 日志根目录路径（如 `~/.workflow/logs/`）
pub(crate) fn cleanup_logs(logs_dir: impl AsRef<Path>) {
    let logs_dir = logs_dir.as_ref();
    if !logs_dir.exists() {
        return;
    }

    let entries = match fs::read_dir(logs_dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    // 收集所有 .log 文件及其修改时间
    let mut log_files: Vec<(std::path::PathBuf, SystemTime)> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "log"))
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((entry.path(), modified))
        })
        .collect();

    // 按修改时间降序排列（最新的在前）
    log_files.sort_by(|a, b| b.1.cmp(&a.1));

    let now = SystemTime::now();
    let max_age = Duration::from_secs(MAX_LOG_AGE_DAYS * 24 * 3600);

    for (i, (path, modified)) in log_files.iter().enumerate() {
        let is_expired = now.duration_since(*modified).map(|age| age > max_age).unwrap_or(false);

        if i >= MAX_LOG_FILES || is_expired {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use filetime::FileTime;
    use rstest::rstest;
    use tempfile::TempDir;

    use super::*;

    /// 创建测试用的日志目录结构（与 path.rs 一致：日志文件直接在 logs_dir 下）
    fn setup_tracing_dir(temp_dir: &TempDir) -> PathBuf {
        let logs_dir = temp_dir.path();
        fs::create_dir_all(logs_dir).unwrap();
        logs_dir.to_path_buf()
    }

    /// 在日志目录中创建一个日志文件
    fn create_log_file(logs_dir: &Path, name: &str) -> PathBuf {
        let path = logs_dir.join(name);
        let mut file = File::create(&path).unwrap();
        writeln!(file, "test log content").unwrap();
        path
    }

    /// 设置文件的修改时间为 N 天前
    fn set_file_age_days(path: &Path, days: u64) {
        let age = Duration::from_secs(days * 24 * 3600);
        let past = SystemTime::now() - age;
        let ft = FileTime::from_system_time(past);
        filetime::set_file_mtime(path, ft).unwrap();
    }

    // ==================== 基本行为测试 ====================

    #[test]
    fn test_cleanup_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let logs_dir = temp_dir.path().join("nonexistent");
        // 不 panic，静默返回
        cleanup_logs(&logs_dir);
    }

    #[test]
    fn test_cleanup_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let _tracing_dir = setup_tracing_dir(&temp_dir);
        // 空目录不 panic
        cleanup_logs(temp_dir.path());
    }

    #[test]
    fn test_cleanup_keeps_recent_files() {
        let temp_dir = TempDir::new().unwrap();
        let tracing_dir = setup_tracing_dir(&temp_dir);

        // 创建 3 个新文件
        create_log_file(&tracing_dir, "cmd-20260201120000-1001.log");
        create_log_file(&tracing_dir, "cmd-20260202120000-1002.log");
        create_log_file(&tracing_dir, "cmd-20260203120000-1003.log");

        cleanup_logs(temp_dir.path());

        // 所有文件都应保留
        let remaining: Vec<_> =
            fs::read_dir(&tracing_dir).unwrap().filter_map(|e| e.ok()).collect();
        assert_eq!(remaining.len(), 3, "All recent files should be kept");
    }

    // ==================== 时间限制测试 ====================

    #[test]
    fn test_cleanup_removes_expired_files() {
        let temp_dir = TempDir::new().unwrap();
        let tracing_dir = setup_tracing_dir(&temp_dir);

        // 创建 1 个新文件和 2 个过期文件（超过 30 天）
        create_log_file(&tracing_dir, "new-20260208120000-1001.log");

        let old1 = create_log_file(&tracing_dir, "old-20251201120000-1002.log");
        set_file_age_days(&old1, 40);

        let old2 = create_log_file(&tracing_dir, "old-20251115120000-1003.log");
        set_file_age_days(&old2, 60);

        cleanup_logs(temp_dir.path());

        let remaining: Vec<_> =
            fs::read_dir(&tracing_dir).unwrap().filter_map(|e| e.ok()).collect();
        assert_eq!(remaining.len(), 1, "Only the recent file should remain");
        assert!(
            remaining[0].file_name().to_string_lossy().contains("new-"),
            "The remaining file should be the new one"
        );
    }

    // ==================== 数量限制测试 ====================

    #[rstest]
    fn test_cleanup_respects_max_file_count() {
        let temp_dir = TempDir::new().unwrap();
        let tracing_dir = setup_tracing_dir(&temp_dir);

        // 创建超过 MAX_LOG_FILES 数量的文件
        let total = MAX_LOG_FILES + 5;
        for i in 0..total {
            let name = format!("cmd-20260208{:06}-{}.log", i, 1000 + i);
            let path = create_log_file(&tracing_dir, &name);
            // 让文件有不同的修改时间，确保排序稳定
            let age = Duration::from_secs((total - i) as u64 * 60);
            let past = SystemTime::now() - age;
            let ft = FileTime::from_system_time(past);
            filetime::set_file_mtime(&path, ft).unwrap();
        }

        cleanup_logs(temp_dir.path());

        let remaining: Vec<_> =
            fs::read_dir(&tracing_dir).unwrap().filter_map(|e| e.ok()).collect();
        assert_eq!(
            remaining.len(),
            MAX_LOG_FILES,
            "Should keep at most {} files",
            MAX_LOG_FILES
        );
    }

    // ==================== 非 .log 文件不受影响 ====================

    #[test]
    fn test_cleanup_ignores_non_log_files() {
        let temp_dir = TempDir::new().unwrap();
        let tracing_dir = setup_tracing_dir(&temp_dir);

        // 创建非 .log 文件
        create_log_file(&tracing_dir, "notes.txt");
        create_log_file(&tracing_dir, "data.json");

        // 创建一个过期的 .log 文件
        let old = create_log_file(&tracing_dir, "old-20251101120000-1001.log");
        set_file_age_days(&old, 60);

        cleanup_logs(temp_dir.path());

        let remaining: Vec<_> =
            fs::read_dir(&tracing_dir).unwrap().filter_map(|e| e.ok()).collect();
        // .txt 和 .json 文件应该保留，过期的 .log 被删除
        assert_eq!(remaining.len(), 2, "Non-log files should not be affected");
    }

    // ==================== 边界情况测试 ====================

    #[test]
    fn test_cleanup_file_near_age_limit() {
        let temp_dir = TempDir::new().unwrap();
        let tracing_dir = setup_tracing_dir(&temp_dir);

        // 29 天的文件（未过期，应保留）
        let within_limit = create_log_file(&tracing_dir, "within-limit-20260110120000-1001.log");
        set_file_age_days(&within_limit, MAX_LOG_AGE_DAYS - 1);

        // 超过 30 天的文件（已过期，应删除）
        let over_limit = create_log_file(&tracing_dir, "over-limit-20260108120000-1002.log");
        set_file_age_days(&over_limit, MAX_LOG_AGE_DAYS + 1);

        cleanup_logs(temp_dir.path());

        let remaining: Vec<_> =
            fs::read_dir(&tracing_dir).unwrap().filter_map(|e| e.ok()).collect();
        assert_eq!(
            remaining.len(),
            1,
            "Only the file within limit should be kept"
        );
        assert!(
            remaining[0].file_name().to_string_lossy().contains("within-limit"),
            "The remaining file should be the one within limit"
        );
    }
}
