use toolkit::log_debug;

use crate::util::safe_push;

/// Push 命令
pub struct PushCommand;

impl Default for PushCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PushCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log_debug!("push: start");
        safe_push(None, false)?;
        log_debug!("push: done");
        Ok(())
    }
}
