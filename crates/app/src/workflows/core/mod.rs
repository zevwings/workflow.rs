//! 核心抽象和基础设施模块
//!
//! 提供工作流的核心抽象，包括上下文管理、阶段抽象和平台通用逻辑。

pub mod context;
pub mod platform;
pub mod stage;

pub use context::{WorkflowContext, WorkflowMode};
pub use platform::{
    add_account_generic, configure_platform, remove_account_generic, switch_account_generic,
    AccountAction, GlobalConfigAccessor, PlatformAccount, PlatformConfigurator, PlatformSettings,
};
pub use stage::{WorkflowExecutor, WorkflowStage};
