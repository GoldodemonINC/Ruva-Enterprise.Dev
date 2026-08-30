mod ast;
mod backend;
mod codegen;
mod colors;
mod debug;
mod json_protocol;
mod lexer;
mod lsp;
mod module;
mod parser;
mod typecheck;
mod vm;

use anyhow::{bail, Result};
use backend::Target;
use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(ClapParser)]
#[command(
    name = "ruva",
    about = "Ruva language compiler — Java structure, Rust power, zero-cost safety",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {

    Compile {

        input: PathBuf,


        #[arg(short, long)]
        output: Option<PathBuf>,


        #[arg(long)]
        release: bool,


        #[arg(long)]
        lazy: bool,


        #[arg(short, long)]
        verbose: bool,
    },


    Build {

        #[arg(default_value = ".")]
        project: PathBuf,


        #[arg(long)]
        release: bool,
    },


    Run {

        input: PathBuf,


        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },


    Check {

        input: PathBuf,


        #[arg(long)]
        all: bool,
    },


    Tokens {

        input: PathBuf,
    },


    Ast {

        input: PathBuf,
    },


    Repl,


    New {

        name: String,
    },


    Fmt {

        input: PathBuf,


        #[arg(long)]
        check: bool,


        #[arg(long)]
        dry_run: bool,


        #[arg(short, long)]
        verbose: bool,
    },


    Lsp,


    Vm {

        input: PathBuf,


        #[arg(long)]
        debug: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output, release, lazy, verbose } => {
            cmd_compile(&input, output.as_deref(), release, lazy, verbose)?;
        }
        Commands::Build { project, release } => {
            cmd_build(&project, release)?;
        }
        Commands::Run { input, args } => {
            cmd_run(&input, &args)?;
        }
        Commands::Check { input, all } => {
            cmd_check(&input, all)?;
        }
        Commands::Tokens { input } => {
            let source = read_source(&input)?;
            debug::print_tokens(&source);
        }
        Commands::Repl => {
            cmd_repl()?;
        }
        Commands::Ast { input } => {
            cmd_ast(&input)?;
        }
        Commands::New { name } => {
            cmd_new(&name)?;
        }
        Commands::Fmt { input, check, dry_run, verbose } => {
            cmd_fmt(&input, check, dry_run, verbose)?;
        }
        Commands::Lsp => {
            cmd_lsp()?;
        }
        Commands::Vm { input, debug } => {
            cmd_vm(&input, debug)?;
        }
    }

    Ok(())
}


fn is_ruva_source(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()), Some("ruva") | Some("rve"))
}

fn read_source(path: &Path) -> Result<String> {
    if !is_ruva_source(path) {
        bail!("Expected a .ruva or .rve file, got: {}", path.display());
    }
    Ok(fs::read_to_string(path)?)
}

fn transpile(source: &str, target: Target, source_path: &Path) -> Result<String> {
    let mut parser = parser::Parser::new(source)?;
    let program = parser.parse_program()?;


    let mut resolver = module::ModuleResolver::new(source_path);
    let program = resolver.resolve_program(&program)?;

    let mut gen = backend::create_generator(target);
    Ok(gen.generate(&program))
}



fn cmd_compile(input: &Path, output: Option<&Path>, release: bool, lazy: bool, verbose: bool) -> Result<()> {
    let source = read_source(input)?;

    if verbose {
        eprintln!("{}", colors::info(&format!("Parsing {}...", input.display())));
    }


    let mut parser = parser::Parser::new(&source)?;
    let program = parser.parse_program()?;

    if lazy {
        eprintln!("{}", colors::success(&format!("{} — syntax OK (lazy mode, no codegen)", input.display())));
        return Ok(());
    }


    let mut resolver = module::ModuleResolver::new(input);
    let program = resolver.resolve_program(&program)?;

    if verbose {
        eprintln!("⟳ Compiling to native...");
    }


    let mut gen = codegen::CodeGen::new();
    let code = gen.generate_rust(&program);
    if gen.has_external_dependencies() {
        bail!("This program imports external crates, which a cargo-free build cannot resolve yet. Use the bytecode VM instead: `rgu run {}", input.display());
    }

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let mut p = input.to_path_buf();
            p.set_extension("");
            if cfg!(target_os = "windows") {
                p.set_extension("exe");
            }
            p
        }
    };

    rustc_build(&code, &out_path, release)?;
    eprintln!("{}", colors::success(&format!("Compiled {} → {} (Rust)", input.display(), out_path.display())));

    Ok(())
}

fn rustc_build(code: &str, out_path: &Path, optimize: bool) -> Result<()> {
    let src_dir = std::env::temp_dir().join("ruva_build");
    fs::create_dir_all(&src_dir)?;
    let src_path = src_dir.join(format!("_ruva_main_{}.rs", std::process::id()));
    fs::write(&src_path, code)?;

    let mut cmd = Command::new("rustc");
    cmd.arg("--edition").arg("2021");
    if optimize {
        cmd.arg("-C").arg("opt-level=3");
        cmd.arg("-C").arg("strip=symbols");
    } else {
        cmd.arg("-C").arg("opt-level=1");
    }
    cmd.arg(&src_path).arg("-o").arg(out_path);

    let status = cmd.status().map_err(|e| anyhow::anyhow!("Failed to run rustc (is it installed?): {e}"))?;
    if !status.success() {
        bail!("Rust compilation failed");
    }
    Ok(())
}

fn cmd_build(project: &Path, release: bool) -> Result<()> {
    let src_dir = project.join("src");
    if !src_dir.exists() {
        bail!("No src/ directory found in {}", project.display());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(&src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if is_ruva_source(&path) {
            files.push(path);
        }
    }

    if files.is_empty() {
        bail!("No .ruva/.rv files found in {}", src_dir.display());
    }

    eprintln!("⟳ Building {} .ruva files...", files.len());

    for file in &files {
        eprintln!("  → {}", file.display());
        cmd_compile(file, None, release, false, false)?;
    }

    eprintln!("✓ Build complete");
    Ok(())
}

fn cmd_vm(input: &Path, debug: bool) -> Result<()> {
    let source = read_source(input)?;
    let mut parser = parser::Parser::new(&source)?;
    let program = parser.parse_program()?;


    let mut resolver = module::ModuleResolver::new(input);
    let program = resolver.resolve_program(&program)?;

    eprintln!("{}", colors::info(&format!("Running {} via bytecode VM...", input.display())));

    let result = vm::compile_and_run(&program, debug)
        .map_err(|e| anyhow::anyhow!("VM error: {}", e))?;


    match &result {
        vm::Value::Nil => {}
        _ => eprintln!("{}", colors::success(&format!("Result: {}", result))),
    }

    Ok(())
}

fn cmd_run(input: &Path, args: &[String]) -> Result<()> {
    let source = read_source(input)?;
    let mut parser = parser::Parser::new(&source)?;
    let program = parser.parse_program()?;

    let mut resolver = module::ModuleResolver::new(input);
    let program = resolver.resolve_program(&program)?;

    let mut gen = codegen::CodeGen::new();
    let code = gen.generate_rust(&program);
    if gen.has_external_dependencies() {
        bail!("This program imports external crates, which a cargo-free build cannot resolve yet. Use the bytecode VM instead: `rgu run {}", input.display());
    }

    let tmp_dir = std::env::temp_dir().join("ruva_run");
    fs::create_dir_all(&tmp_dir)?;
    let bin = tmp_dir.join(if cfg!(target_os = "windows") { "program.exe" } else { "program" });

    eprintln!("⟳ Compiling...");
    rustc_build(&code, &bin, false)?;
    let status = Command::new(&bin).args(args).status()?;
    if !status.success() {
        bail!("Execution failed (exit {})", status.code().unwrap_or(-1));
    }
    Ok(())
}

fn cmd_check(input: &Path, all: bool) -> Result<()> {
    if all {

        let dir = input.parent().unwrap_or(input);
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if is_ruva_source(&path) {
                check_file(&path)?;
            }
        }
    } else {
        check_file(input)?;
    }
    Ok(())
}

fn check_file(path: &Path) -> Result<()> {
    let source = read_source(path)?;
    match parser::Parser::new(&source) {
        Ok(mut parser) => match parser.parse_program() {
            Ok(program) => {

                let mut resolver = module::ModuleResolver::new(path);
                let program = match resolver.resolve_program(&program) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{}", colors::warning(&format!("Module resolution: {}", e)));
                        program
                    }
                };

                let mut checker = typecheck::TypeChecker::new();
                let diagnostics = checker.check(&program);
                let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == typecheck::DiagnosticKind::Error).collect();
                let warnings: Vec<_> = diagnostics.iter().filter(|d| d.kind == typecheck::DiagnosticKind::Warning).collect();
                for w in &warnings {
                    eprintln!("{}", colors::warning(&w.to_string()));
                }
                if errors.is_empty() {
                    eprintln!("{}", colors::success(&format!("{} — no errors ({} warnings)", path.display(), warnings.len())));
                } else {
                    for err in &errors {
                        eprintln!("{}", colors::error(&err.to_string()));
                    }
                    std::process::exit(1);
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Parse error in {}: {}", path.display(), e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("✗ Lex error in {}: {}", path.display(), e);
            std::process::exit(1);
        }
    }
}

fn cmd_repl() -> Result<()> {
    eprintln!("Ruva REPL v0.1.0");
    eprintln!("Type expressions and statements. Ctrl+D to exit.");
    eprintln!("Commands:");
    eprintln!("  :quit, :q    - Exit the REPL");
    eprintln!("  :clear       - Clear the buffer");
    eprintln!("  :transpile   - Transpile the current buffer");
    eprintln!("");

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("ruva> ");
        stdout.flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            eprintln!();
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }


        if line == ":quit" || line == ":q" {
            break;
        }

        if line == ":clear" {
            buffer.clear();
            eprintln!("Buffer cleared.");
            continue;
        }

        if line == ":transpile" {
            match transpile(&buffer, Target::Rust, Path::new("repl")) {
                Ok(code) => {
                    eprintln!("-- Generated Rust --");
                    eprintln!("{}", code);
                    eprintln!("-------------------");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            continue;
        }


        buffer.push_str(line);
        buffer.push('\n');


        match transpile(&buffer, Target::Rust, Path::new("repl")) {
            Ok(code) => {
                eprintln!("-- Generated Rust --");
                eprintln!("{}", code);
                eprintln!("-------------------");
            }
            Err(e) => {
                eprintln!("Error: {}", e);

                buffer = buffer.lines().collect::<Vec<_>>().join("\n");
                buffer.push('\n');
            }
        }
    }

    Ok(())
}

fn cmd_ast(input: &Path) -> Result<()> {
    let source = read_source(input)?;
    let mut parser = parser::Parser::new(&source)?;
    let program = parser.parse_program()?;
    println!("{:#?}", program);
    Ok(())
}

fn cmd_fmt(input: &Path, check: bool, dry_run: bool, verbose: bool) -> Result<()> {

    let mut stats = (0i64, 0i64, 0i64);


    if input.is_dir() {

        for entry in fs::read_dir(input)? {
            let entry = entry?;
            let path = entry.path();
            if is_ruva_source(&path) {
                match format_single_file(&path, check, dry_run, verbose) {
                    Ok(changed) => {
                        stats.0 += 1;
                        if changed { stats.1 += 1; }
                    }
                    Err(e) => {
                        eprintln!("{}", colors::error(&format!("{}: {}", path.display(), e)));
                        stats.2 += 1;
                    }
                }
            }
        }
    } else {

        match format_single_file(input, check, dry_run, verbose) {
            Ok(changed) => {
                stats.0 += 1;
                if changed { stats.1 += 1; }
            }
            Err(e) => {
                eprintln!("{}", colors::error(&format!("{}: {}", input.display(), e)));
                stats.2 += 1;
            }
        }
    }

    if stats.1 == 0 && stats.2 == 0 {
        eprintln!("{}", colors::success(&format!("{} files checked, all formatted correctly", stats.0)));
    } else if check {
        eprintln!("{}", colors::error(&format!("{} files checked, {} need formatting", stats.0, stats.1)));
        std::process::exit(1);
    } else {
        eprintln!("{}", colors::success(&format!("{} files checked, {} reformatted", stats.0, stats.1)));
    }

    Ok(())
}

fn format_single_file(path: &Path, check: bool, dry_run: bool, verbose: bool) -> Result<bool> {
    let source = read_source(path)?;
    let formatted = ruva_format(&source);
    let changed = source != formatted;

    if changed && verbose {
        eprintln!("  {}", colors::info(&format!("reformatting {}", path.display())));
    }

    if check && changed {
        eprintln!("  {}", colors::warning(&format!("{} needs formatting", path.display())));
    }

    if changed && !check && !dry_run {
        fs::write(path, &formatted)?;
    }

    Ok(changed)
}



fn ruva_format(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut prev_blank = false;
    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim_end();


        if in_block_comment {
            output.push_str(trimmed);
            output.push('\n');
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.contains("/*") && !trimmed.contains("*/") {
            in_block_comment = true;
        }

        let is_blank = trimmed.is_empty();


        if is_blank {
            if !prev_blank {
                output.push('\n');
            }
            prev_blank = true;
            continue;
        }

        prev_blank = false;
        output.push_str(trimmed);
        output.push('\n');
    }


    let result = output.trim_end_matches('\n');
    format!("{}\n", result)
}

fn cmd_new(name: &str) -> Result<()> {
    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("src"))?;


    fs::write(
        project_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{}"
path = "src/main.ruva"
"#,
            name, name
        ),
    )?;


    fs::write(
        project_dir.join("src/main.ruva"),
        r#"// Welcome to Ruva!
// This is a simple hello world program.

fn main() {
    println!("Hello, {}!", "Ruva")
}
"#,
    )?;

    eprintln!("{}", colors::success(&format!("Created new Ruva project: {}", name)));
    eprintln!("  {}", colors::dim(&format!("cd {} && ruva run src/main.ruva", name)));

    Ok(())
}

fn cmd_lsp() -> Result<()> {
    eprintln!("{}", colors::info("Starting Ruva LSP server..."));
    eprintln!("{}", colors::dim("Listening on stdin/stdout (JSON-RPC over Content-Length headers)"));

    let mut server = lsp::LspServer::new();
    server.run();

    Ok(())
}

