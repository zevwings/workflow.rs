//! 表格行 trait
//!
//! 提供 Tabled trait 和 impl_tabled 宏

use crate::output::table::builder::{Alignment, TableBuilder};

/// 表格行 trait
///
/// 实现此 trait 的类型可以自动转换为表格行。
/// 提供表头和数据行的映射。
pub trait Tabled {
    /// 返回表格的列头
    fn headers() -> Vec<String>;

    /// 将当前实例转换为表格行数据
    fn row(&self) -> Vec<String>;
}

/// 宏：简化 Tabled trait 的实现
///
/// 自动生成 `headers()` 和 `row()` 方法。
///
/// # 用法
///
/// ```rust,no_run
/// use prompt::Tabled;
/// use toolkit::impl_tabled;
///
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// impl_tabled!(User, ["Name", "Age"], |s| vec![s.name.clone(), s.age.to_string()]);
/// ```
#[macro_export]
macro_rules! impl_tabled {
    ($type:ty, [$($header:expr),+], |$self:ident| $body:expr) => {
        impl $crate::Tabled for $type {
            fn headers() -> Vec<String> {
                vec![$($header.to_string()),+]
            }

            fn row(&self) -> Vec<String> {
                let $self = self;
                $body
            }
        }
    };
}

// 支持 Tabled trait 的适配器
impl TableBuilder {
    /// 从实现了 Tabled trait 的数据创建表格构建器
    ///
    /// # 参数
    ///
    /// * `data` - 要显示的数据，必须实现 `Tabled` trait
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use prompt::{TableBuilder, Tabled, TableStyle};
    ///
    /// struct User {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl Tabled for User {
    ///     fn headers() -> Vec<String> {
    ///         vec!["Name".to_string(), "Age".to_string()]
    ///     }
    ///
    ///     fn row(&self) -> Vec<String> {
    ///         vec![self.name.clone(), self.age.to_string()]
    ///     }
    /// }
    ///
    /// let users = vec![
    ///     User { name: "Alice".to_string(), age: 30 },
    /// ];
    ///
    /// let table = TableBuilder::from_tabled(users)
    ///     .with_title("Users")
    ///     .with_style(TableStyle::Modern)
    ///     .render();
    /// ```
    pub fn from_tabled<T: Tabled>(data: Vec<T>) -> Self {
        if data.is_empty() {
            return Self {
                headers: Vec::new(),
                rows: Vec::new(),
                border: true,
                row_line: true,
                alignment: Alignment::Left,
                title: None,
                max_width: None,
                column_alignments: Vec::new(),
            };
        }

        // 从第一个元素获取表头
        let headers = T::headers();
        let rows: Vec<Vec<String>> = data.iter().map(|item| item.row()).collect();

        Self {
            headers,
            rows,
            border: true,
            row_line: true,
            alignment: Alignment::Left,
            title: None,
            max_width: None,
            column_alignments: Vec::new(),
        }
    }
}
