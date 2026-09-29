# Compiler architecture

```text
UTF-8 source -> Logos lexer -> recursive-descent parser -> AST
            -> semantic validation -> JOCKY IR -> interpreter + stdlib
                                            -> LLVM IR -> clang object
                                                       -> Rust native runner
```

The workspace separates lexer, AST, parser, semantic validation, IR, codegen,
runtime, native-runner, CLI and analysis crates. LLVM codegen emits a native
entrypoint with JSON operation constants and explicit failure branches. The Rust
runtime exposes a checked C ABI; runtime failures return nonzero status. This is
a genuine LLVM-to-object path, not an optimizing compiler for arbitrary Rust.

The interpreter and native binary share collectors and analysis rules. Each
collection is analyzed before filters; findings retain capability, binding and
row index. Evidence digests are computed when reports are materialized. Timestamps
are intentionally excluded from deterministic parity comparisons where needed.

`agent/mock` has no host-collection dependencies and uses RFC 5737 documentation
addresses. The stdlib dispatcher enforces mock isolation. The editor executes
the same CLI with fixed argv, a temporary directory, a sanitized environment,
two concurrent slots and a ten-second timeout. Its BUILD action emits LLVM IR;
native executable packaging remains a local CLI operation.

Tests cover tokenization, parsing, semantics, filters, collectors, each analysis
rule, mock counts, and Linux/Windows workspace builds. CI links and runs native
executables and compares deterministic evidence with interpreter output.
