use jocky_ir::{IrOp, IrProgram};

pub fn emit_pseudo_native(ir: &IrProgram) -> String {
    let mut out = String::from("; JOCKY safe forensic execution plan\n");
    for investigation in &ir.investigations {
        out.push_str(&format!("investigation \"{}\"\n", investigation.title));
        for op in &investigation.ops {
            match op {
                IrOp::Collect { binding, capability, args } => out.push_str(&format!(
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
    let mut steps = Vec::new();
    for investigation in &ir.investigations {
        steps.push(("jocky_runtime_begin", serde_json::to_vec(&investigation.target)?));
        for op in &investigation.ops {
            steps.push(("jocky_runtime_op", serde_json::to_vec(op)?));
        }
    }
    let mut module = String::from("; ModuleID = 'jocky'\nsource_filename = \"jocky\"\n");
    for (index, (_, bytes)) in steps.iter().enumerate() {
        let escaped = bytes.iter().map(|byte| format!("\\{byte:02X}")).collect::<String>();
        module.push_str(&format!("@jocky_data_{index} = private constant [{} x i8] c\"{escaped}\", align 1\n", bytes.len()));
    }
    module.push_str("declare ptr @jocky_runtime_new()\n");
    module.push_str("declare i32 @jocky_runtime_begin(ptr, ptr, i64)\n");
    module.push_str("declare i32 @jocky_runtime_op(ptr, ptr, i64)\n");
    module.push_str("declare i32 @jocky_runtime_finish(ptr)\n");
    module.push_str("declare void @jocky_runtime_free(ptr)\n");
    module.push_str("define i32 @main() {\nentry:\n  %session = call ptr @jocky_runtime_new()\n  %missing = icmp eq ptr %session, null\n");
    let first = if steps.is_empty() { "done".to_string() } else { "step0".to_string() };
    module.push_str(&format!("  br i1 %missing, label %allocation_failed, label %{first}\n"));
    for (index, (function, bytes)) in steps.iter().enumerate() {
        let next = if index + 1 == steps.len() { "done".to_string() } else { format!("step{}", index + 1) };
        module.push_str(&format!(
            "step{index}:\n  %status{index} = call i32 @{function}(ptr %session, ptr @jocky_data_{index}, i64 {})\n  %ok{index} = icmp eq i32 %status{index}, 0\n  br i1 %ok{index}, label %{next}, label %operation_failed\n",
            bytes.len()
        ));
    }
    module.push_str("done:\n  %final_status = call i32 @jocky_runtime_finish(ptr %session)\n  call void @jocky_runtime_free(ptr %session)\n  ret i32 %final_status\n");
    module.push_str("operation_failed:\n  call void @jocky_runtime_free(ptr %session)\n  ret i32 1\n");
    module.push_str("allocation_failed:\n  ret i32 1\n}\n");
    Ok(module)
}

#[cfg(test)]
mod tests {
    use jocky_ast::Target;
    use jocky_ir::{IrInvestigation, IrOp, IrProgram};

    #[test]
    fn lowers_each_operation_to_a_native_runtime_call() {
        let ir = IrProgram { investigations: vec![IrInvestigation {
            title: "x".into(), target: Target::Host("localhost".into()),
            ops: vec![IrOp::Report { name: "report".into(), includes: vec![] }],
        }] };
        let module = super::emit_llvm_ir(&ir).unwrap();
        assert!(module.contains("call i32 @jocky_runtime_begin"));
        assert!(module.contains("call i32 @jocky_runtime_op"));
        assert!(!module.contains("jocky_runtime_execute_plan"));
    }
}
