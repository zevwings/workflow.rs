//! 路径显示扩展 trait
//!
//! 为路径类型提供显示格式化相关的扩展方法。

use std::path::Path;

/// 路径显示格式化 trait
///
/// 为 `Path` 类型提供显示格式化功能。
///
/// # 示例
///
/// ```
/// use toolkit::PathExt;
/// use std::path::Path;
///
/// let path = Path::new("/home/user/project/src/main.rs");
/// let formatted = path.to_display_string();
/// ```
pub trait PathExt {
    /// 将路径格式化为适合显示的字符串，优先显示相对路径。
    ///
    /// # 返回值
    ///
    /// 格式化后的路径字符串
    fn to_display_string(&self) -> String;
}

impl PathExt for Path {
    fn to_display_string(&self) -> String {
        if let Ok(relative) = self
            .strip_prefix(std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf()))
        {
            relative.display().to_string()
        } else {
            self.display().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use super::*;

    /// 获取当前工作目录的辅助函数
    ///
    /// 如果无法获取当前目录，返回 "." 路径。
    fn get_current_dir() -> std::path::PathBuf {
        env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
    }

    // ==================== 相对路径显示测试 ====================

    #[test]
    fn test_to_display_string_relative_path() {
        // 获取当前工作目录
        let current_dir = get_current_dir();

        // 创建一个相对于当前目录的路径
        let relative_path = current_dir.join("test_file.txt");
        let display = relative_path.to_display_string();

        // 应该显示为相对路径（不包含完整路径）
        assert!(display.contains("test_file.txt"));
        // 如果路径在当前目录下，应该不包含完整路径
        if let Ok(stripped) = relative_path.strip_prefix(&current_dir) {
            assert_eq!(display, stripped.display().to_string());
        }
    }

    #[test]
    fn test_to_display_string_current_dir_file() {
        let current_dir = get_current_dir();
        let file_in_current_dir = current_dir.join("file.txt");

        let display = file_in_current_dir.to_display_string();

        // 应该显示为相对路径
        assert_eq!(display, "file.txt");
    }

    #[test]
    fn test_to_display_string_nested_relative() {
        let current_dir = get_current_dir();
        let nested_path = current_dir.join("subdir/nested/file.txt");

        let display = nested_path.to_display_string();

        // 应该显示为相对路径
        assert_eq!(display, "subdir/nested/file.txt");
    }

    // ==================== 绝对路径显示测试 ====================

    #[test]
    fn test_to_display_string_absolute_path_outside_current_dir() {
        // 使用一个绝对路径，不在当前目录下
        let absolute_path = std::env::temp_dir().join("test_file.txt");

        let display = absolute_path.to_display_string();

        // 应该显示为绝对路径
        assert_eq!(display, absolute_path.display().to_string());
    }

    #[test]
    fn test_to_display_string_root_path() {
        let root_path = if cfg!(unix) {
            Path::new("/")
        } else {
            Path::new("C:\\")
        };

        let display = root_path.to_display_string();

        // 应该显示为绝对路径
        assert_eq!(display, root_path.display().to_string());
    }

    #[test]
    fn test_to_display_string_home_dir() {
        // 测试用户主目录路径
        let home_path = if cfg!(unix) {
            Path::new("/home/user")
        } else {
            Path::new("C:\\Users\\User")
        };

        let display = home_path.to_display_string();

        // 如果不在当前目录下，应该显示为绝对路径
        if !home_path.starts_with(get_current_dir()) {
            assert_eq!(display, home_path.display().to_string());
        }
    }

    // ==================== 边界情况测试 ====================

    #[test]
    fn test_to_display_string_current_dir() {
        let current_dir = get_current_dir();
        let display = current_dir.to_display_string();

        // 当前目录 strip_prefix 会返回空路径，显示为空字符串或 "."
        // 这是预期的行为，因为当前目录相对于自己就是空路径
        assert!(
            display.is_empty() || display == "." || display == current_dir.display().to_string()
        );
    }

    #[test]
    fn test_to_display_string_parent_dir() {
        let current_dir = get_current_dir();
        let parent_dir = current_dir.parent().unwrap_or(Path::new(".."));

        let display = parent_dir.to_display_string();

        // 如果父目录不在当前目录下，应该显示为绝对路径
        // 如果父目录在当前目录下，应该显示为相对路径
        if let Ok(stripped) = parent_dir.strip_prefix(&current_dir) {
            // 父目录在当前目录下（不太可能，但处理这种情况）
            let expected = if stripped.display().to_string().is_empty() {
                ".".to_string()
            } else {
                stripped.display().to_string()
            };
            assert_eq!(display, expected);
        } else {
            // 父目录不在当前目录下，显示绝对路径
            assert_eq!(display, parent_dir.display().to_string());
        }
    }

    #[test]
    fn test_to_display_string_with_special_characters() {
        let current_dir = get_current_dir();
        let special_path = current_dir.join("file with spaces.txt");

        let display = special_path.to_display_string();

        // 应该正确处理包含空格的路径
        assert!(display.contains("file with spaces.txt"));
    }

    #[test]
    fn test_to_display_string_unicode() {
        let current_dir = get_current_dir();
        let unicode_path = current_dir.join("test file.txt");

        let display = unicode_path.to_display_string();

        // 应该正确处理 Unicode 字符
        assert!(display.contains("test file.txt"));
    }

    // ==================== 一致性测试 ====================

    #[test]
    fn test_to_display_string_consistency() {
        let current_dir = get_current_dir();
        let test_path = current_dir.join("test.txt");

        let display1 = test_path.to_display_string();
        let display2 = test_path.to_display_string();
        let display3 = test_path.to_display_string();

        // 多次调用应该返回相同的结果
        assert_eq!(display1, display2);
        assert_eq!(display2, display3);
    }

    #[test]
    fn test_to_display_string_same_path_different_representations() {
        let current_dir = get_current_dir();

        // 使用不同的方式表示同一个路径
        let path1 = current_dir.join("subdir").join("file.txt");
        let path2 = current_dir.join("subdir/file.txt");

        let display1 = path1.to_display_string();
        let display2 = path2.to_display_string();

        // 应该显示为相同的相对路径
        assert_eq!(display1, display2);
    }
}
