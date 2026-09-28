use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use anyhow::Result;
use serde_json::{json, Value};

use crate::filesystem::checked_file;

pub fn read(path: &str) -> Result<Value> {
    let (path, _) = checked_file(path)?;
    let file = File::open(&path)?;
    let reader = BufReader::new(file.take(1024 * 1024));
    let lines = reader.lines().take(1000).collect::<std::io::Result<Vec<_>>>()?;
    let truncated = lines.len() == 1000;
    Ok(json!({"path": path, "lines": lines, "truncated": truncated}))
}
