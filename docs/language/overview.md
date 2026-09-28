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

Semantic analysis rejects unsupported calls and invalid argument types. Investigation titles, report names, and filter values may describe any threat without triggering a false positive. Local execution accepts `host("localhost")`, `host("127.0.0.1")`, or the current host name. Remote agent targets are declarative and cannot be run through the local CLI.

`analyze source where field contains "value"` and `equals` filter collected JSON records case-insensitively. The result is bound as `source_analysis` and can be included in a report.

Call arguments are string, integer, or boolean literals, or references to earlier collection bindings. The safe function set is listed in [the standard library](../../stdlib/README.md). File-based collection requires `JOCKY_READ_ROOTS`; collection failures stop execution rather than entering an error object into evidence.

## Grammar

```text
program       := investigation+
investigation := 'investigation' string '{' target statement* '}'
target        := 'target' ('host' | 'agent') '(' string ')'
statement     := 'collect' identifier '.' identifier '(' arguments? ')' 'as' identifier
               | 'analyze' identifier 'where' identifier ('contains' | 'equals') string
               | 'report' string '{' ('include' identifier)* '}'
arguments     := argument (',' argument)*
argument      := string | integer | 'true' | 'false' | identifier
```

Strings support `\\`, `\"`, `\n`, `\r`, and `\t` escapes. `//` starts a line comment. Bindings must be unique within an investigation and must be defined before use. A filter binds `<source>_analysis`.
