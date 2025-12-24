//! Table 模块测试
//!
//! 测试表格构建器和表格样式的核心功能。

use tabled::Tabled;
use workflow::base::table::{TableBuilder, TableStyle};

#[derive(Tabled, Clone)]
struct TestUser {
    name: String,
    age: u32,
    email: String,
}

#[test]
fn test_table_builder_new() {
    let users = vec![
        TestUser {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        },
        TestUser {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
        },
    ];

    let _builder = TableBuilder::new(users);
    // 验证可以创建 TableBuilder
    assert!(true);
}

#[test]
fn test_table_builder_with_title() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    let builder = TableBuilder::new(users).with_title("Users List");
    let output = builder.render();

    // 验证标题出现在输出中
    assert!(output.contains("Users List"));
}

#[test]
fn test_table_builder_with_style() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    let builder = TableBuilder::new(users.clone()).with_style(TableStyle::Modern);
    let output_modern = builder.render();

    let builder = TableBuilder::new(users).with_style(TableStyle::Compact);
    let output_compact = builder.render();

    // 验证不同样式产生不同的输出
    assert_ne!(output_modern, output_compact);
}

#[test]
fn test_table_builder_with_max_width() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    let builder = TableBuilder::new(users).with_max_width(20);
    let output = builder.render();

    // 验证输出不为空
    assert!(!output.is_empty());
}

#[test]
fn test_table_builder_empty_data() {
    let users: Vec<TestUser> = vec![];

    let builder = TableBuilder::new(users);
    let output = builder.render();

    // 空数据应该返回空字符串
    assert_eq!(output, "");
}

#[test]
fn test_table_builder_empty_data_with_title() {
    let users: Vec<TestUser> = vec![];

    let builder = TableBuilder::new(users).with_title("Empty Table");
    let output = builder.render();

    // 空数据带标题应该显示标题和 "(No data)"
    assert!(output.contains("Empty Table"));
    assert!(output.contains("(No data)"));
}

#[test]
fn test_table_builder_display_trait() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    let builder = TableBuilder::new(users);
    let output = format!("{}", builder);

    // 验证 Display trait 正常工作
    assert!(!output.is_empty());
}

#[test]
fn test_table_style_variants() {
    // 验证所有 TableStyle 变体都可以使用
    let _styles = vec![
        TableStyle::Default,
        TableStyle::Modern,
        TableStyle::Compact,
        TableStyle::Minimal,
        TableStyle::Grid,
    ];

    assert!(true);
}

#[test]
fn test_table_builder_chain_calls() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    // 测试链式调用
    let output = TableBuilder::new(users)
        .with_title("Users")
        .with_style(TableStyle::Modern)
        .with_max_width(80)
        .render();

    assert!(!output.is_empty());
    assert!(output.contains("Users"));
}
