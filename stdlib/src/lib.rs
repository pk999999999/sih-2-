use anyhow::{bail, Result};
use serde_json::Value;

pub mod filesystem;
pub mod hashing;
pub mod logs;
pub mod network;
pub mod process;
pub mod reporting;
pub mod system;
pub mod timeline;

pub fn invoke(capability: &str, args: &[Value]) -> Result<Value> {
    match (capability, args) {
        ("system.info", []) => system::info(),
        ("process.list", []) => process::list(),
        ("network.connections", []) => network::connections(),
        ("filesystem.metadata", [Value::String(path)]) => filesystem::metadata(path),
        ("filesystem.hash", [Value::String(path)]) => filesystem::hash(path),
        ("logs.read", [Value::String(path)]) => logs::read(path),
        ("hashing.sha256", [Value::String(value)]) => Ok(hashing::sha256(value)),
        ("timeline.build", [value]) => timeline::build(value),
        ("reporting.summary", [value]) => reporting::summary(value),
        _ => bail!("unsupported capability or arguments: {capability}"),
    }
}
