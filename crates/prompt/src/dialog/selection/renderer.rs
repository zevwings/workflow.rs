//! 选项列表渲染器模块
//!
//! 提供 select 和 multiselect 共享的渲染逻辑

use crate::dialog::Result;
use crate::style::theme::Theme;
use crossterm::cursor;
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::ClearType;
use std::io::Write;

/// 选项渲染器 trait
///
/// 定义如何渲染单个选项，允许 select 和 multiselect 有不同的渲染方式
pub(super) trait OptionRenderer {
    /// 渲染单个选项
    ///
    /// # 参数
    /// - `index`: 选项索引
    /// - `option_text`: 选项文本
    /// - `is_current`: 是否是当前光标位置
    /// - `theme`: 主题样式
    ///
    /// # 返回
    /// 渲染后的行文本（不包含换行符）
    fn render_option(
        &self,
        index: usize,
        option_text: &str,
        is_current: bool,
        theme: &Theme,
    ) -> String;
}

/// 默认分页大小
pub(super) const DEFAULT_PAGE_SIZE: usize = 10;

/// 选项列表渲染参数
#[derive(Debug)]
pub(super) struct RenderOptionsParams<'a, OR: OptionRenderer, O: std::fmt::Display> {
    /// 选项列表
    pub options: &'a [O],
    /// 当前光标位置
    pub current_index: usize,
    /// 已渲染的行数（用于清除）
    pub rendered_lines: usize,
    /// 主题样式
    pub theme: &'a Theme,
    /// 选项渲染器实现
    pub renderer: &'a OR,
    /// 提示文本
    pub hint_text: &'a str,
    /// 搜索查询（如果为 Some，显示搜索框）
    pub search_query: Option<&'a str>,
    /// 分页大小（每页显示的选项数量），None 表示使用默认值
    pub page_size: Option<usize>,
}

/// 选项列表渲染器
pub(super) struct OptionListRenderer;

impl OptionListRenderer {
    /// 渲染选项列表（带搜索框和分页）
    ///
    /// # 参数
    /// - `params`: 渲染参数
    ///
    /// # 返回
    /// 渲染的总行数（搜索框 + 可见选项数 + 提示行）
    pub(super) fn render_options_with_search<OR: OptionRenderer, O: std::fmt::Display>(
        params: &RenderOptionsParams<'_, OR, O>,
    ) -> Result<usize> {
        let mut stdout = std::io::stdout();
        let has_search = params.search_query.is_some();
        let search_lines = if has_search { 1 } else { 0 };

        // 计算分页参数
        let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let total_options = params.options.len();

        // 计算滚动窗口：确保当前选中项在可见区域内
        let (start_index, end_index) = if total_options <= page_size {
            // 选项数量小于等于页面大小，显示全部
            (0, total_options)
        } else {
            // 需要分页，计算滚动窗口
            // 策略：保持当前选中项在窗口中间位置（如果可能）
            let half_page = page_size / 2;
            let start = if params.current_index < half_page {
                // 接近列表开头
                0
            } else if params.current_index >= total_options - half_page {
                // 接近列表结尾
                total_options.saturating_sub(page_size)
            } else {
                // 在中间位置
                params.current_index.saturating_sub(half_page)
            };
            let end = (start + page_size).min(total_options);
            (start, end)
        };

        let visible_count = end_index - start_index;
        // 总行数 = 搜索行 + 可见选项数 + 提示行（1）+ 分页信息行（如果需要分页则为1，否则为0）
        let has_pagination = total_options > page_size;
        let pagination_lines = if has_pagination { 1 } else { 0 };
        let total_lines = search_lines + visible_count + 1 + pagination_lines;

        // 清除已渲染的行
        if params.rendered_lines > 0 {
            clear_rendered_lines(params.rendered_lines)?;
        }

        // 渲染搜索框（如果有）
        if let Some(query) = params.search_query {
            execute!(stdout, cursor::MoveToColumn(0))?;
            execute!(stdout, ResetColor)?;
            let search_label = params.theme.hint.apply("搜索: ", params.theme.enable_color);
            let search_text = params.theme.answer.apply(query, params.theme.enable_color);
            write!(stdout, "{}{}", search_label, search_text)?;
            execute!(stdout, ResetColor)?;
            writeln!(stdout)?;
        }

        // 渲染可见窗口内的选项
        for (visible_index, option) in
            params.options.iter().enumerate().skip(start_index).take(visible_count)
        {
            execute!(stdout, cursor::MoveToColumn(0))?;
            execute!(stdout, ResetColor)?;

            let is_current = visible_index == params.current_index;
            let option_text = option.to_string();
            let rendered_line = params.renderer.render_option(
                visible_index,
                &option_text,
                is_current,
                params.theme,
            );

            write!(stdout, "{}", rendered_line)?;
            execute!(stdout, ResetColor)?;
            writeln!(stdout)?;
        }

        // 显示分页信息（如果需要）
        if has_pagination {
            render_pagination_info(params.theme, start_index, end_index, total_options)?;
        }

        // 显示提示信息
        render_hint(params.theme, params.hint_text)?;

        // 隐藏光标
        execute!(stdout, cursor::Hide)?;
        stdout.flush()?;
        Ok(total_lines)
    }
}

/// 渲染分页信息
fn render_pagination_info(
    theme: &Theme,
    start_index: usize,
    end_index: usize,
    total: usize,
) -> Result<()> {
    let mut stdout = std::io::stdout();
    execute!(stdout, cursor::MoveToColumn(0))?;
    let info = format!(
        "Showing {}-{} of {} items",
        start_index + 1,
        end_index,
        total
    );
    let styled = theme.hint.apply(&info, theme.enable_color);
    writeln!(stdout, "{}", styled)?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

/// 渲染提示信息
fn render_hint(theme: &Theme, hint_text: &str) -> Result<()> {
    let mut stdout = std::io::stdout();
    execute!(stdout, cursor::MoveToColumn(0))?;
    let hint_styled = theme.hint.apply(hint_text, theme.enable_color);
    writeln!(stdout, "{}", hint_styled)?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

/// 清除已渲染的行
fn clear_rendered_lines(rendered_lines: usize) -> Result<()> {
    let mut stdout = std::io::stdout();

    // 上移到已渲染的第一行
    execute!(stdout, cursor::MoveUp(rendered_lines as u16))?;

    // 清除所有已渲染的行
    for i in 0..rendered_lines {
        write!(stdout, "\r")?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::CurrentLine))?;
        if i < rendered_lines - 1 {
            execute!(stdout, cursor::MoveDown(1))?;
        }
    }

    // 清除后，光标在最后一个清除行（提示行）
    // 需要回到第一个选项行：上移 (rendered_lines - 1) 行
    if rendered_lines > 1 {
        execute!(stdout, cursor::MoveUp((rendered_lines - 1) as u16))?;
    }

    Ok(())
}

/// 清除并显示结果（带搜索框支持）
///
/// # 参数
/// - `rendered_lines`: 实际渲染的行数（由 render_options_with_search 返回）
/// - `message`: 提示消息
/// - `result_text`: 结果文本
/// - `theme`: 主题样式
pub(super) fn clear_and_display_result_with_search(
    rendered_lines: usize,
    message: &str,
    result_text: &str,
    theme: &Theme,
) -> Result<()> {
    let mut stdout = std::io::stdout();

    // 需要清除的行数 = 渲染的行数 + 提示行（"? 请选择一个选项"）
    let lines_to_clear = rendered_lines + 1;

    // 当前光标在提示信息行的下一行（因为 render_hint 输出了换行符）
    // 先向上移动一行回到提示信息行
    execute!(stdout, cursor::MoveUp(1))?;

    // 从提示信息行开始向上清除所有行
    // 先清除当前行（提示信息行）
    write!(stdout, "\r")?;
    execute!(stdout, ResetColor)?;
    execute!(stdout, crossterm::terminal::Clear(ClearType::CurrentLine))?;

    // 向上移动并清除每一行（包括所有选项行和提示行）
    for _ in 0..(lines_to_clear - 1) {
        execute!(stdout, cursor::MoveUp(1))?;
        write!(stdout, "\r")?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::CurrentLine))?;
    }

    // 此时光标在提示行位置（"? 请选择一个选项"），显示格式化的结果："> [title] [value]"
    let prefix = theme.success.apply("> ", theme.enable_color);
    let title = theme.title.apply(message, theme.enable_color);
    let answer = theme.answer.apply(result_text, theme.enable_color);

    write!(stdout, "{}{} {}", prefix, title, answer)?;
    writeln!(stdout)?;
    // 确保光标在新行的开头，以便后续消息输出正确对齐
    execute!(stdout, cursor::MoveToColumn(0))?;
    execute!(stdout, ResetColor)?;
    execute!(stdout, cursor::Show)?;
    stdout.flush()?;
    Ok(())
}
