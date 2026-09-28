use std::collections::BTreeMap;

use jocky_ast::Predicate;
use jocky_ir::{IrOp, IrProgram};
use serde_json::{json, Value};
use time::OffsetDateTime;

#[derive(Debug, Default)]
pub struct Runtime {
    bindings: BTreeMap<String, Value>,
}

impl Runtime {
    pub fn execute(&mut self, ir: &IrProgram) -> Value {
        let mut reports = Vec::new();
        for investigation in &ir.investigations {
            self.bindings.clear();
            for op in &investigation.ops {
                match op {
                    IrOp::Collect {
                        binding,
                        capability,
                        args,
                    } => {
                        let value = safe_collect(capability, args);
                        self.bindings.insert(binding.clone(), value);
                    }
                    IrOp::Filter { source, destination, predicate } => {
                        let value = self.bindings.get(source).cloned().unwrap_or(Value::Null);
                        self.bindings.insert(destination.clone(), filter_rows(&value, predicate));
                    }
                    IrOp::Report { name, includes } => {
                        let mut included = BTreeMap::new();
                        for include in includes {
                            included.insert(
                                include.clone(),
                                self.bindings.get(include).cloned().unwrap_or(Value::Null),
                            );
                        }
                        reports.push(json!({
                            "name": name,
                            "generated_at": OffsetDateTime::now_utc().to_string(),
                            "evidence": included
                        }));
                    }
                }
            }
        }
        json!({ "reports": reports })
    }
}

fn filter_rows(value: &Value, predicate: &Predicate) -> Value {
    let matches = |row: &Value| {
        let (field, expected, contains) = match predicate {
            Predicate::Contains { field, value } => (field, value, true),
            Predicate::Equals { field, value } => (field, value, false),
        };
        let actual = match row.get(field) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            _ => return false,
        };
        if contains {
            actual.to_lowercase().contains(&expected.to_lowercase())
        } else {
            actual.eq_ignore_ascii_case(expected)
        }
    };
    match value {
        Value::Array(rows) => Value::Array(rows.iter().filter(|row| matches(row)).cloned().collect()),
        Value::Object(map) => {
            let mut result = map.clone();
            for key in ["processes", "connections"] {
                if let Some(Value::Array(rows)) = map.get(key) {
                    result.insert(key.to_string(), Value::Array(rows.iter().filter(|row| matches(row)).cloned().collect()));
                    return Value::Object(result);
                }
            }
            Value::Array(if matches(value) { vec![value.clone()] } else { vec![] })
        }
        _ => Value::Array(Vec::new()),
    }
}

fn safe_collect(capability: &str, _args: &[String]) -> Value {
    match capability {
        "system.info" => local_system_info(),
        "process.list" => local_processes(),
        "network.connections" => local_connections(),
        _ => json!({"error": "unsupported capability"}),
    }
}

#[cfg(target_os = "linux")]
fn local_system_info() -> Value { jocky_agent_linux::collect_system_info() }
#[cfg(target_os = "windows")]
fn local_system_info() -> Value { jocky_agent_windows::collect_system_info() }
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn local_system_info() -> Value { json!({"error": "unsupported platform"}) }

#[cfg(target_os = "linux")]
fn local_processes() -> Value { jocky_agent_linux::collect_processes().map(|rows| json!(rows)).unwrap_or_else(|error| json!({"error": error.to_string()})) }
#[cfg(target_os = "windows")]
fn local_processes() -> Value { jocky_agent_windows::collect_processes().map(|rows| json!(rows)).unwrap_or_else(|error| json!({"error": error.to_string()})) }
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn local_processes() -> Value { json!({"error": "unsupported platform"}) }

#[cfg(target_os = "linux")]
fn local_connections() -> Value { jocky_agent_linux::collect_network_connections().unwrap_or_else(|error| json!({"error": error.to_string()})) }
#[cfg(target_os = "windows")]
fn local_connections() -> Value { jocky_agent_windows::collect_network_connections().unwrap_or_else(|error| json!({"error": error.to_string()})) }
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn local_connections() -> Value { json!({"error": "unsupported platform"}) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_matching_process_names() {
        let input = json!([
            {"pid": 1, "name": "powershell.exe"},
            {"pid": 2, "name": "explorer.exe"}
        ]);
        let predicate = Predicate::Contains { field: "name".into(), value: "PowerShell".into() };
        assert_eq!(filter_rows(&input, &predicate), json!([{"pid": 1, "name": "powershell.exe"}]));
    }
}
