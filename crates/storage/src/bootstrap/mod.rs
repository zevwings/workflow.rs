//! Bootstrap Module
//!
//! 服务引导模块，按依赖顺序组织各个服务的注册逻辑。

use di::InjectionError;

mod config;
mod git;
mod github;
mod jira;

/// 注册所有 storage 服务
pub fn register_storage() -> Result<(), InjectionError> {
    // 按依赖顺序注册服务
    config::register_config()?;
    git::register_git()?;
    jira::register_jira()?;
    github::register_github()?;

    Ok(())
}
