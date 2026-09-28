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
                    args.iter().map(|arg| format!("{arg:?}")).collect::<Vec<_>>().join(", ")
                )),
                IrOp::Filter { source, destination, predicate } => {
                    out.push_str(&format!("  filter %{destination} = %{source} where {predicate:?}\n"));
                }
                IrOp::Report { name, includes } => {
                    out.push_str(&format!("  report \"{name}\" [{}]\n", includes.join(", ")));
                }
            }
        }
    }
    out
}

pub fn emit_llvm_ir(ir: &IrProgram) -> serde_json::Result<String> {
    let plan = serde_json::to_vec(ir)?;
    let escaped = plan.iter().map(|byte| format!("\\{byte:02X}")).collect::<String>();
    let size = plan.len() + 1;
    Ok(format!(
        "; ModuleID = 'jocky'\nsource_filename = \"jocky\"\n@jocky_plan = private constant [{size} x i8] c\"{escaped}\\00\", align 1\ndeclare i32 @jocky_runtime_execute_plan(ptr, i64)\ndefine i32 @main() {{\nentry:\n  %status = call i32 @jocky_runtime_execute_plan(ptr @jocky_plan, i64 {})\n  ret i32 %status\n}}\n",
        plan.len()
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn emits_executable_entry_with_plan() {
        let ir = jocky_ir::IrProgram { investigations: vec![] };
        let module = super::emit_llvm_ir(&ir).unwrap();
        assert!(module.contains("call i32 @jocky_runtime_execute_plan"));
        assert!(module.contains("define i32 @main()"));
    }
}
