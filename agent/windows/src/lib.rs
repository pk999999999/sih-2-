use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WindowsProcess {
    pub pid: u32,
    pub name: String,
}

pub fn collect_system_info() -> serde_json::Value {
    serde_json::json!({
        "os": "windows",
        "hostname": std::env::var("COMPUTERNAME").unwrap_or_default(),
        "arch": std::env::consts::ARCH
    })
}

pub fn collect_processes() -> anyhow::Result<Vec<WindowsProcess>> {
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
    let mut processes: Vec<_> = system.processes().iter().map(|(pid, process)| WindowsProcess {
        pid: pid.as_u32(),
        name: process.name().to_string_lossy().into_owned(),
    }).collect();
    processes.sort_by_key(|process| process.pid);
    Ok(processes)
}

pub fn collect_network_connections() -> anyhow::Result<serde_json::Value> {
    let output = std::process::Command::new("netstat").arg("-ano").output()?;
    if !output.status.success() {
        anyhow::bail!("netstat collection failed");
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let rows = text.lines().filter_map(parse_netstat_row).take(2000).collect::<Vec<_>>();
    Ok(serde_json::json!({"connections": rows, "source": "netstat -ano"}))
}

fn parse_netstat_row(line: &str) -> Option<serde_json::Value> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    match fields.as_slice() {
        [protocol, local, remote, state, pid] if protocol.eq_ignore_ascii_case("TCP") => Some(serde_json::json!({
            "protocol": "tcp", "local_address": local, "remote_address": remote,
            "state": state.to_ascii_lowercase(), "pid": pid.parse::<u32>().ok()?
        })),
        [protocol, local, remote, pid] if protocol.eq_ignore_ascii_case("UDP") => Some(serde_json::json!({
            "protocol": "udp", "local_address": local, "remote_address": remote,
            "state": "unconnected", "pid": pid.parse::<u32>().ok()?
        })),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_netstat_lines() {
        let row = super::parse_netstat_row("TCP    127.0.0.1:80    0.0.0.0:0    LISTENING    123").unwrap();
        assert_eq!(row["pid"], 123);
        assert_eq!(row["state"], "listening");
    }
}
