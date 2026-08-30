// RGu — the Ruva compiler and driver.
//
// This binary is Ruva's own compiler front-end, independent of cargo. It links
// the Ruva library and drives a `.rve`/`.ruva` source file *directly through the
// bytecode VM* (no transpilation, no external build tool). `rgu build` transpiles
// to a chosen backend. Once the `rgu` binary is built, it needs no other
// toolchain: it reads source, lexes, parses, resolves modules, and interprets.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use ruva::backend::Target;
use ruva::{colors, module, parser, vm};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("RGu — Ruva compiler and driver v{VERSION}");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  rgu run <file.rve> [--debug]        Run a file directly via the bytecode VM");
    eprintln!("  rgu check <file.rve>                Parse + resolve modules (no execution)");
    eprintln!("  rgu build <file.rve> [--target <t>] Transpile to a backend source file");
    eprintln!("  rgu --version                       Print version");
    eprintln!("  rgu --help                          Show this help");
    eprintln!();
    eprintln!("The run path uses RGu's own bytecode interpreter — no cargo, no build step.");
}

fn is_ruva_source(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()), Some("ruva") | Some("rve"))
}

fn read_source(path: &Path) -> Result<String> {
    if !is_ruva_source(path) {
        bail!("Expected a .rve or .ruva file, got: {}", path.display());
    }
    Ok(fs::read_to_string(path)?)
}

/// Parse + resolve modules into a program (shared by run/check/build).
fn load_program(path: &Path) -> Result<ruva::ast::Program> {
    let source = read_source(path)?;
    let mut parser = parser::Parser::new(&source).map_err(|e| anyhow::anyhow!("{e}"))?;
    let program = parser.parse_program()?;
    let mut resolver = module::ModuleResolver::new(path);
    resolver.resolve_program(&program).map_err(|e| anyhow::anyhow!("{e}"))
}

fn cmd_run(input: &Path, debug: bool) -> Result<()> {
    let program = load_program(input)?;
    eprintln!("{}", colors::info(&format!("Running {} via RGu VM...", input.display())));
    let result = vm::compile_and_run(&program, debug)
        .map_err(|e| anyhow::anyhow!("VM error: {e}"))?;
    match &result {
        vm::Value::Nil => {}
        _ => eprintln!("{}", colors::success(&format!("Result: {result}"))),
    }
    Ok(())
}

fn cmd_check(input: &Path) -> Result<()> {
    let _program = load_program(input)?;
    eprintln!("{}", colors::success(&format!("{} — parse + module resolution OK", input.display())));
    Ok(())
}

fn cmd_build(input: &Path, target: Target) -> Result<()> {
    let program = load_program(input)?;
    let mut gen = ruva::backend::create_generator(target);
    let code = gen.generate(&program);
    let out_path = input.with_extension(target.file_extension().trim_start_matches('.'));
    fs::write(&out_path, &code)?;
    eprintln!("{}", colors::success(&format!("Transpiled {} -> {} ({})", input.display(), out_path.display(), target)));
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut it = args.iter().skip(1);

    let Some(sub) = it.next() else {
        print_usage();
        return Ok(());
    };

    match sub.as_str() {
        "help" | "--help" | "-h" => {
            print_usage();
        }
        "version" | "--version" | "-V" => {
            println!("rgu {VERSION}");
        }
        "run" => {
            let input = it.next().ok_or_else(|| anyhow::anyhow!("`rgu run` needs a file path"))?;
            let debug = it.any(|a| a == "--debug");
            cmd_run(&PathBuf::from(input), debug)?;
        }
        "check" => {
            let input = it.next().ok_or_else(|| anyhow::anyhow!("`rgu check` needs a file path"))?;
            cmd_check(&PathBuf::from(input))?;
        }
        "build" => {
            let input = it.next().ok_or_else(|| anyhow::anyhow!("`rgu build` needs a file path"))?;
            let mut target = Target::Rust;
            while let Some(a) = it.next() {
                if a == "--target" {
                    let t = it.next().ok_or_else(|| anyhow::anyhow!("`--target` needs a value"))?;
                    target = t.parse().map_err(|e| anyhow::anyhow!("{e}"))?;
                } else {
                    bail!("Unexpected argument `{a}`");
                }
            }
            cmd_build(&PathBuf::from(input), target)?;
        }
        other => {
            bail!("Unknown command `{other}`. See `rgu --help`.");
        }
    }

    Ok(())
}