//! Branch 模块测试
//!
//! 包含 Branch 模块的所有测试文件。
//!
//! 注意：
//! - naming_advanced 和 naming_utils 的单元测试已迁移到 src/lib/branch/naming.rs 中
//! - types 的纯逻辑单元测试已迁移到 src/lib/branch/types.rs 的 `#[cfg(test)]` 模块中
//! - 本目录仅保留需要交互式输入或外部资源的集成测试

pub mod types; // 分支类型集成测试（交互式功能和外部资源）
