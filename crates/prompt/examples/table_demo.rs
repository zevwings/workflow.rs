//! 表格渲染示例
//!
//! 演示 Table 的构建和渲染功能。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example table_demo
//! ```

use prompt::{table, Alignment, TableBuilder, TableStyle};

fn main() -> prompt::Result<()> {
    println!("表格渲染功能演示");
    println!("================");
    println!();

    // 演示 1：基本表格
    demo_basic_table()?;

    // 演示 2：不同样式
    demo_table_styles()?;

    // 演示 3：对齐方式
    demo_alignment()?;

    // 演示 4：带标题的表格
    demo_table_with_title()?;

    // 演示 5：实际应用场景
    demo_real_world_usage()?;

    println!("\n=== 所有演示完成 ===");
    Ok(())
}

/// 演示 1：基本表格
fn demo_basic_table() -> prompt::Result<()> {
    println!("\n=== Demo 1: 基本表格 ===\n");

    // 使用便捷函数创建表格
    let t = table(vec!["Name", "Age", "City"])
        .add_row(vec!["Alice", "30", "New York"])
        .add_row(vec!["Bob", "25", "London"])
        .add_row(vec!["Charlie", "35", "Tokyo"]);

    t.print()?;

    Ok(())
}

/// 演示 2：不同样式
fn demo_table_styles() -> prompt::Result<()> {
    println!("\n=== Demo 2: 表格样式 ===\n");

    let headers = vec!["Product", "Price", "Stock"];
    let rows = vec![
        vec!["Apple", "$1.00", "100"],
        vec!["Banana", "$0.50", "200"],
        vec!["Orange", "$0.75", "150"],
    ];

    // Modern 样式
    println!("Modern 样式：");
    let mut t = TableBuilder::new(headers.clone());
    for row in &rows {
        t = t.add_row(row.clone());
    }
    t.with_style(TableStyle::Modern).print()?;

    println!();

    // Compact 样式
    println!("Compact 样式：");
    let mut t = TableBuilder::new(headers.clone());
    for row in &rows {
        t = t.add_row(row.clone());
    }
    t.with_style(TableStyle::Compact).print()?;

    println!();

    // Minimal 样式
    println!("Minimal 样式：");
    let mut t = TableBuilder::new(headers.clone());
    for row in &rows {
        t = t.add_row(row.clone());
    }
    t.with_style(TableStyle::Minimal).print()?;

    Ok(())
}

/// 演示 3：对齐方式
fn demo_alignment() -> prompt::Result<()> {
    println!("\n=== Demo 3: 对齐方式 ===\n");

    // 全局左对齐（默认）
    println!("左对齐：");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_alignment(Alignment::Left)
        .print()?;

    println!();

    // 全局居中对齐
    println!("居中对齐：");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_alignment(Alignment::Center)
        .print()?;

    println!();

    // 全局右对齐
    println!("右对齐：");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_alignment(Alignment::Right)
        .print()?;

    println!();

    // 列级别对齐
    println!("列级别对齐（名称左对齐，数量居中，价格右对齐）：");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_column_alignments(vec![Alignment::Left, Alignment::Center, Alignment::Right])
        .print()?;

    Ok(())
}

/// 演示 4：带标题的表格
fn demo_table_with_title() -> prompt::Result<()> {
    println!("\n=== Demo 4: 带标题的表格 ===\n");

    table(vec!["Feature", "Status", "Note"])
        .add_row(vec!["Authentication", "Done", "OAuth2 implemented"])
        .add_row(vec!["Authorization", "In Progress", "RBAC pending"])
        .add_row(vec!["Logging", "Done", "Structured logs"])
        .with_title("Project Status")
        .print()?;

    Ok(())
}

/// 演示 5：实际应用场景
fn demo_real_world_usage() -> prompt::Result<()> {
    println!("\n=== Demo 5: 实际应用场景 ===\n");

    // Git 分支列表
    println!("Git 分支列表：");
    table(vec!["Branch", "Last Commit", "Author"])
        .add_row(vec!["main", "2 hours ago", "Alice"])
        .add_row(vec!["feature/auth", "30 minutes ago", "Bob"])
        .add_row(vec!["bugfix/login", "1 day ago", "Charlie"])
        .with_style(TableStyle::Modern)
        .print()?;

    println!();

    // 测试结果
    println!("测试结果：");
    table(vec!["Test Suite", "Passed", "Failed", "Skipped"])
        .add_row(vec!["Unit Tests", "45", "2", "0"])
        .add_row(vec!["Integration Tests", "12", "0", "1"])
        .add_row(vec!["E2E Tests", "8", "1", "0"])
        .with_column_alignments(vec![
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
        ])
        .print()?;

    println!();

    // 依赖信息
    println!("依赖信息：");
    table(vec!["Package", "Version", "License"])
        .add_row(vec!["crossterm", "0.27", "MIT"])
        .add_row(vec!["serde", "1.0", "MIT/Apache-2.0"])
        .add_row(vec!["tokio", "1.35", "MIT"])
        .with_style(TableStyle::Compact)
        .print()?;

    Ok(())
}
