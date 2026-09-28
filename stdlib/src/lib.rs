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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorized_file_and_log_collection() {
        let root = std::env::current_dir().unwrap();
        std::env::set_var("JOCKY_READ_ROOTS", &root);
        let path = root.join("Cargo.toml").to_string_lossy().into_owned();
        let arg = [Value::String(path)];
        let metadata = invoke("filesystem.metadata", &arg).unwrap();
        assert!(metadata["size"].as_u64().unwrap() > 0);
        let digest = invoke("filesystem.hash", &arg).unwrap();
        assert_eq!(digest["digest"].as_str().unwrap().len(), 64);
        let log = invoke("logs.read", &arg).unwrap();
        assert!(!log["lines"].as_array().unwrap().is_empty());
        std::env::remove_var("JOCKY_READ_ROOTS");
    }
}
