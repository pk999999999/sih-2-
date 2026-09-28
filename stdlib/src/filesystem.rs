use std::fs::{self, File};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) fn checked_file(path: &str) -> Result<(PathBuf, fs::Metadata)> {
    let roots = std::env::var_os("JOCKY_READ_ROOTS")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();
    checked_file_in_roots(path, &roots)
}

fn checked_file_in_roots(path: &str, roots: &[PathBuf]) -> Result<(PathBuf, fs::Metadata)> {
    if roots.is_empty() {
        bail!("JOCKY_READ_ROOTS must contain an authorized directory for file and log access");
    }
    let canonical = Path::new(path).canonicalize().with_context(|| format!("resolve {path}"))?;
    let allowed = roots.iter().filter_map(|root| root.canonicalize().ok()).any(|root| canonical.starts_with(root));
    if !allowed {
        bail!("path is outside authorized read roots");
    }
    let metadata = fs::metadata(&canonical)?;
    if !metadata.is_file() {
        bail!("only regular files may be inspected");
    }
    if metadata.len() > MAX_FILE_BYTES {
        bail!("file exceeds 64 MiB collection limit");
    }
    Ok((canonical, metadata))
}

pub fn metadata(path: &str) -> Result<Value> {
    let (path, metadata) = checked_file(path)?;
    Ok(json!({
        "path": path,
        "size": metadata.len(),
        "readonly": metadata.permissions().readonly(),
        "modified_unix_seconds": metadata.modified().ok().and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok()).map(|duration| duration.as_secs())
    }))
}

pub fn hash(path: &str) -> Result<Value> {
    let (path, metadata) = checked_file(path)?;
    let mut file = File::open(&path)?;
    let mut hasher = Sha256::new();
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 { break; }
        copied += count as u64;
        if copied > MAX_FILE_BYTES { bail!("file changed during collection and exceeded limit"); }
        hasher.update(&buffer[..count]);
    }
    let current_len = file.seek(std::io::SeekFrom::End(0))?;
    if current_len != metadata.len() || copied != metadata.len() {
        bail!("file changed during collection");
    }
    Ok(json!({"path": path, "size": copied, "algorithm": "sha256", "digest": format!("{:x}", hasher.finalize())}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denies_without_authorized_root() {
        assert!(checked_file_in_roots("Cargo.toml", &[]).is_err());
    }

    #[test]
    fn denies_path_outside_root() {
        let root = std::env::current_dir().unwrap();
        let parent = root.parent().unwrap();
        assert!(checked_file_in_roots("Cargo.toml", &[parent.join("not-a-real-root")]).is_err());
    }
}
