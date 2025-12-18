//! 测试工具模块
//!
//! 提供各种测试辅助工具和容器，包括临时文件管理等。
//!
//! ## 使用方式
//!
//! 在测试文件中，可以通过以下方式使用：
//!
//! ```rust
//! use utils::temp::TempManager;
//!
//! #[test]
//! fn my_test() {
//!     let temp_manager = TempManager::new().unwrap();
//!     // 使用临时目录...
//! }
//! ```

pub mod temp;
