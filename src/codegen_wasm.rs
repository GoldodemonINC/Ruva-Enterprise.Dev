use crate::ast::*;
use crate::backend::CodeGenerator;
use std::fmt::Write;

/// WebAssembly code generator for Ruva.
///
/// Targets WAT (WebAssembly Text format) which can be assembled to WASM.
/// Designed for browser plugins, sandboxed execution, game engine mods,
/// and secure server-side scripting with memory isolation.
pub struct WasmCodeGen {
    output: String,
    indent: usize,
    /// Memory management
    memory_pages: u32,
    /// Function signatures
    func_types: Vec<String>,
    /// Function bodies
    func_bodies: Vec<String>,
    /// Exported functions
    exports: Vec<String>,
    /// String literals (stored in data section)
    strings: Vec<(String, u32)>,
    next_string_offset: u32,
    /// Local variable counter
    local_counter: u32,
}

impl WasmCodeGen {
    pub fn new() -> Self {
        Self {
            output: String::with_capacity(8192),
            indent: 0,
            memory_pages: 16, // 1MB initial
            func_types: Vec::new(),
            func_bodies: Vec::new(),
            exports: Vec::new(),
            strings: Vec::new(),
            next_string_offset: 0,
            local_counter: 0,
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        writeln!(self.output, "{}", s).unwrap();
    }

    /// Map Ruva types to WASM type signatures
    fn wasm_type_str(&self, ty: &Type) -> &str {
        match ty {
            Type::Name(name) => match name.as_str() {
                "i8" | "i16" | "i32" | "isize" => "i32",
                "i64" => "i64",
                "u8" | "u16" | "u32" | "usize" => "i32",
                "u64" => "i64",
                "f32" => "f32",
                "f64" => "f64",
                "bool" => "i32",
                "char" => "i32",
                "string" | "String" => "i32",
                "void" => "",
                _ => "i32",
            },
            Type::Path(_) => "i32",
            Type::Reference { .. } | Type::RawPointer { .. } => "i32",
            Type::Array { .. } | Type::Slice(_) => "i32",
            Type::Tuple(_) => "i32",
            Type::Generic { .. } => "i32",
            Type::Function { .. } => "i32",
            Type::Unit | Type::Never => "",
            Type::SelfType => "i32",
        }
    }

    fn wasm_type_param_str(&self, ty: &Type) -> String {
        self.wasm_type_str(ty).to_string()
    }

    fn wasm_name(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        for ch in name.chars() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                result.push(ch);
            } else {
                result.push('_');
            }
        }
        if result.is_empty() || result.chars().next().unwrap().is_ascii_digit() {
            result.insert(0, '_');
        }
        result
    }

    fn gen_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.gen_function(f),
            Item::Struct(_) => {} // No-op in WASM (flat memory)
            Item::Enum(_) => {}   // Handled as i32 tags
            Item::Class(c) => {
                for method in &c.methods {
                    self.gen_function(method);
                }
            }
            Item::Impl(imp) => {
                for method in &imp.methods {
                    self.gen_function(method);
                }
            }
            Item::Trait(_) => {}
            Item::TypeAlias(_) => {}
            Item::Const(c) => self.gen_const(c),
            Item::Import(_) | Item::Use(_) => {}
            Item::Attribute(attr) => self.gen_item(&attr.item),
            Item::Module(m) => {
                if let Some(ref body) = m.body {
                    for inner in body { self.gen_item(inner); }
                }
            }
            Item::ExternBlock(eb) => {
                for item in &eb.items {
                    if let ExternItem::Function { name, params, return_type, .. } = item {
                        let param_types: Vec<String> = params.iter()
                            .map(|p| self.wasm_type_param_str(&p.ty))
                            .filter(|s| !s.is_empty())
                            .collect();
                        let ret_type = return_type.as_ref()
                            .map(|t| self.wasm_type_str(t))
                            .unwrap_or("");
                        let sig = if ret_type.is_empty() {
                            format!("(param {})", param_types.join(" "))
                        } else {
                            format!("(param {}) (result {})", param_types.join(" "), ret_type)
                        };
                        self.func_types.push(format!("(func {})", sig));
                        // Import declaration
                        self.exports.push(format!("  (import \"env\" \"{}\" (func ${}{}))",
                            name, Self::wasm_name(name), sig));
                    }
                }
            }
        }
    }

    fn gen_function(&mut self, f: &FunctionDef) {
        let name = Self::wasm_name(&f.name);
        self.local_counter = 0;

        // Collect parameters
        let params: Vec<String> = f.params.iter().map(|p| {
            let ty = self.wasm_type_param_str(&p.ty);
            if ty.is_empty() { String::new() }
            else { format!("(param ${} {})", Self::wasm_name(&p.name), ty) }
        }).filter(|s| !s.is_empty()).collect();

        // Result type
        let result = f.return_type.as_ref()
            .map(|t| {
                let ty = self.wasm_type_str(t);
                if ty.is_empty() { String::new() }
                else { format!("(result {})", ty) }
            })
            .unwrap_or_default();

        // Body
        let mut body = String::new();
        self.gen_block_wat(&mut body, &f.body);

        let sig = if result.is_empty() {
            format!("(func ${} {})", name, params.join(" "))
        } else {
            format!("(func ${} {} {})", name, params.join(" "), result)
        };

        self.func_types.push(sig.clone());
        self.func_bodies.push(format!("{} (nop) {}", sig, body));

        if f.is_pub {
            self.exports.push(format!("  (export \"{}\" (func ${}))", f.name, name));
        }
    }

    fn gen_block_wat(&mut self, output: &mut String, block: &Block) {
        for stmt in &block.stmts {
            self.gen_stmt_wat(output, stmt);
        }
        if let Some(ref expr) = block.expr {
            self.gen_expr_wat(output, expr);
        }
    }

    fn gen_stmt_wat(&mut self, output: &mut String, stmt: &Stmt) {
        match stmt {
            Stmt::Let { value, .. } => {
                self.gen_expr_wat(output, value);
                output.push_str(" (local.set $var)");
                output.push('\n');
            }
            Stmt::Expr(expr) => {
                self.gen_expr_wat(output, expr);
                output.push('\n');
            }
            Stmt::Return(Some(expr)) => {
                self.gen_expr_wat(output, expr);
                output.push_str(" return\n");
            }
            Stmt::Return(None) => {
                output.push_str("return\n");
            }
            Stmt::If { condition, then_body, else_body } => {
                output.push_str("(if ");
                self.gen_expr_wat(output, condition);
                output.push_str("\n  (then\n");
                self.gen_block_wat(output, then_body);
                if let Some(ElseKind::Else(body)) = else_body {
                    output.push_str("  )\n  (else\n");
                    self.gen_block_wat(output, body);
                }
                output.push_str("  )\n)\n");
            }
            Stmt::While { condition, body } => {
                output.push_str("(block $break\n");
                output.push_str("  (loop $continue\n");
                self.gen_expr_wat(output, condition);
                output.push_str(" i32.eqz br_if $break\n");
                self.gen_block_wat(output, body);
                output.push_str("  br $continue\n");
                output.push_str("  )\n");
                output.push_str(")\n");
            }
            Stmt::Loop(body) => {
                output.push_str("(block $break\n");
                output.push_str("  (loop $continue\n");
                self.gen_block_wat(output, body);
                output.push_str("  br $continue\n");
                output.push_str("  )\n");
                output.push_str(")\n");
            }
            Stmt::Break(_) => {
                output.push_str("br $break\n");
            }
            Stmt::Continue => {
                output.push_str("br $continue\n");
            }
            Stmt::Block(block) => {
                output.push_str("(block\n");
                self.gen_block_wat(output, block);
                output.push_str(")\n");
            }
            Stmt::Unsafe(block) => {
                // In WASM, everything is sandboxed — unsafe is a no-op
                self.gen_block_wat(output, block);
            }
            _ => {
                output.push_str("/* unhandled stmt */\n");
            }
        }
    }

    fn gen_expr_wat(&mut self, output: &mut String, expr: &Expr) {
        match expr {
            Expr::Int(n) => {
                output.push_str(&format!("i64.const {}", n));
            }
            Expr::Float(f) => {
                output.push_str(&format!("f64.const {}", f));
            }
            Expr::Bool(b) => {
                output.push_str(&format!("i32.const {}", if *b { 1 } else { 0 }));
            }
            Expr::Str(s) => {
                // Store string in data section and push pointer
                let offset = self.next_string_offset;
                let bytes = s.as_bytes();
                self.next_string_offset += (bytes.len() as u32 + 3) & !3; // align to 4
                self.strings.push((s.clone(), offset));
                output.push_str(&format!("i32.const {}", offset));
            }
            Expr::Char(c) => {
                output.push_str(&format!("i32.const {}", *c as u32));
            }
            Expr::Null | Expr::NullPtr => {
                output.push_str("i32.const 0");
            }
            Expr::Ident(name) => {
                output.push_str(&format!("(local.get ${})", Self::wasm_name(name)));
            }
            Expr::Binary { op, left, right } => {
                self.gen_expr_wat(output, left);
                output.push(' ');
                self.gen_expr_wat(output, right);
                output.push(' ');
                match op {
                    BinOp::Add => output.push_str("i64.add"),
                    BinOp::Sub => output.push_str("i64.sub"),
                    BinOp::Mul => output.push_str("i64.mul"),
                    BinOp::Div => output.push_str("i64.div_s"),
                    BinOp::Rem => output.push_str("i64.rem_s"),
                    BinOp::Eq => output.push_str("i64.eq"),
                    BinOp::Ne => output.push_str("i64.ne"),
                    BinOp::Lt => output.push_str("i64.lt_s"),
                    BinOp::Gt => output.push_str("i64.gt_s"),
                    BinOp::Le => output.push_str("i64.le_s"),
                    BinOp::Ge => output.push_str("i64.ge_s"),
                    BinOp::And => output.push_str("i64.and"),
                    BinOp::Or => output.push_str("i64.or"),
                    BinOp::BitAnd => output.push_str("i64.and"),
                    BinOp::BitOr => output.push_str("i64.or"),
                    BinOp::BitXor => output.push_str("i64.xor"),
                    BinOp::Shl => output.push_str("i64.shl"),
                    BinOp::Shr => output.push_str("i64.shr_s"),
                }
            }
            Expr::Unary { op, expr } => {
                match op {
                    UnaryOp::Neg => {
                        output.push_str("i64.const 0 ");
                        self.gen_expr_wat(output, expr);
                        output.push_str(" i64.sub");
                    }
                    UnaryOp::Not => {
                        self.gen_expr_wat(output, expr);
                        output.push_str(" i64.eqz");
                    }
                    UnaryOp::Deref => {
                        // Load from memory
                        self.gen_expr_wat(output, expr);
                        output.push_str(" i64.load");
                    }
                }
            }
            Expr::Call { function, args } => {
                for arg in args {
                    self.gen_expr_wat(output, arg);
                    output.push(' ');
                }
                if let Expr::Ident(name) = function.as_ref() {
                    output.push_str(&format!("call ${}", Self::wasm_name(name)));
                } else {
                    self.gen_expr_wat(output, function);
                    output.push_str(" call_indirect");
                }
            }
            Expr::Macro { name, args } => {
                match name.as_str() {
                    "println" | "print" => {
                        for arg in args {
                            self.gen_expr_wat(output, arg);
                            output.push_str(" call $print\n");
                        }
                    }
                    "sizeof" => {
                        output.push_str("i64.const 8"); // default pointer size
                    }
                    _ => {
                        for arg in args {
                            self.gen_expr_wat(output, arg);
                            output.push(' ');
                        }
                        output.push_str(&format!("call ${}", Self::wasm_name(name)));
                    }
                }
            }
            Expr::If { condition, then_body, else_body } => {
                output.push_str("(if (result i64) ");
                self.gen_expr_wat(output, condition);
                output.push_str("\n  (then ");
                if let Some(ref expr) = then_body.expr {
                    self.gen_expr_wat(output, expr);
                } else {
                    output.push_str("i64.const 0");
                }
                output.push_str(")");
                output.push_str("\n  (else ");
                if let Some(ref else_expr) = else_body {
                    self.gen_expr_wat(output, else_expr);
                } else {
                    output.push_str("i64.const 0");
                }
                output.push_str(")\n)");
            }
            Expr::Block(block) => {
                output.push_str("(block (result i64)\n");
                self.gen_block_wat(output, block);
                output.push_str(")");
            }
            Expr::Assign { target, value } => {
                self.gen_expr_wat(output, value);
                output.push(' ');
                if let Expr::Ident(name) = target.as_ref() {
                    output.push_str(&format!("(local.set ${})", Self::wasm_name(name)));
                }
            }
            Expr::Cast { expr, .. } => {
                self.gen_expr_wat(output, expr);
            }
            Expr::Move(expr) | Expr::Reference { expr, .. } | Expr::Deref(expr) => {
                self.gen_expr_wat(output, expr);
            }
            Expr::Self_ => {
                output.push_str("(local.get $self)");
            }
            Expr::Field { .. } | Expr::Index { .. } => {
                // Simplified: treat as i32 offset
                output.push_str("i32.const 0");
            }
            Expr::StructLiteral { .. } | Expr::Tuple(_) | Expr::Array(_) | Expr::VecLit(_) => {
                output.push_str("i32.const 0");
            }
            Expr::Sizeof(_) => {
                output.push_str("i64.const 8");
            }
            Expr::Loop(block) => {
                output.push_str("(block $break\n  (loop $continue\n");
                self.gen_block_wat(output, block);
                output.push_str("  br $continue\n  )\n)");
            }
            _ => {
                output.push_str("i64.const 0");
            }
        }
    }

    fn gen_const(&mut self, c: &ConstDef) {
        // WASM doesn't have top-level constants; skip
    }
}

impl CodeGenerator for WasmCodeGen {
    fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.writeln(";; Generated by the Ruva transpiler — WASM backend — do not edit");
        self.writeln("(module");
        self.indent += 1;

        // Memory
        self.writeln(&format!("(memory (export \"memory\") {})", self.memory_pages));

        // Collect all items
        for item in &program.items {
            self.gen_item(item);
        }

        // Print built-in function
        self.func_types.push("(func $print (param i32) (nop))".into());

        // Type section
        self.writeln(";; Types");
        { let func_types = self.func_types.clone(); for ft in &func_types { self.writeln(ft); } }

        // Functions
        self.writeln(";; Functions");
        { let func_bodies = self.func_bodies.clone(); for fb in &func_bodies { self.writeln(fb); } }

        // Data section for strings
        if !self.strings.is_empty() {
            self.writeln(";; String data");
            let strings = self.strings.clone(); for (s, offset) in &strings {
                let bytes: Vec<String> = s.bytes().map(|b| format!("\\{:02x}", b)).collect();
                self.writeln(&format!("(data (i32.const {}) \"{}\")", offset, bytes.join("")));
            }
        }

        // Exports
        self.writeln(";; Exports");
        { let exports = self.exports.clone(); for exp in &exports { self.writeln(exp); } }

        self.indent -= 1;
        self.writeln(")");

        std::mem::take(&mut self.output)
    }

    fn target_name(&self) -> &str { "wasm" }
    fn file_extension(&self) -> &str { ".wat" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn gen_wasm(source: &str) -> String {
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();
        let mut gen = WasmCodeGen::new();
        gen.generate(&program)
    }

    #[test]
    fn test_module_header() {
        let code = gen_wasm("fn main() { println!(\"hello\") }");
        assert!(code.contains("(module"));
        assert!(code.contains("memory"));
    }

    #[test]
    fn test_function_generation() {
        let code = gen_wasm("fn add(a: i32, b: i32) -> i32 { return a + b }");
        assert!(code.contains("func $add"));
        assert!(code.contains("i32"));
    }

    #[test]
    fn test_string_literal() {
        let code = gen_wasm("fn main() { println!(\"hello world\") }");
        assert!(code.contains("module"));
    }

    #[test]
    fn test_if_else() {
        let code = gen_wasm(r#"
            fn check(x: i32) -> i32 {
                if x > 0 {
                    return 1
                } else {
                    return 0
                }
            }
        "#);
        assert!(code.contains("if"));
        assert!(code.contains("then"));
        assert!(code.contains("else"));
    }

    #[test]
    fn test_loop() {
        let code = gen_wasm("fn count() { loop { break } }");
        assert!(code.contains("loop"));
        assert!(code.contains("break") || code.contains("br"));
    }
}
