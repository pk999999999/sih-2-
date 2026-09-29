# JOCKY language specification (prototype 0.1)

JOCKY is a bounded, declarative forensic DSL, not Rust and not a general-purpose
programming language. It does not expose arbitrary Rust standard-library APIs.
Source is UTF-8. Identifiers use ASCII letters, digits and underscores, starting
with a letter or underscore. Keywords are case-sensitive. `//` starts a comment.
Statements do not end in semicolons. Strings support `\\`, `\"`, `\n`, `\r`, `\t`.
Integer literals are non-negative signed 64-bit values; booleans are `true/false`.

```text
program       = investigation+
investigation = "investigation" STRING "{" target statement* "}"
target        = "target" ("host" | "agent") "(" STRING ")"
statement     = collect | analyze | report
collect       = "collect" IDENT "." IDENT "(" arguments? ")" "as" IDENT
arguments     = (STRING | INTEGER | BOOLEAN | IDENT) ("," arguments)?
analyze       = "analyze" IDENT "where" IDENT ("contains" | "equals") STRING
report        = "report" STRING "{" ("include" IDENT)* "}"
```

Each investigation has an independent binding scope. Collection aliases cannot
be reused. Bindings must exist before use. Analyze creates `<source>_analysis`.
Contains and equals compare scalar values case-insensitively. Collection errors
stop execution; they do not produce fabricated evidence. Targets are enforced:
local CLI accepts localhost, 127.0.0.1 or the current hostname; agent targets must
use authorized API jobs rather than silently collecting from the CLI host.

## Capability signatures

| Capability | Arguments | Result |
|---|---|---|
| system.info | none | host object |
| process.list | none | process array |
| process.modules | positive u32 PID | module collection |
| network.connections | none | connection collection |
| filesystem.metadata / filesystem.hash | path string | metadata / SHA-256 |
| logs.read / logs.syslog | path string | bounded lines / events |
| logs.journal | none | Linux journal events |
| logs.windows_events | System, Application or Security | Windows events |
| hashing.sha256 | string | SHA-256 value |
| timeline.build | binding | timestamp-sorted events |
| reporting.summary | binding | counts and digest |

Filesystem/log reads require `JOCKY_READ_ROOTS`; paths are resolved beneath
authorized directories using capability-based opens. Platform-specific APIs
fail explicitly on unsupported systems. Collection limits and omitted/truncated
flags are evidence semantics: see existing stdlib implementation and tests.

## Mock execution and findings

`jocky-cli run examples/mock_investigation.jky --mock` or
`JOCKY_MOCK_MODE=true` uses fixed synthetic host/process/network/log fixtures.
Mock mode never falls back to real file or process-module reads. Such unsupported
mock calls fail. Hashing, timeline and summaries operate on their supplied values.
Reports carry evidence digests, and output includes `mock` and `findings`.

Automatic rules: miner in process name HIGH (0.75), temporary path MEDIUM (0.45),
remote port 3333 or 4444 HIGH (0.55), failed-password/login or event 4625 MEDIUM
(0.40). Confidence is a heuristic score, not a calibrated probability. Indicators
use cautious wording and are never instructions to terminate a process.

## CLI

`check FILE` validates; `compile FILE --format json|plan|llvm` emits IR;
`run FILE [--mock]` interprets; `build FILE --output PATH` invokes clang and Cargo
to link native code to the Rust runtime. Native build requires a source checkout,
Rust and clang. Cross-compilation/toolchain provisioning is not automatic.
