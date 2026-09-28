# JOCKY Language Overview

JOCKY programs are investigation plans. Each plan selects an authorized target, collects evidence through allow-listed forensic calls, optionally analyzes collected bindings, and emits reports.

```jocky
investigation "Name" {
  target host("localhost")
  collect system.info() as sys
  analyze sys where os equals "windows"
  report "name" {
    include sys
  }
}
```

## Safety

Semantic analysis rejects unsupported calls. Investigation titles, report names, and filter values may describe any threat without triggering a false positive. The runtime uses local read-only collectors for `system.info`, `process.list`, and `network.connections`.

`analyze source where field contains "value"` and `equals` filter collected JSON records case-insensitively. The result is bound as `source_analysis` and can be included in a report.
