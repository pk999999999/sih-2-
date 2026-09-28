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
        #[arg(long)]
        runtime_lib: Option<PathBuf>,
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
        Command::Build { file, output, runtime_lib } => {
            let program = load_checked(&file)?;
            let ir = jocky_ir::lower(&program);
            let llvm = jocky_codegen::emit_llvm_ir(&ir)?;
            let llvm_path = output.with_extension("ll");
            fs::write(&llvm_path, llvm).with_context(|| format!("write {}", llvm_path.display()))?;
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let mut native_libs = Vec::new();
            let library = match runtime_lib {
                Some(path) => path,
                None => {
                    let target = std::env::var_os("CARGO_TARGET_DIR")
                        .map(PathBuf::from).unwrap_or_else(|| root.join("target"));
                    let name = if cfg!(target_os = "windows") { "jocky_runtime.lib" } else { "libjocky_runtime.a" };
                    let path = target.join("release").join(name);
                    if cfg!(target_os = "windows") {
                        let output = ProcessCommand::new("cargo")
                            .args(["rustc", "--release", "-p", "jocky-runtime", "--lib", "--manifest-path"])
                            .arg(root.join("Cargo.toml"))
                            .args(["--", "--print", "native-static-libs"])
                            .output().context("build JOCKY runtime and query native libraries")?;
                        anyhow::ensure!(output.status.success(), "runtime build failed: {}", String::from_utf8_lossy(&output.stderr));
                        let diagnostics = String::from_utf8_lossy(&output.stderr);
                        native_libs = diagnostics.lines()
                            .filter_map(|line| line.split_once("native-static-libs:").map(|(_, libs)| libs))
                            .flat_map(str::split_whitespace).map(str::to_string).collect();
                        anyhow::ensure!(!native_libs.is_empty(), "rustc did not report native static libraries");
                    } else if !path.exists() {
                        let status = ProcessCommand::new("cargo")
                            .args(["build", "--release", "-p", "jocky-runtime", "--manifest-path"])
                            .arg(root.join("Cargo.toml"))
                            .status().context("build JOCKY runtime")?;
                        anyhow::ensure!(status.success(), "runtime build failed");
                    }
                    path
                }
            };
            anyhow::ensure!(library.is_file(), "runtime library not found: {}", library.display());
            let mut command = ProcessCommand::new("clang");
            command.arg(&llvm_path).arg(&library).arg("-o").arg(&output);
            command.args(native_libs);
            if cfg!(target_os = "linux") {
                command.args(["-ldl", "-lpthread", "-lm"]);
            }
            let status = command.status().context("invoke clang")?;
            anyhow::ensure!(status.success(), "native link failed");
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
