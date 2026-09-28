use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jocky", version, about = "JOCKY forensic DSL compiler and runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Check { file: PathBuf },
    Compile {
        file: PathBuf,
        #[arg(long, default_value = "json")]
        format: String,
    },
    Run { file: PathBuf },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { file } => {
            let program = load_checked(&file)?;
            println!("ok: {} investigation(s)", program.investigations.len());
        }
        Command::Compile { file, format } => {
            let program = load_checked(&file)?;
            let ir = jocky_ir::lower(&program);
            match format.as_str() {
                "json" => println!("{}", jocky_ir::to_json(&ir)?),
                "plan" => println!("{}", jocky_codegen::emit_pseudo_native(&ir)),
                "llvm" => print!("{}", jocky_codegen::emit_llvm_ir(&ir)?),
                other => anyhow::bail!("unknown output format: {other}"),
            }
        }
        Command::Run { file } => {
            let program = load_checked(&file)?;
            let ir = jocky_ir::lower(&program);
            let mut runtime = jocky_runtime::Runtime::default();
            println!("{}", serde_json::to_string_pretty(&runtime.execute(&ir)?)?);
        }
    }
    Ok(())
}

fn load_checked(file: &PathBuf) -> anyhow::Result<jocky_ast::Program> {
    let source = fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let program = jocky_parser::parse_program(&source)?;
    jocky_semantic::analyze(&program)?;
    Ok(program)
}
