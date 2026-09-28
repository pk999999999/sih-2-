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
    Ok(system.processes().iter().map(|(pid, process)| WindowsProcess {
        pid: pid.as_u32(),
        name: process.name().to_string_lossy().into_owned(),
    }).collect())
}

pub fn collect_network_connections() -> anyhow::Result<serde_json::Value> {
    let output = std::process::Command::new("netstat").arg("-ano").output()?;
    if !output.status.success() {
        anyhow::bail!("netstat collection failed");
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let rows: Vec<_> = text.lines()
        .filter(|line| line.trim_start().starts_with("TCP") || line.trim_start().starts_with("UDP"))
        .take(2000)
        .map(|line| line.trim().to_string())
        .collect();
    Ok(serde_json::json!({"connections": rows, "source": "netstat -ano"}))
}
