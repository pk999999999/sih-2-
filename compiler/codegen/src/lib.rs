use jocky_ir::{IrOp, IrProgram};

pub fn emit_pseudo_native(ir: &IrProgram) -> String {
    let mut out = String::from("; JOCKY safe forensic execution plan\n");
    for investigation in &ir.investigations {
        out.push_str(&format!("investigation \"{}\"\n", investigation.title));
        for op in &investigation.ops {
            match op {
                IrOp::Collect {
                    binding,
                    capability,
                    args,
                } => out.push_str(&format!(
                    "  collect %{binding} = call @{capability}({})\n",
                    args.join(", ")
                )),
                IrOp::Filter { source, predicate } => {
                    out.push_str(&format!("  filter %{source} where {predicate}\n"));
                }
                IrOp::Report { name, includes } => {
                    out.push_str(&format!("  report \"{name}\" [{}]\n", includes.join(", ")));
                }
            }
        }
    }
    out
}

#[cfg(feature = "llvm")]
pub fn emit_llvm_ir(ir: &IrProgram) -> String {
    let mut module = String::from("; LLVM backend prototype: forensic calls remain runtime imports\n");
    module.push_str("declare i32 @jocky_runtime_execute_plan(ptr)\n");
    module.push_str("define i32 @main() {\nentry:\n");
    module.push_str(&format!("  ; investigations: {}\n", ir.investigations.len()));
    module.push_str("  ret i32 0\n}\n");
    module
}

#[cfg(not(feature = "llvm"))]
pub fn emit_llvm_ir(_ir: &IrProgram) -> String {
    "; LLVM backend disabled in this build. Enable the `llvm` feature and link Inkwell/LLVM in a full toolchain environment.\n".into()
}

