//! 浏览器操作模块
//!
//! 本模块提供了在浏览器中打开 URL 的功能。

use color_eyre::Result;

/// 浏览器操作模块
///
/// 提供在默认浏览器中打开 URL 的功能。
pub struct Browser;

impl Browser {
    /// 在浏览器中打开 URL
    ///
    /// 使用系统默认浏览器打开指定的 URL。
    ///
    /// # 参数
    ///
    /// * `url` - 要打开的 URL
    ///
    /// # 错误
    ///
    /// 如果无法打开浏览器，返回相应的错误信息。
    pub fn open(url: &str) -> Result<()> {
        open::that(url)?;
        Ok(())
    }
}
