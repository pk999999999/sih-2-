use anyhow::{bail, Result};
use serde_json::Value;

pub fn info() -> Result<Value> {
    #[cfg(target_os = "linux")]
    { return Ok(jocky_agent_linux::collect_system_info()); }
    #[cfg(target_os = "windows")]
    { return Ok(jocky_agent_windows::collect_system_info()); }
    #[allow(unreachable_code)]
    bail!("unsupported platform")
}
