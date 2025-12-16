//! 补全完整性测试入口文件
//!
//! 此文件作为独立的测试目标，使 `cargo test --test completeness` 可以运行。
//! 它使用 `include!` 宏直接包含 `tests/completion/completeness.rs` 中的测试代码。

include!("completion/completeness.rs");
