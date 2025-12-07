//! 表格输出工具
//!
//! 提供统一的表格输出接口，使用 tabled 库。

use crate::base::util::Logger;
use tabled::{Table, Tabled};

/// 打印表格
pub fn print_table<T: Tabled>(data: Vec<T>) {
    println!("{}", Table::new(data));
}

/// 打印表格（带标题）
pub fn print_table_with_title<T: Tabled>(title: &str, data: Vec<T>) {
    println!("{}\n", title);
    println!("{}", Table::new(data));
}

/// 从结构体创建表格字符串
///
/// # 示例
/// ```rust
/// #[derive(Tabled)]
/// struct PR {
///     number: u32,
///     title: String,
///     author: String,
///     status: String,
/// }
///
/// let prs = vec![
///     PR { number: 123, title: "Fix bug".to_string(), author: "Alice".to_string(), status: "Open".to_string() },
/// ];
///
/// let table_str = create_table(prs);
/// println!("{}", table_str);
/// ```
pub fn create_table<T: Tabled>(data: Vec<T>) -> String {
    Table::new(data).to_string()
}
