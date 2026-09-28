# Compiler Architecture

Pipeline:

1. `jocky-lexer` tokenizes source.
2. `jocky-parser` builds the AST.
3. `jocky-semantic` validates bindings and safety allow-lists.
4. `jocky-ir` lowers to JSON-serializable IR.
5. `jocky-codegen` emits a readable plan or executable LLVM IR.
6. `jocky-runtime` executes the same audited operations for interpreted and native runs.

LLVM lowering emits one call for each investigation boundary and each IR operation. It checks every runtime status, stops on an error, and frees the session. The linked Rust static library implements collection, filtering, and reporting. `jocky build <file> --output <path>` writes `.ll`, builds the runtime library when needed, and invokes Clang. `jocky compile <file> --format llvm` emits the IR for inspection.

Both execution modes reject nonlocal targets, propagate collector errors, and produce the same report evidence for deterministic inputs. Native code generation does not inline platform collectors into LLVM; it keeps them in the audited runtime ABI.

Each report includes a SHA-256 digest for every included evidence value. The digest covers the runtime's JSON serialization of that value, allowing an analyst to verify that a report's evidence has not changed after generation.
