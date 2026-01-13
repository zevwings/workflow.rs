//! 选项渲染和选择处理的 trait 定义

use crate::base::interactive::style::Theme;

/// 选项渲染器 trait
///
/// 定义如何渲染单个选项，允许 select 和 multiselect 有不同的渲染方式
pub trait OptionRenderer {
    /// 渲染单个选项
    ///
    /// # 参数
    /// - `index`: 选项索引
    /// - `option_text`: 选项文本
    /// - `is_current`: 是否是当前光标位置
    /// - `theme`: 主题样式
    ///
    /// # 返回
    /// 渲染后的行文本（不包含换行符）
    fn render_option(
        &self,
        index: usize,
        option_text: &str,
        is_current: bool,
        theme: &Theme,
    ) -> String;
}
