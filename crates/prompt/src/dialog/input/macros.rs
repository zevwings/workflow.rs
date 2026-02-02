//! 输入提示宏
//!
//! 提供格式化字符串的便捷方式，智能判断是否需要格式化：
//! - 简单字符串字面量：直接传递，不调用 `format!()`
//! - 格式化字符串：使用 `format!()` 进行格式化
//! - 变量或表达式：直接传递，不调用 `format!()`
//!
//! # Examples
//!
//! ```rust,no_run
//! use toolkit::input;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 简单字符串（直接传递，不格式化）
//! let name = input!("Enter your name")
//!     .default("John Doe")
//!     .prompt()?;
//!
//! // 格式化字符串（使用 format!）
//! let value = input!("Enter {} name:", "branch")
//!     .default("main")
//!     .prompt()?;
//!
//! // 变量（直接传递，不格式化）
//! let prompt = "Enter value:";
//! let value = input!(prompt)
//!     .prompt()?;
//! # Ok(())
//! # }
//! ```
#[macro_export]
macro_rules! input {
    // 格式化字符串：input!("Message {}", var) 或 input!("Message {}", var1, var2)
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::InputBuilder::new(format!($fmt, $($arg),+))
    };
    // 简单字符串字面量：input!("Message") - 直接传递，不格式化
    ($msg:literal) => {
        $crate::InputBuilder::new($msg)
    };
    // 变量或其他表达式：input!(var) - 直接传递，不格式化
    ($expr:expr) => {
        $crate::InputBuilder::new($expr)
    };
}
