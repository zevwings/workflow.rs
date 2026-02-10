//! 验证结果格式化器 trait
//!
//! 定义验证结果到展示格式的转换接口。

/// 验证结果格式化器
///
/// 将验证结果转换为表格行并格式化输出到控制台。
pub trait VerificationResultFormatter {
    /// 格式化并显示验证结果
    fn format(&self);
}

impl VerificationResultFormatter for Box<dyn VerificationResultFormatter> {
    fn format(&self) {
        (**self).format();
    }
}
