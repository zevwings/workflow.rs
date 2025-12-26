//! Mock 相关工具
//!
//! 提供 HTTP Mock 测试的完整功能，包括 MockServer、模板系统、验证器和场景预设库。

pub mod scenarios;
pub mod server;
pub mod templates;
pub mod validators;

// 重新导出常用类型，保持向后兼容
pub use server::{setup_mock_server, MockServer};
