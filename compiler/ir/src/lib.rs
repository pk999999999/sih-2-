use jocky_ast::{Program, Statement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrProgram {
    pub investigations: Vec<IrInvestigation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrInvestigation {
    pub title: String,
    pub target: String,
    pub ops: Vec<IrOp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IrOp {
    Collect {
        binding: String,
        capability: String,
        args: Vec<String>,
    },
    Filter {
        source: String,
        predicate: String,
    },
    Report {
        name: String,
        includes: Vec<String>,
    },
}

pub fn lower(program: &Program) -> IrProgram {
    IrProgram {
        investigations: program
            .investigations
            .iter()
            .map(|investigation| IrInvestigation {
                title: investigation.title.clone(),
                target: format!("{:?}", investigation.target),
                ops: investigation
                    .statements
                    .iter()
                    .map(|statement| match statement {
                        Statement::Collect { call, alias } => IrOp::Collect {
                            binding: alias.clone(),
                            capability: format!("{}.{}", call.namespace, call.function),
                            args: call.args.iter().map(|arg| format!("{arg:?}")).collect(),
                        },
                        Statement::Analyze { source, predicate } => IrOp::Filter {
                            source: source.clone(),
                            predicate: format!("{predicate:?}"),
                        },
                        Statement::Report { name, includes } => IrOp::Report {
                            name: name.clone(),
                            includes: includes.clone(),
                        },
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub fn to_json(ir: &IrProgram) -> serde_json::Result<String> {
    serde_json::to_string_pretty(ir)
}

