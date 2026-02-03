use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute};
use std::io::Write;

/// 在用户取消交互时输出统一的提示信息。
pub fn print_cancelled_message() -> std::io::Result<()> {
    let theme = crate::style::theme::get_theme();
    let mut stdout = std::io::stdout();

    // 使用 warning 样式统一展示取消提示，前缀使用统一的警告 emoji
    let emoji = "⚠";
    let text = theme.warning.apply(&format!("{} User cancelled", emoji), theme.enable_color);

    // 清掉当前行（例如 input 残留的 `> `），然后在该行输出取消提示
    let _ = execute!(
        stdout,
        cursor::MoveToColumn(0),
        Clear(ClearType::UntilNewLine),
    );

    // 输出一行取消提示（前面加一个空格，与其它文案对齐）
    writeln!(stdout, "{}", text)?;
    Ok(())
}
