use anyhow::{bail, Result};
use serde_json::Value;

pub fn connections() -> Result<Value> {
    #[cfg(target_os = "linux")]
    { return jocky_agent_linux::collect_network_connections(); }
    #[cfg(target_os = "windows")]
    { return jocky_agent_windows::collect_network_connections(); }
    #[allow(unreachable_code)]
    bail!("unsupported platform")
}
