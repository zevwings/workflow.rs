//! Log 配置处理模块

use crate::base::interactive::{
    ConfirmFormField, FormBuilder, GroupConfig, InputFormField, SelectFormField,
};
use crate::base::settings::settings::{default_download_base_dir, LogSettings};
use crate::commands::setup::types::{CollectedConfig, LogConfig};
use crate::{br, info};
use color_eyre::{eyre::WrapErr, Result};

/// 处理 Log 配置
pub fn handle_log_config(existing: &CollectedConfig) -> Result<LogConfig> {
    // 显示标题
    br!();
    info!("  Log Configuration (Optional)");
    br!('─', 65);

    // 显示检测信息（借鉴 Go 版本）
    let has_log_config = existing.log_output_folder_name.is_some()
        || existing.log_download_base_dir.is_some()
        || existing.enable_trace_console.is_some();
    if has_log_config {
        info!("Log configuration detected.");
    } else {
        info!("No log configuration detected.");
    }
    br!();

    // 使用 FormBuilder 收集配置
    let form_result =
        build_log_form(existing).run().wrap_err("Failed to collect Log configuration")?;

    // 处理结果
    parse_log_result(form_result, existing)
}

/// 构建 Log 配置表单
fn build_log_form(existing: &CollectedConfig) -> FormBuilder {
    let default_folder_name = LogSettings::default_log_folder();
    let is_custom_folder_name = existing
        .log_output_folder_name
        .as_ref()
        .map(|name| name != &default_folder_name)
        .unwrap_or(false);
    let default_dir = default_download_base_dir();
    let is_custom_dir = existing
        .log_download_base_dir
        .as_ref()
        .map(|dir| dir != &default_dir)
        .unwrap_or(false);
    let current_trace_console = existing.enable_trace_console.unwrap_or(false);

    FormBuilder::new().add_group(
        "log",
        |g| {
            g.add_step(|s| {
                s.add_confirm(
                    ConfirmFormField::new(
                        "should_configure_log_folder",
                        "Do you want to configure log output folder name?",
                    )
                    .default(is_custom_folder_name),
                )
            })
            .step_if("should_configure_log_folder", "yes", |s| {
                let folder_name_prompt = if is_custom_folder_name {
                    "Log output folder name (press Enter to keep)".to_string()
                } else {
                    format!(
                        "Log output folder name (press Enter to use default: {})",
                        default_folder_name
                    )
                };
                let mut field = InputFormField::new("log_output_folder_name", &folder_name_prompt)
                    .allow_empty(true);
                if is_custom_folder_name {
                    if let Some(ref existing_name) = existing.log_output_folder_name {
                        field = field.default(existing_name.clone());
                    }
                } else {
                    field = field.default(default_folder_name.clone());
                }
                s.add_input(field)
            })
            .add_step(|s| {
                s.add_confirm(
                    ConfirmFormField::new(
                        "should_configure_doc_dir",
                        "Do you want to configure document base directory?",
                    )
                    .default(is_custom_dir),
                )
            })
            .step_if("should_configure_doc_dir", "yes", |s| {
                let base_dir_prompt = if is_custom_dir {
                    "Document base directory (press Enter to keep)".to_string()
                } else {
                    format!(
                        "Document base directory (press Enter to use default: {})",
                        default_dir
                    )
                };
                let mut field = InputFormField::new("log_download_base_dir", &base_dir_prompt)
                    .allow_empty(true);
                if is_custom_dir {
                    if let Some(ref existing_dir) = existing.log_download_base_dir {
                        field = field.default(existing_dir.clone());
                    }
                } else {
                    field = field.default(default_dir.clone());
                }
                s.add_input(field)
            })
            .add_step(|s| {
                // Tracing Console Output
                let trace_console_options = vec![
                    "Enable (output to both file and console)".to_string(),
                    "Disable (output to file only)".to_string(),
                ];
                let default_idx = if current_trace_console { 0 } else { 1 };
                s.add_select(
                    SelectFormField::new(
                        "trace_console_mode",
                        "Please select trace console output mode",
                        trace_console_options,
                    )
                    .default(default_idx)
                    .result_title("Trace console output mode"),
                )
            })
        },
        GroupConfig::optional().with_title("Log Configuration (Optional)"),
    )
}

/// 解析 Log 配置结果
fn parse_log_result(
    form_result: crate::base::interactive::FormResult,
    existing: &CollectedConfig,
) -> Result<LogConfig> {
    let default_folder_name = LogSettings::default_log_folder();
    let default_dir = default_download_base_dir();
    let is_custom_folder_name = existing
        .log_output_folder_name
        .as_ref()
        .map(|name| name != &default_folder_name)
        .unwrap_or(false);
    let is_custom_dir = existing
        .log_download_base_dir
        .as_ref()
        .map(|dir| dir != &default_dir)
        .unwrap_or(false);

    // 如果用户选择不配置 Log 组，使用现有值
    let (log_output_folder_name, log_download_base_dir, enable_trace_console) = if form_result
        .has("should_configure_log_folder")
        || form_result.has("should_configure_doc_dir")
        || form_result.has("trace_console_mode")
    {
        // 用户配置了 Log 组，处理配置
        let log_output_folder_name =
            if form_result.get_bool_opt("should_configure_log_folder") == Some(true) {
                if let Some(input_value) = form_result.get("log_output_folder_name") {
                    if input_value.is_empty() || input_value == default_folder_name {
                        None
                    } else {
                        Some(input_value.clone())
                    }
                } else {
                    None
                }
            } else if is_custom_folder_name {
                existing.log_output_folder_name.clone()
            } else {
                None
            };

        let log_download_base_dir =
            if form_result.get_bool_opt("should_configure_doc_dir") == Some(true) {
                if let Some(input_value) = form_result.get("log_download_base_dir") {
                    if input_value.is_empty() || input_value == default_dir {
                        None
                    } else {
                        Some(input_value.clone())
                    }
                } else {
                    None
                }
            } else if is_custom_dir {
                existing.log_download_base_dir.clone()
            } else {
                None
            };

        // Tracing 配置
        // 注意：Select 字段现在返回选项值（String），而不是索引
        let enable_trace_console = if let Some(mode) = form_result.get("trace_console_mode") {
            if mode == "Enable (output to both file and console)" {
                Some(true)
            } else {
                None
            }
        } else {
            existing.enable_trace_console
        };

        (
            log_output_folder_name,
            log_download_base_dir,
            enable_trace_console,
        )
    } else {
        // 用户选择不配置 Log 组，使用现有值
        (
            existing.log_output_folder_name.clone(),
            existing.log_download_base_dir.clone(),
            existing.enable_trace_console,
        )
    };

    Ok(LogConfig {
        output_folder_name: log_output_folder_name,
        download_base_dir: log_download_base_dir,
        enable_trace_console,
    })
}
