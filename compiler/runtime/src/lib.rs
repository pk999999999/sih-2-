use std::collections::BTreeMap;

use jocky_ast::{Argument, Literal, Predicate};
use jocky_ir::{IrOp, IrProgram};
use serde_json::{json, Value};
use time::OffsetDateTime;

#[derive(Debug, Default)]
pub struct Runtime {
    bindings: BTreeMap<String, Value>,
}

impl Runtime {
    pub fn execute(&mut self, ir: &IrProgram) -> Value {
        let mut reports = Vec::new();
        for investigation in &ir.investigations {
            self.bindings.clear();
            for op in &investigation.ops {
                match op {
                    IrOp::Collect {
                        binding,
                        capability,
                        args,
                    } => {
                        let resolved = args.iter().map(|arg| match arg {
                            Argument::Literal(Literal::String(s)) => json!(s),
                            Argument::Literal(Literal::Integer(i)) => json!(i),
                            Argument::Literal(Literal::Boolean(b)) => json!(b),
                            Argument::Binding(name) => self.bindings.get(name).cloned().unwrap_or(Value::Null),
                        }).collect::<Vec<_>>();
                        let value = jocky_stdlib::invoke(capability, &resolved)
                            .unwrap_or_else(|error| json!({"error": error.to_string()}));
                        self.bindings.insert(binding.clone(), value);
                    }
                    IrOp::Filter { source, destination, predicate } => {
                        let value = self.bindings.get(source).cloned().unwrap_or(Value::Null);
                        self.bindings.insert(destination.clone(), filter_rows(&value, predicate));
                    }
                    IrOp::Report { name, includes } => {
                        let mut included = BTreeMap::new();
                        for include in includes {
                            included.insert(
                                include.clone(),
                                self.bindings.get(include).cloned().unwrap_or(Value::Null),
                            );
                        }
                        reports.push(json!({
                            "name": name,
                            "generated_at": OffsetDateTime::now_utc().to_string(),
                            "evidence": included
                        }));
                    }
                }
            }
        }
        json!({ "reports": reports })
    }
}

fn filter_rows(value: &Value, predicate: &Predicate) -> Value {
    let matches = |row: &Value| {
        let (field, expected, contains) = match predicate {
            Predicate::Contains { field, value } => (field, value, true),
            Predicate::Equals { field, value } => (field, value, false),
        };
        let actual = match row.get(field) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            _ => return false,
        };
        if contains {
            actual.to_lowercase().contains(&expected.to_lowercase())
        } else {
            actual.eq_ignore_ascii_case(expected)
        }
    };
    match value {
        Value::Array(rows) => Value::Array(rows.iter().filter(|row| matches(row)).cloned().collect()),
        Value::Object(map) => {
            let mut result = map.clone();
            for key in ["processes", "connections"] {
                if let Some(Value::Array(rows)) = map.get(key) {
                    result.insert(key.to_string(), Value::Array(rows.iter().filter(|row| matches(row)).cloned().collect()));
                    return Value::Object(result);
                }
            }
            Value::Array(if matches(value) { vec![value.clone()] } else { vec![] })
        }
        _ => Value::Array(Vec::new()),
    }
}

#[no_mangle]
pub extern "C" fn jocky_runtime_execute_plan(plan: *const u8, len: usize) -> i32 {
    let result = std::panic::catch_unwind(|| -> Result<(), Box<dyn std::error::Error>> {
        if plan.is_null() || len > 16 * 1024 * 1024 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid JOCKY plan buffer").into());
        }
        let bytes = unsafe { std::slice::from_raw_parts(plan, len) };
        let ir: IrProgram = serde_json::from_slice(bytes)?;
        let output = Runtime::default().execute(&ir);
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    });
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => { eprintln!("jocky runtime: {error}"); 1 },
        Err(_) => { eprintln!("jocky runtime panic"); 2 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_matching_process_names() {
        let input = json!([
            {"pid": 1, "name": "powershell.exe"},
            {"pid": 2, "name": "explorer.exe"}
        ]);
        let predicate = Predicate::Contains { field: "name".into(), value: "PowerShell".into() };
        assert_eq!(filter_rows(&input, &predicate), json!([{"pid": 1, "name": "powershell.exe"}]));
    }
}
