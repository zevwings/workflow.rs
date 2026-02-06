//! 路径展开工具
//!
//! 提供路径字符串展开功能，支持 Unix 风格的 `~` 展开和 Windows 风格的环境变量展开。
//!
//! 使用 `shellexpand` 处理 `~` 展开和 Unix 风格的环境变量（`$VAR`、`${VAR}`），
//! 同时保留自定义逻辑处理 Windows 风格的 `%VAR%` 环境变量。

use std::env;
use std::path::PathBuf;

// //! 路径操作错误类型

use thiserror::Error;

/// 路径操作错误
#[derive(Debug, Error)]
pub enum PathExpandError {
    /// 环境变量错误
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    /// 路径展开错误
    #[error("Path expansion error: {0}")]
    Expansion(String),
}

/// 展开路径字符串
///
/// 支持的路径格式：
/// - Unix: `~` 和 `~/path` - 展开为用户主目录
/// - Unix: `$VAR` 和 `${VAR}` - 展开环境变量（通过 shellexpand）
/// - Windows: `%VAR%` 和 `%VAR%\path` - 展开环境变量（自定义处理）
/// - 绝对路径: 直接使用
///
/// # 示例
///
/// ```text
/// // Unix
/// expand("~/Documents/Workflow") -> "/home/user/Documents/Workflow"
/// expand("~") -> "/home/user"
/// expand("$HOME/Documents") -> "/home/user/Documents"
/// expand("${HOME}/Documents") -> "/home/user/Documents"
///
/// // Windows
/// expand("%USERPROFILE%\\Documents\\Workflow") -> "C:\\Users\\User\\Documents\\Workflow"
/// expand("%APPDATA%\\workflow") -> "C:\\Users\\User\\AppData\\Roaming\\workflow"
///
/// // 绝对路径
/// expand("/absolute/path") -> "/absolute/path"
/// expand("C:\\absolute\\path") -> "C:\\absolute\\path"
/// ```
pub fn expand(path_str: &str) -> Result<PathBuf, PathExpandError> {
    // 先处理 Windows 风格的环境变量展开 %VAR%
    // 因为 shellexpand 不支持这种格式
    let expanded = if path_str.contains('%') {
        expand_windows_env_vars(path_str)?
    } else {
        path_str.to_string()
    };

    // 使用 shellexpand 处理 ~ 展开和 Unix 风格的环境变量
    let result = shellexpand::full(&expanded)
        .map_err(|e| PathExpandError::Expansion(format!("Failed to expand path: {}", e)))?;

    Ok(PathBuf::from(result.as_ref()))
}

/// 展开 Windows 风格的环境变量 `%VAR%`
fn expand_windows_env_vars(path_str: &str) -> Result<String, PathExpandError> {
    let mut result = String::new();
    let mut chars = path_str.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut var_name = String::new();
            loop {
                match chars.next() {
                    Some('%') => break,
                    Some(c) => var_name.push(c),
                    None => break,
                }
            }

            if !var_name.is_empty() {
                let var_value = env::var(&var_name).map_err(PathExpandError::EnvVar)?;
                result.push_str(&var_value);
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_tilde() {
        // 测试 ~ 展开为主目录
        let result = expand("~");
        assert!(result.is_ok());

        let expanded = result.unwrap();
        // 验证展开后不再包含 ~
        assert!(!expanded.to_string_lossy().starts_with('~'));
        // 验证是绝对路径
        assert!(expanded.is_absolute());
    }

    #[test]
    fn test_expand_tilde_with_path() {
        // 测试 ~/path 格式
        let result = expand("~/Documents/test");
        assert!(result.is_ok());

        let expanded = result.unwrap();
        assert!(expanded.is_absolute());
        assert!(expanded.to_string_lossy().ends_with("Documents/test"));
    }

    #[test]
    fn test_expand_absolute_path() {
        // 绝对路径应该保持不变
        let path = "/absolute/path/to/file";
        let result = expand(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), path);
    }

    #[test]
    fn test_expand_relative_path() {
        // 相对路径应该保持不变
        let path = "relative/path";
        let result = expand(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), path);
    }

    #[test]
    fn test_expand_unix_env_var_dollar() {
        // 设置测试环境变量
        env::set_var("TEST_EXPAND_VAR", "/test/value");

        let result = expand("$TEST_EXPAND_VAR/subdir");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), "/test/value/subdir");

        env::remove_var("TEST_EXPAND_VAR");
    }

    #[test]
    fn test_expand_unix_env_var_braces() {
        // 设置测试环境变量
        env::set_var("TEST_EXPAND_VAR2", "/braces/value");

        let result = expand("${TEST_EXPAND_VAR2}/subdir");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), "/braces/value/subdir");

        env::remove_var("TEST_EXPAND_VAR2");
    }

    #[test]
    fn test_expand_windows_env_var() {
        // 设置测试环境变量
        env::set_var("TEST_WIN_VAR", "/win/value");

        let result = expand("%TEST_WIN_VAR%/subdir");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), "/win/value/subdir");

        env::remove_var("TEST_WIN_VAR");
    }

    #[test]
    fn test_expand_windows_env_var_missing() {
        // 确保环境变量不存在
        env::remove_var("NONEXISTENT_WIN_VAR_12345");

        let result = expand("%NONEXISTENT_WIN_VAR_12345%/path");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PathExpandError::EnvVar(_)));
    }

    #[test]
    fn test_expand_combined_env_vars() {
        // 设置多个测试环境变量
        env::set_var("TEST_BASE", "/base");
        env::set_var("TEST_SUB", "subdir");

        let result = expand("$TEST_BASE/$TEST_SUB/file");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), "/base/subdir/file");

        env::remove_var("TEST_BASE");
        env::remove_var("TEST_SUB");
    }

    #[test]
    fn test_expand_empty_string() {
        let result = expand("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), "");
    }

    #[test]
    fn test_expand_no_expansion_needed() {
        // 不需要展开的普通路径
        let path = "simple/path/without/special/chars";
        let result = expand(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string_lossy(), path);
    }

    #[test]
    fn test_expand_windows_env_vars_internal() {
        // 测试内部函数
        env::set_var("TEST_INTERNAL", "internal_value");

        let result = expand_windows_env_vars("%TEST_INTERNAL%");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "internal_value");

        env::remove_var("TEST_INTERNAL");
    }

    #[test]
    fn test_expand_windows_env_vars_multiple() {
        // 测试多个 Windows 环境变量
        env::set_var("TEST_A", "aaa");
        env::set_var("TEST_B", "bbb");

        let result = expand_windows_env_vars("%TEST_A%/%TEST_B%");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "aaa/bbb");

        env::remove_var("TEST_A");
        env::remove_var("TEST_B");
    }

    #[test]
    fn test_expand_windows_env_vars_no_percent() {
        // 不包含 % 的字符串
        let result = expand_windows_env_vars("normal/path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "normal/path");
    }
}
