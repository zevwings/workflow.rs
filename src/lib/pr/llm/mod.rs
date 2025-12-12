//! Pull Request LLM 服务
//!
//! 本模块提供了使用 LLM 生成 Pull Request 内容的功能。
//! 根据不同的业务场景，提供不同的生成方法。
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use workflow::pr::llm::{CreateGenerator, RewordGenerator, SummaryGenerator, FileSummaryGenerator};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 示例 diff 数据
//! let git_diff = "diff --git a/src/main.rs b/src/main.rs\n@@ -1,3 +1,5 @@\n fn main() {\n+    println!(\"Hello\");\n }";
//! let pr_diff = "diff --git a/src/main.rs b/src/main.rs\n@@ -1,3 +1,5 @@\n fn main() {\n+    println!(\"Hello\");\n }";
//! let file_diff = "diff --git a/src/main.rs b/src/main.rs\n@@ -1,3 +1,5 @@\n fn main() {\n+    println!(\"Hello\");\n }";
//!
//! // 生成 PR 内容（用于创建 PR）
//! let content = CreateGenerator::generate(
//!     "Add user authentication",
//!     Some(vec!["feature-login".to_string()]),
//!     Some(git_diff.to_string()),
//! )?;
//!
//! // 基于 PR diff 生成标题和描述（用于更新 PR）
//! let reword = RewordGenerator::reword_from_diff(pr_diff, Some("Old title"))?;
//!
//! // 生成 PR 总结文档
//! let summary = SummaryGenerator::summarize_pr("PR Title", pr_diff)?;
//!
//! // 生成单个文件的修改总结
//! let file_summary = FileSummaryGenerator::summarize_file_change("src/main.rs", file_diff)?;
//! # Ok(())
//! # }
//! ```

mod create;
mod file_summary;
mod helpers;
mod reword;
mod summary;

// 重新导出公共 API
pub use create::{CreateGenerator, PullRequestContent};
pub use file_summary::FileSummaryGenerator;
pub use helpers::extract_json_from_markdown;
pub use reword::{PullRequestReword, RewordGenerator};
pub use summary::{PullRequestSummary, SummaryGenerator};
