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
//! use workflow::base::interactive::*;
//!
//! fn main() -> Result<()> {
//!     // 输入提示
//!     let name = input("请输入您的姓名")
//!         .default("John Doe")
//!         .prompt()?;
//!
//!     // 确认提示
//!     let confirmed = confirm("是否继续？")
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

mod dialog; // 交互式对话框（input, confirm, select, multiselect, form）
mod output; // 输出功能（message, table, spinner）
mod style;

// 重新导出公共 API（仅导出外部使用的 API，内部实现不导出）
pub use dialog::{
    confirm, form, input, multiselect, select, validators, Condition, ConfirmBuilder,
    ConfirmFormField, FormBuilder, FormExecutor, FormResult, InputBuilder, InputFormField,
    MultiSelectBuilder, MultiSelectFormField, NestedFormField, PasswordFormField, PromptError,
    Result, SelectBuilder, SelectFormField, Validator,
};
pub use output::{
    progress_bar, spinner, table, Alignment, Message, MessageRef, Progress, ProgressBar,
    ProgressBarBuilder, Spinner, SpinnerBuilder, TableBuilder, TableStyle,
};
