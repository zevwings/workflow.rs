//! 路径显示格式化

use std::path::Path;

/// 路径显示格式化 trait
///
/// 为 `Path` 类型提供显示格式化功能。
///
/// # 示例
///
/// ```
/// use workflow::util::format::PathDisplay;
/// use std::path::Path;
///
/// let path = Path::new("/home/user/project/src/main.rs");
/// let formatted = path.to_display_string();
/// ```
pub trait PathDisplay {
    /// 将路径格式化为适合显示的字符串，优先显示相对路径。
    ///
    /// # 返回值
    ///
    /// 格式化后的路径字符串
    fn to_display_string(&self) -> String;
}

impl PathDisplay for Path {
    fn to_display_string(&self) -> String {
        if let Ok(relative) = self.strip_prefix(std::env::current_dir().unwrap_or_default()) {
            relative.display().to_string()
        } else {
            self.display().to_string()
        }
    }
}
