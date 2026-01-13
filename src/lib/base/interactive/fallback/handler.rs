//! 类型安全的 Fallback 处理器

use crate::base::interactive::config::PromptConfig;
use crate::base::interactive::error::Result;
use crate::base::interactive::fallback::options::FallbackOptions;
use crate::base::interactive::terminal::Terminal;

/// 类型安全的 Fallback 处理器接口
/// 用于提供类型安全的 fallback 处理，避免类型转换
pub trait FallbackHandler<T> {
    /// 格式化提示文本（用于显示）
    fn format_prompt_text(&self, message: &str) -> String;

    /// 格式化答案文本（用于显示结果）
    fn format_answer(&self, result: &T) -> String;

    /// 处理一行输入（用于 fallback 模式）
    /// 返回处理结果和错误
    fn process_line_input(&self, input: &str) -> std::result::Result<T, String>;

    /// 获取默认结果（当输入为空或无效时使用）
    fn get_default_result(&self) -> T;
}

/// 执行 fallback 模式的通用框架（类型安全版本）
/// 使用泛型提供类型安全，避免类型转换
///
/// 参数:
///   - terminal: 终端接口
///   - message: 原始提示消息
///   - config: 提示配置
///   - handler: 类型安全的 fallback 处理器
///   - options: fallback 选项
///
/// 返回:
///   - result: 处理结果（类型 T）
///   - error: 错误
#[allow(dead_code)]
pub fn execute_fallback<T, TR: Terminal>(
    terminal: &mut TR,
    message: &str,
    config: &PromptConfig,
    handler: &dyn FallbackHandler<T>,
    options: &FallbackOptions<T>,
) -> Result<T> {
    use color_eyre::eyre;

    // 格式化提示文本
    let prompt_text = handler.format_prompt_text(message);

    // 保存光标位置（在提示行的开始）
    terminal.write_flush(&prompt_text)?;

    // 如果设置了显示选项，显示选项列表
    if options.show_options {
        if let Some(format_options) = &options.format_options {
            format_options(terminal)?;
        }
    }

    // 显示输入提示（如果有）
    if let Some(input_prompt) = &options.input_prompt {
        terminal.write_flush(input_prompt)?;
    }

    // 读取一行输入
    let input = match terminal.read_line() {
        Ok(line) => line,
        Err(_) => {
            // 如果读取失败（可能是空输入），返回默认值并显示格式化结果
            let default_result = handler.get_default_result();
            if let Some(result_display) = &options.result_display {
                let prompt_msg = if let Some(format_prompt) = &config.format_prompt {
                    format_prompt(message)
                } else {
                    message.to_string()
                };
                result_display(
                    terminal,
                    &prompt_msg,
                    &default_result,
                    handler,
                    message,
                    config,
                )?;
            }
            return Ok(default_result);
        }
    };

    // 处理输入
    let result = match handler.process_line_input(&input) {
        Ok(r) => r,
        Err(e) => {
            // 处理失败，返回默认值
            let default_result = handler.get_default_result();
            if let Some(result_display) = &options.result_display {
                let prompt_msg = if let Some(format_prompt) = &config.format_prompt {
                    format_prompt(message)
                } else {
                    message.to_string()
                };
                result_display(
                    terminal,
                    &prompt_msg,
                    &default_result,
                    handler,
                    message,
                    config,
                )?;
            }
            return Err(eyre::eyre!("{}", e));
        }
    };

    // 显示格式化结果
    if let Some(result_display) = &options.result_display {
        let prompt_msg = if let Some(format_prompt) = &config.format_prompt {
            format_prompt(message)
        } else {
            message.to_string()
        };
        result_display(terminal, &prompt_msg, &result, handler, message, config)?;
    }

    Ok(result)
}

/// 类型别名，保持 API 简洁
pub type ExecuteFallback<T, TR> =
    fn(&mut TR, &str, &PromptConfig, &dyn FallbackHandler<T>, &FallbackOptions<T>) -> Result<T>;
