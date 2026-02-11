//! 交互式提示模块
//!
//! 本模块提供完全基于 Rust 的交互式提示库，遵循 Rust 最佳实践。
//! 提供输入提示、确认提示、选择提示、表单、消息输出、加载指示器和表格显示等功能。
//!
//! ## 设计原则
//!
//! 1. **类型安全**：充分利用 Rust 的类型系统，编译期保证正确性
//! 2. **零成本抽象**：使用 Trait 和泛型，运行时无额外开销
//! 3. **所有权清晰**：明确的所有权语义，避免不必要的克隆
//! 4. **错误处理**：使用 `Result<T>` 进行错误处理，提供清晰的错误信息
//! 5. **可组合性**：模块化设计，易于组合和扩展
//! 6. **性能优先**：避免不必要的分配，使用零拷贝技术
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use prompt::*;
//! use prompt::{input, confirm};
//!
//! fn main() -> Result<()> {
//!     // 输入提示（使用宏）
//!     let name = input!("请输入您的姓名")
//!         .default("John Doe")
//!         .prompt()?;
//!
//!     // 确认提示（使用宏）
//!     let confirmed = confirm!("是否继续？")
//!         .default(true)
//!         .prompt()?;
//!
//!     // 消息输出
//!     let msg = Message::global();
//!     msg.info("这是一条信息")?;
//!     msg.success("操作成功")?;
//!
//!     Ok(())
//! }
//! ```

pub(crate) mod backend; // 终端后端抽象（内部使用）
pub(crate) mod dialog; // 交互式对话框（input, confirm, select, multiselect）
pub(crate) mod error; // 错误类型定义
pub(crate) mod form; // 表单构建器（FormBuilder, FormResult）
pub(crate) mod output; // 输出功能（message, table, spinner）
pub(crate) mod style {
    pub(crate) mod theme;
}

// 重新导出公共 API（仅导出外部使用的 API，内部实现不导出）
// 注意：confirm, input, select, multiselect 现在只通过宏使用（confirm!, input!, select!, multiselect!）
pub use dialog::{
    validators, ConfirmBuilder, FuzzyFilter, InputBuilder, MultiSelectBuilder, SelectBuilder,
    Validator,
};
pub use error::{is_user_cancelled, PromptError, Result};
pub use form::{
    form, Condition, ConfirmFormField, FormBuilder, FormExecutor, FormModel, FormResult,
    GroupConfig, InputFormField, MultiSelectFormField, NestedFormField, PasswordFormField,
    SelectFormField,
};
pub use output::{
    progress_bar, spinner, table, terminal_state, Alignment, Message, MessageRef, Progress,
    ProgressBar, ProgressBarBuilder, Spinner, SpinnerBuilder, TableBuilder, TableStyle, Tabled,
};
pub use style::theme::{get_theme, set_theme, Style, Theme};
