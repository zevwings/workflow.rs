//! 路径展开工具
//!
//! 提供路径字符串展开功能，支持 Unix 风格的 `~` 展开和 Windows 风格的环境变量展开。
//!
//! 使用 `shellexpand` 处理 `~` 展开和 Unix 风格的环境变量（`$VAR`、`${VAR}`），
//! 同时保留自定义逻辑处理 Windows 风格的 `%VAR%` 环境变量。

use crate::paths::PathError;
use std::env;
use std::path::PathBuf;

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
pub fn expand(path_str: &str) -> Result<PathBuf, PathError> {
    // 先处理 Windows 风格的环境变量展开 %VAR%
    // 因为 shellexpand 不支持这种格式
    let expanded = if path_str.contains('%') {
        expand_windows_env_vars(path_str)?
    } else {
        path_str.to_string()
    };

    // 使用 shellexpand 处理 ~ 展开和 Unix 风格的环境变量
    let result = shellexpand::full(&expanded)
        .map_err(|e| PathError::Expansion(format!("Failed to expand path: {}", e)))?;

    Ok(PathBuf::from(result.as_ref()))
}

/// 展开 Windows 风格的环境变量 `%VAR%`
fn expand_windows_env_vars(path_str: &str) -> Result<String, PathError> {
    let mut result = String::new();
    let mut chars = path_str.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut var_name = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '%' {
                    chars.next();
                    break;
                }
                var_name.push(chars.next().unwrap());
            }

            if !var_name.is_empty() {
                let var_value = env::var(&var_name).map_err(PathError::EnvVar)?;
                result.push_str(&var_value);
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}
