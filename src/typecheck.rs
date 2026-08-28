use crate::ast::*;

// ─── Structured Type Representation ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Var(TypeVar),
    Primitive(String),
    Named(String),
    Generic(String, Vec<Ty>),
    Reference(Box<Ty>, bool),
    RawPointer(Box<Ty>, bool),
    Array(Box<Ty>),
    Slice(Box<Ty>),
    Tuple(Vec<Ty>),
    FnPointer { params: Vec<Ty>, ret: Box<Ty> },
    Unit,
    Never,
    Inferred,
}

type TypeVar = usize;

// ─── Type Variable Table ───────────────────────────────────────────────────

struct TypeTable {
    next_var: TypeVar,
}

impl TypeTable {
    fn new() -> Self {
        Self { next_var: 0 }
    }

    fn fresh_var(&mut self) -> Ty {
        let v = self.next_var;
        self.next_var += 1;
        Ty::Var(v)
    }
}

// ─── Structured Type Checker ───────────────────────────────────────────────

struct Scope {
    variables: std::collections::HashMap<String, Ty>,
    used: std::collections::HashMap<String, bool>,
}

pub struct TypeChecker {
    scopes: Vec<Scope>,
    functions: std::collections::HashMap<String, FunctionSig>,
    struct_fields: std::collections::HashMap<String, Vec<(String, Ty)>>,
    type_aliases: std::collections::HashMap<String, Ty>,
    /// Methods keyed by receiver type: TypeName -> method name -> signature
    class_methods: std::collections::HashMap<String, std::collections::HashMap<String, FunctionSig>>,
    /// Enclosing class while checking its methods (used to resolve `self.field`)
    current_class: Option<String>,
    diagnostics: Vec<Diagnostic>,
    current_return_type: Option<Ty>,
    in_unsafe_block: bool,
    in_unsafe_fn: bool,
    type_table: TypeTable,
    /// Tracks the current source location for error reporting
    current_line: usize,
    current_col: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Clone)]
struct FunctionSig {
    params: Vec<(String, Ty)>,
    return_type: Option<Ty>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticKind {
    Error,
    Warning,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            DiagnosticKind::Error => write!(f, "error at {}:{}: {}", self.line, self.col, self.message),
            DiagnosticKind::Warning => write!(f, "warning at {}:{}: {}", self.line, self.col, self.message),
        }
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            scopes: vec![Scope {
                variables: std::collections::HashMap::new(),
                used: std::collections::HashMap::new(),
            }],
            functions: std::collections::HashMap::new(),
            struct_fields: std::collections::HashMap::new(),
            type_aliases: std::collections::HashMap::new(),
            class_methods: std::collections::HashMap::new(),
            current_class: None,
            diagnostics: vec![],
            current_return_type: None,
            in_unsafe_block: false,
            in_unsafe_fn: false,
            type_table: TypeTable::new(),
            current_line: 0,
            current_col: 0,
            warning_count: 0,
            error_count: 0,
        };

        // Register built-in constructors
        for b in &["Some", "None", "Ok", "Err", "Self"] {
            checker.define_var(b.to_string(), Ty::Named(b.to_string()), 0);
        }
        checker
    }

    /// Check a program and return diagnostics.
    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        // First pass: collect function signatures and struct definitions
        for item in &program.items {
            self.register_item(item);
        }

        // Second pass: check all items
        for item in &program.items {
            self.check_item(item);
        }

        // Report unused variables
        self.report_unused();

        std::mem::take(&mut self.diagnostics)
    }

    // ─── Registration Pass ─────────────────────────────────────────────

    fn register_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                let params: Vec<(String, Ty)> = f.params.iter().map(|p| {
                    (p.name.clone(), self.ast_type_to_ty(&p.ty))
                }).collect();
                let ret_type = f.return_type.as_ref().map(|t| self.ast_type_to_ty(t));
                self.functions.insert(f.name.clone(), FunctionSig { params, return_type: ret_type });
                self.define_var(f.name.clone(), Ty::Named(f.name.clone()), f.span.line);
            }
            Item::Class(c) => {
                let fields: Vec<(String, Ty)> = c.fields.iter().map(|f| {
                    (f.name.clone(), self.ast_type_to_ty(&f.ty))
                }).collect();
                self.struct_fields.insert(c.name.clone(), fields);
                self.define_var(c.name.clone(), Ty::Named(c.name.clone()), c.span.line);
                // Register methods keyed by the receiver type so same-named
                // methods on different classes don't collide
                for m in &c.methods {
                    let params: Vec<(String, Ty)> = m.params.iter().map(|p| {
                        (p.name.clone(), self.ast_type_to_ty(&p.ty))
                    }).collect();
                    let ret = m.return_type.as_ref().map(|t| self.ast_type_to_ty(t));
                    self.class_methods.entry(c.name.clone()).or_default()
                        .insert(m.name.clone(), FunctionSig { params, return_type: ret });
                }
            }
            Item::Struct(s) => {
                let fields: Vec<(String, Ty)> = s.fields.iter().map(|f| {
                    (f.name.clone(), self.ast_type_to_ty(&f.ty))
                }).collect();
                self.struct_fields.insert(s.name.clone(), fields);
                self.define_var(s.name.clone(), Ty::Named(s.name.clone()), s.span.line);
            }
            Item::Enum(e) => {
                self.define_var(e.name.clone(), Ty::Named(e.name.clone()), e.span.line);
                // Register enum variants as constructors
                for v in &e.variants {
                    let ret = Ty::Generic(e.name.clone(), vec![]);
                    self.functions.insert(v.name.clone(), FunctionSig {
                        params: v.fields.iter().enumerate().map(|(i, t)| {
                            (format!("__{}_{}", v.name, i), self.ast_type_to_ty(t))
                        }).collect(),
                        return_type: Some(ret),
                    });
                }
            }
            Item::Impl(imp) => {
                let target = self.type_name(&imp.self_type);
                for m in &imp.methods {
                    let params: Vec<(String, Ty)> = m.params.iter().map(|p| {
                        (p.name.clone(), self.ast_type_to_ty(&p.ty))
                    }).collect();
                    let ret = m.return_type.as_ref().map(|t| self.ast_type_to_ty(t));
                    if let Some(ref tname) = target {
                        self.class_methods.entry(tname.clone()).or_default()
                            .insert(m.name.clone(), FunctionSig { params, return_type: ret });
                    } else {
                        self.functions.insert(m.name.clone(), FunctionSig { params, return_type: ret });
                    }
                }
            }
            Item::Trait(t) => {
                for m in &t.methods {
                    let params: Vec<(String, Ty)> = m.params.iter().map(|p| {
                        (p.name.clone(), self.ast_type_to_ty(&p.ty))
                    }).collect();
                    let ret = m.return_type.as_ref().map(|t| self.ast_type_to_ty(t));
                    self.class_methods.entry(t.name.clone()).or_default()
                        .insert(m.name.clone(), FunctionSig { params, return_type: ret });
                }
            }
            Item::Use(u) => {
                if u.wildcard {
                    if let Some(last) = u.path.last() {
                        self.define_var(last.to_string(), Ty::Named(last.to_string()), 0);
                    }
                } else if !u.selective.is_empty() {
                    for item in &u.selective {
                        let name = item.alias.as_ref().unwrap_or(&item.name);
                        self.define_var(name.to_string(), Ty::Inferred, 0);
                    }
                } else if let Some(ref alias) = u.alias {
                    self.define_var(alias.to_string(), Ty::Inferred, 0);
                } else if let Some(last) = u.path.last() {
                    self.define_var(last.to_string(), Ty::Named(last.to_string()), 0);
                }
            }
            Item::TypeAlias(ta) => {
                // Register `type X = Y` so declared types referencing X resolve to Y
                let resolved = self.ast_type_to_ty(&ta.ty);
                self.type_aliases.insert(ta.name.clone(), resolved.clone());
                self.define_var(ta.name.clone(), resolved, 0);
            }
            Item::Const(c) => {
                let ty = c.ty.as_ref().map(|t| self.ast_type_to_ty(t))
                    .unwrap_or_else(|| self.infer_type(&c.value));
                self.define_var(c.name.clone(), ty, 0);
            }
            Item::Module(m) => {
                self.define_var(m.name.clone(), Ty::Named(m.name.clone()), 0);
                if let Some(ref body) = m.body {
                    for inner in body {
                        self.register_item(inner);
                    }
                }
            }
            Item::ExternBlock(eb) => {
                // Security: validate ABI string
                let valid_abis = ["C", "system", "cdecl", "stdcall", "fastcall", "vectorcall", "thiscall", "unwind"];
                if !valid_abis.contains(&eb.abi.as_str()) {
                    self.warn(
                        format!("Unknown ABI '{}' in extern block — expected one of: {}",
                            eb.abi, valid_abis.join(", ")),
                        0, 0,
                    );
                }

                for ei in &eb.items {
                    match ei {
                        ExternItem::Function { name, params, return_type, .. } => {
                            // Security: warn about dangerous FFI functions
                            let dangerous = ["exec", "system", "popen", "eval", "dlopen",
                                "LoadLibrary", "CreateProcess", "ShellExecute"];
                            if dangerous.iter().any(|d| name.eq_ignore_ascii_case(d)) {
                                self.warn(
                                    format!("FFI function '{}' is potentially dangerous — ensure input is validated",
                                        name),
                                    0, 0,
                                );
                            }

                            // Security: warn about functions returning raw pointers without safety annotation
                            if let Some(ref ret) = return_type {
                                if matches!(ret, Type::RawPointer { .. }) {
                                    self.warn(
                                        format!("FFI function '{}' returns a raw pointer — caller must ensure memory safety",
                                            name),
                                        0, 0,
                                    );
                                }
                            }

                            let ps: Vec<(String, Ty)> = params.iter().map(|p| {
                                (p.name.clone(), self.ast_type_to_ty(&p.ty))
                            }).collect();
                            let ret = return_type.as_ref().map(|t| self.ast_type_to_ty(t));
                            self.functions.insert(name.clone(), FunctionSig { params: ps, return_type: ret });
                        }
                        ExternItem::Static { name, ty, is_mut, .. } => {
                            // Security: warn about mutable statics
                            if *is_mut {
                                self.warn(
                                    format!("Mutable static '{}' in FFI — requires unsafe access and careful synchronization",
                                        name),
                                    0, 0,
                                );
                            }
                            self.define_var(name.clone(), self.ast_type_to_ty(ty), 0);
                        }
                        ExternItem::Const { name, ty, .. } => {
                            self.define_var(name.clone(), self.ast_type_to_ty(ty), 0);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // ─── Checking Pass ─────────────────────────────────────────────────

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                self.current_line = f.span.line;
                self.current_col = f.span.col;
                self.check_function(f);
            }
            Item::Class(c) => {
                self.current_line = c.span.line;
                self.current_col = c.span.col;
                let saved_class = self.current_class.take();
                self.current_class = Some(c.name.clone());
                for method in &c.methods {
                    self.check_function(method);
                }
                self.current_class = saved_class;
            }
            Item::Impl(imp) => {
                self.current_line = imp.span.line;
                self.current_col = imp.span.col;
                let saved_class = self.current_class.take();
                self.current_class = self.type_name(&imp.self_type);
                for method in &imp.methods {
                    self.check_function(method);
                }
                self.current_class = saved_class;
            }
            Item::Trait(t) => {
                for method in &t.methods {
                    if let Some(ref body) = method.default_body {
                        self.push_scope();
                        for p in &method.params {
                            if !matches!(p.ty, Type::SelfType) {
                                let ty = self.ast_type_to_ty(&p.ty);
                                self.define_var(p.name.clone(), ty, 0);
                            }
                        }
                        self.check_block(body);
                        self.pop_scope();
                    }
                }
            }
            Item::Module(m) => {
                if let Some(ref body) = m.body {
                    self.push_scope();
                    for inner in body {
                        self.check_item(inner);
                    }
                    self.pop_scope();
                }
            }
            Item::ExternBlock(eb) => {
                for ei in &eb.items {
                    if let ExternItem::Function { params, .. } = ei {
                        // Extern functions can reference types that may not be in scope yet
                        // Just verify parameter types are valid
                        for p in params {
                            self.verify_type(&p.ty, 0);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_function(&mut self, f: &FunctionDef) {
        self.current_line = f.span.line;
        self.current_col = f.span.col;
        let old_return = self.current_return_type.clone();
        self.current_return_type = f.return_type.as_ref().map(|t| self.ast_type_to_ty(t));

        let was_unsafe_fn = self.in_unsafe_fn;
        if f.is_unsafe {
            self.in_unsafe_fn = true;
        }

        self.push_scope();

        for p in &f.params {
            if !matches!(p.ty, Type::SelfType) {
                let ty = self.ast_type_to_ty(&p.ty);
                self.define_var(p.name.clone(), ty, f.span.line);
            }
        }

        self.check_block(&f.body);

        // Check return type presence
        if let Some(ref ret_ty) = self.current_return_type {
            if !self.ty_is_unit(ret_ty) && !self.block_has_return(&f.body) {
                self.warn(
                    format!("Function '{}' may not return a value of type {}", f.name, self.ty_to_string(ret_ty)),
                    f.span.line, 0,
                );
            }
        }

        self.pop_scope();
        self.current_return_type = old_return;
        self.in_unsafe_fn = was_unsafe_fn;
    }

    fn block_has_return(&self, block: &Block) -> bool {
        for stmt in &block.stmts {
            if matches!(stmt, Stmt::Return(Some(_))) {
                return true;
            }
            if let Stmt::If { then_body, else_body, .. } = stmt {
                if let Some(ElseKind::Else(else_body)) = else_body {
                    if self.block_has_return(then_body) && self.block_has_return(else_body) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        if let Some(ref expr) = block.expr {
            self.check_expr(expr);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { pattern, ty, value, .. } => {
                let val_ty = self.infer_type(value);
                self.check_expr(value);

                let names = self.pattern_names(pattern);
                for name in names {
                    // Determine the variable's type
                    let var_ty = if let Some(declared) = ty {
                        let declared_ty = self.ast_type_to_ty(declared);
                        if !self.ty_is_inferred(&val_ty) && !self.types_compatible(&declared_ty, &val_ty) {
                            self.error(
                                format!("Type mismatch: expected '{}', got '{}'",
                                    self.ty_to_string(&declared_ty), self.ty_to_string(&val_ty)),
                                0, 0,
                            );
                        }
                        declared_ty
                    } else {
                        val_ty.clone()
                    };

                    if self.is_defined_in_current_scope(&name) {
                        self.warn(format!("Variable '{}' shadows a previous binding", name), 0, 0);
                    }
                    self.define_var(name, var_ty, 0);
                }
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e);
                    if let Some(ref expected) = self.current_return_type.clone() {
                        let actual = self.infer_type(e);
                        if !self.ty_is_inferred(&actual) && !self.types_compatible(expected, &actual)
                            && !self.ty_mentions_impl(expected) {
                            self.error(
                                format!("Return type mismatch: expected '{}', got '{}'",
                                    self.ty_to_string(expected), self.ty_to_string(&actual)),
                                0, 0,
                            );
                        }
                    }
                } else {
                    if let Some(ref expected) = self.current_return_type.clone() {
                        if !self.ty_is_unit(expected) {
                            self.error(
                                format!("Function expects return type '{}', but return has no value",
                                    self.ty_to_string(expected)),
                                0, 0,
                            );
                        }
                    }
                }
            }
            Stmt::If { condition, then_body, else_body } => {
                self.check_expr(condition);
                let cond_ty = self.infer_type(condition);
                if !self.ty_is_inferred(&cond_ty) && !self.ty_is_bool(&cond_ty) && !self.ty_is_unit(&cond_ty) {
                    self.warn(
                        format!("Condition should be bool, got '{}'", self.ty_to_string(&cond_ty)),
                        0, 0,
                    );
                }
                self.check_block(then_body);
                if let Some(else_kind) = else_body {
                    match else_kind {
                        ElseKind::If(cond, body) => {
                            self.check_expr(cond);
                            self.check_block(body);
                        }
                        ElseKind::Else(body) => {
                            self.check_block(body);
                        }
                    }
                }
            }
            Stmt::For { pattern, iterable, body } => {
                self.check_expr(iterable);
                self.push_scope();
                let names = self.pattern_names(pattern);
                for name in names {
                    self.define_var(name, Ty::Inferred, 0);
                }
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition);
                let cond_ty = self.infer_type(condition);
                if !self.ty_is_inferred(&cond_ty) && !self.ty_is_bool(&cond_ty) && !self.ty_is_unit(&cond_ty) {
                    self.warn(
                        format!("Condition should be bool, got '{}'", self.ty_to_string(&cond_ty)),
                        0, 0,
                    );
                }
                self.check_block(body);
            }
            Stmt::WhileLet { pattern, value, body } => {
                self.check_expr(value);
                self.push_scope();
                let names = self.pattern_names(pattern);
                for name in names {
                    self.define_var(name, Ty::Inferred, 0);
                }
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::Loop(body) => {
                self.check_block(body);
            }
            Stmt::Break(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e);
                }
            }
            Stmt::Continue => {}
            Stmt::Match { expr, arms } => {
                self.check_expr(expr);
                for arm in arms {
                    self.push_scope();
                    let names = self.pattern_names(&arm.pattern);
                    for name in names {
                        self.define_var(name, Ty::Inferred, 0);
                    }
                    if let Some(ref guard) = arm.guard {
                        self.check_expr(guard);
                    }
                    self.check_expr(&arm.body);
                    self.pop_scope();
                }
            }
            Stmt::TryCatch { try_body, catch_param, catch_body } => {
                self.check_block(try_body);
                self.push_scope();
                self.define_var(catch_param.to_string(), Ty::Inferred, 0);
                self.check_block(catch_body);
                self.pop_scope();
            }
            Stmt::Block(block) => {
                self.push_scope();
                self.check_block(block);
                self.pop_scope();
            }
            Stmt::Unsafe(block) => {
                self.push_scope();
                self.in_unsafe_block = true;
                self.check_block(block);
                self.in_unsafe_block = false;
                self.pop_scope();
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => {
                // Set span from identifier for error reporting
                self.current_col = self.current_col.max(1);
                if !self.is_defined(name) {
                    self.error(format!("Variable '{}' is not defined", name), self.current_line, self.current_col);
                } else {
                    self.mark_used(name);
                }
            }
            Expr::Binary { op, left, right } => {
                self.check_expr(left);
                self.check_expr(right);
                // Type check binary operations
                let left_ty = self.infer_type(left);
                let right_ty = self.infer_type(right);
                if !self.ty_is_inferred(&left_ty) && !self.ty_is_inferred(&right_ty) {
                    match op {
                        // Comparison operators: operands must be the same type
                        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                            if !self.types_compatible(&left_ty, &right_ty) {
                                self.error(
                                    format!("Cannot compare '{}' with '{}'",
                                        self.ty_to_string(&left_ty), self.ty_to_string(&right_ty)),
                                    0, 0,
                                );
                            }
                        }
                        // Logical operators: operands must be bool
                        BinOp::And | BinOp::Or => {
                            if !self.ty_is_bool(&left_ty) || !self.ty_is_bool(&right_ty) {
                                self.error(
                                    format!("Logical operator requires bool operands, got '{}' and '{}'",
                                        self.ty_to_string(&left_ty), self.ty_to_string(&right_ty)),
                                    0, 0,
                                );
                            }
                        }
                        // Arithmetic operators: operands must be numeric and compatible
                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                            let left_numeric = self.ty_is_numeric(&left_ty);
                            let right_numeric = self.ty_is_numeric(&right_ty);
                            let string_concat = matches!(op, BinOp::Add)
                                && self.ty_is_string(&left_ty) && self.ty_is_string(&right_ty);
                            if string_concat {
                                // "a" + "b" is string concatenation, not arithmetic
                            } else if !left_numeric || !right_numeric {
                                self.error(
                                    format!("Arithmetic operator '{}' requires numeric operands, got '{}' and '{}'",
                                        op, self.ty_to_string(&left_ty), self.ty_to_string(&right_ty)),
                                    0, 0,
                                );
                            } else if !self.types_compatible(&left_ty, &right_ty) {
                                self.error(
                                    format!("Arithmetic on incompatible types: '{}' and '{}'",
                                        self.ty_to_string(&left_ty), self.ty_to_string(&right_ty)),
                                    0, 0,
                                );
                            }
                        }
                        // Bitwise operators: operands must be numeric
                        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                            if !self.ty_is_numeric(&left_ty) && !self.ty_is_bool(&left_ty) {
                                self.error(
                                    format!("Bitwise operator requires numeric or bool operands, got '{}'",
                                        self.ty_to_string(&left_ty)),
                                    0, 0,
                                );
                            }
                        }
                    }
                }
            }
            Expr::Unary { op, expr } => {
                self.check_expr(expr);
                if matches!(op, UnaryOp::Not) {
                    let ty = self.infer_type(expr);
                    if !self.ty_is_inferred(&ty) && !self.ty_is_bool(&ty) {
                        self.warn(format!("Logical not applied to non-bool type '{}'", self.ty_to_string(&ty)), 0, 0);
                    }
                }
            }
            Expr::Call { function, args } => {
                self.check_expr(function);
                for arg in args {
                    self.check_expr(arg);
                }
                // Check argument count and types
                if let Expr::Ident(name) = function.as_ref() {
                    if let Some(sig) = self.functions.get(name).cloned() {
                        let expected = sig.params.len();
                        let actual = args.len();
                        if expected != actual {
                            self.error(
                                format!("Function '{}' expects {} arguments, got {}", name, expected, actual),
                                0, 0,
                            );
                        } else {
                            // Type-check each argument
                            for (i, (param_name, param_ty)) in sig.params.iter().enumerate() {
                                if !self.ty_is_inferred(param_ty) {
                                    let arg_ty = self.infer_type(&args[i]);
                                    if !self.ty_is_inferred(&arg_ty) && !self.types_compatible(param_ty, &arg_ty)
                                        && !self.ty_mentions_impl(param_ty) {
                                        self.error(
                                            format!("Argument '{}' to '{}' has type '{}', expected '{}'",
                                                param_name, name, self.ty_to_string(&arg_ty), self.ty_to_string(param_ty)),
                                            0, 0,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Expr::MethodCall { object, args, method } => {
                self.check_expr(object);
                for arg in args {
                    self.check_expr(arg);
                }
                // Check method exists and argument count
                if let Some(sig) = self.method_sig(object, method) {
                    let expected = sig.params.iter().filter(|(name, _)| name != "self").count();
                    let actual = args.len();
                    if expected != actual {
                        self.error(
                            format!("Method '{}' expects {} arguments, got {}", method, expected, actual),
                            0, 0,
                        );
                    } else {
                        // Type-check arguments (skip self parameter)
                        let non_self_params: Vec<_> = sig.params.iter()
                            .filter(|(name, _)| name != "self")
                            .collect();
                        for (i, (param_name, param_ty)) in non_self_params.iter().enumerate() {
                            if !self.ty_is_inferred(param_ty) {
                                let arg_ty = self.infer_type(&args[i]);
                                if !self.ty_is_inferred(&arg_ty) && !self.types_compatible(param_ty, &arg_ty)
                                    && !self.ty_mentions_impl(param_ty) {
                                    self.error(
                                        format!("Argument '{}' to method '{}' has type '{}', expected '{}'",
                                            param_name, method, self.ty_to_string(&arg_ty), self.ty_to_string(param_ty)),
                                        0, 0,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Expr::Assign { target, value } => {
                self.check_expr(target);
                self.check_expr(value);
                if let Expr::Ident(name) = target.as_ref() {
                    if !self.is_defined(name) {
                        self.error(format!("Cannot assign to undefined variable '{}'", name), 0, 0);
                    } else {
                        // Type-check assignment
                        let target_ty = self.lookup_var_type(name);
                        let value_ty = self.infer_type(value);
                        if !self.ty_is_inferred(&target_ty) && !self.ty_is_inferred(&value_ty) {
                            if !self.types_compatible(&target_ty, &value_ty) {
                                self.error(
                                    format!("Cannot assign '{}' to variable of type '{}'",
                                        self.ty_to_string(&value_ty), self.ty_to_string(&target_ty)),
                                    0, 0,
                                );
                            }
                        }
                    }
                }
            }
            Expr::CompoundAssign { target, value, .. } => {
                self.check_expr(target);
                self.check_expr(value);
                if let Expr::Ident(name) = target.as_ref() {
                    if !self.is_defined(name) {
                        self.error(format!("Cannot assign to undefined variable '{}'", name), 0, 0);
                    }
                }
            }
            Expr::Index { object, index } => {
                self.check_expr(object);
                self.check_expr(index);
            }
            Expr::Field { object, .. } => {
                self.check_expr(object);
            }
            Expr::If { condition, then_body, else_body } => {
                self.check_expr(condition);
                self.check_block(then_body);
                if let Some(ref else_expr) = else_body {
                    self.check_expr(else_expr);
                }
            }
            Expr::Block(block) => {
                self.push_scope();
                self.check_block(block);
                self.pop_scope();
            }
            Expr::Match { expr, arms } => {
                self.check_expr(expr);
                for arm in arms {
                    self.push_scope();
                    let names = self.pattern_names(&arm.pattern);
                    for name in names {
                        self.define_var(name, Ty::Inferred, 0);
                    }
                    if let Some(ref guard) = arm.guard {
                        self.check_expr(guard);
                    }
                    self.check_expr(&arm.body);
                    self.pop_scope();
                }
            }
            Expr::Closure { params, body, .. } => {
                self.push_scope();
                for p in params {
                    let ty = p.ty.as_ref().map(|t| self.ast_type_to_ty(t)).unwrap_or(Ty::Inferred);
                    self.define_var(p.name.clone(), ty, 0);
                }
                self.check_expr(body);
                self.pop_scope();
            }
            Expr::Array(elements) => {
                for el in elements {
                    self.check_expr(el);
                }
            }
            Expr::Tuple(elements) => {
                for el in elements {
                    self.check_expr(el);
                }
            }
            Expr::ArrayRepeat { value, size } => {
                self.check_expr(value);
                self.check_expr(size);
            }
            Expr::Range { start, end, .. } => {
                self.check_expr(start);
                self.check_expr(end);
            }
            Expr::Cast { expr, .. } => {
                self.check_expr(expr);
            }
            Expr::Reference { expr, .. } => {
                self.check_expr(expr);
            }
            Expr::Deref(expr) => {
                self.check_expr(expr);
                // Check if dereferencing a raw pointer outside unsafe
                let expr_ty = self.infer_type(expr);
                if !self.in_unsafe_block && !self.in_unsafe_fn {
                    if self.ty_is_raw_pointer(&expr_ty) {
                        self.error(
                            "Dereferencing raw pointers requires an unsafe block".into(),
                            0, 0,
                        );
                    }
                }
            }
            Expr::Move(expr) => {
                self.check_expr(expr);
            }
            Expr::VecLit(elements) => {
                for el in elements {
                    self.check_expr(el);
                }
            }
            Expr::StructLiteral { name, fields } => {
                self.check_expr(name);
                for (_, val) in fields {
                    self.check_expr(val);
                }
            }
            Expr::Try(expr) => {
                self.check_expr(expr);
            }
            Expr::Macro { args, .. } => {
                for arg in args {
                    self.check_expr(arg);
                }
            }
            Expr::Int(_) | Expr::Float(_) | Expr::Str(_) | Expr::Char(_)
            | Expr::Bool(_) | Expr::Null | Expr::Self_ => {}
            Expr::Loop(body) => {
                self.check_block(body);
            }
            Expr::Path(_) => {}
            Expr::UnsafeBlock(body) => {
                self.push_scope();
                self.in_unsafe_block = true;
                self.check_block(body);
                self.in_unsafe_block = false;
                self.pop_scope();
            }
            Expr::Sizeof(_) => {
                if !self.in_unsafe_block && !self.in_unsafe_fn {
                    self.warn("sizeof is typically used in unsafe/FFI contexts".into(), 0, 0);
                }
            }
            Expr::Offsetof { .. } => {
                if !self.in_unsafe_block && !self.in_unsafe_fn {
                    self.warn("offsetof is typically used in unsafe/FFI contexts".into(), 0, 0);
                }
            }
            Expr::NullPtr => {
                if !self.in_unsafe_block && !self.in_unsafe_fn {
                    self.warn("null_mut() is typically used in unsafe/FFI contexts".into(), 0, 0);
                }
            }
            Expr::FString(parts) => {
                for part in parts {
                    if let crate::ast::FStringPart::Expr(expr) = part {
                        self.check_expr(expr);
                    }
                }
            }
            Expr::OptionalChaining { object, .. } => {
                self.check_expr(object);
            }
            Expr::NullCoalesce { left, right } => {
                self.check_expr(left);
                self.check_expr(right);
            }
            Expr::Assert { condition, message } => {
                self.check_expr(condition);
                if let Some(ref msg) = message {
                    self.check_expr(msg);
                }
            }
            Expr::AssertEq { left, right, message } => {
                self.check_expr(left);
                self.check_expr(right);
                if let Some(ref msg) = message {
                    self.check_expr(msg);
                }
            }
            Expr::AssertNe { left, right, message } => {
                self.check_expr(left);
                self.check_expr(right);
                if let Some(ref msg) = message {
                    self.check_expr(msg);
                }
            }
        }
    }

    // ─── Type Inference ────────────────────────────────────────────────

    fn infer_type(&self, expr: &Expr) -> Ty {
        match expr {
            Expr::Int(_) => Ty::Primitive("i64".into()),
            Expr::Float(_) => Ty::Primitive("f64".into()),
            Expr::Str(_) => Ty::Primitive("string".into()),
            Expr::Char(_) => Ty::Primitive("char".into()),
            Expr::Bool(_) => Ty::Primitive("bool".into()),
            Expr::Null => Ty::Named("Option".into()),
            Expr::NullPtr => Ty::RawPointer(Box::new(Ty::Inferred), true),
            Expr::FString(_) => Ty::Primitive("string".into()),
            Expr::OptionalChaining { .. } => Ty::Named("Option".into()),
            Expr::NullCoalesce { left, .. } => self.infer_type(left),
            Expr::Assert { .. } | Expr::AssertEq { .. } | Expr::AssertNe { .. } => Ty::Primitive("()".into()),

            Expr::Ident(name) => {
                // A local variable shadows any function of the same name.
                let var_ty = self.lookup_var_type(name);
                if !matches!(&var_ty, Ty::Named(n) if n == name) {
                    return var_ty;
                }
                // `name` refers to a function (registered as Ty::Named(name)):
                // a bare function name used as a value has its function-pointer type,
                // e.g. passing `double` where `fn(i32) -> i32` is expected.
                if let Some(sig) = self.functions.get(name) {
                    return Ty::FnPointer {
                        params: sig.params.iter().map(|(_, t)| t.clone()).collect(),
                        ret: Box::new(sig.return_type.clone().unwrap_or(Ty::Unit)),
                    };
                }
                var_ty
            },

            Expr::Binary { op, left, right: _ } => {
                match op {
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt
                    | BinOp::Le | BinOp::Ge | BinOp::And | BinOp::Or => Ty::Primitive("bool".into()),
                    _ => self.infer_type(left),
                }
            }

            Expr::Unary { op, expr } => {
                match op {
                    UnaryOp::Not => Ty::Primitive("bool".into()),
                    UnaryOp::Neg => self.infer_type(expr),
                    UnaryOp::Deref => {
                        if let Ty::RawPointer(inner, _) = self.infer_type(expr) {
                            *inner
                        } else if let Ty::Reference(inner, _) = self.infer_type(expr) {
                            *inner
                        } else {
                            Ty::Inferred
                        }
                    }
                }
            }

            Expr::Reference { expr, is_mut } => {
                Ty::Reference(Box::new(self.infer_type(expr)), *is_mut)
            }

            Expr::Call { function, args: _ } => {
                if let Expr::Ident(name) = function.as_ref() {
                    if let Some(sig) = self.functions.get(name) {
                        return sig.return_type.clone().unwrap_or(Ty::Unit);
                    }
                } else if let Expr::Path(parts) = function.as_ref() {
                    // Type::constructor(...) — resolve via the type's methods
                    if parts.len() >= 2 {
                        if let Some(methods) = self.class_methods.get(&parts[0]) {
                            if let Some(sig) = methods.get(parts.last().unwrap()) {
                                return sig.return_type.clone().unwrap_or(Ty::Unit);
                            }
                        }
                    }
                }
                Ty::Inferred
            }

            Expr::MethodCall { object, method, args: _ } => {
                if let Some(sig) = self.method_sig(object, method) {
                    return sig.return_type.clone().unwrap_or(Ty::Unit);
                }
                // Built-in methods
                if method == "is_some" || method == "is_none" || method == "is_ok" || method == "is_err" {
                    return Ty::Primitive("bool".into());
                }
                Ty::Inferred
            }

            Expr::Field { object, field } => {
                let obj_ty = self.infer_type(object);
                if let Expr::Ident(name) = object.as_ref() {
                    if let Some(fields) = self.struct_fields.get(name) {
                        if let Some((_, fty)) = fields.iter().find(|(n, _)| n == field) {
                            return fty.clone();
                        }
                    }
                }
                // self.field resolves through the current class's fields
                if matches!(object.as_ref(), Expr::Self_) {
                    if let Some(cls) = &self.current_class {
                        if let Some(fields) = self.struct_fields.get(cls) {
                            if let Some((_, fty)) = fields.iter().find(|(n, _)| n == field) {
                                return fty.clone();
                            }
                        }
                    }
                }
                // Check generic types (e.g., Option.is_some)
                if let Ty::Generic(name, _) = &obj_ty {
                    if name == "Option" && (field == "is_some" || field == "is_none") {
                        return Ty::Primitive("bool".into());
                    }
                }
                Ty::Inferred
            }

            Expr::Index { object, .. } => {
                let obj_ty = self.infer_type(object);
                if let Ty::Array(inner) = obj_ty {
                    *inner
                } else if let Ty::Slice(inner) = obj_ty {
                    *inner
                } else {
                    Ty::Inferred
                }
            }

            Expr::Closure { return_type, body, .. } => {
                if let Some(rt) = return_type {
                    self.ast_type_to_ty(rt)
                } else {
                    self.infer_type(body)
                }
            }

            Expr::Cast { ty, .. } => self.ast_type_to_ty(ty),

            Expr::If { then_body, .. } => {
                // Use the then_body type: trailing expression, or last statement
                if let Some(ref expr) = then_body.expr {
                    self.infer_type(expr)
                } else if let Some(Stmt::Expr(e)) = then_body.stmts.last() {
                    self.infer_type(e)
                } else {
                    Ty::Unit
                }
            }

            Expr::Block(block) => {
                if let Some(ref expr) = block.expr {
                    self.infer_type(expr)
                } else if let Some(Stmt::Expr(e)) = block.stmts.last() {
                    self.infer_type(e)
                } else {
                    Ty::Unit
                }
            }

            Expr::Path(parts) => {
                // Enum variant reference like Status::Active resolves to the parent enum type.
                // This makes `let s = Status::Active` have type `Status`, so it can be
                // passed to functions expecting `&Status`.
                if parts.len() >= 2 {
                    let parent = parts[0].as_str();
                    // Only collapse to the parent type when the first segment is a known
                    // type (enum, struct, class, or built-in generic). Module-style paths
                    // (e.g. std::io::read) keep their full name.
                    if self.is_known_type(parent) {
                        Ty::Named(parent.to_string())
                    } else {
                        Ty::Named(parts.join("::"))
                    }
                } else {
                    Ty::Named(parts.join("::"))
                }
            }

            Expr::Array(items) => {
                if let Some(first) = items.first() {
                    Ty::Array(Box::new(self.infer_type(first)))
                } else {
                    Ty::Array(Box::new(Ty::Inferred))
                }
            }

            Expr::ArrayRepeat { value, .. } => {
                Ty::Array(Box::new(self.infer_type(value)))
            }

            Expr::Tuple(items) => {
                Ty::Tuple(items.iter().map(|e| self.infer_type(e)).collect())
            }

            Expr::StructLiteral { name, .. } => {
                if let Expr::Ident(n) = name.as_ref() {
                    Ty::Named(n.clone())
                } else {
                    Ty::Inferred
                }
            }

            Expr::Macro { name, .. } => {
                match name.as_str() {
                    "println" | "eprintln" | "print" | "eprint" => Ty::Unit,
                    "vec" => Ty::Inferred,
                    "format" => Ty::Primitive("string".into()),
                    _ => Ty::Inferred,
                }
            }

            Expr::Try(inner) => {
                let inner_ty = self.infer_type(inner);
                // Try to extract the Ok type from Result<T, E>
                if let Ty::Generic(_, args) = &inner_ty {
                    if args.len() >= 1 {
                        return args[0].clone();
                    }
                }
                Ty::Inferred
            }

            Expr::Self_ => {
                // Look up Self type from current context
                Ty::Inferred
            }

            Expr::Loop(_) => Ty::Unit,
            Expr::Move(inner) => self.infer_type(inner),

            Expr::Deref(inner) => {
                let inner_ty = self.infer_type(inner);
                if let Ty::RawPointer(inner, _) = inner_ty {
                    *inner
                } else if let Ty::Reference(inner, _) = inner_ty {
                    *inner
                } else {
                    Ty::Inferred
                }
            }

            Expr::Range { .. } => Ty::Inferred,
            Expr::Assign { value, .. } => self.infer_type(value),
            Expr::CompoundAssign { .. } => Ty::Unit,
            Expr::Match { .. } => Ty::Inferred,
            Expr::UnsafeBlock(block) => {
                if let Some(ref expr) = block.expr {
                    self.infer_type(expr)
                } else {
                    Ty::Unit
                }
            }
            Expr::VecLit(items) => {
                if let Some(first) = items.first() {
                    Ty::Array(Box::new(self.infer_type(first)))
                } else {
                    Ty::Array(Box::new(Ty::Inferred))
                }
            }
            Expr::Sizeof(_) => Ty::Primitive("usize".into()),
            Expr::Offsetof { .. } => Ty::Primitive("usize".into()),
        }
    }

    // ─── Type Utility Methods ──────────────────────────────────────────

    fn ast_type_to_ty(&self, ast_ty: &Type) -> Ty {
        match ast_ty {
            Type::Name(name) => {
                // Resolve user-defined type aliases (type Meter = f64)
                if let Some(alias_ty) = self.type_aliases.get(name) {
                    return alias_ty.clone();
                }
                // Normalize common type aliases
                match name.as_str() {
                    "string" => Ty::Primitive("string".into()),
                    "bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                    | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                    | "f32" | "f64" | "char" => Ty::Primitive(name.clone()),
                    _ => Ty::Named(name.clone()),
                }
            }
            Type::Path(parts) => Ty::Named(parts.join("::")),
            Type::Reference { inner, is_mut } => {
                Ty::Reference(Box::new(self.ast_type_to_ty(inner)), *is_mut)
            }
            Type::RawPointer { inner, is_mut } => {
                Ty::RawPointer(Box::new(self.ast_type_to_ty(inner)), *is_mut)
            }
            Type::Array { inner, .. } => Ty::Array(Box::new(self.ast_type_to_ty(inner))),
            Type::Slice(inner) => Ty::Slice(Box::new(self.ast_type_to_ty(inner))),
            Type::Tuple(types) => {
                Ty::Tuple(types.iter().map(|t| self.ast_type_to_ty(t)).collect())
            }
            Type::Generic { name, args } => {
                Ty::Generic(name.clone(), args.iter().map(|a| self.ast_type_to_ty(a)).collect())
            }
            Type::Function { params, return_type } => {
                Ty::FnPointer {
                    params: params.iter().map(|p| self.ast_type_to_ty(p)).collect(),
                    ret: Box::new(self.ast_type_to_ty(return_type)),
                }
            }
            Type::Unit => Ty::Unit,
            Type::Never => Ty::Never,
            Type::SelfType => Ty::Inferred, // Resolved at check time
        }
    }

    fn ty_to_string(&self, ty: &Ty) -> String {
        match ty {
            Ty::Var(v) => format!("?{}", v),
            Ty::Primitive(s) | Ty::Named(s) => s.clone(),
            Ty::Generic(name, args) => {
                let arg_strs: Vec<String> = args.iter().map(|a| self.ty_to_string(a)).collect();
                format!("{}<{}>", name, arg_strs.join(", "))
            }
            Ty::Reference(inner, is_mut) => {
                let prefix = if *is_mut { "&mut " } else { "&" };
                format!("{}{}", prefix, self.ty_to_string(inner))
            }
            Ty::RawPointer(inner, is_mut) => {
                let prefix = if *is_mut { "*mut " } else { "*const " };
                format!("{}{}", prefix, self.ty_to_string(inner))
            }
            Ty::Array(inner) => format!("[{}]", self.ty_to_string(inner)),
            Ty::Slice(inner) => format!("[{}]", self.ty_to_string(inner)),
            Ty::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(|t| self.ty_to_string(t)).collect();
                format!("({})", inner.join(", "))
            }
            Ty::FnPointer { params, ret } => {
                let ps: Vec<String> = params.iter().map(|p| self.ty_to_string(p)).collect();
                format!("fn({}) -> {}", ps.join(", "), self.ty_to_string(ret))
            }
            Ty::Unit => "()".into(),
            Ty::Never => "!".into(),
            Ty::Inferred => "_".into(),
        }
    }

    fn types_compatible(&self, a: &Ty, b: &Ty) -> bool {
        // Normalize type vars
        let a = self.resolve(a);
        let b = self.resolve(b);

        // Normalize string/String (literals infer as Primitive("string"); signatures use
        // either lowercase "string" or capitalized "String", which parses as Named)
        let a = if matches!(&a, Ty::Primitive(s) if s == "string") { Ty::Primitive("String".into()) } else { a };
        let b = if matches!(&b, Ty::Primitive(s) if s == "string") { Ty::Primitive("String".into()) } else { b };
        let a = if matches!(&a, Ty::Named(s) if s == "String" || s == "string" || s == "str") { Ty::Primitive("String".into()) } else { a };
        let b = if matches!(&b, Ty::Named(s) if s == "String" || s == "string" || s == "str") { Ty::Primitive("String".into()) } else { b };

        if a == b { return true; }

        // Inferred matches anything
        if matches!(&a, Ty::Inferred) || matches!(&b, Ty::Inferred) { return true; }
        if matches!(&a, Ty::Var(_)) || matches!(&b, Ty::Var(_)) { return true; }

        // Option variants (None, Some, Ok, Err) are compatible with their Option/Result types
        let a_is_option = matches!(&a, Ty::Generic(s, _) if s == "Option")
            || matches!(&a, Ty::Named(s) if s == "Option" || s == "None" || s == "Some" || s.starts_with("Option::"));
        let b_is_option = matches!(&b, Ty::Generic(s, _) if s == "Option")
            || matches!(&b, Ty::Named(s) if s == "Option" || s == "None" || s == "Some" || s.starts_with("Option::"));
        if a_is_option && b_is_option { return true; }
        let a_is_result = matches!(&a, Ty::Generic(s, _) if s == "Result")
            || matches!(&a, Ty::Named(s) if s == "Result" || s == "Ok" || s == "Err" || s.starts_with("Result::"));
        let b_is_result = matches!(&b, Ty::Generic(s, _) if s == "Result")
            || matches!(&b, Ty::Named(s) if s == "Result" || s == "Ok" || s == "Err" || s.starts_with("Result::"));
        if a_is_result && b_is_result { return true; }

        // Single-letter generic type variables (T, K, V, E, U, N) are unconstrained: match anything.
        // Note: generic params in signatures parse as plain Names (e.g. Named("T")).
        if let Ty::Generic(name, args) = &a {
            if args.is_empty() && name.len() == 1 && name.chars().next().unwrap().is_uppercase() { return true; }
        }
        if let Ty::Generic(name, args) = &b {
            if args.is_empty() && name.len() == 1 && name.chars().next().unwrap().is_uppercase() { return true; }
        }
        if let Ty::Named(name) = &a {
            if name.len() == 1 && name.chars().next().unwrap().is_uppercase() { return true; }
        }
        if let Ty::Named(name) = &b {
            if name.len() == 1 && name.chars().next().unwrap().is_uppercase() { return true; }
        }
        // A bare type name matches its generic instantiation: Vec vs Vec<T>, Pair vs Pair<A, B>
        if let (Ty::Named(n), Ty::Generic(g, _)) = (&a, &b) { if n == g { return true; } }
        if let (Ty::Generic(g, _), Ty::Named(n)) = (&a, &b) { if n == g { return true; } }

        match (&a, &b) {
            // Allow numeric coercion between primitive numeric types
            (Ty::Primitive(a), Ty::Primitive(b)) => {
                let numeric = ["i8", "i16", "i32", "i64", "i128", "isize",
                               "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64"];
                let a_is_num = numeric.contains(&a.as_str());
                let b_is_num = numeric.contains(&b.as_str());
                a_is_num && b_is_num
            }
            // &T coerces to *const T / *mut T (FFI) and vice versa
            (Ty::RawPointer(a_inner, _), Ty::Reference(b_inner, _))
            | (Ty::Reference(a_inner, _), Ty::RawPointer(b_inner, _)) => {
                self.types_compatible(a_inner, b_inner)
            }
            // Auto-ref / auto-deref: &T accepts T, &str accepts string, etc.
            (Ty::Reference(a_inner, _), b) => self.types_compatible(a_inner, b),
            (a, Ty::Reference(b_inner, _)) => self.types_compatible(a, b_inner),
            (Ty::RawPointer(a_inner, a_mut), Ty::RawPointer(b_inner, b_mut)) => {
                a_mut == b_mut && self.types_compatible(a_inner, b_inner)
            }
            (Ty::Generic(a_name, a_args), Ty::Generic(b_name, b_args)) => {
                a_name == b_name && a_args.len() == b_args.len()
                    && a_args.iter().zip(b_args.iter()).all(|(a, b)| self.types_compatible(a, b))
            }
            (Ty::Tuple(a_types), Ty::Tuple(b_types)) => {
                a_types.len() == b_types.len()
                    && a_types.iter().zip(b_types.iter()).all(|(a, b)| self.types_compatible(a, b))
            }
            (Ty::Array(a_inner), Ty::Array(b_inner)) => self.types_compatible(a_inner, b_inner),
            (Ty::Slice(a_inner), Ty::Slice(b_inner)) => self.types_compatible(a_inner, b_inner),
            (Ty::Array(a_inner), Ty::Slice(b_inner)) | (Ty::Slice(a_inner), Ty::Array(b_inner)) => {
                self.types_compatible(a_inner, b_inner)
            }
            // Vec<T> derefs/coerces to [T] (and vice versa)
            (Ty::Slice(inner), Ty::Generic(name, args)) if name == "Vec" && args.len() == 1 => {
                self.types_compatible(inner, &args[0])
            }
            (Ty::Generic(name, args), Ty::Slice(inner)) if name == "Vec" && args.len() == 1 => {
                self.types_compatible(&args[0], inner)
            }
            (Ty::FnPointer { params: a_p, ret: a_r }, Ty::FnPointer { params: b_p, ret: b_r }) => {
                a_p.len() == b_p.len()
                    && a_p.iter().zip(b_p.iter()).all(|(a, b)| self.types_compatible(a, b))
                    && self.types_compatible(a_r, b_r)
            }
            _ => false,
        }
    }

    fn resolve(&self, ty: &Ty) -> Ty {
        // For now, just return the type as-is (no type variable unification yet)
        ty.clone()
    }

    fn ty_is_inferred(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Inferred | Ty::Var(_))
    }

    fn ty_is_bool(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Primitive(s) if s == "bool")
    }

    fn ty_is_unit(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Unit)
    }

    fn ty_is_numeric(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Primitive(s) if matches!(s.as_str(),
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
            | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            | "f32" | "f64"
        ))
    }

    fn ty_is_string(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Primitive(s) if s == "string" || s == "String")
            || matches!(ty, Ty::Named(s) if s == "String" || s == "string")
    }

    /// Resolve a method's signature by the receiver's type first (so same-named
    /// methods on different classes don't collide), falling back to the global
    /// function table (builtins, free functions used as methods).
    fn method_sig(&self, object: &Expr, method: &str) -> Option<FunctionSig> {
        let receiver_ty = self.infer_type(object);
        let class = match &receiver_ty {
            Ty::Named(n) => Some(n.clone()),
            Ty::Reference(inner, _) => match inner.as_ref() {
                Ty::Named(n) => Some(n.clone()),
                _ => None,
            },
            Ty::Generic(n, _) => Some(n.clone()),
            _ => None,
        };
        if let Some(cls) = class {
            if let Some(methods) = self.class_methods.get(&cls) {
                if let Some(sig) = methods.get(method) {
                    return Some(sig.clone());
                }
            }
        }
        self.functions.get(method).cloned()
    }

    /// Extract the base type name from a type AST node (impl target etc.)
    fn type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Name(n) => Some(n.clone()),
            Type::Path(p) => p.last().cloned(),
            Type::Generic { name, .. } => Some(name.clone()),
            Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => self.type_name(inner),
            _ => None,
        }
    }

    /// Does this type mention an `impl Trait` anywhere (e.g. `&impl Area`, `&[impl Drawable]`)?
    /// Such parameters accept any concrete type, since we don't track trait impls.
    fn ty_mentions_impl(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Named(s) => s.starts_with("impl "),
            Ty::Reference(inner, _) | Ty::RawPointer(inner, _) => self.ty_mentions_impl(inner),
            Ty::Array(inner) | Ty::Slice(inner) => self.ty_mentions_impl(inner),
            Ty::Generic(_, args) => args.iter().any(|t| self.ty_mentions_impl(t)),
            Ty::Tuple(types) => types.iter().any(|t| self.ty_mentions_impl(t)),
            _ => false,
        }
    }

    fn ty_is_raw_pointer(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::RawPointer(_, _))
    }

    // ─── Scope Management ──────────────────────────────────────────────

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            variables: std::collections::HashMap::new(),
            used: std::collections::HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, name: String, ty: Ty, _line: usize) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.used.insert(name.clone(), false);
            scope.variables.insert(name, ty);
        }
    }

    fn is_defined(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.variables.contains_key(name) {
                return true;
            }
        }
        false
    }

    fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().map_or(false, |s| s.variables.contains_key(name))
    }

    fn mark_used(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(used) = scope.used.get_mut(name) {
                *used = true;
                return;
            }
        }
    }

    fn lookup_var_type(&self, name: &str) -> Ty {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.variables.get(name) {
                return ty.clone();
            }
        }
        Ty::Inferred
    }

    fn verify_type(&self, ty: &Type, line: usize) {
        // Verify a type reference is valid
        match ty {
            Type::Name(name) => {
                if !self.is_known_type(name) {
                    // Don't error on type names we don't know — they might be from external crates
                }
            }
            Type::Generic { args, .. } => {
                for arg in args {
                    self.verify_type(arg, line);
                }
            }
            Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => {
                self.verify_type(inner, line);
            }
            Type::Array { inner, .. } | Type::Slice(inner) => {
                self.verify_type(inner, line);
            }
            Type::Tuple(types) => {
                for t in types {
                    self.verify_type(t, line);
                }
            }
            _ => {}
        }
    }

    fn is_known_type(&self, name: &str) -> bool {
        // Check built-in types
        let normalized = if name == "string" { "String" } else { name };
        matches!(normalized,
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
            | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            | "f32" | "f64" | "bool" | "char" | "String"
            | "Self" | "Option" | "Result" | "Vec" | "HashMap"
            | "str"
        ) || self.scopes.iter().any(|s| s.variables.contains_key(name))
    }

    // ─── Unsafe Checking Helpers ───────────────────────────────────────

    fn is_unsafe_required_fn(&self, name: &str) -> bool {
        matches!(name,
            "transmute" | "size_of" | "align_of" | "offset_of"
            | "read_volatile" | "write_volatile"
            | "copy" | "copy_nonoverlapping"
            | "ptr::read" | "ptr::write"
        ) || name.starts_with("asm") || name.starts_with("llvm")
    }

    fn is_unsafe_type(&self, ty: &Ty) -> bool {
        self.ty_is_raw_pointer(ty)
    }

    fn report_unused(&mut self) {
        let unused: Vec<String> = if let Some(scope) = self.scopes.first() {
            scope.used.iter()
                .filter(|(name, used)| !**used && name.as_str() != "main" && !name.starts_with('_'))
                .map(|(name, _)| name.clone())
                .collect()
        } else {
            vec![]
        };
        for name in unused {
            self.warn(format!("Variable '{}' is never used", name), 0, 0);
        }
    }

    fn error(&mut self, message: String, line: usize, col: usize) {
        let (line, col) = if line == 0 && col == 0 {
            (self.current_line, self.current_col)
        } else {
            (line, col)
        };
        self.diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Error,
            message,
            line,
            col,
        });
        self.error_count += 1;
    }

    fn warn(&mut self, message: String, line: usize, col: usize) {
        let (line, col) = if line == 0 && col == 0 {
            (self.current_line, self.current_col)
        } else {
            (line, col)
        };
        self.diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Warning,
            message,
            line,
            col,
        });
        self.warning_count += 1;
    }

    fn pattern_names(&self, pattern: &Pattern) -> Vec<String> {
        match pattern {
            Pattern::Ident(name) => vec![name.clone()],
            Pattern::Tuple(patterns) => {
                let mut names = vec![];
                for p in patterns {
                    names.extend(self.pattern_names(p));
                }
                names
            }
            Pattern::Enum { fields, .. } => {
                let mut names = vec![];
                for f in fields {
                    names.extend(self.pattern_names(f));
                }
                names
            }
            Pattern::Struct { fields, .. } => {
                let mut names = vec![];
                for (_, p) in fields {
                    names.extend(self.pattern_names(p));
                }
                names
            }
            Pattern::Mut(name) => vec![name.clone()],
            Pattern::Reference(inner) => self.pattern_names(inner),
            Pattern::Or(patterns) => {
                let mut names = vec![];
                for p in patterns {
                    names.extend(self.pattern_names(p));
                }
                names
            }
            _ => vec![],
        }
    }

    // ─── Legacy compat: infer_type used as String in some places ───────
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn check_source(source: &str) -> Vec<Diagnostic> {
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();
        let mut checker = TypeChecker::new();
        checker.check(&program)
    }

    fn has_error(diagnostics: &[Diagnostic], msg: &str) -> bool {
        diagnostics.iter().any(|d| d.kind == DiagnosticKind::Error && d.message.contains(msg))
    }

    fn has_warning(diagnostics: &[Diagnostic], msg: &str) -> bool {
        diagnostics.iter().any(|d| d.kind == DiagnosticKind::Warning && d.message.contains(msg))
    }

    #[test]
    fn test_undefined_variable() {
        let errors = check_source(r#"
            fn main() {
                let x = y
            }
        "#);
        assert!(has_error(&errors, "'y' is not defined"));
    }

    #[test]
    fn test_duplicate_binding_warning() {
        let diagnostics = check_source(r#"
            fn main() {
                let x = 1
                let x = 2
            }
        "#);
        assert!(has_warning(&diagnostics, "shadows"));
    }

    #[test]
    fn test_valid_code_no_errors() {
        let diagnostics = check_source(r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b
            }
            fn main() {
                let result = add(1, 2)
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_argument_count_mismatch() {
        let diagnostics = check_source(r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b
            }
            fn main() {
                let x = add(1)
            }
        "#);
        assert!(has_error(&diagnostics, "expects 2 arguments, got 1"));
    }

    #[test]
    fn test_argument_count_correct() {
        let diagnostics = check_source(r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b
            }
            fn main() {
                let x = add(1, 2)
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_class() {
        let diagnostics = check_source(r#"
            class Dog {
                pub let name: string
                pub fn new(name: string) -> Self {
                    return Self { name }
                }
                pub fn bark(&self) {
                    println!("{}", self.name)
                }
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_assign_to_undefined() {
        let diagnostics = check_source(r#"
            fn main() {
                x = 5
            }
        "#);
        assert!(has_error(&diagnostics, "Cannot assign to undefined variable 'x'"));
    }

    #[test]
    fn test_class_no_false_positives() {
        let diagnostics = check_source(r#"
            class Calculator {
                pub fn add(&self, a: i32, b: i32) -> i32 {
                    return a + b
                }
            }
            fn main() {
                let calc = Calculator {}
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_return_type_inference() {
        let diagnostics = check_source(r#"
            fn is_positive(x: i32) -> bool {
                if x > 0 {
                    return true
                } else {
                    return false
                }
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_type_inference_literals() {
        let checker = TypeChecker::new();
        assert_eq!(checker.infer_type(&Expr::Int(42)), Ty::Primitive("i64".into()));
        assert_eq!(checker.infer_type(&Expr::Float(3.14)), Ty::Primitive("f64".into()));
        assert_eq!(checker.infer_type(&Expr::Str("hello".into())), Ty::Primitive("string".into()));
        assert_eq!(checker.infer_type(&Expr::Bool(true)), Ty::Primitive("bool".into()));
    }

    #[test]
    fn test_return_type_mismatch() {
        let diagnostics = check_source(r#"
            fn get_number() -> i32 {
                return "hello"
            }
        "#);
        assert!(has_error(&diagnostics, "Return type mismatch"));
    }

    #[test]
    fn test_assignment_type_mismatch() {
        let diagnostics = check_source(r#"
            fn main() {
                let x: i32 = "hello"
            }
        "#);
        assert!(has_error(&diagnostics, "Type mismatch"));
    }

    #[test]
    fn test_unsafe_deref_error() {
        let diagnostics = check_source(r#"
            fn main() {
                let p: *mut i32 = null_mut()
                let x = *p
            }
        "#);
        assert!(has_error(&diagnostics, "unsafe block"));
    }

    #[test]
    fn test_unsafe_deref_ok() {
        let diagnostics = check_source(r#"
            fn main() {
                let p: *mut i32 = null_mut()
                unsafe {
                    let x = *p
                }
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_unsafe_fn_propagates() {
        let diagnostics = check_source(r#"
            unsafe fn get_raw() -> *mut i32 {
                let p: *mut i32 = null_mut()
                let x = *p
                return p
            }
        "#);
        // Inside an unsafe fn, dereferencing is allowed
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_condition_type_warning() {
        let diagnostics = check_source(r#"
            fn main() {
                let x = 42
                if x {
                    println!("yes")
                }
            }
        "#);
        assert!(has_warning(&diagnostics, "Condition should be bool"));
    }

    fn errors_only(diagnostics: &[Diagnostic]) -> Vec<String> {
        diagnostics.iter()
            .filter(|d| d.kind == DiagnosticKind::Error)
            .map(|d| d.message.clone())
            .collect()
    }

    #[test]
    fn test_enum_match_bindings() {
        let diagnostics = check_source(r#"
            enum Shape {
                Circle(f64),
                Rect(f64, f64),
            }
            fn area(s: &Shape) -> f64 {
                match s {
                    Shape::Circle(r) => 3.14 * r * r,
                    Shape::Rect(w, h) => w * h,
                }
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_enum_variant_path_type() {
        // `let s = Status::Active` should infer the parent enum type
        let diagnostics = check_source(r#"
            enum Status { Active, Inactive }
            fn is_active(s: &Status) -> bool {
                return true
            }
            fn main() {
                let s = Status::Active
                is_active(&s)
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_or_pattern_in_match() {
        let diagnostics = check_source(r#"
            enum Day { Mon, Tue, Wed }
            fn is_weekend(d: Day) -> bool {
                match d {
                    Day::Mon | Day::Tue => false,
                    Day::Wed => true,
                }
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_string_string_compat() {
        // `String` in a signature must accept string literals (Primitive("string"))
        let diagnostics = check_source(r#"
            fn greet(name: String) -> String {
                return name
            }
            fn main() {
                greet("Alice")
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_none_return_compat() {
        let diagnostics = check_source(r#"
            fn find(x: i32) -> Option<i32> {
                if x > 0 {
                    return Option::Some(x)
                }
                return Option::None
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_methods_do_not_collide() {
        // Same-named methods on different classes must resolve by receiver type
        let diagnostics = check_source(r#"
            class Dog {
                pub fn speak(&self) -> string {
                    return "woof"
                }
            }
            class Cat {
                pub fn speak(&self, loud: bool) -> string {
                    return "meow"
                }
            }
            fn main() {
                let d = Dog {}
                let c = Cat {}
                d.speak()
                c.speak(true)
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_array_repeat_literal() {
        let diagnostics = check_source(r#"
            fn main() {
                let items = [0; 100]
                let x = items[0]
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_tuple_field_access() {
        let diagnostics = check_source(r#"
            fn main() {
                let point = (3.0, 4.0)
                let x = point.0
                let y = point.1
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_const_item() {
        let diagnostics = check_source(r#"
            const MAX_SIZE: i32 = 1024
            fn main() {
                println!("{}", MAX_SIZE)
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_type_alias_resolution() {
        let diagnostics = check_source(r#"
            type Meter = f64
            fn main() {
                let distance: Meter = 100.0
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_fn_value_passing() {
        // A bare function name passed where a fn pointer is expected
        let diagnostics = check_source(r#"
            fn double(x: i32) -> i32 {
                return x * 2
            }
            fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
                return f(x)
            }
            fn main() {
                apply(double, 5)
            }
        "#);
        let errs = errors_only(&diagnostics);
        assert!(errs.is_empty(), "Unexpected errors: {:?}", errs);
    }
}
