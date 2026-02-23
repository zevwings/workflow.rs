//! SSH 服务实现
//!
//! 通过系统 ssh-keygen、ssh-add 命令实现 SshService trait。

mod service;

pub(crate) use service::SshServiceImpl;
