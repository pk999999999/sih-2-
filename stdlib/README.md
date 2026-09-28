# JOCKY Standard Library

The `jocky-stdlib` Rust crate implements read-only forensic capabilities. The semantic analyzer allow-lists and type-checks these functions.

## Namespaces

- `system.info()` - host metadata.
- `process.list()` - process inventory.
- `process.modules(pid)` - executable mappings or loaded modules visible for an authorized process ID.
- `filesystem.metadata(path)` - metadata without modifying file contents.
- `filesystem.hash(path)` - SHA-256 file hashing.
- `network.connections()` - local connection table.
- `logs.read(path)` - bounded UTF-8 log lines from an authorized file on either platform.
- `logs.syslog(path)` - bounded syslog lines as event records, without inventing timestamps.
- `logs.journal()` - last 100 Linux journal entries via `journalctl`.
- `logs.windows_events(channel)` - last 100 Windows events from `System`, `Application`, or `Security` via `wevtutil`.
- `hashing.sha256(value)` - digest helper.
- `timeline.build(binding)` - RFC 3339 timestamp sorting; undated/invalid records are counted and omitted.
- `reporting.summary(binding)` - record count and SHA-256 digest of JSON evidence.

`filesystem` and file-based `logs` calls require `JOCKY_READ_ROOTS` to name authorized directories. They open files relative to directory capabilities and read through the resulting handles, which prevents symlink escape from a configured root. This is not a substitute for OS permissions or a process sandbox. Native journal and event-channel APIs obey OS access controls. `hashing.sha256` hashes a DSL string literal, while `timeline` and `reporting` take a previously collected binding. Process, module, and network snapshots carry observation timestamps; undated syslog lines are not silently assigned an event time.

No standard-library function bypasses security controls, disables products, persists code, steals secrets, exploits privileges, or performs destructive changes.
