//! 项目常量模块
//!
//! 统一管理项目中使用的字符串常量、错误消息、API URL 等，
//! 提升代码一致性、维护性和可读性。

pub mod errors;
pub mod git;
pub mod messages;
pub mod network;
pub mod validation;

// 重新导出常用常量模块
pub use errors::*;
pub use git::*;
pub use messages::*;
pub use validation::*;
