use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

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
    Build {
        file: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
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
        Command::Build { file, output } => {
            let program = load_checked(&file)?;
            let ir = jocky_ir::lower(&program);
            let llvm = jocky_codegen::emit_llvm_ir(&ir)?;
            let llvm_path = output.with_extension("ll");
            let object_path = output.with_extension(if cfg!(target_os = "windows") { "obj" } else { "o" });
            fs::write(&llvm_path, llvm).with_context(|| format!("write {}", llvm_path.display()))?;
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let status = ProcessCommand::new("clang")
                .arg("-c").arg(&llvm_path).arg("-o").arg(&object_path)
                .status().context("compile LLVM IR with clang")?;
            anyhow::ensure!(status.success(), "LLVM object compilation failed");
            let object_path = object_path.canonicalize()?;
            let status = ProcessCommand::new("cargo")
                .args(["build", "--release", "-p", "jocky-native-runner", "--manifest-path"])
                .arg(root.join("Cargo.toml"))
                .env("JOCKY_OBJECT_PATH", &object_path)
                .status().context("link JOCKY native runner")?;
            anyhow::ensure!(status.success(), "native link failed");
            let target = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from).unwrap_or_else(|| root.join("target"));
            let binary = target.join("release").join(if cfg!(target_os = "windows") { "jocky-native-runner.exe" } else { "jocky-native-runner" });
            fs::copy(&binary, &output).with_context(|| format!("copy native executable to {}", output.display()))?;
            println!("{}", output.display());
        }
    }
    Ok(())
}

fn load_checked(file: &PathBuf) -> anyhow::Result<jocky_ast::Program> {
    let source = fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let program = jocky_parser::parse_program(&source).map_err(|error| {
        let offset = match &error {
            jocky_parser::ParseError::Lex(jocky_lexer::LexError::InvalidToken(offset)) => *offset,
            jocky_parser::ParseError::Unexpected { offset, .. } => *offset,
            jocky_parser::ParseError::Eof(_) => source.len(),
        };
        let before = &source[..offset.min(source.len())];
        let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        anyhow::anyhow!("{}:{line}:{column}: {error}", file.display())
    })?;
    jocky_semantic::analyze(&program).with_context(|| format!("{}: semantic analysis", file.display()))?;
    Ok(program)
}
