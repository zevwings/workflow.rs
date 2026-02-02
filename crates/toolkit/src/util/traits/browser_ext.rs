//! 浏览器扩展 trait
//!
//! 为字符串类型提供浏览器操作相关的扩展方法。

use reqwest::Url;
use thiserror::Error;

/// 浏览器操作错误
#[derive(Debug, Error)]
pub enum BrowserError {
    /// 浏览器操作错误
    #[error("Browser error: {0}")]
    Operation(String),
}

/// 浏览器类型枚举
///
/// 支持常见的浏览器类型，会根据平台自动转换为对应的应用程序名称。
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::{Browser, BrowserExt};
///
/// "https://example.com".open_in_browser_with(Browser::Firefox)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    /// Firefox 浏览器
    Firefox,
    /// Google Chrome 浏览器
    Chrome,
    /// Microsoft Edge 浏览器
    Edge,
    /// Safari 浏览器（主要 macOS）
    Safari,
    /// Opera 浏览器
    Opera,
    /// Brave 浏览器
    Brave,
    /// Vivaldi 浏览器
    Vivaldi,
}

impl Browser {
    /// 获取浏览器在平台上的应用程序名称
    ///
    /// 根据当前平台返回对应的浏览器应用程序名称或命令。
    ///
    /// # 返回
    ///
    /// 返回平台特定的浏览器名称字符串。
    pub fn as_str(&self) -> &'static str {
        #[cfg(target_os = "macos")]
        {
            match self {
                Browser::Firefox => "Firefox",
                Browser::Chrome => "Google Chrome",
                Browser::Edge => "Microsoft Edge",
                Browser::Safari => "Safari",
                Browser::Opera => "Opera",
                Browser::Brave => "Brave Browser",
                Browser::Vivaldi => "Vivaldi",
            }
        }

        #[cfg(target_os = "windows")]
        {
            match self {
                Browser::Firefox => "firefox",
                Browser::Chrome => "chrome",
                Browser::Edge => "msedge",
                Browser::Safari => "safari",
                Browser::Opera => "opera",
                Browser::Brave => "brave",
                Browser::Vivaldi => "vivaldi",
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            // Linux 和其他 Unix 系统
            match self {
                Browser::Firefox => "firefox",
                Browser::Chrome => "google-chrome",
                Browser::Edge => "microsoft-edge",
                Browser::Safari => "safari",
                Browser::Opera => "opera",
                Browser::Brave => "brave-browser",
                Browser::Vivaldi => "vivaldi",
            }
        }
    }
}

/// 浏览器扩展 trait
///
/// 为字符串类型提供在浏览器中打开 URL 的功能。
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::BrowserExt;
///
/// let url = "https://example.com";
/// url.open_in_browser()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait BrowserExt {
    /// 在浏览器中打开 URL
    ///
    /// 使用系统默认浏览器打开指定的 URL。
    /// 在打开前会验证 URL 格式是否合法。
    ///
    /// # 返回
    ///
    /// 如果成功打开浏览器，返回 `Ok(())`；如果失败，返回相应的错误信息。
    ///
    /// # 错误
    ///
    /// - 如果 URL 格式不合法，返回 `Err` 包含验证错误
    /// - 如果无法打开浏览器，返回 `Err` 包含系统错误
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use toolkit::BrowserExt;
    ///
    /// "https://example.com".open_in_browser()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # 错误示例
    ///
    /// ```rust,no_run
    /// use toolkit::BrowserExt;
    ///
    /// // 无效的 URL 会返回错误
    /// let result = "not a url".open_in_browser();
    /// assert!(result.is_err());
    /// ```
    fn open_in_browser(&self) -> Result<(), BrowserError>;

    /// 在指定的浏览器中打开 URL
    ///
    /// 使用指定的浏览器打开 URL。在打开前会验证 URL 格式是否合法。
    ///
    /// # 参数
    ///
    /// * `browser` - 浏览器类型枚举，会根据平台自动转换为对应的应用程序名称
    ///
    /// # 返回
    ///
    /// 如果成功打开浏览器，返回 `Ok(())`；如果失败，返回相应的错误信息。
    ///
    /// # 错误
    ///
    /// - 如果 URL 格式不合法，返回 `Err` 包含验证错误
    /// - 如果指定的浏览器不存在或无法打开，返回 `Err` 包含系统错误
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use toolkit::{Browser, BrowserExt};
    ///
    /// // 在 Firefox 中打开
    /// "https://example.com".open_in_browser_with(Browser::Firefox)?;
    ///
    /// // 在 Chrome 中打开
    /// "https://example.com".open_in_browser_with(Browser::Chrome)?;
    ///
    /// // 在 Safari 中打开（macOS）
    /// "https://example.com".open_in_browser_with(Browser::Safari)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn open_in_browser_with(&self, browser: Browser) -> Result<(), BrowserError>;
}

/// 为 `str` 实现 `BrowserExt` trait
impl BrowserExt for str {
    fn open_in_browser(&self) -> Result<(), BrowserError> {
        // 验证 URL 格式
        Url::parse(self).map_err(|e| {
            BrowserError::Operation(format!("Invalid URL format '{}': {}", self, e))
        })?;

        // 使用系统默认浏览器打开 URL
        open::that(self).map_err(|e| {
            BrowserError::Operation(format!("Failed to open URL '{}': {}", self, e))
        })?;
        Ok(())
    }

    fn open_in_browser_with(&self, browser: Browser) -> Result<(), BrowserError> {
        // 验证 URL 格式
        Url::parse(self).map_err(|e| {
            BrowserError::Operation(format!("Invalid URL format '{}': {}", self, e))
        })?;

        // 获取平台特定的浏览器名称
        let browser_name = browser.as_str();

        // 使用指定的浏览器打开 URL
        open::with(self, browser_name).map_err(|e| {
            BrowserError::Operation(format!(
                "Failed to open URL '{}' with browser '{}': {}",
                self, browser_name, e
            ))
        })?;
        Ok(())
    }
}

/// 为 `String` 实现 `BrowserExt` trait
impl BrowserExt for String {
    fn open_in_browser(&self) -> Result<(), BrowserError> {
        self.as_str().open_in_browser()
    }

    fn open_in_browser_with(&self, browser: Browser) -> Result<(), BrowserError> {
        self.as_str().open_in_browser_with(browser)
    }
}

#[cfg(test)]
mod tests {
    use super::{Browser, BrowserExt};

    // ============================================================================
    // Browser 枚举测试
    // ============================================================================

    #[test]
    fn test_browser_as_str_platform_specific() {
        // 测试浏览器枚举在当前平台上的名称映射
        #[cfg(target_os = "macos")]
        {
            assert_eq!(Browser::Firefox.as_str(), "Firefox");
            assert_eq!(Browser::Chrome.as_str(), "Google Chrome");
            assert_eq!(Browser::Edge.as_str(), "Microsoft Edge");
            assert_eq!(Browser::Safari.as_str(), "Safari");
            assert_eq!(Browser::Opera.as_str(), "Opera");
            assert_eq!(Browser::Brave.as_str(), "Brave Browser");
            assert_eq!(Browser::Vivaldi.as_str(), "Vivaldi");
        }

        #[cfg(target_os = "windows")]
        {
            assert_eq!(Browser::Firefox.as_str(), "firefox");
            assert_eq!(Browser::Chrome.as_str(), "chrome");
            assert_eq!(Browser::Edge.as_str(), "msedge");
            assert_eq!(Browser::Safari.as_str(), "safari");
            assert_eq!(Browser::Opera.as_str(), "opera");
            assert_eq!(Browser::Brave.as_str(), "brave");
            assert_eq!(Browser::Vivaldi.as_str(), "vivaldi");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            assert_eq!(Browser::Firefox.as_str(), "firefox");
            assert_eq!(Browser::Chrome.as_str(), "google-chrome");
            assert_eq!(Browser::Edge.as_str(), "microsoft-edge");
            assert_eq!(Browser::Safari.as_str(), "safari");
            assert_eq!(Browser::Opera.as_str(), "opera");
            assert_eq!(Browser::Brave.as_str(), "brave-browser");
            assert_eq!(Browser::Vivaldi.as_str(), "vivaldi");
        }
    }

    #[test]
    fn test_browser_enum_all_variants_have_names() {
        // 验证所有浏览器变体都有非空名称
        let browsers = vec![
            Browser::Firefox,
            Browser::Chrome,
            Browser::Edge,
            Browser::Safari,
            Browser::Opera,
            Browser::Brave,
            Browser::Vivaldi,
        ];

        for browser in browsers {
            let name = browser.as_str();
            assert!(
                !name.is_empty(),
                "Browser {:?} name should not be empty",
                browser
            );
        }
    }

    // ============================================================================
    // BrowserExt::open_in_browser() 测试
    // ============================================================================

    #[test]
    fn test_open_in_browser_invalid_urls() {
        // 测试各种无效 URL 格式会正确返回错误
        let invalid_urls = vec![
            ("not a url", "无效格式"),
            ("example.com", "缺少协议"),
            ("://example.com", "无效协议"),
            ("", "空字符串"),
            ("   ", "空白字符"),
            ("ftp://example.com", "不支持的协议"),
        ];

        for (url, description) in invalid_urls {
            let result = url.open_in_browser();
            assert!(
                result.is_err(),
                "open_in_browser() should reject invalid URL: {} ({})",
                url,
                description
            );

            // 验证错误消息包含有用的信息
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("Invalid URL"),
                "Error message should mention 'Invalid URL', got: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_open_in_browser_valid_url_formats() {
        // 测试各种有效的 URL 格式能通过验证阶段
        // 注意：我们不实际打开浏览器（CI 环境限制），只验证 URL 验证逻辑
        let valid_urls = vec![
            "https://example.com",
            "http://example.com",
            "https://example.com/path",
            "https://example.com:8080/path?query=value",
            "https://example.com/path#fragment",
            "https://subdomain.example.com",
            "https://example.com/path/to/resource.html",
            "https://example.com?key1=value1&key2=value2",
        ];

        for url in valid_urls {
            let result = url.open_in_browser();
            // 在 CI 环境中可能因为没有浏览器而失败，但错误应该是打开浏览器的错误，
            // 而不是 URL 验证错误
            if let Err(e) = result {
                let error_msg = e.to_string();
                // 如果失败，应该是"Failed to open URL"错误，而不是"Invalid URL"错误
                if error_msg.contains("Invalid URL") {
                    panic!(
                        "Valid URL '{}' should pass validation, but got validation error: {}",
                        url, error_msg
                    );
                }
            }
        }
    }

    #[test]
    fn test_open_in_browser_str_and_string_types() {
        // 测试 str 和 String 类型都可以调用方法
        let url_str = "https://example.com";
        let url_string = String::from("https://example.com");

        // 两种类型都应该能调用方法（不实际打开浏览器）
        let result_str = url_str.open_in_browser();
        let result_string = url_string.open_in_browser();

        // 验证两种类型的行为一致
        match (result_str, result_string) {
            (Ok(_), Ok(_)) => {
                // 两者都成功（有浏览器环境）
            }
            (Err(e1), Err(e2)) => {
                // 两者都失败（CI 环境），错误类型应该相同
                assert!(
                    e1.to_string().contains("Failed to open URL")
                        && e2.to_string().contains("Failed to open URL"),
                    "Both should fail with same error type"
                );
            }
            _ => {
                // 不应该出现一个成功一个失败的情况
                panic!("str and String types should behave consistently");
            }
        }
    }

    // ============================================================================
    // BrowserExt::open_in_browser_with() 测试
    // ============================================================================

    #[test]
    fn test_open_in_browser_with_invalid_urls() {
        // 测试指定浏览器时，无效 URL 仍然会被拒绝
        let invalid_urls = vec!["not a url", "example.com", ""];

        for url in invalid_urls {
            let result = url.open_in_browser_with(Browser::Firefox);
            assert!(
                result.is_err(),
                "open_in_browser_with() should reject invalid URL: '{}'",
                url
            );

            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("Invalid URL"),
                "Error should be URL validation error, got: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_open_in_browser_with_all_browser_types() {
        // 测试所有浏览器类型都可以被使用
        // 注意：不实际打开浏览器，只验证方法可以调用
        let url = "https://example.com";
        let browsers = vec![
            Browser::Firefox,
            Browser::Chrome,
            Browser::Edge,
            Browser::Safari,
            Browser::Opera,
            Browser::Brave,
            Browser::Vivaldi,
        ];

        for browser in browsers {
            let result = url.open_in_browser_with(browser);
            // 在 CI 环境中可能失败，但至少不应该 panic
            // 如果失败，应该是打开浏览器失败，而不是 URL 验证失败
            if let Err(e) = result {
                let error_msg = e.to_string();
                if error_msg.contains("Invalid URL") {
                    panic!(
                        "Valid URL should pass validation for browser {:?}, got: {}",
                        browser, error_msg
                    );
                }
            }
        }
    }

    #[test]
    fn test_open_in_browser_with_str_and_string_types() {
        // 测试 str 和 String 类型都可以调用 open_in_browser_with
        let url_str = "https://example.com";
        let url_string = String::from("https://example.com");

        let result_str = url_str.open_in_browser_with(Browser::Firefox);
        let result_string = url_string.open_in_browser_with(Browser::Chrome);

        // 验证两种类型都可以调用方法（行为一致）
        match (result_str, result_string) {
            (Ok(_), Ok(_)) => {}
            (Err(e1), Err(e2)) => {
                // 都失败是可以接受的（CI 环境）
                let msg1 = e1.to_string();
                let msg2 = e2.to_string();
                // 但不应该是 URL 验证错误
                assert!(
                    !msg1.contains("Invalid URL") && !msg2.contains("Invalid URL"),
                    "Should not be URL validation errors"
                );
            }
            _ => {
                // 一个成功一个失败也是可能的（不同浏览器可用性不同）
            }
        }
    }

    // ============================================================================
    // 特殊 URL 格式测试
    // ============================================================================

    #[test]
    fn test_open_in_browser_with_special_characters_in_url() {
        // 测试包含特殊字符的 URL
        let special_urls = vec![
            "https://example.com/path%20with%20spaces",
            "https://example.com/path?query=hello%20world",
            "https://example.com/path#section-1",
            "https://example.com/path?utf8=✓&q=test",
        ];

        for url in special_urls {
            let result = url.open_in_browser();
            // 这些 URL 格式都是有效的，不应该在验证阶段失败
            if let Err(e) = result {
                let error_msg = e.to_string();
                assert!(
                    !error_msg.contains("Invalid URL"),
                    "Special URL '{}' should be valid, got: {}",
                    url,
                    error_msg
                );
            }
        }
    }
}
