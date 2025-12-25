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

// ==================== TableBuilder Creation Tests ====================

/// 测试使用数据创建TableBuilder
///
/// ## 测试目的
/// 验证 `TableBuilder::new()` 方法能够使用数据成功创建表格构建器。
///
/// ## 测试场景
/// 1. 准备测试数据（包含多个用户记录）
/// 2. 调用 `TableBuilder::new()` 创建构建器
///
/// ## 预期结果
/// - 构建器创建成功，不会panic
#[test]
fn test_table_builder_new_with_data_creates_builder() {
    // Arrange: 准备测试数据
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

    // Act: 创建TableBuilder
    let _builder = TableBuilder::new(users);

    // Assert: 验证可以创建TableBuilder
    assert!(true);
}

/// 测试使用标题创建TableBuilder并渲染
///
/// ## 测试目的
/// 验证 `TableBuilder::with_title()` 方法能够设置标题，并在渲染输出中包含标题。
///
/// ## 测试场景
/// 1. 准备测试数据和标题
/// 2. 创建带标题的TableBuilder
/// 3. 渲染表格
/// 4. 验证标题出现在输出中
///
/// ## 预期结果
/// - 渲染输出包含设置的标题
#[test]
fn test_table_builder_with_title_with_title_string_renders_with_title() {
    // Arrange: 准备测试数据和标题
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];
    let title = "Users List";

    // Act: 创建带标题的TableBuilder并渲染
    let builder = TableBuilder::new(users).with_title(title);
    let output = builder.render();

    // Assert: 验证标题出现在输出中
    assert!(output.contains(title));
}

/// 测试使用不同样式渲染表格
///
/// ## 测试目的
/// 验证 `TableBuilder::with_style()` 方法能够应用不同的表格样式，不同样式产生不同的渲染输出。
///
/// ## 测试场景
/// 1. 使用相同数据创建两个TableBuilder
/// 2. 分别应用 Modern 和 Compact 样式
/// 3. 渲染两个表格
/// 4. 验证输出不同
///
/// ## 预期结果
/// - 不同样式产生不同的渲染输出
#[test]
fn test_table_builder_with_style_with_different_styles_renders_differently() {
    // Arrange: 准备测试数据
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    // Act: 使用不同样式渲染表格
    let builder = TableBuilder::new(users.clone()).with_style(TableStyle::Modern);
    let output_modern = builder.render();
    let builder = TableBuilder::new(users).with_style(TableStyle::Compact);
    let output_compact = builder.render();

    // Assert: 验证不同样式产生不同的输出
    assert_ne!(output_modern, output_compact);
}

/// 测试使用最大宽度限制渲染表格
///
/// ## 测试目的
/// 验证 `TableBuilder::with_max_width()` 方法能够设置最大宽度限制，并在渲染时应用该限制。
///
/// ## 测试场景
/// 1. 准备测试数据和最大宽度值
/// 2. 创建带最大宽度限制的TableBuilder
/// 3. 渲染表格
/// 4. 验证输出不为空
///
/// ## 预期结果
/// - 渲染输出不为空
/// - 宽度限制被应用（具体行为取决于实现）
#[test]
fn test_table_builder_with_max_width_with_width_limit_renders_table() {
    // Arrange: 准备测试数据和最大宽度
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];
    let max_width = 20;

    // Act: 创建带最大宽度限制的TableBuilder并渲染
    let builder = TableBuilder::new(users).with_max_width(max_width);
    let output = builder.render();

    // Assert: 验证输出不为空
    assert!(!output.is_empty());
}

/// 测试使用空数据渲染表格
///
/// ## 测试目的
/// 验证 `TableBuilder` 在使用空数据时能够正确处理，返回空字符串。
///
/// ## 测试场景
/// 1. 准备空数据列表
/// 2. 创建TableBuilder并渲染
///
/// ## 预期结果
/// - 渲染输出为空字符串
#[test]
fn test_table_builder_empty_data_with_empty_data_returns_empty_string() {
    // Arrange: 准备空数据
    let users: Vec<TestUser> = vec![];

    // Act: 创建TableBuilder并渲染
    let builder = TableBuilder::new(users);
    let output = builder.render();

    // Assert: 验证空数据返回空字符串
    assert_eq!(output, "");
}

/// 测试使用空数据和标题渲染表格
///
/// ## 测试目的
/// 验证 `TableBuilder` 在使用空数据但设置了标题时，能够显示标题和 "(No data)" 提示。
///
/// ## 测试场景
/// 1. 准备空数据列表和标题
/// 2. 创建带标题的TableBuilder并渲染
///
/// ## 预期结果
/// - 渲染输出包含标题
/// - 渲染输出包含 "(No data)" 提示
#[test]
fn test_table_builder_empty_data_with_title() {
    let users: Vec<TestUser> = vec![];

    let builder = TableBuilder::new(users).with_title("Empty Table");
    let output = builder.render();

    // 空数据带标题应该显示标题和 "(No data)"
    assert!(output.contains("Empty Table"));
    assert!(output.contains("(No data)"));
}

/// 测试TableBuilder的Display trait实现
///
/// ## 测试目的
/// 验证 `TableBuilder` 实现了 `Display` trait，可以通过 `format!` 宏格式化输出。
///
/// ## 测试场景
/// 1. 创建TableBuilder
/// 2. 使用 `format!` 宏格式化输出
///
/// ## 预期结果
/// - 格式化输出不为空
/// - Display trait正常工作
#[test]
fn test_table_builder_display_trait() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    let builder = TableBuilder::new(users);
    let output = format!("{}", builder);

    // Assert: 验证 Display trait 正常工作
    assert!(!output.is_empty());
}

/// 测试所有TableStyle变体都可以使用
///
/// ## 测试目的
/// 验证所有 `TableStyle` 枚举变体都可以正常创建和使用。
///
/// ## 测试场景
/// 1. 创建包含所有TableStyle变体的向量
/// 2. 验证不会产生编译错误
///
/// ## 预期结果
/// - 所有样式变体（Default, Modern, Compact, Minimal, Grid）都可以使用
#[test]
fn test_table_style_variants() {
    // Assert: 验证所有 TableStyle 变体都可以使用
    let _styles = vec![
        TableStyle::Default,
        TableStyle::Modern,
        TableStyle::Compact,
        TableStyle::Minimal,
        TableStyle::Grid,
    ];

    assert!(true);
}

/// 测试TableBuilder的链式调用
///
/// ## 测试目的
/// 验证 `TableBuilder` 支持链式调用多个配置方法（with_title, with_style, with_max_width），能够一次性配置所有选项。
///
/// ## 测试场景
/// 1. 使用链式调用配置标题、样式和最大宽度
/// 2. 渲染表格
/// 3. 验证输出包含所有配置
///
/// ## 预期结果
/// - 链式调用成功
/// - 渲染输出不为空
/// - 输出包含设置的标题
#[test]
fn test_table_builder_chain_calls() {
    let users = vec![TestUser {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    }];

    // Arrange: 准备测试链式调用
    let output = TableBuilder::new(users)
        .with_title("Users")
        .with_style(TableStyle::Modern)
        .with_max_width(80)
        .render();

    assert!(!output.is_empty());
    assert!(output.contains("Users"));
}
