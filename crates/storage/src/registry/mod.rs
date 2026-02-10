//! Registry Module
//!
//! 服务注册模块，按依赖顺序组织各个服务的注册逻辑。

use registry::RegistryError;

mod config;
mod git;
mod github;
mod jira;
mod verify;

/// 注册所有 storage 服务
pub fn register_storage() -> Result<(), RegistryError> {
    // 按依赖顺序注册服务
    config::register_config()?;
    git::register_git()?;
    jira::register_jira()?;
    github::register_github()?;
    verify::register_verify()?;

    Ok(())
}
