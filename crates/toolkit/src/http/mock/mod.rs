//! HTTP 模块测试 Mock 工具
//!
//! 提供 HTTP 模块单元测试中使用的 Mock 服务器和辅助工具。
//!
//! ⚠️ **注意**：此模块仅在测试时编译，不会被打包到正式代码中。
//! 由 `http/mod.rs` 中的 `#[cfg(test)] mod mock;` 控制。

#[cfg(test)]
pub mod server;

#[cfg(test)]
pub use server::HttpMockServer;
