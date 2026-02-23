//! 路径管理
//!
//! 统一管理所有路径信息，包括：
//! - 配置文件路径（存储在 `~/.workflow/config/` 目录下）
//! - 安装路径（二进制文件和补全脚本的安装路径和名称）
//! - Completion 目录路径
//!
//! ## 使用方式
//!
//! 路径展开等 API 通过 `toolkit::expand` 访问：
//! ```rust
//! use toolkit::expand;
//! let path = expand("~/config").unwrap();
//! ```

mod expand;

// ==================== 路径工具方法 ====================

/// 展开路径字符串
///
/// 支持的路径格式：
/// - Unix: `~` 和 `~/path` - 展开为用户主目录
/// - Unix: `$VAR` 和 `${VAR}` - 展开环境变量
/// - Windows: `%VAR%` 和 `%VAR%\path` - 展开环境变量
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
pub use expand::{expand, PathExpandError};
