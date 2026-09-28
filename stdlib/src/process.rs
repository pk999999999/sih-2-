use anyhow::{bail, Result};
use serde_json::Value;

pub fn list() -> Result<Value> {
    #[cfg(target_os = "linux")]
    { return Ok(serde_json::to_value(jocky_agent_linux::collect_processes()?)?); }
    #[cfg(target_os = "windows")]
    { return Ok(serde_json::to_value(jocky_agent_windows::collect_processes()?)?); }
    #[allow(unreachable_code)]
    bail!("unsupported platform")
}
