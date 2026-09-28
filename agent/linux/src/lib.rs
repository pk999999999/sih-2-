use std::fs;

use anyhow::Context;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LinuxProcess {
    pub pid: u32,
    pub name: String,
}

pub fn collect_processes() -> anyhow::Result<Vec<LinuxProcess>> {
    let mut processes = Vec::new();
    for entry in fs::read_dir("/proc").context("read /proc")? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(pid) = name.to_string_lossy().parse::<u32>().ok() else {
            continue;
        };
        let name = fs::read_to_string(entry.path().join("comm"))
            .unwrap_or_default()
            .trim()
            .to_string();
        processes.push(LinuxProcess { pid, name });
    }
    Ok(processes)
}

pub fn collect_system_info() -> serde_json::Value {
    serde_json::json!({
        "os": "linux",
        "hostname": fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default().trim(),
        "kernel": fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default().trim()
    })
}

pub fn collect_network_connections() -> anyhow::Result<serde_json::Value> {
    let mut rows = Vec::new();
    for (protocol, file) in [
        ("tcp", "/proc/net/tcp"),
        ("tcp6", "/proc/net/tcp6"),
        ("udp", "/proc/net/udp"),
        ("udp6", "/proc/net/udp6"),
    ] {
        let contents = fs::read_to_string(file).with_context(|| format!("read {file}"))?;
        for line in contents.lines().skip(1).take(2000) {
            let columns: Vec<_> = line.split_whitespace().collect();
            if columns.len() > 3 {
                rows.push(serde_json::json!({
                    "protocol": protocol,
                    "local_address_raw": columns[1],
                    "remote_address_raw": columns[2],
                    "state_raw": columns[3]
                }));
            }
        }
    }
    Ok(serde_json::json!({"connections": rows, "source": "/proc/net"}))
}
