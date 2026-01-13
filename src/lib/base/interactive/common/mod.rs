//! 交互式提示公共模块
//!
//! 提供 select 和 multiselect 等模块共享的渲染和交互逻辑

mod renderer;
mod traits;

pub use renderer::OptionListRenderer;
pub use traits::OptionRenderer;
