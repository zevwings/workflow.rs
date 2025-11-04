use anyhow::Result;

/// 浏览器操作模块
pub struct Browser;

impl Browser {
    /// 在浏览器中打开 URL
    pub fn open(url: &str) -> Result<()> {
        open::that(url)?;
        Ok(())
    }
}
