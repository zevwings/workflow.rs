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
/// use prompt::impl_tabled;
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

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用结构体
    struct TestUser {
        name: String,
        age: u32,
        active: bool,
    }

    impl Tabled for TestUser {
        fn headers() -> Vec<String> {
            vec!["Name".to_string(), "Age".to_string(), "Active".to_string()]
        }

        fn row(&self) -> Vec<String> {
            vec![
                self.name.clone(),
                self.age.to_string(),
                self.active.to_string(),
            ]
        }
    }

    // ========================================================================
    // Tabled trait 测试
    // ========================================================================

    #[test]
    fn test_tabled_headers() {
        let headers = TestUser::headers();
        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0], "Name");
        assert_eq!(headers[1], "Age");
        assert_eq!(headers[2], "Active");
    }

    #[test]
    fn test_tabled_row() {
        let user = TestUser {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        };

        let row = user.row();
        assert_eq!(row.len(), 3);
        assert_eq!(row[0], "Alice");
        assert_eq!(row[1], "30");
        assert_eq!(row[2], "true");
    }

    #[test]
    fn test_tabled_row_with_special_values() {
        let user = TestUser {
            name: "".to_string(),
            age: 0,
            active: false,
        };

        let row = user.row();
        assert_eq!(row[0], "");
        assert_eq!(row[1], "0");
        assert_eq!(row[2], "false");
    }

    // ========================================================================
    // TableBuilder::from_tabled 测试
    // ========================================================================

    #[test]
    fn test_from_tabled_with_data() {
        let users = vec![
            TestUser {
                name: "Alice".to_string(),
                age: 30,
                active: true,
            },
            TestUser {
                name: "Bob".to_string(),
                age: 25,
                active: false,
            },
        ];

        let builder = TableBuilder::from_tabled(users);

        // 验证 headers 被正确设置
        assert_eq!(builder.headers.len(), 3);
        assert_eq!(builder.headers[0], "Name");

        // 验证 rows 被正确设置
        assert_eq!(builder.rows.len(), 2);
        assert_eq!(builder.rows[0][0], "Alice");
        assert_eq!(builder.rows[1][0], "Bob");
    }

    #[test]
    fn test_from_tabled_empty_data() {
        let users: Vec<TestUser> = vec![];
        let builder = TableBuilder::from_tabled(users);

        // 空数据应该返回空的 headers 和 rows
        assert!(builder.headers.is_empty());
        assert!(builder.rows.is_empty());
    }

    #[test]
    fn test_from_tabled_single_item() {
        let users = vec![TestUser {
            name: "Charlie".to_string(),
            age: 35,
            active: true,
        }];

        let builder = TableBuilder::from_tabled(users);

        assert_eq!(builder.headers.len(), 3);
        assert_eq!(builder.rows.len(), 1);
        assert_eq!(builder.rows[0][0], "Charlie");
    }

    #[test]
    fn test_from_tabled_default_settings() {
        let users = vec![TestUser {
            name: "Test".to_string(),
            age: 20,
            active: true,
        }];

        let builder = TableBuilder::from_tabled(users);

        // 验证默认设置
        assert!(builder.border);
        assert!(builder.row_line);
        assert!(matches!(builder.alignment, Alignment::Left));
        assert!(builder.title.is_none());
        assert!(builder.max_width.is_none());
        assert!(builder.column_alignments.is_empty());
    }

    #[test]
    fn test_from_tabled_chained_with_options() {
        let users = vec![TestUser {
            name: "Test".to_string(),
            age: 20,
            active: true,
        }];

        let builder = TableBuilder::from_tabled(users)
            .with_title("User List")
            .with_border(false)
            .with_max_width(80);

        assert_eq!(builder.title, Some("User List".to_string()));
        assert!(!builder.border);
        assert_eq!(builder.max_width, Some(80));
    }

    #[test]
    fn test_from_tabled_render() {
        let users = vec![TestUser {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        }];

        let table = TableBuilder::from_tabled(users).render();

        // 验证渲染结果包含数据
        assert!(table.contains("Alice"));
        assert!(table.contains("30"));
        assert!(table.contains("true"));
        assert!(table.contains("Name"));
    }

    // ========================================================================
    // 使用 impl_tabled 宏的测试
    // ========================================================================

    // 定义一个简单的结构体用于宏测试
    struct SimpleItem {
        id: u32,
        value: String,
    }

    impl_tabled!(SimpleItem, ["ID", "Value"], |s| vec![
        s.id.to_string(),
        s.value.clone()
    ]);

    #[test]
    fn test_impl_tabled_macro_headers() {
        let headers = SimpleItem::headers();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0], "ID");
        assert_eq!(headers[1], "Value");
    }

    #[test]
    fn test_impl_tabled_macro_row() {
        let item = SimpleItem {
            id: 42,
            value: "test".to_string(),
        };
        let row = item.row();
        assert_eq!(row.len(), 2);
        assert_eq!(row[0], "42");
        assert_eq!(row[1], "test");
    }

    #[test]
    fn test_impl_tabled_macro_with_from_tabled() {
        let items = vec![
            SimpleItem {
                id: 1,
                value: "one".to_string(),
            },
            SimpleItem {
                id: 2,
                value: "two".to_string(),
            },
        ];

        let builder = TableBuilder::from_tabled(items);
        assert_eq!(builder.headers.len(), 2);
        assert_eq!(builder.rows.len(), 2);
    }

    // ========================================================================
    // Unicode 内容测试
    // ========================================================================

    struct UnicodeItem {
        chinese: String,
        emoji: String,
    }

    impl Tabled for UnicodeItem {
        fn headers() -> Vec<String> {
            vec!["中文".to_string(), "Emoji".to_string()]
        }

        fn row(&self) -> Vec<String> {
            vec![self.chinese.clone(), self.emoji.clone()]
        }
    }

    #[test]
    fn test_tabled_unicode_headers() {
        let headers = UnicodeItem::headers();
        assert_eq!(headers[0], "中文");
        assert_eq!(headers[1], "Emoji");
    }

    #[test]
    fn test_tabled_unicode_row() {
        let item = UnicodeItem {
            chinese: "你好".to_string(),
            emoji: "🎉".to_string(),
        };

        let row = item.row();
        assert_eq!(row[0], "你好");
        assert_eq!(row[1], "🎉");
    }

    #[test]
    fn test_from_tabled_unicode() {
        let items = vec![UnicodeItem {
            chinese: "测试".to_string(),
            emoji: "✓".to_string(),
        }];

        let table = TableBuilder::from_tabled(items).render();
        assert!(table.contains("中文"));
        assert!(table.contains("测试"));
        assert!(table.contains("✓"));
    }
}
