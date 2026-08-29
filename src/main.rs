mod ast;
mod backend;
mod codegen;
mod codegen_python;
mod codegen_zig;
mod colors;
mod debug;
mod features;
mod json_protocol;
mod lexer;
mod lsp;
mod module;
mod parser;
mod typecheck;

use anyhow::{bail, Result};
use backend::Target;
use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Read, Write};
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

#[derive(ValueEnum, Clone)]
enum CliTarget {
    Rust,
    Zig,
    Python,
    C,
    Cpp,
    Wasm,
}

impl From<CliTarget> for Target {
    fn from(t: CliTarget) -> Self {
        match t {
            CliTarget::Rust => Target::Rust,
            CliTarget::Zig => Target::Zig,
            CliTarget::Python => Target::Python,
            CliTarget::C => Target::C,
            CliTarget::Cpp => Target::Cpp,
            CliTarget::Wasm => Target::Wasm,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .ruva file to a native executable
    Compile {
        /// Input .ruva file
        input: PathBuf,

        /// Output executable path (defaults to input name without extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target language (rust, zig, python)
        #[arg(long, value_enum, default_value_t = CliTarget::Rust)]
        target: CliTarget,

        /// Build in release mode (optimized)
        #[arg(long)]
        release: bool,

        /// Lazy compilation: only check for errors, don't generate code
        #[arg(long)]
        lazy: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Build a .ruva project (compiles all .ruva files in src/)
    Build {
        /// Project directory (defaults to current directory)
        #[arg(default_value = ".")]
        project: PathBuf,

        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run a .ruva file directly
    Run {
        /// Input .ruva file
        input: PathBuf,

        /// Arguments to pass to the program
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Check .ruva files for syntax and type errors
    Check {
        /// Input .ruva file
        input: PathBuf,

        /// Check all .ruva files in a directory
        #[arg(long)]
        all: bool,
    },

    /// Transpile .ruva files to target language source code
    Transpile {
        /// Input .ruva file
        input: PathBuf,

        /// Output file (defaults to input with target extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target language (rust, zig, python)
        #[arg(long, value_enum, default_value_t = CliTarget::Rust)]
        target: CliTarget,

        /// Print generated code to stdout instead of writing a file
        #[arg(short, long)]
        stdout: bool,
    },

    /// Print the token stream for debugging
    Tokens {
        /// Input .ruva file
        input: PathBuf,
    },

    /// Print the AST for a .ruva file (for debugging)
    Ast {
        /// Input .ruva file
        input: PathBuf,
    },

    /// Start an interactive REPL
    Repl,

    /// Transpile from stdin (pipe mode)
    Pipe {
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target language (rust, zig, python)
        #[arg(long, value_enum, default_value_t = CliTarget::Rust)]
        target: CliTarget,
    },

    /// Create a new Ruva project
    New {
        /// Project name
        name: String,
    },

    /// Format .ruva source files
    Fmt {
        /// Input file or directory
        input: PathBuf,

        /// Check only, don't modify files
        #[arg(long)]
        check: bool,

        /// Dry run, show what would change
        #[arg(long)]
        dry_run: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start the Ruva Language Server (LSP)
    Lsp,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output, target, release, lazy, verbose } => {
            cmd_compile(&input, output.as_deref(), target.into(), release, lazy, verbose)?;
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
        Commands::Transpile { input, output, target, stdout } => {
            cmd_transpile(&input, output.as_deref(), target.into(), stdout)?;
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
        Commands::Pipe { output, target } => {
            cmd_pipe(output.as_deref(), target.into())?;
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
    }

    Ok(())
}

fn read_source(path: &Path) -> Result<String> {
    let ext = path.extension().and_then(|e| e.to_str());
    if ext != Some("ruva") {
        bail!("Expected .ruva file, got: {}", path.display());
    }
    Ok(fs::read_to_string(path)?)
}

fn transpile(source: &str, target: Target, source_path: &Path) -> Result<String> {
    let mut parser = parser::Parser::new(source)?;
    let program = parser.parse_program()?;

    // Resolve modules (inline stdlib, file-based modules)
    let mut resolver = module::ModuleResolver::new(source_path);
    let program = resolver.resolve_program(&program)?;

    let mut gen = backend::create_generator(target);
    Ok(gen.generate(&program))
}

// ─── Commands ────────────────────────────────────────────────────────────────

fn cmd_compile(input: &Path, output: Option<&Path>, target: Target, release: bool, lazy: bool, verbose: bool) -> Result<()> {
    let source = read_source(input)?;

    if verbose {
        eprintln!("{}", colors::info(&format!("Parsing {}...", input.display())));
    }

    // Parse
    let mut parser = parser::Parser::new(&source)?;
    let program = parser.parse_program()?;

    if lazy {
        eprintln!("{}", colors::success(&format!("{} — syntax OK (lazy mode, no codegen)", input.display())));
        return Ok(());
    }

    // Resolve modules (inline stdlib, file-based modules)
    let mut resolver = module::ModuleResolver::new(input);
    let program = resolver.resolve_program(&program)?;

    if verbose {
        eprintln!("{}", colors::info(&format!("Transpiling to {}...", target)));
    }

    // Transpile using the selected backend
    let mut gen = backend::create_generator(target);
    let code = gen.generate(&program);

    match target {
        Target::Rust => {
            // Rust: compile via cargo
            if verbose {
                eprintln!("⟳ Compiling to native...");
            }

            let tmp_dir = std::env::temp_dir().join("ruva_build");
            fs::create_dir_all(&tmp_dir)?;

            let cargo_toml = tmp_dir.join("Cargo.toml");
            let profile = if release { "release" } else { "dev" };
            let mut cargo_content = gen.generate_cargo_toml();
            if code.contains("macroquad::") && !cargo_content.contains("macroquad") {
                cargo_content.push_str("macroquad = \"0.4\"\n");
            }
            cargo_content.push_str(&format!("\n[profile.{}]\nopt-level = 3\n", profile));
            fs::write(&cargo_toml, cargo_content)?;
            fs::create_dir_all(tmp_dir.join("src"))?;
            fs::write(tmp_dir.join("src/main.rs"), &code)?;

            let mut cmd = Command::new("cargo");
            cmd.arg("build");
            if release {
                cmd.arg("--release");
            }
            cmd.arg("--quiet");
            cmd.current_dir(&tmp_dir);

            let status = cmd.status()?;
            if !status.success() {
                bail!("Compilation failed");
            }

            let bin_name = if cfg!(target_os = "windows") {
                "ruva_program.exe"
            } else {
                "ruva_program"
            };

            let mut bin_path = tmp_dir.join("target").join(profile).join(bin_name);
            if !bin_path.exists() {
                bin_path = tmp_dir.join("target/debug").join(bin_name);
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

            fs::copy(&bin_path, &out_path)?;
            eprintln!("{}", colors::success(&format!("Compiled {} → {} (Rust)", input.display(), out_path.display())));
        }
        Target::Zig => {
            // Zig: write .zig file and compile with zig build-exe
            let out_path = match output {
                Some(p) => p.to_path_buf(),
                None => {
                    let mut p = input.to_path_buf();
                    p.set_extension("zig");
                    p
                }
            };
            fs::write(&out_path, &code)?;

            if verbose {
                eprintln!("⟳ Compiling with zig...");
            }

            let mut cmd = Command::new("zig");
            cmd.arg("build-exe");
            cmd.arg(out_path.to_str().unwrap());

            let status = cmd.status()?;
            if !status.success() {
                bail!("Zig compilation failed");
            }

            eprintln!("{}", colors::success(&format!("Compiled {} → {} (Zig)", input.display(), out_path.display())));
        }
        Target::Python => {
            // Python: write .py file (interpreted, no compilation needed)
            let out_path = match output {
                Some(p) => p.to_path_buf(),
                None => {
                    let mut p = input.to_path_buf();
                    p.set_extension("py");
                    p
                }
            };
            fs::write(&out_path, &code)?;
            eprintln!("{}", colors::success(&format!("Transpiled {} → {} (Python)", input.display(), out_path.display())));
        }
        _ => bail!("Target {:?} not yet implemented", target),
    }

    Ok(())
}fn cmd_build(project: &Path, release: bool) -> Result<()> {
    let src_dir = project.join("src");
    if !src_dir.exists() {
        bail!("No src/ directory found in {}", project.display());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(&src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ruva") {
            files.push(path);
        }
    }

    if files.is_empty() {
        bail!("No .ruva files found in {}", src_dir.display());
    }

    eprintln!("⟳ Building {} .ruva files...", files.len());

    for file in &files {
        eprintln!("  → {}", file.display());
        cmd_compile(file, None, Target::Rust, release, false, false)?;
    }

    eprintln!("✓ Build complete");
    Ok(())
}

fn cmd_run(input: &Path, args: &[String]) -> Result<()> {
    let source = read_source(input)?;
    let code = transpile(&source, Target::Rust, input)?;

    let tmp_dir = std::env::temp_dir().join("ruva_build");
    fs::create_dir_all(&tmp_dir)?;

    let cargo_toml = tmp_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        fs::write(
            &cargo_toml,
            r#"[package]
name = "ruva_program"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )?;
        fs::create_dir_all(tmp_dir.join("src"))?;
    }

    let src_path = tmp_dir.join("src/main.rs");
    fs::write(&src_path, &code)?;

    eprintln!("⟳ Compiling...");
    let status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .current_dir(&tmp_dir)
        .status()?;

    if !status.success() {
        bail!("Compilation or execution failed");
    }

    Ok(())
}

fn cmd_check(input: &Path, all: bool) -> Result<()> {
    if all {
        // Check all .ruva files in the directory
        let dir = input.parent().unwrap_or(input);
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ruva") {
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
                // Resolve modules before type-checking
                let mut resolver = module::ModuleResolver::new(path);
                let program = match resolver.resolve_program(&program) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{}", colors::warning(&format!("Module resolution: {}", e)));
                        program
                    }
                };
                // Run type checker
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

fn cmd_transpile(input: &Path, output: Option<&Path>, target: Target, stdout: bool) -> Result<()> {
    let source = read_source(input)?;
    let code = transpile(&source, target, input)?;

    if stdout {
        print!("{}", code);
        return Ok(());
    }

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let mut p = input.to_path_buf();
            p.set_extension(target.file_extension());
            p
        }
    };

    fs::write(&out_path, &code)?;
    eprintln!("{}", colors::success(&format!("Transpiled {} → {} ({})", input.display(), out_path.display(), target)));
    Ok(())
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

        // Handle special commands
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
                    eprintln!("── Generated Rust ──");
                    eprintln!("{}", code);
                    eprintln!("───────────────────");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            continue;
        }

        // Add to buffer
        buffer.push_str(line);
        buffer.push('\n');

        // Try to transpile
        match transpile(&buffer, Target::Rust, Path::new("repl")) {
            Ok(code) => {
                eprintln!("── Generated Rust ──");
                eprintln!("{}", code);
                eprintln!("───────────────────");
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                // Remove the last line so user can retry
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

fn cmd_pipe(output: Option<&Path>, target: Target) -> Result<()> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;    let code = transpile(&source, target, Path::new("stdin"))?;

    if let Some(out_path) = output {
        fs::write(out_path, &code)?;
        eprintln!("{}", colors::success(&format!("Wrote to {} ({})", out_path.display(), target)));
    } else {
        print!("{}", code);
    }

    Ok(())
}

fn cmd_fmt(input: &Path, check: bool, dry_run: bool, verbose: bool) -> Result<()> {
    // Simple Ruva formatter: normalize indentation, trailing whitespace, and blank lines
    let mut stats = (0i64, 0i64, 0i64); // checked, changed, errors

    if input.is_dir() {
        // Format all .ruva files in directory
        for entry in fs::read_dir(input)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ruva") {
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
        // Format single file
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

/// Minimal Ruva formatter: strips trailing whitespace, normalizes blank lines,
/// ensures file ends with newline.
fn ruva_format(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut prev_blank = false;
    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim_end();

        // Track block comments
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

        // Collapse multiple blank lines into one
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

    // Ensure file ends with exactly one newline
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

    // Create Cargo.toml
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

    // Create main.ruva
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