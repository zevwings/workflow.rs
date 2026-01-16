//! Log 配置处理模块

use crate::config::settings::LogSettings;
use crate::core::prompt::{Condition, ConfirmFormField, FormBuilder, GroupConfig, SelectFormField};
use crate::{br, info};
use color_eyre::{eyre::WrapErr, Result};

/// 处理 Log 配置
///
/// 参考 Go 版本的实现逻辑：
/// 1. 检测现有配置（检查日志级别是否存在）
/// 2. 启用确认：询问是否启用日志（已有配置时默认 Yes）
/// 3. 不启用则清空日志级别并返回
/// 4. 日志级别选择：选项：error、warn、info、debug（默认 info）
/// 5. 已有配置时显示当前级别
pub fn handle_log_config(existing: LogSettings) -> Result<LogSettings> {
    // 显示标题
    br!();

    // 显示检测信息（借鉴 Go 版本）
    // Go 版本检查：日志级别是否存在
    let has_log_level = existing.level.is_some();
    if has_log_level {
        info!("Log configuration detected.");
    } else {
        info!("No log configuration detected.");
    }

    // 使用 FormBuilder 收集配置
    // 如果已有配置，会在表单的第一个 step 中询问是否启用
    // 如果没有配置，会在表单的第一个 step 中询问是否启用
    let form_result = build_log_form(&existing, has_log_level)
        .run()
        .wrap_err("Failed to collect Log configuration")?;

    // 检查是否选择启用日志
    if let Some(enable) = form_result.get_bool_opt("enable_logging") {
        if !enable {
            // 不启用则清空日志级别并返回
            let mut result = existing.clone();
            result.level = None;
            return Ok(result);
        }
    }

    // 处理结果
    parse_log_result(form_result, &existing)
}

/// 构建 Log 配置表单
///
/// 如果已有配置，会在第一个 step 中询问是否启用（默认 Yes）
/// 如果没有配置，会在第一个 step 中询问是否启用（默认 No）
fn build_log_form(existing: &LogSettings, has_log_level: bool) -> FormBuilder {
    // 日志级别选项（按 Go 版本：error、warn、info、debug）
    let log_levels = vec![
        "error".to_string(),
        "warn".to_string(),
        "info".to_string(),
        "debug".to_string(),
    ];

    // 找到当前级别的索引（默认 info）
    let current_level = existing.level.as_deref().unwrap_or("info");
    let default_idx = log_levels.iter().position(|level| level == current_level).unwrap_or(2); // 默认为 info

    let log_level_prompt = if has_log_level {
        format!("Please select log level [current: {}]", current_level)
    } else {
        "Please select log level".to_string()
    };

    FormBuilder::new().add_group(
        "log",
        |mut g| {
            // 第一步：询问是否启用日志
            // 已有配置时默认 Yes，否则默认 No
            let default_enable = has_log_level;
            g = g.add_step(|s| {
                s.add_confirm(
                    ConfirmFormField::new("enable_logging", "Do you want to enable logging?")
                        .default(default_enable)
                        .result_title("Enable logging"),
                )
            });

            // 第二步：日志级别选择
            // 只有当用户选择启用时才执行
            g.add_step({
                let log_levels_clone = log_levels.clone();
                let log_level_prompt_clone = log_level_prompt.clone();
                let default_idx_clone = default_idx;
                move |s| {
                    // 创建条件函数：只有当用户选择启用时才执行
                    let condition: Option<Condition> =
                        Some(Box::new(move |result: &crate::prompt::FormResult| {
                            result.get_bool_opt("enable_logging").unwrap_or_default()
                        }));

                    let mut field = SelectFormField::new(
                        "log_level",
                        &log_level_prompt_clone,
                        log_levels_clone,
                    )
                    .default(default_idx_clone)
                    .result_title("Log level");

                    // 如果有条件，添加条件
                    if let Some(cond) = condition {
                        field = field.condition(cond);
                    }

                    s.add_select(field)
                }
            })
        },
        GroupConfig::required().with_title("Log Configuration (Optional)"),
    )
}

/// 解析 Log 配置结果
fn parse_log_result(
    form_result: crate::prompt::FormResult,
    existing: &LogSettings,
) -> Result<LogSettings> {
    // 处理日志级别
    let level = if let Some(selected_level) = form_result.get("log_level") {
        Some(selected_level.clone())
    } else {
        // 用户选择不配置 Log 组，使用现有值
        existing.level.clone()
    };

    let mut result = existing.clone();
    result.level = level;
    Ok(result)
}
