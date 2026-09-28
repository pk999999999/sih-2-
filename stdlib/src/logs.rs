use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use anyhow::{bail, Result};
use serde_json::{json, Value};

use crate::filesystem::checked_file;

pub fn read(path: &str) -> Result<Value> {
    let (path, _) = checked_file(path)?;
    let file = File::open(&path)?;
    let reader = BufReader::new(file.take(1024 * 1024));
    let lines = reader.lines().take(1000).collect::<std::io::Result<Vec<_>>>()?;
    let truncated = lines.len() == 1000;
    Ok(json!({"path": path, "lines": lines, "truncated": truncated}))
}

pub fn syslog(path: &str) -> Result<Value> {
    let content = read(path)?;
    let lines = content["lines"].as_array().expect("logs.read returns lines");
    let events = lines.iter().filter_map(Value::as_str).map(|line| json!({
        "message": line,
        "source": "syslog"
    })).collect::<Vec<_>>();
    Ok(json!({"path": content["path"], "events": events, "truncated": content["truncated"]}))
}

#[cfg(target_os = "linux")]
pub fn journal() -> Result<Value> {
    use std::process::Command;
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    let output = Command::new("journalctl")
        .args(["--no-pager", "--output=json", "-n", "100"])
        .output()?;
    if !output.status.success() {
        bail!("journalctl failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    if output.stdout.len() > 2 * 1024 * 1024 { bail!("journal output exceeds 2 MiB limit"); }
    let mut events = Vec::new();
    for line in output.stdout.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let entry: Value = serde_json::from_slice(line)?;
        let micros = entry.get("__REALTIME_TIMESTAMP")
            .and_then(Value::as_str).and_then(|value| value.parse::<i128>().ok());
        let timestamp = micros.and_then(|value| OffsetDateTime::from_unix_timestamp_nanos(value * 1000).ok())
            .and_then(|value| value.format(&Rfc3339).ok());
        events.push(json!({
            "timestamp": timestamp,
            "message": entry.get("MESSAGE"),
            "pid": entry.get("_PID"),
            "unit": entry.get("_SYSTEMD_UNIT"),
            "priority": entry.get("PRIORITY")
        }));
    }
    let truncated = events.len() >= 100;
    Ok(json!({"events": events, "source": "journalctl", "truncated": truncated}))
}

#[cfg(not(target_os = "linux"))]
pub fn journal() -> Result<Value> { bail!("logs.journal is supported on Linux only") }

#[cfg(target_os = "windows")]
pub fn windows_events(channel: &str) -> Result<Value> {
    use std::process::Command;

    if !["System", "Application", "Security"].contains(&channel) {
        bail!("unsupported Windows event channel");
    }
    let output = Command::new("wevtutil")
        .args(["qe", channel, "/c:100", "/f:xml"])
        .output()?;
    if !output.status.success() {
        bail!("wevtutil query failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    if output.stdout.len() > 2 * 1024 * 1024 { bail!("event output exceeds 2 MiB limit"); }
    let text = decode_windows_output(&output.stdout)?;
    let events = parse_windows_events(&text)?;
    let truncated = events.len() >= 100;
    Ok(json!({"channel": channel, "events": events, "source": "wevtutil", "truncated": truncated}))
}

#[cfg(not(target_os = "windows"))]
pub fn windows_events(_channel: &str) -> Result<Value> { bail!("logs.windows_events is supported on Windows only") }

#[cfg(target_os = "windows")]
fn decode_windows_output(bytes: &[u8]) -> Result<String> {
    if bytes.starts_with(&[0xff, 0xfe]) {
        let units = bytes[2..].chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect::<Vec<_>>();
        return Ok(String::from_utf16(&units)?);
    }
    Ok(String::from_utf8(bytes.to_vec())?)
}

#[cfg(target_os = "windows")]
fn parse_windows_events(text: &str) -> Result<Vec<Value>> {
    let trimmed = text.trim_start_matches('\u{feff}').trim();
    let body = if trimmed.starts_with("<?xml") {
        trimmed.split_once("?>").map(|(_, body)| body).unwrap_or(trimmed)
    } else { trimmed };
    let wrapped = format!("<Events>{body}</Events>");
    let document = roxmltree::Document::parse(&wrapped)?;
    let mut events = Vec::new();
    for event in document.root_element().children().filter(|node| node.has_tag_name("Event")) {
        let system = event.children().find(|node| node.has_tag_name("System"));
        let field = |name| system.and_then(|node| node.children().find(|child| child.has_tag_name(name)));
        let event_id = field("EventID").and_then(|node| node.text()).and_then(|text| text.parse::<u32>().ok());
        let timestamp = field("TimeCreated").and_then(|node| node.attribute("SystemTime"));
        let provider = field("Provider").and_then(|node| node.attribute("Name"));
        let computer = field("Computer").and_then(|node| node.text());
        let data = event.descendants().filter(|node| node.has_tag_name("Data"))
            .filter_map(|node| node.text()).map(str::to_string).collect::<Vec<_>>();
        events.push(json!({"timestamp": timestamp, "event_id": event_id, "provider": provider, "computer": computer, "data": data}));
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    #[test]
    fn parses_event_xml() {
        let xml = r#"<Event><System><Provider Name="Test"/><EventID>42</EventID><TimeCreated SystemTime="2026-01-01T00:00:00Z"/></System><EventData><Data>value</Data></EventData></Event>"#;
        let events = super::parse_windows_events(xml).unwrap();
        assert_eq!(events[0]["event_id"], 42);
        assert_eq!(events[0]["data"][0], "value");
    }
}
