//! Golden/snapshot tests for the Rust transpiler backend.
//!
//! Each test reads a .ruva source file, transpiles it via the Rust backend,
//! and compares the output against a golden file in tests/golden/.
//!
//! To regenerate all golden files after an intentional change:
//!   GOLDEN_BLESS=1 cargo test --test golden_tests
//!
//! Error golden tests verify that error/diagnostic output remains stable.
//! Syntax errors use `transpile`, type errors use `check`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// All golden test cases: (input_name, input_extension)
const GOLDEN_CASES: &[&str] = &[
    "hello",
    "variables",
    "control_flow",
    "functions",
    "structs",
    "enums",
    "classes",
    "generics",
    "traits",
    "error_handling",
    "macros",
    "extern_ffi",
    "closures_iter",
];

fn project_root() -> PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest)
}

fn transpile_ruva_to_rust(input_path: &std::path::Path) -> String {
    let root = project_root();
    let exe = if cfg!(target_os = "windows") {
        root.join("target/debug/ruva.exe")
    } else {
        root.join("target/debug/ruva")
    };

    // Build the binary if it doesn't exist yet
    if !exe.exists() {
        let status = Command::new("cargo")
            .args(["build"])
            .current_dir(&root)
            .status()
            .expect("Failed to run cargo build");
        assert!(status.success(), "cargo build failed");
    }

    let output = Command::new(&exe)
        .args(["transpile"])
        .arg(input_path)
        .arg("--target")
        .arg("rust")
        .arg("--stdout")
        .current_dir(&root)
        .output()
        .expect("Failed to run ruva transpile");

    assert!(
        output.status.success(),
        "ruva transpile failed on {}: {}",
        input_path.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("Output is not valid UTF-8")
}

fn golden_path(name: &str) -> PathBuf {
    project_root().join(format!("tests/golden/{}.golden", name))
}

/// Resolve a source file that may be named `.rve` or `.ruva`, so the harness works
/// both before and after the extension rename.
fn resolve_ruva_file(dir: &Path, name: &str) -> PathBuf {
    for ext in [".rve", ".ruva"] {
        let p = dir.join(format!("{}{}", name, ext));
        if p.exists() { return p; }
    }
    dir.join(format!("{}.ruva", name)) // default; the harness surfaces a clear error if missing
}

fn input_path(name: &str) -> PathBuf {
    resolve_ruva_file(&project_root().join("tests/transpiler_golden"), name)
}

// Error golden helpers

const ERROR_GOLDEN_CASES: &[&str] = &[
    "syntax_unterminated_string",
    "syntax_unexpected_token",
    "syntax_mismatched_braces",
    "syntax_unterminated_comment",
    "type_undefined_var",
    "type_mismatch",
    "type_arg_count",
    "type_assign_undef",
    "type_bool_arithmetic",
];

fn error_input_path(name: &str) -> PathBuf {
    resolve_ruva_file(&project_root().join("tests/transpiler_golden/errors"), name)
}

fn error_golden_path(name: &str) -> PathBuf {
    project_root().join(format!("tests/golden/errors/{}.golden", name))
}

fn ruva_exe() -> PathBuf {
    let root = project_root();
    if cfg!(target_os = "windows") {
        root.join("target/debug/ruva.exe")
    } else {
        root.join("target/debug/ruva")
    }
}

fn ensure_built() {
    let exe = ruva_exe();
    if !exe.exists() {
        let status = Command::new("cargo")
            .args(["build"])
            .current_dir(project_root())
            .status()
            .expect("Failed to run cargo build");
        assert!(status.success(), "cargo build failed");
    }
}

/// Filter out noise from stderr: built-in variable unused warnings and success lines.
/// Keeps actual errors and meaningful warnings.
fn filter_stderr(raw: &str) -> String {
    raw.lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Skip built-in variable unused warnings
            if trimmed.contains("is never used") && trimmed.contains("Variable '") {
                return false;
            }
            // Skip success lines from check command
            if trimmed.starts_with('✓') {
                return false;
            }
            !trimmed.is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Run `transpile` on a file and capture filtered stderr (for parser/syntax errors).
fn run_transpile_error(input_path: &std::path::Path) -> String {
    ensure_built();
    let exe = ruva_exe();
    let root = project_root();

    let output = Command::new(&exe)
        .args(["transpile"])
        .arg(input_path)
        .arg("--target")
        .arg("rust")
        .arg("--stdout")
        .current_dir(&root)
        .output()
        .expect("Failed to run ruva transpile");

    let stderr = String::from_utf8_lossy(&output.stderr);
    filter_stderr(&stderr).trim().to_string()
}

/// Run `check` on a file and capture filtered stderr (for type/diagnostic errors).
fn run_check_error(input_path: &std::path::Path) -> String {
    ensure_built();
    let exe = ruva_exe();
    let root = project_root();

    let output = Command::new(&exe)
        .args(["check"])
        .arg(input_path)
        .current_dir(&root)
        .output()
        .expect("Failed to run ruva check");

    let stderr = String::from_utf8_lossy(&output.stderr);
    filter_stderr(&stderr).trim().to_string()
}

#[test]
fn test_golden_files_exist() {
    let root = project_root();
    assert!(
        root.join("tests/golden").exists(),
        "tests/golden/ directory does not exist"
    );
    assert!(
        root.join("tests/transpiler_golden").exists(),
        "tests/transpiler_golden/ directory does not exist"
    );
    for name in GOLDEN_CASES {
        let golden = golden_path(name);
        assert!(
            golden.exists(),
            "Missing golden file: {}",
            golden.display()
        );
        let input = input_path(name);
        assert!(
            input.exists(),
            "Missing input file: {}",
            input.display()
        );
    }
}

macro_rules! golden_test {
    ($name:ident, $case:expr) => {
        #[test]
        fn $name() {
            let case = $case;
            let input = input_path(case);
            let golden = golden_path(case);
            let bless = std::env::var("GOLDEN_BLESS").is_ok();

            let actual = transpile_ruva_to_rust(&input);

            if bless {
                fs::write(&golden, &actual).expect("Failed to write golden file");
                eprintln!("Blessed golden file: {}", golden.display());
                return;
            }

            let expected = fs::read_to_string(&golden)
                .unwrap_or_else(|e| panic!("Failed to read golden file {}: {}", golden.display(), e));

            if actual != expected {
                let diff = simple_diff(&expected, &actual);
                panic!(
                    "Golden test '{}' failed!\n\nInput: {}\nGolden: {}\n\nDiff:\n{}",
                    case,
                    input.display(),
                    golden.display(),
                    diff
                );
            }
        }
    };
}

/// Produce a minimal unified-style diff between expected and actual.
fn simple_diff(expected: &str, actual: &str) -> String {
    let exp_lines: Vec<&str> = expected.lines().collect();
    let act_lines: Vec<&str> = actual.lines().collect();
    let mut diff = String::new();
    let max = exp_lines.len().max(act_lines.len());

    for i in 0..max {
        match (exp_lines.get(i), act_lines.get(i)) {
            (Some(e), Some(a)) if e != a => {
                diff.push_str(&format!("  line {}: - {}\n", i + 1, e));
                diff.push_str(&format!("  line {}: + {}\n", i + 1, a));
            }
            (Some(e), None) => {
                diff.push_str(&format!("  line {}: - {}\n", i + 1, e));
            }
            (None, Some(a)) => {
                diff.push_str(&format!("  line {}: + {}\n", i + 1, a));
            }
            _ => {}
        }
    }

    if diff.is_empty() {
        "  (no differences)\n".to_string()
    } else {
        diff
    }
}

// Generate one test function per golden case
golden_test!(golden_hello, "hello");
golden_test!(golden_variables, "variables");
golden_test!(golden_control_flow, "control_flow");
golden_test!(golden_functions, "functions");
golden_test!(golden_structs, "structs");
golden_test!(golden_enums, "enums");
golden_test!(golden_classes, "classes");
golden_test!(golden_generics, "generics");
golden_test!(golden_traits, "traits");
golden_test!(golden_error_handling, "error_handling");
golden_test!(golden_macros, "macros");
golden_test!(golden_extern_ffi, "extern_ffi");
golden_test!(golden_closures_iter, "closures_iter");

// Error golden tests

#[test]
fn test_error_golden_files_exist() {
    for name in ERROR_GOLDEN_CASES {
        let golden = error_golden_path(name);
        assert!(
            golden.exists(),
            "Missing error golden file: {}",
            golden.display()
        );
        let input = error_input_path(name);
        assert!(
            input.exists(),
            "Missing error input file: {}",
            input.display()
        );
    }
}

/// Macro for syntax error golden tests (parser failures via `transpile`).
macro_rules! syntax_error_test {
    ($name:ident, $case:expr) => {
        #[test]
        fn $name() {
            let case = $case;
            let input = error_input_path(case);
            let golden = error_golden_path(case);
            let bless = std::env::var("GOLDEN_BLESS").is_ok();

            let actual = run_transpile_error(&input);

            if bless {
                fs::write(&golden, &actual).expect("Failed to write error golden file");
                eprintln!("Blessed error golden file: {}", golden.display());
                return;
            }

            let expected = fs::read_to_string(&golden)
                .unwrap_or_else(|e| panic!("Failed to read error golden file {}: {}", golden.display(), e));

            if actual != expected {
                let diff = simple_diff(&expected, &actual);
                panic!(
                    "Error golden test '{}' failed!\n\nInput: {}\nGolden: {}\n\nDiff:\n{}",
                    case,
                    input.display(),
                    golden.display(),
                    diff
                );
            }
        }
    };
}

/// Macro for type error golden tests (diagnostic failures via `check`).
macro_rules! type_error_test {
    ($name:ident, $case:expr) => {
        #[test]
        fn $name() {
            let case = $case;
            let input = error_input_path(case);
            let golden = error_golden_path(case);
            let bless = std::env::var("GOLDEN_BLESS").is_ok();

            let actual = run_check_error(&input);

            if bless {
                fs::write(&golden, &actual).expect("Failed to write error golden file");
                eprintln!("Blessed error golden file: {}", golden.display());
                return;
            }

            let expected = fs::read_to_string(&golden)
                .unwrap_or_else(|e| panic!("Failed to read error golden file {}: {}", golden.display(), e));

            if actual != expected {
                let diff = simple_diff(&expected, &actual);
                panic!(
                    "Error golden test '{}' failed!\n\nInput: {}\nGolden: {}\n\nDiff:\n{}",
                    case,
                    input.display(),
                    golden.display(),
                    diff
                );
            }
        }
    };
}

// Syntax error cases (parser failures)
syntax_error_test!(error_golden_syntax_unterminated_string, "syntax_unterminated_string");
syntax_error_test!(error_golden_syntax_unexpected_token, "syntax_unexpected_token");
syntax_error_test!(error_golden_syntax_mismatched_braces, "syntax_mismatched_braces");
syntax_error_test!(error_golden_syntax_unterminated_comment, "syntax_unterminated_comment");

// Type error cases (diagnostic failures)
type_error_test!(error_golden_type_undefined_var, "type_undefined_var");
type_error_test!(error_golden_type_mismatch, "type_mismatch");
type_error_test!(error_golden_type_arg_count, "type_arg_count");
type_error_test!(error_golden_type_assign_undef, "type_assign_undef");
type_error_test!(error_golden_type_bool_arithmetic, "type_bool_arithmetic");
