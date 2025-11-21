use anyhow::Result;
use clap_complete::shells::Shell;

/// Shell 检测工具
///
/// 提供 Shell 类型检测功能。
pub struct Detect;

impl Detect {
    /// 检测当前 shell 类型并返回 Shell
    ///
    /// 根据 `SHELL` 环境变量检测当前 shell 类型。
    /// 支持的 shell 类型：zsh、bash、fish、powershell、elvish。
    ///
    /// # 返回
    ///
    /// 返回检测到的 `Shell` 类型。
    ///
    /// # 错误
    ///
    /// 如果 shell 类型不支持，返回相应的错误信息。
    pub fn shell() -> Result<Shell> {
        Shell::from_env()
            .or_else(|| {
                // 如果 from_env() 失败，尝试从 SHELL 环境变量解析
                std::env::var("SHELL").ok().and_then(Shell::from_shell_path)
            })
            .ok_or_else(|| {
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
                anyhow::anyhow!("Unsupported shell: {}", shell)
            })
    }
}
