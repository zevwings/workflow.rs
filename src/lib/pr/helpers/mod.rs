//! PR 辅助函数模块
//!
//! 按职责拆分为多个子模块：
//! - `url` - URL 相关函数（提取 PR ID、仓库信息等）
//! - `generation` - 内容生成相关函数（生成 commit 标题、PR body 等）
//! - `resolution` - PR ID 解析相关函数（获取当前分支 PR ID、解析 PR ID 等）

pub mod generation;
pub mod resolution;
pub mod url;

// 统一导出所有公共函数
pub use generation::{generate_commit_title, generate_pull_request_body};
pub use resolution::{get_current_branch_pr_id, resolve_pull_request_id};
pub use url::{extract_github_repo_from_url, extract_pull_request_id_from_url};
