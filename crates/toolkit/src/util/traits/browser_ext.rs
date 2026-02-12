//! 浏览器扩展 trait
//!
//! 为字符串类型提供浏览器操作相关的扩展方法。

use thiserror::Error;
use url::Url;

/// 浏览器操作错误
#[derive(Debug, Error)]
pub enum BrowserError {
    /// 无效的 URL 格式
    #[error("Invalid URL format '{url}': {reason}")]
    InvalidUrl { url: String, reason: String },

    /// 打开浏览器失败
    #[error("Failed to open URL '{url}': {reason}")]
    OpenFailed { url: String, reason: String },

    /// 打开浏览器失败（指定浏览器）
    #[error("Failed to open URL '{url}' with browser '{browser}': {reason}")]
    OpenWithBrowserFailed {
        url: String,
        browser: String,
        reason: String,
    },
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
        Url::parse(self).map_err(|e| BrowserError::InvalidUrl {
            url: self.to_string(),
            reason: e.to_string(),
        })?;

        // 使用系统默认浏览器打开 URL
        open::that(self).map_err(|e| BrowserError::OpenFailed {
            url: self.to_string(),
            reason: e.to_string(),
        })?;
        Ok(())
    }

    fn open_in_browser_with(&self, browser: Browser) -> Result<(), BrowserError> {
        // 验证 URL 格式
        Url::parse(self).map_err(|e| BrowserError::InvalidUrl {
            url: self.to_string(),
            reason: e.to_string(),
        })?;

        // 获取平台特定的浏览器名称
        let browser_name = browser.as_str();

        // 使用指定的浏览器打开 URL
        open::with(self, browser_name).map_err(|e| BrowserError::OpenWithBrowserFailed {
            url: self.to_string(),
            browser: browser_name.to_string(),
            reason: e.to_string(),
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
    use crate::util::traits::{Browser, BrowserError, BrowserExt};

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
        // 注意：ftp://example.com 是有效的 URL 格式，Url::parse() 会接受它
        let invalid_urls = vec![
            ("not a url", "无效格式"),
            ("example.com", "缺少协议"),
            ("://example.com", "无效协议"),
            ("", "空字符串"),
            ("   ", "空白字符"),
        ];

        for (url, description) in invalid_urls {
            let result = url.open_in_browser();
            assert!(
                matches!(result, Err(BrowserError::InvalidUrl { .. })),
                "open_in_browser() should reject invalid URL: {} ({})",
                url,
                description
            );
        }
    }
}
