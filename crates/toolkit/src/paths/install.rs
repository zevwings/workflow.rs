//! 安装路径管理
//!
//! 提供二进制文件和补全脚本的安装路径相关功能。

use std::path::PathBuf;

use crate::paths::base::local_base_dir;
use crate::paths::PathError;
use crate::util::fs::DirectoryWalker;

/// 获取所有命令名称
///
/// 返回所有 Workflow CLI 命令的名称列表，这些名称同时用于：
/// - 二进制文件名（workflow）
/// - 补全脚本命令名（用于生成补全脚本）
///
/// # 返回
///
/// 返回包含所有命令名称的静态字符串切片数组。
///
/// # 示例
///
/// ```
/// use toolkit::paths::Paths;
///
/// let names = Paths::command_names();
/// assert_eq!(names, ["workflow"]);
/// ```
pub fn command_names() -> &'static [&'static str] {
    &["workflow"]
}

/// 获取二进制文件安装目录
///
/// 返回二进制文件安装的系统目录路径。
///
/// # 返回
///
/// 返回安装目录路径的字符串。
///
/// # 示例
///
/// ```
/// use toolkit::paths::Paths;
///
/// let dir = Paths::binary_install_dir();
/// // Unix: "/usr/local/bin"
/// // Windows: "%LOCALAPPDATA%\\Programs\\workflow\\bin"
/// ```
pub fn binary_install_dir() -> String {
    if cfg!(target_os = "windows") {
        // Windows: 使用 dirs::data_local_dir() 获取 %LOCALAPPDATA%
        dirs::data_local_dir()
            .map(|d| d.join("Programs").join("workflow").join("bin"))
            .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("bin")))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "C:\\Users\\User\\Programs\\workflow\\bin".to_string())
    } else {
        // Unix-like: 使用 /usr/local/bin
        "/usr/local/bin".to_string()
    }
}

/// 获取所有二进制文件的完整路径
///
/// 基于 `command_names()` 和 `binary_install_dir()` 构建完整路径。
///
/// # 返回
///
/// 返回包含所有二进制文件完整路径的字符串向量。
///
/// # 示例
///
/// ```
/// use toolkit::paths::Paths;
///
/// let paths = Paths::binary_paths();
/// assert_eq!(paths, vec![
///     "/usr/local/bin/workflow".to_string(),
/// ]);
/// ```
pub fn binary_paths() -> Vec<String> {
    let install_dir = binary_install_dir();
    let install_path = PathBuf::from(&install_dir);
    command_names()
        .iter()
        .map(|name| {
            let binary_name = binary_name(name);
            install_path
                .join(&binary_name)
                .to_string_lossy()
                .to_string()
        })
        .collect()
}

/// 获取平台特定的二进制文件名
///
/// 在 Windows 上添加 .exe 扩展名，其他平台保持不变。
///
/// # 参数
///
/// * `name` - 二进制文件的基础名称（不含扩展名）
///
/// # 返回
///
/// 返回平台特定的二进制文件名。
///
/// # 示例
///
/// ```
/// use toolkit::paths::Paths;
///
/// let name = Paths::binary_name("workflow");
/// // Windows: "workflow.exe"
/// // Unix: "workflow"
/// ```
pub fn binary_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

/// 获取补全脚本目录路径（强制本地）
///
/// 返回 `~/.workflow/completions/`（总是本地路径）。
/// Shell 补全脚本是本地安装的，不需要同步。
///
/// # 返回
///
/// 返回补全脚本目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法获取本地目录，返回相应的错误信息。
pub fn completion_dir() -> Result<PathBuf, PathError> {
    // 确保使用本地路径
    let completion_dir = local_base_dir()?.join("completions");

    // 确保目录存在
    _ = DirectoryWalker::new(&completion_dir).ensure_exists();

    Ok(completion_dir)
}
