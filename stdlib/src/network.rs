use anyhow::{bail, Result};
use serde_json::Value;

pub fn connections() -> Result<Value> {
    #[cfg(target_os = "linux")]
    { return with_timestamp(jocky_agent_linux::collect_network_connections()?); }
    #[cfg(target_os = "windows")]
    { return with_timestamp(jocky_agent_windows::collect_network_connections()?); }
    #[allow(unreachable_code)]
    bail!("unsupported platform")
}

fn with_timestamp(mut value: Value) -> Result<Value> {
    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)?;
    if let Some(rows) = value.get_mut("connections").and_then(Value::as_array_mut) {
        for row in rows {
            if let Some(map) = row.as_object_mut() {
                map.insert("timestamp".into(), Value::String(timestamp.clone()));
            }
        }
    }
    Ok(value)
}
