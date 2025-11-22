//! 用户确认辅助函数
//!
//! 提供统一的用户确认接口，简化 `Confirm::new()` 的使用。

use anyhow::{Context, Result};
use dialoguer::Confirm;

/// 确认函数
///
/// 如果用户确认，返回 `Ok(true)`。
/// 如果用户取消：
///   - 如果提供了 `cancel_message`，返回错误
///   - 如果没有提供 `cancel_message`，返回 `Ok(false)`
///
/// # 参数
/// * `prompt` - 提示信息
/// * `default` - 默认选择（true 表示默认确认，false 表示默认取消）
/// * `cancel_message` - 取消时的错误消息（可选）
///   - 如果为 `Some(msg)`，取消时返回错误
///   - 如果为 `None`，取消时返回 `Ok(false)`
///
/// # 返回
/// - 用户确认：返回 `Ok(true)`
/// - 用户取消且 `cancel_message` 为 `Some`：返回错误
/// - 用户取消且 `cancel_message` 为 `None`：返回 `Ok(false)`
///
/// # 使用场景
/// - 需要根据用户选择执行不同逻辑：`confirm(prompt, default, None)?`
/// - 需要用户确认才能继续，取消则终止：`confirm(prompt, default, Some("Operation cancelled."))?`
pub fn confirm(prompt: &str, default: bool, message: Option<&str>) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .context("Failed to get user confirmation")?;

    if !confirmed {
        if let Some(msg) = message {
            anyhow::bail!("{}", msg);
        }
    }

    Ok(confirmed)
}
