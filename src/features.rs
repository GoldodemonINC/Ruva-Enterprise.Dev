use crate::ast::*;

#[derive(Debug, Clone)]
pub struct SecurityDiagnostic {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for SecurityDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "security at {}:{}: {}", self.line, self.col, self.message)
    }
}

fn decorator_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(name) => Some(name.clone()),
        Expr::Call { function, .. } => decorator_name(function),
        Expr::Path(parts) => parts.last().cloned(),
        _ => None,
    }
}

fn collect_let_bindings(block: &Block) -> Vec<String> {
    let mut names = Vec::new();
    for stmt in &block.stmts {
        if let Stmt::Let { pattern, is_mut: false, .. } = stmt {
            collect_idents_from_pattern(pattern, &mut names);
        }
    }
    names
}

fn collect_idents_from_pattern(pat: &Pattern, out: &mut Vec<String>) {
    match pat {
        Pattern::Ident(name) | Pattern::Mut(name) => out.push(name.clone()),
        Pattern::Tuple(pats) => {
            for p in pats {
                collect_idents_from_pattern(p, out);
            }
        }
        _ => {}
    }
}

// ── Unsafe detection ─────────────────────────────────────────────────────

fn block_has_unsafe(block: &Block) -> Option<Span> {
    block.stmts.iter()
        .filter_map(|s| stmt_has_unsafe(s))
        .chain(block.expr.as_ref().and_then(|e| expr_has_unsafe(e)))
        .next()
}

fn stmt_has_unsafe(stmt: &Stmt) -> Option<Span> {
    match stmt {
        Stmt::Unsafe(_) => Some(Span { line: 0, col: 0 }),
        Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
            expr_has_unsafe(value)
        }
        Stmt::If { condition, then_body, else_body } => {
            expr_has_unsafe(condition)
                .or_else(|| block_has_unsafe(then_body))
                .or_else(|| match else_body {
                    Some(ElseKind::If(e, b)) => expr_has_unsafe(e).or_else(|| block_has_unsafe(b)),
                    Some(ElseKind::Else(b)) => block_has_unsafe(b),
                    None => None,
                })
        }
        Stmt::For { iterable, body, .. } => {
            expr_has_unsafe(iterable).or_else(|| block_has_unsafe(body))
        }
        Stmt::While { condition, body } => {
            expr_has_unsafe(condition).or_else(|| block_has_unsafe(body))
        }
        Stmt::Loop(block) | Stmt::Block(block) => block_has_unsafe(block),
        Stmt::Match { expr, arms } => {
            expr_has_unsafe(expr).or_else(|| arms.iter().filter_map(|arm| expr_has_unsafe(&arm.body)).next())
        }
        _ => None,
    }
}

fn expr_has_unsafe(expr: &Expr) -> Option<Span> {
    match expr {
        Expr::UnsafeBlock(_) => Some(Span { line: 0, col: 0 }),
        Expr::Binary { left, right, .. } => expr_has_unsafe(left).or_else(|| expr_has_unsafe(right)),
        Expr::Call { function, args } | Expr::MethodCall { object: function, args, .. } => {
            expr_has_unsafe(function).or_else(|| args.iter().filter_map(|a| expr_has_unsafe(a)).next())
        }
        Expr::If { condition, then_body, else_body } => {
            expr_has_unsafe(condition)
                .or_else(|| block_has_unsafe(then_body))
                .or_else(|| else_body.as_ref().and_then(|e| expr_has_unsafe(e)))
        }
        Expr::Match { expr: scrutinee, arms } => {
            expr_has_unsafe(scrutinee).or_else(|| arms.iter().filter_map(|arm| expr_has_unsafe(&arm.body)).next())
        }
        Expr::Block(block) | Expr::Loop(block) => block_has_unsafe(block),
        Expr::Array(elems) | Expr::Tuple(elems) | Expr::VecLit(elems) => {
            elems.iter().filter_map(|e| expr_has_unsafe(e)).next()
        }
        Expr::StructLiteral { fields, .. } => {
            fields.iter().filter_map(|(_, v)| expr_has_unsafe(v)).next()
        }
        Expr::FString(parts) => parts.iter().filter_map(|p| match p {
            FStringPart::Expr(e) => expr_has_unsafe(e),
            _ => None,
        }).next(),
        _ => None,
    }
}

// ── Mutation detection for tamper_proof ───────────────────────────────────

fn block_mutates_any(block: &Block, names: &std::collections::HashSet<&str>) -> Option<String> {
    block.stmts.iter()
        .filter_map(|s| stmt_mutates_any(s, names))
        .chain(block.expr.as_ref().and_then(|e| expr_mutates_any(e, names)))
        .next()
}

fn stmt_mutates_any(stmt: &Stmt, names: &std::collections::HashSet<&str>) -> Option<String> {
    match stmt {
        Stmt::Let { value, .. } => expr_mutates_any(value, names),
        Stmt::Expr(e) | Stmt::Return(Some(e)) => expr_mutates_any(e, names),
        Stmt::If { condition, then_body, else_body } => {
            expr_mutates_any(condition, names)
                .or_else(|| block_mutates_any(then_body, names))
                .or_else(|| match else_body {
                    Some(ElseKind::If(e, b)) => expr_mutates_any(e, names).or_else(|| block_mutates_any(b, names)),
                    Some(ElseKind::Else(b)) => block_mutates_any(b, names),
                    None => None,
                })
        }
        Stmt::For { iterable, body, .. } => {
            expr_mutates_any(iterable, names).or_else(|| block_mutates_any(body, names))
        }
        Stmt::While { condition, body } => {
            expr_mutates_any(condition, names).or_else(|| block_mutates_any(body, names))
        }
        Stmt::Loop(block) | Stmt::Block(block) => block_mutates_any(block, names),
        Stmt::Match { expr, arms } => {
            expr_mutates_any(expr, names).or_else(|| arms.iter().filter_map(|arm| expr_mutates_any(&arm.body, names)).next())
        }
        _ => None,
    }
}

fn expr_mutates_any(expr: &Expr, names: &std::collections::HashSet<&str>) -> Option<String> {
    match expr {
        Expr::Assign { target, .. } | Expr::CompoundAssign { target, .. } => {
            if let Expr::Ident(name) = target.as_ref() {
                if names.contains(name.as_str()) {
                    return Some(name.clone());
                }
            }
            None
        }
        Expr::Binary { left, right, .. } => {
            expr_mutates_any(left, names).or_else(|| expr_mutates_any(right, names))
        }
        Expr::If { condition, then_body, else_body } => {
            expr_mutates_any(condition, names)
                .or_else(|| block_mutates_any(then_body, names))
                .or_else(|| else_body.as_ref().and_then(|e| expr_mutates_any(e, names)))
        }
        Expr::Match { expr: scrutinee, arms } => {
            expr_mutates_any(scrutinee, names).or_else(|| arms.iter().filter_map(|arm| expr_mutates_any(&arm.body, names)).next())
        }
        Expr::Block(block) | Expr::Loop(block) => block_mutates_any(block, names),
        _ => None,
    }
}

// ── Public API ───────────────────────────────────────────────────────────

pub fn check_security_annotations(program: &Program) -> Vec<SecurityDiagnostic> {
    let mut diagnostics = Vec::new();
    for item in &program.items {
        check_item_annotations(item, &mut diagnostics);
    }
    diagnostics
}

fn check_item_annotations(item: &Item, diagnostics: &mut Vec<SecurityDiagnostic>) {
    match item {
        Item::Decorated(dec) => {
            for expr in &dec.decorators {
                if let Some(name) = decorator_name(expr) {
                    match name.as_str() {
                        "safe" => check_safe_annotation(&dec.definition, diagnostics),
                        "tamper_proof" => check_tamper_proof_annotation(&dec.definition, diagnostics),
                        "rate_limited" => {
                            let item_name = match dec.definition.as_ref() {
                                Item::Function(f) => f.name.clone(),
                                Item::Class(c) => c.name.clone(),
                                _ => "unknown".to_string(),
                            };
                            diagnostics.push(SecurityDiagnostic {
                                message: format!("@rate_limited on '{}' — rate limiting must be enforced at the call site or via middleware", item_name),
                                line: 0, col: 0,
                            });
                        }
                        _ => {}
                    }
                }
            }
            check_item_annotations(&dec.definition, diagnostics);
        }
        Item::Function(_) | Item::Class(_) | Item::Impl(_) | Item::Trait(_) => {}
        Item::Module(m) => {
            if let Some(ref body) = m.body {
                for inner in body { check_item_annotations(inner, diagnostics); }
            }
        }
        _ => {}
    }
}


// ── @safe ────────────────────────────────────────────────────────────────

fn check_method_unsafe(method: &FunctionDef, context: &str, diagnostics: &mut Vec<SecurityDiagnostic>) {
    if method.is_unsafe {
        diagnostics.push(SecurityDiagnostic {
            message: format!("@safe on {} but method '{}' is declared unsafe", context, method.name),
            line: method.span.line, col: method.span.col,
        });
    }
    if let Some(span) = block_has_unsafe(&method.body) {
        diagnostics.push(SecurityDiagnostic {
            message: format!("@safe on {} but method '{}' contains unsafe operations", context, method.name),
            line: span.line, col: span.col,
        });
    }
}

fn check_safe_annotation(item: &Item, diagnostics: &mut Vec<SecurityDiagnostic>) {
    match item {
        Item::Function(f) => {
            if f.is_unsafe {
                diagnostics.push(SecurityDiagnostic {
                    message: format!("@safe on '{}' but function is declared unsafe", f.name),
                    line: f.span.line, col: f.span.col,
                });
            }
            if let Some(span) = block_has_unsafe(&f.body) {
                diagnostics.push(SecurityDiagnostic {
                    message: format!("@safe on '{}' but body contains unsafe operations", f.name),
                    line: span.line, col: span.col,
                });
            }
        }
        Item::Class(c) => {
            for method in &c.methods {
                check_method_unsafe(method, &format!("class '{}'", c.name), diagnostics);
            }
        }
        Item::Impl(imp) => {
            for method in &imp.methods {
                check_method_unsafe(method, "impl block", diagnostics);
            }
        }
        _ => {}
    }
}

// ── @tamper_proof ────────────────────────────────────────────────────────

fn check_tamper_proof_annotation(item: &Item, diagnostics: &mut Vec<SecurityDiagnostic>) {
    match item {
        Item::Function(f) => check_tamper_proof_body(&f.body, &f.name, diagnostics),
        Item::Class(c) => {
            for method in &c.methods {
                check_tamper_proof_body(&method.body, &format!("{}.{}", c.name, method.name), diagnostics);
            }
        }
        Item::Impl(imp) => {
            for method in &imp.methods {
                check_tamper_proof_body(&method.body, &method.name, diagnostics);
            }
        }
        _ => {}
    }
}

fn check_tamper_proof_body(block: &Block, context_name: &str, diagnostics: &mut Vec<SecurityDiagnostic>) {
    let bindings = collect_let_bindings(block);
    if bindings.is_empty() { return; }

    let binding_set: std::collections::HashSet<&str> = bindings.iter().map(|s| s.as_str()).collect();
    if let Some(var_name) = block_mutates_any(block, &binding_set) {
        diagnostics.push(SecurityDiagnostic {
            message: format!("@tamper_proof on '{}' but variable '{}' is mutated", context_name, var_name),
            line: 0, col: 0,
        });
    }
}

// ── Legacy types (kept for backward compat) ──────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel { Safe, Trusted, Unsafe }

#[derive(Debug, Clone)]
pub enum Annotation {
    Safe, Trusted, Unsafe, Immutable, TamperProof,
    NoMemoryEdit, Checksum, RateLimited, ConnectionIsolated,
}

impl Annotation {
    pub fn from_attribute(name: &str) -> Option<Self> {
        match name {
            "safe" => Some(Annotation::Safe),
            "trusted" => Some(Annotation::Trusted),
            "unsafe" => Some(Annotation::Unsafe),
            "immutable" => Some(Annotation::Immutable),
            "tamper_proof" => Some(Annotation::TamperProof),
            "no_memory_edit" => Some(Annotation::NoMemoryEdit),
            "checksum" => Some(Annotation::Checksum),
            "rate_limited" => Some(Annotation::RateLimited),
            "connection_isolated" => Some(Annotation::ConnectionIsolated),
            _ => None,
        }
    }
}

pub struct FeatureFlags {
    pub security_level: SecurityLevel,
    pub annotations: Vec<Annotation>,
}

pub struct FeatureChecker { pub flags: FeatureFlags }

impl FeatureChecker {
    pub fn new() -> Self {
        Self { flags: FeatureFlags { security_level: SecurityLevel::Safe, annotations: Vec::new() } }
    }
}

#[derive(Debug, Clone)]
pub enum CodegenHint {
    UseAtomic, UseSafeMemory, InsertBoundsCheck, UsePoolAllocator, UseLockFreeStructure,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn check(source: &str) -> Vec<SecurityDiagnostic> {
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();
        check_security_annotations(&program)
    }

    #[test]
    fn safe_rejects_unsafe_block() {
        let d = check(r#"@safe fn f() { unsafe { let p: *mut i32 = null_mut() } }"#);
        assert!(!d.is_empty() && d[0].message.contains("@safe"));
    }

    #[test]
    fn safe_rejects_unsafe_fn() {
        let d = check(r#"@safe unsafe fn helper() { }"#);
        assert!(!d.is_empty() && d[0].message.contains("unsafe"));
    }

    #[test]
    fn safe_passes_clean_code() {
        assert!(check(r#"@safe fn f() { let x = 42 }"#).is_empty());
    }

    #[test]
    fn tamper_proof_rejects_mutation() {
        let d = check(r#"@tamper_proof fn f() { let c = "x"; c = "y" }"#);
        assert!(!d.is_empty() && d[0].message.contains("c"));
    }

    #[test]
    fn tamper_proof_allows_reads() {
        assert!(check(r#"@tamper_proof fn f() { let c = "x"; let y = c }"#).is_empty());
    }

    #[test]
    fn tamper_proof_compound_assign_flagged() {
        let d = check(r#"@tamper_proof fn f() { let c = 0; c += 1 }"#);
        assert!(!d.is_empty() && d[0].message.contains("c"));
    }

    #[test]
    fn rate_limited_emits_diagnostic() {
        let d = check(r#"@rate_limited fn handle() { }"#);
        assert!(d.len() == 1 && d[0].message.contains("@rate_limited"));
    }

    #[test]
    fn no_annotations_clean() {
        assert!(check(r#"fn main() { let x = 1 }"#).is_empty());
    }

    #[test]
    fn nested_unsafe_in_safe_flagged() {
        let d = check(r#"@safe fn f() { if true { unsafe { let p: *mut i32 = null_mut() } } }"#);
        assert!(!d.is_empty());
    }
}
