# Compiler Architecture

Pipeline:

1. `jocky-lexer` tokenizes source.
2. `jocky-parser` builds the AST.
3. `jocky-semantic` validates bindings and safety allow-lists.
4. `jocky-ir` lowers to JSON-serializable IR.
5. `jocky-codegen` emits an execution plan or LLVM placeholder module.
6. `jocky-runtime` interprets safe IR operations.

The LLVM backend is structured behind `jocky-codegen`. In this prototype it emits an import-oriented module skeleton when built with the `llvm` feature. Full native lowering should keep forensic collection in audited runtime calls rather than embedding unsafe endpoint logic.

