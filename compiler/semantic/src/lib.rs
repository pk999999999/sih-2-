use std::collections::HashSet;

use jocky_ast::{ForensicCall, Program, Statement};
use thiserror::Error;

const SAFE_CALLS: &[(&str, &str)] = &[
    ("system", "info"),
    ("process", "list"),
    ("network", "connections"),
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SemanticError {
    #[error("unsafe or unsupported forensic call {0}.{1}")]
    UnsafeCall(String, String),
    #[error("duplicate binding {0}")]
    DuplicateBinding(String),
    #[error("unknown binding {0}")]
    UnknownBinding(String),
}

pub fn analyze(program: &Program) -> Result<(), SemanticError> {
    for investigation in &program.investigations {
        let mut bindings = HashSet::new();
        for statement in &investigation.statements {
            match statement {
                Statement::Collect { call, alias } => {
                    validate_call(call)?;
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

fn validate_call(call: &ForensicCall) -> Result<(), SemanticError> {
    if SAFE_CALLS
        .iter()
        .any(|(namespace, function)| *namespace == call.namespace && *function == call.function)
    {
        Ok(())
    } else {
        Err(SemanticError::UnsafeCall(
            call.namespace.clone(),
            call.function.clone(),
        ))
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
}
