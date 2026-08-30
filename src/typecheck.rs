use crate::ast::*;




const NUMERIC_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64",
];


const PRIMITIVE_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64", "bool", "char",
];



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



struct Scope {
    bindings: std::collections::HashMap<String, (Ty, bool, bool)>,
}

pub struct TypeChecker {
    scopes: Vec<Scope>,
    functions: std::collections::HashMap<String, FunctionSig>,
    struct_fields: std::collections::HashMap<String, Vec<(String, Ty)>>,
    diagnostics: Vec<Diagnostic>,
    current_return_type: Option<Ty>,
    in_unsafe_block: bool,
    in_unsafe_fn: bool,

    current_line: usize,
    current_col: usize,
    pub warning_count: usize,
    pub error_count: usize,

    /// Maps type variable IDs to their resolved types.
    type_var_values: std::collections::HashMap<usize, Option<Ty>>,
    /// Counter for generating fresh type variable IDs.
    next_type_var: usize,
}

#[derive(Clone)]
struct FunctionSig {
    params: Vec<(String, Ty)>,
    return_type: Option<Ty>,
    /// Generic parameter names and their corresponding type variable IDs.
    /// E.g. for `fn id<T>(x: T) -> T`, this would be [("T", 0)].
    generic_params: Vec<(String, usize)>,
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
                bindings: std::collections::HashMap::new(),
            }],
            functions: std::collections::HashMap::new(),
            struct_fields: std::collections::HashMap::new(),
            diagnostics: vec![],
            current_return_type: None,
            in_unsafe_block: false,
            in_unsafe_fn: false,
            current_line: 0,
            current_col: 0,
            warning_count: 0,
            error_count: 0,
            type_var_values: std::collections::HashMap::new(),
            next_type_var: 0,
        };


        for b in &["Some", "None", "Ok", "Err", "Self"] {
            checker.define_var(b.to_string(), Ty::Named(b.to_string()), false, 0);
        }
        checker
    }


    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {

        for item in &program.items {
            self.register_item(item);
        }


        for item in &program.items {
            self.check_item(item);
        }


        self.report_unused();

        std::mem::take(&mut self.diagnostics)
    }



    fn register_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                // Create type variables for generic parameters
                let mut generic_params = Vec::new();
                for gp in &f.generics {
                    let var_id = self.next_type_var;
                    self.next_type_var += 1;
                    self.type_var_values.insert(var_id, None);
                    generic_params.push((gp.name.clone(), var_id));
                }

                // Build a mapping from generic param names to their type variables
                let generic_map: std::collections::HashMap<String, Ty> = generic_params
                    .iter()
                    .map(|(name, id)| (name.clone(), Ty::Var(*id)))
                    .collect();

                let params: Vec<(String, Ty)> = f.params.iter().map(|p| {
                    let ty = self.ast_type_to_ty_with_generics(&p.ty, &generic_map);
                    (p.name.clone(), ty)
                }).collect();
                let ret_type = f.return_type.as_ref().map(|t| self.ast_type_to_ty_with_generics(t, &generic_map));
                self.functions.insert(f.name.clone(), FunctionSig { params, return_type: ret_type, generic_params });
                self.define_var(f.name.clone(), Ty::Named(f.name.clone()), false, f.span.line);
            }
            Item::Class(c) => {
                let fields: Vec<(String, Ty)> = c.fields.iter().map(|f| {
                    (f.name.clone(), self.ast_type_to_ty(&f.ty))
                }).collect();
                self.struct_fields.insert(c.name.clone(), fields);
                self.define_var(c.name.clone(), Ty::Named(c.name.clone()), false, c.span.line);

                for m in &c.methods {
                    let params: Vec<(String, Ty)> = m.params.iter().map(|p| {
                        (p.name.clone(), self.ast_type_to_ty(&p.ty))
                    }).collect();
                    let ret = m.return_type.as_ref().map(|t| self.ast_type_to_ty(t));                    self.functions.insert(m.name.clone(), FunctionSig { params, return_type: ret, generic_params: vec![] });
                }
            }

            Item::Struct(s) => {
                let fields: Vec<(String, Ty)> = s.fields.iter().map(|f| {
                    (f.name.clone(), self.ast_type_to_ty(&f.ty))
                }).collect();
                self.struct_fields.insert(s.name.clone(), fields);
                self.define_var(s.name.clone(), Ty::Named(s.name.clone()), false, s.span.line);
            }
            Item::Enum(e) => {
                self.define_var(e.name.clone(), Ty::Named(e.name.clone()), false, e.span.line);

                for v in &e.variants {
                    let ret = Ty::Generic(e.name.clone(), vec![]);
                    self.functions.insert(v.name.clone(), FunctionSig {
                        params: v.fields.iter().enumerate().map(|(i, t)| {
                            (format!("__{}_{}", v.name, i), self.ast_type_to_ty(t))
                        }).collect(),
                        return_type: Some(ret),
                        generic_params: vec![],
                    });
                }
            }
            Item::Impl(imp) => {
                for m in &imp.methods {
                    let params: Vec<(String, Ty)> = m.params.iter().map(|p| {
                        (p.name.clone(), self.ast_type_to_ty(&p.ty))
                    }).collect();
                    let ret = m.return_type.as_ref().map(|t| self.ast_type_to_ty(t));
                    self.functions.insert(m.name.clone(), FunctionSig { params, return_type: ret, generic_params: vec![] });
                }
            }
            Item::Trait(t) => {
                for m in &t.methods {
                    let params: Vec<(String, Ty)> = m.params.iter().map(|p| {
                        (p.name.clone(), self.ast_type_to_ty(&p.ty))
                    }).collect();
                    let ret = m.return_type.as_ref().map(|t| self.ast_type_to_ty(t));
                    self.functions.insert(m.name.clone(), FunctionSig { params, return_type: ret, generic_params: vec![] });
                }
            }
            Item::Use(u) => {
                if u.wildcard {
                    if let Some(last) = u.path.last() {
                        self.define_var(last.to_string(), Ty::Named(last.to_string()), false, 0);
                    }
                } else if !u.selective.is_empty() {
                    for item in &u.selective {
                        let name = item.alias.as_ref().unwrap_or(&item.name);
                        self.define_var(name.to_string(), Ty::Inferred, false, 0);
                    }
                } else if let Some(ref alias) = u.alias {
                    self.define_var(alias.to_string(), Ty::Inferred, false, 0);
                } else if let Some(last) = u.path.last() {
                    self.define_var(last.to_string(), Ty::Named(last.to_string()), false, 0);
                }
            }
            Item::TypeAlias(ta) => {
                self.define_var(ta.name.clone(), self.ast_type_to_ty(&ta.ty), false, 0);
            }
            Item::Module(m) => {
                self.define_var(m.name.clone(), Ty::Named(m.name.clone()), false, 0);
                if let Some(ref body) = m.body {
                    for inner in body {
                        self.register_item(inner);
                    }
                }
            }
            Item::ExternBlock(eb) => {

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

                            let dangerous = ["exec", "system", "popen", "eval", "dlopen",
                                "LoadLibrary", "CreateProcess", "ShellExecute"];
                            if dangerous.iter().any(|d| name.eq_ignore_ascii_case(d)) {
                                self.warn(
                                    format!("FFI function '{}' is potentially dangerous — ensure input is validated",
                                        name),
                                    0, 0,
                                );
                            }


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
                            self.functions.insert(name.clone(), FunctionSig { params: ps, return_type: ret, generic_params: vec![] });
                        }
                        ExternItem::Static { name, ty, is_mut, .. } => {

                            if *is_mut {
                                self.warn(
                                    format!("Mutable static '{}' in FFI — requires unsafe access and careful synchronization",
                                        name),
                                    0, 0,
                                );
                            }
                            self.define_var(name.clone(), self.ast_type_to_ty(ty), false, 0);
                        }
                        ExternItem::Const { name, ty, .. } => {
                            self.define_var(name.clone(), self.ast_type_to_ty(ty), false, 0);
                        }
                    }
                }
            }
            _ => {}
        }
    }



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
                for method in &c.methods {
                    self.check_function(method);
                }
            }
            Item::Impl(imp) => {
                self.current_line = imp.span.line;
                self.current_col = imp.span.col;
                for method in &imp.methods {
                    self.check_function(method);
                }
            }
            Item::Trait(t) => {
                for method in &t.methods {
                    if let Some(ref body) = method.default_body {
                        self.push_scope();
                        for p in &method.params {
                            if !matches!(p.ty, Type::SelfType) {
                                let ty = self.ast_type_to_ty(&p.ty);
                                self.define_var(p.name.clone(), ty, false, 0);
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


                let _ = eb;
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
                self.define_var(p.name.clone(), ty, false, f.span.line);
            }
        }

        self.check_block(&f.body);


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
            Stmt::Let { pattern, ty, value, is_mut } => {
                let val_ty = self.infer_type(value);
                self.check_expr(value);

                let names = self.pattern_names(pattern);
                for name in names {

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
                    self.define_var(name, var_ty, *is_mut, 0);
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
                        if !self.ty_is_inferred(&actual) && !self.types_compatible(expected, &actual) {
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
                let is_mut = matches!(pattern, Pattern::Mut(_));
                let names = self.pattern_names(pattern);
                for name in names {
                    self.define_var(name, Ty::Inferred, is_mut, 0);
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
                let is_mut = matches!(pattern, Pattern::Mut(_));
                let names = self.pattern_names(pattern);
                for name in names {
                    self.define_var(name, Ty::Inferred, is_mut, 0);
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
                        self.define_var(name, Ty::Inferred, false, 0);
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
                self.define_var(catch_param.to_string(), Ty::Inferred, false, 0);
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

                let left_ty = self.infer_type(left);
                let right_ty = self.infer_type(right);
                if !self.ty_is_inferred(&left_ty) && !self.ty_is_inferred(&right_ty) {
                    match op {

                        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                            if !self.types_compatible(&left_ty, &right_ty) {
                                self.error(
                                    format!("Cannot compare '{}' with '{}'",
                                        self.ty_to_string(&left_ty), self.ty_to_string(&right_ty)),
                                    0, 0,
                                );
                            }
                        }

                        BinOp::And | BinOp::Or => {
                            if !self.ty_is_bool(&left_ty) || !self.ty_is_bool(&right_ty) {
                                self.error(
                                    format!("Logical operator requires bool operands, got '{}' and '{}'",
                                        self.ty_to_string(&left_ty), self.ty_to_string(&right_ty)),
                                    0, 0,
                                );
                            }
                        }

                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                            let left_numeric = self.ty_is_numeric(&left_ty);
                            let right_numeric = self.ty_is_numeric(&right_ty);
                            if !left_numeric || !right_numeric {
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
                            // For generic functions, instantiate fresh type variables
                            // so each call site gets its own type variable bindings
                            let (fresh_sig, _fresh_vars) = if !sig.generic_params.is_empty() {
                                let mut fresh_map = std::collections::HashMap::new();
                                let mut fresh_vars = Vec::new();
                                for (gp_name, _old_id) in &sig.generic_params {
                                    let new_id = self.next_type_var;
                                    self.next_type_var += 1;
                                    self.type_var_values.insert(new_id, None);
                                    fresh_map.insert(gp_name.clone(), Ty::Var(new_id));
                                    fresh_vars.push(new_id);
                                }
                                // Substitute fresh type variables into the signature
                                let fresh_params: Vec<(String, Ty)> = sig.params.iter().map(|(n, t)| {
                                    (n.clone(), self.substitute_type_vars(t, &fresh_map))
                                }).collect();
                                let fresh_ret = sig.return_type.as_ref().map(|t| self.substitute_type_vars(t, &fresh_map));
                                (FunctionSig { params: fresh_params, return_type: fresh_ret, generic_params: sig.generic_params.clone() }, fresh_vars)
                            } else {
                                (sig.clone(), Vec::new())
                            };

                            for (i, (param_name, param_ty)) in fresh_sig.params.iter().enumerate() {
                                if !self.ty_is_inferred(param_ty) {
                                    let arg_ty = self.infer_type(&args[i]);
                                    if !self.ty_is_inferred(&arg_ty) && !self.types_compatible(param_ty, &arg_ty) {
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

                if let Some(sig) = self.functions.get(method).cloned() {
                    let expected = sig.params.iter().filter(|(name, _)| name != "self").count();
                    let actual = args.len();
                    if expected != actual {
                        self.error(
                            format!("Method '{}' expects {} arguments, got {}", method, expected, actual),
                            0, 0,
                        );
                    } else {

                        let non_self_params: Vec<_> = sig.params.iter()
                            .filter(|(name, _)| name != "self")
                            .collect();
                        for (i, (param_name, param_ty)) in non_self_params.iter().enumerate() {
                            if !self.ty_is_inferred(param_ty) {
                                let arg_ty = self.infer_type(&args[i]);
                                if !self.ty_is_inferred(&arg_ty) && !self.types_compatible(param_ty, &arg_ty) {
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
                    } else if !self.is_mutable(name) {
                        self.error(format!("Cannot assign to immutable variable '{}'", name), 0, 0);
                    } else {

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
                    } else if !self.is_mutable(name) {
                        self.error(format!("Cannot assign to immutable variable '{}'", name), 0, 0);
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
                        self.define_var(name, Ty::Inferred, false, 0);
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
                    self.define_var(p.name.clone(), ty, false, 0);
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
            Expr::TryCatch(tc) => {
                for stmt in &tc.try_body.stmts {
                    self.check_stmt(stmt);
                }
                for clause in &tc.catch_clauses {
                    for stmt in &clause.body.stmts {
                        self.check_stmt(stmt);
                    }
                }
            }
            Expr::Throw(th) => {
                self.check_expr(&th.value);
            }
            Expr::Comptime(ct) => {
                for stmt in &ct.body.stmts {
                    self.check_stmt(stmt);
                }
            }
            Expr::ListComp(lc) => {
                self.check_expr(&lc.iterable);
                self.check_expr(&lc.element);
                if let Some(ref cond) = lc.condition {
                    self.check_expr(cond);
                }
            }
        }
    }



    fn infer_type(&mut self, expr: &Expr) -> Ty {
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

            Expr::Ident(name) => self.lookup_var_type(name),

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

            Expr::Call { function, args } => {
                if let Expr::Ident(name) = function.as_ref() {
                    if let Some(sig) = self.functions.get(name).cloned() {
                        // For generic functions, instantiate fresh type variables
                        if !sig.generic_params.is_empty() {
                            let mut fresh_map = std::collections::HashMap::new();
                            for (gp_name, _old_id) in &sig.generic_params {
                                let new_id = self.next_type_var;
                                self.next_type_var += 1;
                                self.type_var_values.insert(new_id, None);
                                fresh_map.insert(gp_name.clone(), Ty::Var(new_id));
                            }
                            // Unify argument types with parameter types
                            let non_self_params: Vec<_> = sig.params.iter()
                                .filter(|(n, _)| n != "self")
                                .collect();
                            for (i, (_pn, pt)) in non_self_params.iter().enumerate() {
                                if i < args.len() {
                                    let fresh_pt = self.substitute_type_vars(pt, &fresh_map);
                                    let arg_ty = self.infer_type(&args[i]);
                                    if !self.ty_is_inferred(&arg_ty) {
                                        self.types_compatible(&fresh_pt, &arg_ty);
                                    }
                                }
                            }
                            // Resolve the return type
                            let ret = sig.return_type.clone().unwrap_or(Ty::Unit);
                            let resolved = self.substitute_type_vars(&ret, &fresh_map);
                            return self.resolve(&resolved);
                        }
                        return self.resolve(&sig.return_type.clone().unwrap_or(Ty::Unit));
                    }
                }
                Ty::Inferred
            }

            Expr::MethodCall { object: _, method, args: _ } => {
                if let Some(sig) = self.functions.get(method) {
                    return sig.return_type.clone().unwrap_or(Ty::Unit);
                }

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

                if let Some(ref expr) = then_body.expr {
                    self.infer_type(expr)
                } else {
                    Ty::Unit
                }
            }

            Expr::Block(block) => {
                if let Some(ref expr) = block.expr {
                    self.infer_type(expr)
                } else {
                    Ty::Unit
                }
            }

            Expr::Path(parts) => Ty::Named(parts.join("::")),

            Expr::Array(items) => {
                if let Some(first) = items.first() {
                    Ty::Array(Box::new(self.infer_type(first)))
                } else {
                    Ty::Array(Box::new(Ty::Inferred))
                }
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

                if let Ty::Generic(_, args) = &inner_ty {
                    if args.len() >= 1 {
                        return args[0].clone();
                    }
                }
                Ty::Inferred
            }

            Expr::Self_ => {

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
            Expr::TryCatch(_) => Ty::Inferred,
            Expr::Throw(_) => Ty::Inferred,
            Expr::Comptime(_) => Ty::Inferred,
            Expr::ListComp(_) => Ty::Array(Box::new(Ty::Inferred)),
        }
    }



    fn ast_type_to_ty(&self, ast_ty: &Type) -> Ty {
        self.ast_type_to_ty_with_generics(ast_ty, &std::collections::HashMap::new())
    }

    fn ast_type_to_ty_with_generics(&self, ast_ty: &Type, generic_map: &std::collections::HashMap<String, Ty>) -> Ty {
        match ast_ty {
            Type::Name(name) => {
                // Check if this is a generic parameter (e.g., T in fn id<T>(x: T))
                if let Some(ty) = generic_map.get(name) {
                    return ty.clone();
                }
                if name == "string" || PRIMITIVE_TYPES.contains(&name.as_str()) {
                    Ty::Primitive(name.clone())
                } else {
                    Ty::Named(name.clone())
                }
            }
            Type::Path(parts) => Ty::Named(parts.join("::")),
            Type::Reference { inner, is_mut } => {
                Ty::Reference(Box::new(self.ast_type_to_ty_with_generics(inner, generic_map)), *is_mut)
            }
            Type::RawPointer { inner, is_mut } => {
                Ty::RawPointer(Box::new(self.ast_type_to_ty_with_generics(inner, generic_map)), *is_mut)
            }
            Type::Array { inner, .. } => Ty::Array(Box::new(self.ast_type_to_ty_with_generics(inner, generic_map))),
            Type::Slice(inner) => Ty::Slice(Box::new(self.ast_type_to_ty_with_generics(inner, generic_map))),
            Type::Tuple(types) => {
                Ty::Tuple(types.iter().map(|t| self.ast_type_to_ty_with_generics(t, generic_map)).collect())
            }
            Type::Generic { name, args } => {
                Ty::Generic(name.clone(), args.iter().map(|a| self.ast_type_to_ty_with_generics(a, generic_map)).collect())
            }
            Type::Function { params, return_type } => {
                Ty::FnPointer {
                    params: params.iter().map(|p| self.ast_type_to_ty_with_generics(p, generic_map)).collect(),
                    ret: Box::new(self.ast_type_to_ty_with_generics(return_type, generic_map)),
                }
            }
            Type::Unit => Ty::Unit,
            Type::Never => Ty::Never,
            Type::SelfType => Ty::Inferred,

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

    fn types_compatible(&mut self, a: &Ty, b: &Ty) -> bool {

        let a = self.resolve(a);
        let b = self.resolve(b);


        let a = if matches!(&a, Ty::Primitive(s) if s == "string") { Ty::Primitive("String".into()) } else { a };
        let b = if matches!(&b, Ty::Primitive(s) if s == "string") { Ty::Primitive("String".into()) } else { b };

        if a == b { return true; }


        if matches!(&a, Ty::Inferred) || matches!(&b, Ty::Inferred) { return true; }

        // When a type variable meets a concrete type, constrain the variable
        if let Ty::Var(id) = &a {
            self.constrain_type_var(*id, b.clone());
            return true;
        }
        if let Ty::Var(id) = &b {
            self.constrain_type_var(*id, a.clone());
            return true;
        }


        let a_is_option = matches!(&a, Ty::Generic(s, _) if s == "Option")
            || matches!(&a, Ty::Named(s) if s == "Option" || s.starts_with("Option::"));
        let b_is_option = matches!(&b, Ty::Generic(s, _) if s == "Option")
            || matches!(&b, Ty::Named(s) if s == "Option" || s.starts_with("Option::"));
        if a_is_option && b_is_option { return true; }
        let a_is_result = matches!(&a, Ty::Generic(s, _) if s == "Result")
            || matches!(&a, Ty::Named(s) if s == "Result" || s.starts_with("Result::"));
        let b_is_result = matches!(&b, Ty::Generic(s, _) if s == "Result")
            || matches!(&b, Ty::Named(s) if s == "Result" || s.starts_with("Result::"));
        if a_is_result && b_is_result { return true; }

        match (&a, &b) {

            (Ty::Primitive(a), Ty::Primitive(b)) => {
                NUMERIC_TYPES.contains(&a.as_str()) && NUMERIC_TYPES.contains(&b.as_str())
            }
            (Ty::Reference(a_inner, a_mut), Ty::Reference(b_inner, b_mut)) => {
                a_mut == b_mut && self.types_compatible(a_inner, b_inner)
            }
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
            (Ty::FnPointer { params: a_p, ret: a_r }, Ty::FnPointer { params: b_p, ret: b_r }) => {
                a_p.len() == b_p.len()
                    && a_p.iter().zip(b_p.iter()).all(|(a, b)| self.types_compatible(a, b))
                    && self.types_compatible(a_r, b_r)
            }
            _ => false,
        }
    }

    fn fresh_type_var(&mut self) -> Ty {
        let id = self.next_type_var;
        self.next_type_var += 1;
        self.type_var_values.insert(id, None);
        Ty::Var(id)
    }

    fn constrain_type_var(&mut self, var: usize, ty: Ty) {
        // If the type is itself a type variable, link them
        if let Ty::Var(other) = &ty {
            if *other == var {
                return; // Already the same variable
            }
        }
        self.type_var_values.insert(var, Some(ty));
    }

    fn resolve(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Var(id) => {
                match self.type_var_values.get(id) {
                    Some(Some(resolved)) => {
                        // Follow the chain — the resolved type might itself be a Var
                        self.resolve(resolved)
                    }
                    _ => ty.clone(), // Unresolved — return the Var as-is
                }
            }
            Ty::Reference(inner, is_mut) => {
                let resolved = self.resolve(inner);
                Ty::Reference(Box::new(resolved), *is_mut)
            }
            Ty::RawPointer(inner, is_mut) => {
                let resolved = self.resolve(inner);
                Ty::RawPointer(Box::new(resolved), *is_mut)
            }
            Ty::Array(inner) => Ty::Array(Box::new(self.resolve(inner))),
            Ty::Slice(inner) => Ty::Slice(Box::new(self.resolve(inner))),
            Ty::Tuple(types) => {
                Ty::Tuple(types.iter().map(|t| self.resolve(t)).collect())
            }
            Ty::Generic(name, args) => {
                Ty::Generic(name.clone(), args.iter().map(|a| self.resolve(a)).collect())
            }
            Ty::FnPointer { params, ret } => {
                Ty::FnPointer {
                    params: params.iter().map(|p| self.resolve(p)).collect(),
                    ret: Box::new(self.resolve(ret)),
                }
            }
            _ => ty.clone(),
        }
    }

    fn substitute_type_vars(&self, ty: &Ty, map: &std::collections::HashMap<String, Ty>) -> Ty {
        match ty {
            Ty::Generic(name, args) => {
                if let Some(replacement) = map.get(name) {
                    if args.is_empty() {
                        return replacement.clone();
                    }
                }
                Ty::Generic(name.clone(), args.iter().map(|a| self.substitute_type_vars(a, map)).collect())
            }
            Ty::Reference(inner, is_mut) => {
                Ty::Reference(Box::new(self.substitute_type_vars(inner, map)), *is_mut)
            }
            Ty::RawPointer(inner, is_mut) => {
                Ty::RawPointer(Box::new(self.substitute_type_vars(inner, map)), *is_mut)
            }
            Ty::Array(inner) => Ty::Array(Box::new(self.substitute_type_vars(inner, map))),
            Ty::Slice(inner) => Ty::Slice(Box::new(self.substitute_type_vars(inner, map))),
            Ty::Tuple(types) => {
                Ty::Tuple(types.iter().map(|t| self.substitute_type_vars(t, map)).collect())
            }
            Ty::FnPointer { params, ret } => {
                Ty::FnPointer {
                    params: params.iter().map(|p| self.substitute_type_vars(p, map)).collect(),
                    ret: Box::new(self.substitute_type_vars(ret, map)),
                }
            }
            _ => ty.clone(),
        }
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
        matches!(ty, Ty::Primitive(s) if NUMERIC_TYPES.contains(&s.as_str()))
    }

    fn ty_is_raw_pointer(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::RawPointer(_, _))
    }



    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            bindings: std::collections::HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, name: String, ty: Ty, is_mut: bool, _line: usize) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, (ty, false, is_mut));
        }
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.scopes.iter().rev()
            .find_map(|s| s.bindings.get(name).map(|(_, _, m)| *m))
            .unwrap_or(false)
    }

    fn is_defined(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.bindings.contains_key(name))
    }

    fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().map_or(false, |s| s.bindings.contains_key(name))
    }

    fn mark_used(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some((_, used, _)) = scope.bindings.get_mut(name) {
                *used = true;
                return;
            }
        }
    }

    fn lookup_var_type(&self, name: &str) -> Ty {
        self.scopes.iter().rev()
            .find_map(|s| s.bindings.get(name).map(|(ty, _, _)| ty.clone()))
            .unwrap_or(Ty::Inferred)
    }

    fn report_unused(&mut self) {
        let unused: Vec<String> = self.scopes.first()
            .map(|scope| {

                scope.bindings.iter()
                    .filter(|(name, &(_, used, _))| !used && name.as_str() != "main" && !name.starts_with('_'))
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .unwrap_or_default();
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
            _ => vec![],
        }
    }

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
        let mut checker = TypeChecker::new();
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

    #[test]
    fn test_immutable_let_rejects_assignment() {
        let diagnostics = check_source(r#"
            fn main() {
                let x = 10
                x = 20
            }
        "#);
        assert!(has_error(&diagnostics, "Cannot assign to immutable variable 'x'"));
    }

    #[test]
    fn test_mutable_let_allows_assignment() {
        let diagnostics = check_source(r#"
            fn main() {
                let mut x = 10
                x = 20
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_immutable_let_rejects_compound_assignment() {
        let diagnostics = check_source(r#"
            fn main() {
                let x = 10
                x += 5
            }
        "#);
        assert!(has_error(&diagnostics, "Cannot assign to immutable variable 'x'"));
    }

    #[test]
    fn test_mutable_let_allows_compound_assignment() {
        let diagnostics = check_source(r#"
            fn main() {
                let mut x = 10
                x += 5
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_mutable_let_allows_minus_equals() {
        let diagnostics = check_source(r#"
            fn main() {
                let mut x = 10
                x -= 3
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_function_param_is_immutable() {
        let diagnostics = check_source(r#"
            fn add_to(x: i32) -> i32 {
                x = x + 1
                return x
            }
        "#);
        assert!(has_error(&diagnostics, "Cannot assign to immutable variable 'x'"));
    }

    #[test]
    fn test_shadowed_binding_preserves_mutability() {
        let diagnostics = check_source(r#"
            fn main() {
                let x = 10
                let mut x = 20
                x = 30
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_for_loop_binding_is_immutable() {
        let diagnostics = check_source(r#"
            fn main() {
                for k in [1, 2, 3] {
                    k = 99
                }
            }
        "#);
        assert!(has_error(&diagnostics, "Cannot assign to immutable variable 'k'"));
    }

    // ─── Generic type variable resolution tests ─────────────────────────────

    #[test]
    fn test_generic_identity_resolves_to_i64() {
        // fn identity<T>(x: T) -> T { return x }
        // identity(42) should resolve T = i64, no errors
        let diagnostics = check_source(r#"
            fn identity<T>(x: T) -> T {
                return x
            }
            fn main() {
                let result = identity(42)
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_generic_identity_resolves_to_string() {
        // identity("hello") should resolve T = string, no errors
        let diagnostics = check_source(r#"
            fn identity<T>(x: T) -> T {
                return x
            }
            fn main() {
                let result = identity("hello")
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_generic_wrong_arg_type_reports_error() {
        // fn first<T, U>(a: T, b: U) -> T { return a }
        // first(42, "hello") is fine, but type mismatch on wrong type should be caught
        let diagnostics = check_source(r#"
            fn identity<T>(x: T) -> T {
                return x
            }
            fn main() {
                let result: i32 = identity("hello")
            }
        "#);
        // The declaration type mismatch should be caught
        assert!(has_error(&diagnostics, "Type mismatch"));
    }

    #[test]
    fn test_generic_two_params_resolves_independently() {
        // fn swap<A, B>(a: A, b: B) -> A { return a }
        // swap(42, "hello") — A = i64, B = string
        let diagnostics = check_source(r#"
            fn pick_first<A, B>(a: A, b: B) -> A {
                return a
            }
            fn main() {
                let result = pick_first(42, "hello")
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_generic_call_site_independence() {
        // Each call site should get its own type variable bindings
        // identity(42) resolves T = i64, identity("hi") resolves T = string
        let diagnostics = check_source(r#"
            fn identity<T>(x: T) -> T {
                return x
            }
            fn main() {
                let a = identity(42)
                let b = identity("hello")
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_non_generic_function_still_works() {
        // Non-generic functions should continue to work as before
        let diagnostics = check_source(r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b
            }
            fn main() {
                let result = add(1, 2)
            }
        "#);
        let errors: Vec<_> = diagnostics.iter().filter(|d| d.kind == DiagnosticKind::Error).collect();
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }
}

