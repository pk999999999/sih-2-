use std::collections::HashSet;

use jocky_ast::{Argument, ForensicCall, Literal, Program, Statement};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SemanticError {
    #[error("unsafe or unsupported forensic call {0}.{1}")]
    UnsafeCall(String, String),
    #[error("duplicate binding {0}")]
    DuplicateBinding(String),
    #[error("unknown binding {0}")]
    UnknownBinding(String),
    #[error("invalid arguments for {0}.{1}: {2}")]
    InvalidArguments(String, String, &'static str),
}

pub fn analyze(program: &Program) -> Result<(), SemanticError> {
    for investigation in &program.investigations {
        let mut bindings = HashSet::new();
        for statement in &investigation.statements {
            match statement {
                Statement::Collect { call, alias } => {
                    validate_call(call, &bindings)?;
                    if !bindings.insert(alias.clone()) {
                        return Err(SemanticError::DuplicateBinding(alias.clone()));
                    }
                }
                Statement::Analyze { source, predicate: _ } => {
                    if !bindings.contains(source) {
                        return Err(SemanticError::UnknownBinding(source.clone()));
                    }
                    let destination = format!("{source}_analysis");
                    if !bindings.insert(destination.clone()) {
                        return Err(SemanticError::DuplicateBinding(destination));
                    }
                }
                Statement::Report { name: _, includes } => {
                    for include in includes {
                        if !bindings.contains(include) {
                            return Err(SemanticError::UnknownBinding(include.clone()));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_call(call: &ForensicCall, bindings: &HashSet<String>) -> Result<(), SemanticError> {
    let expected = match (call.namespace.as_str(), call.function.as_str()) {
        ("system", "info") | ("process", "list") | ("network", "connections") => 0,
        ("filesystem", "metadata") | ("filesystem", "hash") | ("logs", "read")
        | ("hashing", "sha256") | ("timeline", "build") | ("reporting", "summary") => 1,
        _ => return Err(SemanticError::UnsafeCall(
            call.namespace.clone(),
            call.function.clone(),
        )),
    };
    if call.args.len() != expected {
        return Err(SemanticError::InvalidArguments(call.namespace.clone(), call.function.clone(), "wrong argument count"));
    }
    match (call.namespace.as_str(), call.function.as_str(), call.args.first()) {
        ("filesystem" | "logs" | "hashing", _, Some(Argument::Literal(Literal::String(_)))) => Ok(()),
        ("timeline" | "reporting", _, Some(Argument::Binding(name))) => {
            if bindings.contains(name) { Ok(()) } else { Err(SemanticError::UnknownBinding(name.clone())) }
        }
        ("system" | "process" | "network", _, None) => Ok(()),
        _ => Err(SemanticError::InvalidArguments(call.namespace.clone(), call.function.clone(), "expected a string literal or prior binding")),
    }
}

#[cfg(test)]
mod tests {
    use jocky_ast::{ForensicCall, Investigation, Program, Statement, Target};

    use super::*;

    #[test]
    fn rejects_unsafe_calls() {
        let program = Program {
            investigations: vec![Investigation {
                title: "x".into(),
                target: Target::Host("h".into()),
                statements: vec![Statement::Collect {
                    call: ForensicCall {
                        namespace: "process".into(),
                        function: "inject".into(),
                        args: vec![],
                    },
                    alias: "x".into(),
                }],
            }],
        };
        assert!(analyze(&program).is_err());
    }

    #[test]
    fn allows_incident_terms_in_investigation_metadata() {
        let program = Program {
            investigations: vec![Investigation {
                title: "Credential theft investigation".into(),
                target: Target::Host("h".into()),
                statements: vec![Statement::Collect {
                    call: ForensicCall {
                        namespace: "process".into(),
                        function: "list".into(),
                        args: vec![],
                    },
                    alias: "credential_incident".into(),
                }],
            }],
        };
        assert!(analyze(&program).is_ok());
    }

    #[test]
    fn validates_argument_types_and_binding_order() {
        let program = jocky_parser::parse_program(r#"investigation "x" { target host("h") collect reporting.summary(missing) as summary }"#).unwrap();
        assert_eq!(analyze(&program), Err(SemanticError::UnknownBinding("missing".into())));
        let program = jocky_parser::parse_program(r#"investigation "x" { target host("h") collect filesystem.hash(1) as digest }"#).unwrap();
        assert!(matches!(analyze(&program), Err(SemanticError::InvalidArguments(..))));
    }
}
