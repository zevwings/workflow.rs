//! 并发任务执行器模块
//!
//! 提供通用的并发任务执行功能，支持并行执行多个任务并收集结果。
//!
//! ## 使用示例
//!
//! ```rust
//! use workflow::base::concurrent::{ConcurrentExecutor, TaskResult};
//! use anyhow::Result;
//!
//! let executor = ConcurrentExecutor::new(5); // 最大并发数 5
//!
//! let tasks: Vec<(String, Box<dyn Fn() -> Result<String, String> + Send + Sync>)> = vec![
//!     ("task1".to_string(), Box::new(|| -> Result<String, String> {
//!         Ok("result1".to_string())
//!     })),
//!     ("task2".to_string(), Box::new(|| -> Result<String, String> {
//!         Ok("result2".to_string())
//!     })),
//! ];
//!
//! let results = executor.execute(tasks)?;
//! for (name, result) in results {
//!     match result {
//!         TaskResult::Success(value) => println!("{}: {}", name, value),
//!         TaskResult::Failure(err) => println!("{}: error - {}", name, err),
//!     }
//! }
//! ```

mod executor;

pub use executor::{ConcurrentExecutor, TaskResult};
