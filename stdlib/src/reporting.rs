use anyhow::Result;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn summary(value: &Value) -> Result<Value> {
    let data = serde_json::to_vec(value)?;
    let record_count = match value {
        Value::Array(rows) => rows.len(),
        Value::Object(map) => map.get("events").and_then(Value::as_array).map(Vec::len).unwrap_or(1),
        Value::Null => 0,
        _ => 1,
    };
    Ok(json!({"record_count": record_count, "sha256": format!("{:x}", Sha256::digest(data))}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_array_records() {
        assert_eq!(summary(&json!([1,2])).unwrap()["record_count"], 2);
    }
}
