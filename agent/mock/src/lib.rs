use serde_json::{json, Value};

pub const TIMESTAMP: &str = "2026-01-01T00:00:00Z";

pub fn system_info() -> Value {
    json!({"hostname":"FORENSIC-PC-001", "platform":"windows", "os":"Windows 11 (synthetic)",
           "architecture":"x86_64", "mock":true, "timestamp":TIMESTAMP})
}

pub fn processes() -> Value {
    let names = ["System", "smss.exe", "csrss.exe", "wininit.exe", "services.exe", "lsass.exe",
        "svchost.exe", "explorer.exe", "dwm.exe", "SearchIndexer.exe", "notepad.exe", "chrome.exe",
        "jocky-agent.exe", "suspicious_miner.exe", "updater.exe"];
    Value::Array(names.iter().enumerate().map(|(i, name)| json!({
        "pid":1000+i, "name":name, "path":if i >= 13 {format!("C:\\Users\\Analyst\\AppData\\Local\\Temp\\{name}")}
            else {format!("C:\\Windows\\System32\\{name}")}, "timestamp":TIMESTAMP, "mock":true
    })).collect())
}

pub fn connections() -> Value {
    Value::Array((0..10).map(|i| json!({"protocol":"tcp", "local_address":"192.0.2.10",
        "local_port":50000+i, "remote_address":if i == 8 {"198.51.100.33"} else {"203.0.113.10"},
        "remote_port":if i == 8 {3333} else if i == 9 {4444} else {443},
        "state":"ESTABLISHED", "pid":if i >= 8 {1013} else {1011}, "mock":true, "timestamp":TIMESTAMP
    })).collect())
}

pub fn logs() -> Value {
    json!({"mock":true, "events":[{"message":"Failed password for synthetic-user", "event_id":4625,
                                    "timestamp":TIMESTAMP}]})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_fixtures() {
        assert_eq!(system_info()["hostname"], "FORENSIC-PC-001");
        assert_eq!(processes().as_array().unwrap().len(), 15);
        assert_eq!(connections().as_array().unwrap().len(), 10);
        assert_eq!(processes(), processes());
        assert_eq!(connections()[8]["remote_port"], 3333);
        assert_eq!(connections()[9]["remote_port"], 4444);
    }
}
