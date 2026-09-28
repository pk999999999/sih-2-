# JOCKY Standard Library

The standard library documents safe, read-only forensic capabilities. The semantic analyzer allow-lists these functions.

## Namespaces

- `system.info()` - host metadata.
- `process.list()` - process inventory.
- `process.modules(pid)` - loaded module inventory for authorized local inspection.
- `filesystem.metadata(path)` - metadata without modifying file contents.
- `filesystem.hash(path)` - SHA-256 file hashing.
- `network.connections()` - local connection table.
- `logs.windows_events(channel)` - Windows event retrieval.
- `logs.syslog(path)` - Linux syslog parsing.
- `hashing.sha256(value)` - digest helper.
- `timeline.build(items)` - timeline normalization.
- `reporting.summary(items)` - report helper.

No standard-library function bypasses security controls, disables products, persists code, steals secrets, exploits privileges, or performs destructive changes.

