use anyhow::Result;
use clap::Parser;
use workflow::commands::pr::update::PRUpdateCommand;

#[derive(Parser)]
#[command(name = "update")]
#[command(about = "Quick update (use PR title as commit message)", long_about = None)]
#[command(version)]
struct Cli;

fn main() -> Result<()> {
    PRUpdateCommand::update()?;
    Ok(())
}

