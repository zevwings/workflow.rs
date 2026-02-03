//! 终端输出协调模块
//!
//! 提供终端输出的协调机制，确保 spinner/progress 与日志输出不冲突。

mod coordinator;
mod layer;

pub use coordinator::{
    register_spinner_handlers, suspend_spinner, resume_spinner, SpinnerHandlers,
};
pub use layer::{SpinnerAwareLayer, SpinnerAwareMakeWriter};
