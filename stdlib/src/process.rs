use anyhow::{bail, Result};
use serde_json::{json, Value};

fn observed_at() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp formatting")
}

pub fn list() -> Result<Value> {
    #[cfg(target_os = "linux")]
    {
        let timestamp = observed_at();
        return Ok(json!(jocky_agent_linux::collect_processes()?.into_iter()
            .map(|process| json!({"pid": process.pid, "name": process.name, "timestamp": timestamp}))
            .collect::<Vec<_>>()));
    }
    #[cfg(target_os = "windows")]
    {
        let timestamp = observed_at();
        return Ok(json!(jocky_agent_windows::collect_processes()?.into_iter()
            .map(|process| json!({"pid": process.pid, "name": process.name, "timestamp": timestamp}))
            .collect::<Vec<_>>()));
    }
    #[allow(unreachable_code)]
    bail!("unsupported platform")
}

pub fn modules(pid: u32) -> Result<Value> {
    #[cfg(target_os = "linux")]
    { return linux_modules(pid); }
    #[cfg(target_os = "windows")]
    { return windows_modules(pid); }
    #[allow(unreachable_code)]
    bail!("unsupported platform")
}

#[cfg(target_os = "linux")]
fn linux_modules(pid: u32) -> Result<Value> {
    use std::collections::BTreeSet;
    use std::io::{BufRead, BufReader, Read};

    let path = format!("/proc/{pid}/maps");
    let file = std::fs::File::open(&path)?;
    let reader = BufReader::new(file.take(4 * 1024 * 1024));
    let mut paths = BTreeSet::new();
    for line in reader.lines().take(20_000) {
        let line = line?;
        let mut fields = line.split_whitespace();
        let _range = fields.next();
        let executable = fields.next().is_some_and(|permissions| permissions.contains('x'));
        let pathname = line.find('/').map(|index| &line[index..]);
        if executable {
            if let Some(pathname) = pathname.filter(|value| value.starts_with('/')) {
                paths.insert(pathname.to_string());
            }
        }
        if paths.len() >= 4096 { break; }
    }
    let timestamp = observed_at();
    Ok(json!({"pid": pid, "modules": paths.into_iter().map(|path| json!({"path": path, "timestamp": timestamp})).collect::<Vec<_>>(), "source": path}))
}

#[cfg(target_os = "windows")]
fn windows_modules(pid: u32) -> Result<Value> {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W,
        TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };
    if snapshot == INVALID_HANDLE_VALUE { return Err(std::io::Error::last_os_error().into()); }
    let result = (|| -> Result<Value> {
        let mut entry: MODULEENTRY32W = unsafe { zeroed() };
        entry.dwSize = size_of::<MODULEENTRY32W>() as u32;
        if unsafe { Module32FirstW(snapshot, &mut entry) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let mut modules = Vec::new();
        let timestamp = observed_at();
        loop {
            let end = entry.szExePath.iter().position(|unit| *unit == 0).unwrap_or(entry.szExePath.len());
            let path = String::from_utf16_lossy(&entry.szExePath[..end]);
            modules.push(json!({"path": path, "size": entry.modBaseSize, "timestamp": timestamp}));
            if modules.len() >= 4096 || unsafe { Module32NextW(snapshot, &mut entry) } == 0 { break; }
        }
        Ok(json!({"pid": pid, "modules": modules, "source": "toolhelp32"}))
    })();
    unsafe { CloseHandle(snapshot) };
    result
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn lists_current_process_modules() {
        let value = super::modules(std::process::id()).unwrap();
        assert!(!value["modules"].as_array().unwrap().is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn lists_current_process_modules() {
        let value = super::modules(std::process::id()).unwrap();
        assert!(!value["modules"].as_array().unwrap().is_empty());
    }
}
