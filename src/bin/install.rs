use anyhow::Result;
use workflow::commands::install::InstallCommand;

fn main() -> Result<()> {
    InstallCommand::install()?;
    Ok(())
}

