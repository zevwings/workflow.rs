#![allow(
    clippy::overly_complex_bool_expr,
    clippy::module_inception,
    clippy::collapsible_match,
    clippy::to_string_trait_impl,
    clippy::unnecessary_unwrap,
    clippy::bool_comparison
)]

//! 端到端集成测试入口
//!
//! 包含完整的用户工作流测试。
//! 这些测试通常运行较慢，需要 Mock 服务器、Git 仓库等。
//!
//! ## 测试类型
//!
//! - **端到端集成测试**：测试多个模块协作的完整工作流
//! - **运行速度**：较慢（分钟级）
//! - **依赖**：需要 Mock 服务器、Git 仓库等完整环境
//!
//! ## 运行方式
//!
//! ```bash
//! # 运行所有端到端集成测试
//! cargo test --test e2e_test
//!
//! # 运行特定的端到端测试
//! cargo test --test e2e_test e2e::end_to_end
//! ```

// 引入端到端集成测试模块
mod e2e;

// 共享测试工具
mod common;
