//! 配置模块
//!
//! 提供配置适配器，实现各种配置提供者 trait。
//! 仅在本 crate 内通过 registry 使用，子模块不对外暴露。

mod global;
mod repo;

pub(crate) use global::{GlobalConfigRepositoryImpl, VerificationServiceImpl};
pub(crate) use repo::RepoConfigRepositoryImpl;
