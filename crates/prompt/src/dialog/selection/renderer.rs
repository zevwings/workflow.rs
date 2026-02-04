//! 选项列表渲染器模块
//!
//! 提供 select 和 multiselect 共享的渲染逻辑

use crate::backend::Backend;
use crate::dialog::Result;
use crate::style::theme::Theme;

/// 选项渲染器 trait
///
/// 定义如何渲染单个选项，允许 select 和 multiselect 有不同的渲染方式
pub(super) trait OptionRenderer {
    /// 渲染单个选项
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
    pub(super) fn render_options_with_search<
        B: Backend,
        OR: OptionRenderer,
        O: std::fmt::Display,
    >(
        backend: &mut B,
        params: &RenderOptionsParams<'_, OR, O>,
    ) -> Result<usize> {
        let has_search = params.search_query.is_some();
        let search_lines = if has_search { 1 } else { 0 };

        // 计算分页参数
        let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let total_options = params.options.len();

        // 计算滚动窗口
        let (start_index, end_index) = if total_options <= page_size {
            (0, total_options)
        } else {
            let half_page = page_size / 2;
            let start = if params.current_index < half_page {
                0
            } else if params.current_index >= total_options - half_page {
                total_options.saturating_sub(page_size)
            } else {
                params.current_index.saturating_sub(half_page)
            };
            let end = (start + page_size).min(total_options);
            (start, end)
        };

        let visible_count = end_index - start_index;
        let has_pagination = total_options > page_size;
        let pagination_lines = if has_pagination { 1 } else { 0 };
        let total_lines = search_lines + visible_count + 1 + pagination_lines;

        // 清除已渲染的行
        if params.rendered_lines > 0 {
            clear_rendered_lines(backend, params.rendered_lines)?;
        }

        // 渲染搜索框
        if let Some(query) = params.search_query {
            backend.move_to_column(0)?;
            let search_label = params.theme.hint.apply("搜索: ", params.theme.enable_color);
            let search_text = params.theme.answer.apply(query, params.theme.enable_color);
            backend.writeln(&format!("{}{}", search_label, search_text))?;
        }

        // 渲染可见窗口内的选项
        for (visible_index, option) in
            params.options.iter().enumerate().skip(start_index).take(visible_count)
        {
            backend.move_to_column(0)?;
            let is_current = visible_index == params.current_index;
            let option_text = option.to_string();
            let rendered_line = params.renderer.render_option(
                visible_index,
                &option_text,
                is_current,
                params.theme,
            );
            backend.writeln(&rendered_line)?;
        }

        // 显示分页信息
        if has_pagination {
            render_pagination_info(backend, params.theme, start_index, end_index, total_options)?;
        }

        // 显示提示信息
        render_hint(backend, params.theme, params.hint_text)?;

        backend.hide_cursor()?;
        backend.flush()?;
        Ok(total_lines)
    }
}

/// 渲染分页信息
fn render_pagination_info<B: Backend>(
    backend: &mut B,
    theme: &Theme,
    start_index: usize,
    end_index: usize,
    total: usize,
) -> Result<()> {
    backend.move_to_column(0)?;
    let info = format!(
        "Showing {}-{} of {} items",
        start_index + 1,
        end_index,
        total
    );
    let styled = theme.hint.apply(&info, theme.enable_color);
    backend.writeln(&styled)?;
    Ok(())
}

/// 渲染提示信息
fn render_hint<B: Backend>(backend: &mut B, theme: &Theme, hint_text: &str) -> Result<()> {
    backend.move_to_column(0)?;
    let hint_styled = theme.hint.apply(hint_text, theme.enable_color);
    backend.writeln(&hint_styled)?;
    Ok(())
}

/// 清除已渲染的行
fn clear_rendered_lines<B: Backend>(backend: &mut B, rendered_lines: usize) -> Result<()> {
    // 上移到已渲染的第一行
    backend.move_up(rendered_lines as u16)?;

    // 清除所有已渲染的行
    for i in 0..rendered_lines {
        backend.move_to_column(0)?;
        backend.clear_line()?;
        if i < rendered_lines - 1 {
            backend.move_down(1)?;
        }
    }

    // 回到第一行
    if rendered_lines > 1 {
        backend.move_up((rendered_lines - 1) as u16)?;
    }

    Ok(())
}

/// 清除并显示结果（带搜索框支持）
pub(super) fn clear_and_display_result_with_search<B: Backend>(
    backend: &mut B,
    rendered_lines: usize,
    message: &str,
    result_text: &str,
    theme: &Theme,
) -> Result<()> {
    // 需要清除的行数 = 渲染的行数 + 提示行
    let lines_to_clear = rendered_lines + 1;

    // 向上移动一行
    backend.move_up(1)?;

    // 清除当前行
    backend.move_to_column(0)?;
    backend.clear_line()?;

    // 向上移动并清除每一行
    for _ in 0..(lines_to_clear - 1) {
        backend.move_up(1)?;
        backend.move_to_column(0)?;
        backend.clear_line()?;
    }

    // 显示结果
    let prefix = theme.success.apply("> ", theme.enable_color);
    let title = theme.title.apply(message, theme.enable_color);
    let answer = theme.answer.apply(result_text, theme.enable_color);

    backend.write(&format!("{}{} {}", prefix, title, answer))?;
    backend.writeln("")?;
    backend.move_to_column(0)?;
    backend.show_cursor()?;
    backend.flush()?;
    Ok(())
}
