use std::collections::BTreeMap;

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
                    IrOp::Filter { source, predicate } => {
                        let value = self.bindings.get(source).cloned().unwrap_or(Value::Null);
                        self.bindings.insert(
                            format!("{source}_analysis"),
                            json!({"source": source, "predicate": predicate, "input": value}),
                        );
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
fn local_processes() -> Value { json!(jocky_agent_linux::collect_processes().unwrap_or_default()) }
#[cfg(target_os = "windows")]
fn local_processes() -> Value { json!(jocky_agent_windows::collect_processes().unwrap_or_default()) }
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn local_processes() -> Value { json!({"error": "unsupported platform"}) }

#[cfg(target_os = "linux")]
fn local_connections() -> Value { jocky_agent_linux::collect_network_connections().unwrap_or_else(|error| json!({"error": error.to_string()})) }
#[cfg(target_os = "windows")]
fn local_connections() -> Value { jocky_agent_windows::collect_network_connections().unwrap_or_else(|error| json!({"error": error.to_string()})) }
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn local_connections() -> Value { json!({"error": "unsupported platform"}) }
