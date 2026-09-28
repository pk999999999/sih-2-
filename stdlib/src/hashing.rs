use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn sha256(value: &str) -> Value {
    json!({"algorithm": "sha256", "digest": format!("{:x}", Sha256::digest(value.as_bytes()))})
}

#[cfg(test)]
mod tests {
    #[test]
    fn known_digest() {
        assert_eq!(super::sha256("abc")["digest"], "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
