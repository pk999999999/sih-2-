# JOCKY Standard Library

The `jocky-stdlib` Rust crate implements read-only forensic capabilities. The semantic analyzer allow-lists and type-checks these functions.

## Namespaces

- `system.info()` - host metadata.
- `process.list()` - process inventory.
- `filesystem.metadata(path)` - metadata without modifying file contents.
- `filesystem.hash(path)` - SHA-256 file hashing.
- `network.connections()` - local connection table.
- `logs.read(path)` - bounded UTF-8 log lines from an authorized file on either platform.
- `hashing.sha256(value)` - digest helper.
- `timeline.build(binding)` - RFC 3339 timestamp sorting; undated/invalid records are counted and omitted.
- `reporting.summary(binding)` - record count and SHA-256 digest of JSON evidence.

`filesystem` and `logs` require `JOCKY_READ_ROOTS` to name authorized directories. This is an opt-in local read policy, not a substitute for OS permissions. `hashing.sha256` hashes a DSL string literal, while `timeline` and `reporting` take a previously collected binding.

No standard-library function bypasses security controls, disables products, persists code, steals secrets, exploits privileges, or performs destructive changes.
