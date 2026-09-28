use jocky_ast::{Argument, Predicate, Program, Statement};
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
        args: Vec<Argument>,
    },
    Filter {
        source: String,
        destination: String,
        predicate: Predicate,
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
                            args: call.args.clone(),
                        },
                        Statement::Analyze { source, predicate } => IrOp::Filter {
                            source: source.clone(),
                            destination: format!("{source}_analysis"),
                            predicate: predicate.clone(),
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
