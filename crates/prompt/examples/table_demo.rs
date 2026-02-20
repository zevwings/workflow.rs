//! Table rendering example.
//!
//! Run: `cargo run -p prompt --example table_demo`

use prompt::{table, Alignment, TableBuilder, TableStyle};

fn main() -> prompt::Result<()> {
    println!("Table rendering demonstration");
    println!("=================");
    println!();
    println!("This example demonstrates how to use Table to display data in a tabular format.");
    println!("Press Ctrl+C or Esc to cancel the table.");
    println!();

    demo_basic_table()?;
    demo_table_styles()?;
    demo_alignment()?;
    demo_table_with_title()?;
    demo_real_world_usage()?;

    println!("\n=== All demos completed ===");
    Ok(())
}

fn demo_basic_table() -> prompt::Result<()> {
    println!("\n=== Demo 1: Basic table ===\n");

    let t = table(vec!["Name", "Age", "City"])
        .add_row(vec!["Alice", "30", "New York"])
        .add_row(vec!["Bob", "25", "London"])
        .add_row(vec!["Charlie", "35", "Tokyo"]);

    t.print()?;

    Ok(())
}

fn demo_table_styles() -> prompt::Result<()> {
    println!("\n=== Demo 2: Table styles ===\n");

    let headers = vec!["Product", "Price", "Stock"];
    let rows = vec![
        vec!["Apple", "$1.00", "100"],
        vec!["Banana", "$0.50", "200"],
        vec!["Orange", "$0.75", "150"],
    ];

    fn print_with_style(headers: &[&str], rows: &[Vec<&str>], style: TableStyle) -> prompt::Result<()> {
        let mut t = TableBuilder::new(headers.to_vec());
        for row in rows {
            t = t.add_row(row.clone());
        }
        t.with_style(style).print()
    }

    println!("Modern style:");
    print_with_style(&headers, &rows, TableStyle::Modern)?;
    println!();
    println!("Compact style:");
    print_with_style(&headers, &rows, TableStyle::Compact)?;
    println!();
    println!("Minimal style:");
    print_with_style(&headers, &rows, TableStyle::Minimal)?;

    Ok(())
}

fn demo_alignment() -> prompt::Result<()> {
    println!("\n=== Demo 3: Alignment ===\n");

    println!("Left alignment:");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_alignment(Alignment::Left)
        .print()?;

    println!();

    println!("Center alignment:");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_alignment(Alignment::Center)
        .print()?;

    println!();

    println!("Right alignment:");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_alignment(Alignment::Right)
        .print()?;

    println!();

    println!("Column level alignment (name left, quantity center, price right):");
    table(vec!["Item", "Quantity", "Price"])
        .add_row(vec!["Widget", "10", "$5.00"])
        .add_row(vec!["Gadget", "5", "$10.00"])
        .with_column_alignments(vec![Alignment::Left, Alignment::Center, Alignment::Right])
        .print()?;

    Ok(())
}

fn demo_table_with_title() -> prompt::Result<()> {
    println!("\n=== Demo 4: Table with title ===\n");

    table(vec!["Feature", "Status", "Note"])
        .add_row(vec!["Authentication", "Done", "OAuth2 implemented"])
        .add_row(vec!["Authorization", "In Progress", "RBAC pending"])
        .add_row(vec!["Logging", "Done", "Structured logs"])
        .with_title("Project Status")
        .print()?;

    Ok(())
}

fn demo_real_world_usage() -> prompt::Result<()> {
    println!("\n=== Demo 5: Real world usage ===\n");

    println!("Git branches list:");
    table(vec!["Branch", "Last Commit", "Author"])
        .add_row(vec!["main", "2 hours ago", "Alice"])
        .add_row(vec!["feature/auth", "30 minutes ago", "Bob"])
        .add_row(vec!["bugfix/login", "1 day ago", "Charlie"])
        .with_style(TableStyle::Modern)
        .print()?;

    println!();

    println!("Test results:");
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

    println!("Dependencies information:");
    table(vec!["Package", "Version", "License"])
        .add_row(vec!["crossterm", "0.27", "MIT"])
        .add_row(vec!["serde", "1.0", "MIT/Apache-2.0"])
        .add_row(vec!["tokio", "1.35", "MIT"])
        .with_style(TableStyle::Compact)
        .print()?;

    Ok(())
}
