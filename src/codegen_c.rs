use crate::ast::*;
use crate::backend::CodeGenerator;

pub struct CCodeGen {
    output: String,
    indent: usize,
    includes: Vec<String>,
    forward_decls: Vec<String>,
    struct_defs: Vec<String>,
    func_defs: Vec<String>,
    enum_defs: Vec<String>,
    typedefs: Vec<String>,
    current_struct: Option<String>,
}

impl CCodeGen {
    pub fn new() -> Self {
        Self {
            output: String::with_capacity(8192),
            indent: 0,
            includes: Vec::new(),
            forward_decls: Vec::new(),
            struct_defs: Vec::new(),
            func_defs: Vec::new(),
            enum_defs: Vec::new(),
            typedefs: Vec::new(),
            current_struct: None,
        }
    }

    fn write_indent(&self, out: &mut String) {
        for _ in 0..self.indent { out.push_str("    "); }
    }

    fn add_include(&mut self, inc: &str) {
        let inc = inc.to_string();
        if !self.includes.contains(&inc) { self.includes.push(inc); }
    }

    fn safe_name(name: &str) -> String {
        match name {
            "type" => "ruva_type".into(), "match" => "ruva_match".into(),
            "default" => "ruva_default".into(), "void" => "ruva_void".into(),
            "int" => "ruva_int".into(), "char" => "ruva_char".into(),
            "double" => "ruva_double".into(), "float" => "ruva_float".into(),
            "short" => "ruva_short".into(), "long" => "ruva_long".into(),
            "unsigned" => "ruva_unsigned".into(), "signed" => "ruva_signed".into(),
            "static" => "ruva_static".into(), "extern" => "ruva_extern".into(),
            "register" => "ruva_register".into(), "volatile" => "ruva_volatile".into(),
            "const" => "ruva_const_kw".into(), "struct" => "ruva_struct".into(),
            "enum" => "ruva_enum".into(), "union" => "ruva_union".into(),
            "if" => "ruva_if".into(), "else" => "ruva_else".into(),
            "for" => "ruva_for".into(), "while" => "ruva_while".into(),
            "do" => "ruva_do".into(), "switch" => "ruva_switch".into(),
            "case" => "ruva_case".into(), "break" => "ruva_break".into(),
            "continue" => "ruva_continue".into(), "return" => "ruva_return".into(),
            "sizeof" => "ruva_sizeof".into(), "NULL" => "RUVA_NULL".into(),
            _ => name.replace("::", "_").replace('.', "_"),
        }
    }

    fn type_str(&mut self, ty: &Type) -> String {
        match ty {
            Type::Name(name) => match name.as_str() {
                "i8" => { self.add_include("<stdint.h>"); "int8_t".into() }
                "i16" => { self.add_include("<stdint.h>"); "int16_t".into() }
                "i32" => { self.add_include("<stdint.h>"); "int32_t".into() }
                "i64" | "isize" => { self.add_include("<stdint.h>"); "int64_t".into() }
                "u8" => { self.add_include("<stdint.h>"); "uint8_t".into() }
                "u16" => { self.add_include("<stdint.h>"); "uint16_t".into() }
                "u32" => { self.add_include("<stdint.h>"); "uint32_t".into() }
                "u64" | "usize" => { self.add_include("<stdint.h>"); "uint64_t".into() }
                "f32" => "float".into(),
                "f64" => "double".into(),
                "bool" => "bool".into(),
                "string" => { self.add_include("<string.h>"); "const char*".into() }
                "char" => "char".into(),
                "void" => "void".into(),
                "null" => "NULL".into(),
                _ => Self::safe_name(name),
            },
            Type::Path(path) => Self::safe_name(&path.join("_")),
            Type::Reference { inner, is_mut: _ } => format!("{}*", self.type_str(inner)),
            Type::Slice(inner) => format!("{}*", self.type_str(inner)),
            Type::Array { inner, size } => {
                if let Some(s) = size {
                    let size_str = self.gen_expr_str(s);
                    format!("{}[{}]", self.type_str(inner), size_str)
                } else {
                    format!("{}*", self.type_str(inner))
                }
            }
            Type::Tuple(types) => {
                let parts: Vec<String> = types.iter().enumerate()
                    .map(|(i, t)| format!("{} field_{};", self.type_str(t), i))
                    .collect();
                format!("struct {{ {} }}", parts.join(" "))
            }
            Type::Generic { name, args } => {
                if args.is_empty() { Self::safe_name(name) }
                else {
                    let arg_strs: Vec<String> = args.iter().map(|a| self.type_str(a)).collect();
                    format!("{}_{}", Self::safe_name(name), arg_strs.join("_"))
                }
            }
            Type::Function { params, return_type } => {
                let ps: Vec<String> = params.iter().map(|p| self.type_str(p)).collect();
                format!("{} (*)({})", self.type_str(return_type), ps.join(", "))
            }
            Type::Unit | Type::Never => "void".into(),
            Type::SelfType => "void*".into(),
            Type::RawPointer { inner, is_mut } => {
                if *is_mut { format!("{}*", self.type_str(inner)) }
                else { format!("const {}*", self.type_str(inner)) }
            }
        }
    }

    fn gen_expr_str(&mut self, expr: &Expr) -> String {
        let mut result = String::new();
        self.gen_expr_to(expr, &mut result);
        result
    }

    // ─── Top-level ─────────────────────────────────────────────────────────

    pub fn generate_c(&mut self, program: &Program) -> String {
        for item in &program.items { self.collect_item(item); }

        let mut result = String::with_capacity(8192);
        result.push_str("/* Generated by the Ruva transpiler (C backend) — do not edit */\n");
        result.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <stdbool.h>\n#include <stdint.h>\n");
        let stds: Vec<&str> = vec!["<stdio.h>", "<stdlib.h>", "<string.h>", "<stdbool.h>", "<stdint.h>"];
        for inc in &self.includes {
            if !stds.contains(&inc.as_str()) { result.push_str(&format!("#include {}\n", inc)); }
        }
        result.push('\n');
        for en in &self.enum_defs { result.push_str(&format!("{}\n\n", en)); }
        for st in &self.struct_defs { result.push_str(&format!("{}\n\n", st)); }
        for td in &self.typedefs { result.push_str(&format!("{}\n", td)); }
        if !self.typedefs.is_empty() { result.push('\n'); }
        for func in &self.func_defs { result.push_str(&format!("{}\n", func)); }
        std::mem::take(&mut result)
    }

    fn collect_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                self.current_struct = None;
                let s = self.gen_function_str(f);
                self.func_defs.push(s);
            }
            Item::Struct(s) => {
                let s = self.gen_struct_str(s);
                self.struct_defs.push(s);
            }
            Item::Class(c) => {
                let s = self.gen_class_str(c);
                self.struct_defs.push(s);
            }
            Item::Enum(e) => {
                let s = self.gen_enum_str(e);
                self.enum_defs.push(s);
            }
            Item::Impl(imp) => self.gen_impl_items(imp),
            Item::Trait(t) => self.gen_trait_as_struct(t),
            Item::TypeAlias(ta) => {
                let ty = self.type_str(&ta.ty);
                self.typedefs.push(format!("typedef {} {};", ty, Self::safe_name(&ta.name)));
            }
            Item::Const(c) => {
                let ty_str = c.ty.as_ref().map(|t| self.type_str(t)).unwrap_or_else(|| "int64_t".into());
                let val = self.gen_expr_str(&c.value);
                self.func_defs.push(format!("#define {} (({})({}))", Self::safe_name(&c.name), ty_str, val));
            }
            Item::Import(_) | Item::Use(_) => {}
            Item::Attribute(attr) => self.collect_item(&attr.item),
            Item::Module(m) => {
                if let Some(ref body) = m.body {
                    for item in body { self.collect_item(item); }
                }
            }
            Item::ExternBlock(eb) => self.gen_extern_block_items(eb),
        }
    }

    fn gen_function_str(&mut self, f: &FunctionDef) -> String {
        let mut result = String::new();
        if f.is_test { result.push_str("/* @test */\n"); }
        let vis = if f.is_pub { "" } else { "static " };
        let ret = match &f.return_type { Some(ty) => self.type_str(ty), None => "void".into() };
        let params: Vec<String> = f.params.iter().map(|p| {
            if matches!(p.ty, Type::SelfType) { "void* self".into() }
            else {
                let ty = self.type_str(&p.ty);
                let r = if p.is_ref { "*" } else { "" };
                format!("{}{}{}", ty, r, Self::safe_name(&p.name))
            }
        }).collect();
        let name = Self::safe_name(&f.name);
        let full_name = if let Some(ref sn) = self.current_struct {
            format!("{}_{}", Self::safe_name(sn), name)
        } else { name };
        result.push_str(&format!("{}{} {}({}) {{\n", vis, ret, full_name, params.join(", ")));
        let saved = self.indent;
        self.indent = 1;
        self.gen_block_to(&f.body, &mut result);
        self.indent = saved;
        result.push_str("}\n");
        result
    }

    fn gen_struct_str(&mut self, s: &StructDef) -> String {
        let mut r = String::new();
        let name = Self::safe_name(&s.name);
        r.push_str(&format!("typedef struct {} {{\n", name));
        for field in &s.fields {
            r.push_str(&format!("    {} {};\n", self.type_str(&field.ty), Self::safe_name(&field.name)));
        }
        r.push_str(&format!("}} {};\n", name));
        if !s.fields.is_empty() {
            let params: Vec<String> = s.fields.iter()
                .map(|f| format!("{} {}", self.type_str(&f.ty), Self::safe_name(&f.name)))
                .collect();
            r.push_str(&format!("\nstatic inline {} {}_new({}) {{\n    {} result;\n", name, name, params.join(", "), name));
            for field in &s.fields {
                let fn_ = Self::safe_name(&field.name);
                r.push_str(&format!("    result.{} = {};\n", fn_, fn_));
            }
            r.push_str("    return result;\n}\n");
        }
        r
    }

    fn gen_class_str(&mut self, c: &ClassDef) -> String {
        let mut r = String::new();
        let name = Self::safe_name(&c.name);
        r.push_str(&format!("typedef struct {} {{\n", name));
        for field in &c.fields {
            let mk = if field.is_mut { "" } else { "const " };
            r.push_str(&format!("    {}{} {};\n", mk, self.type_str(&field.ty), Self::safe_name(&field.name)));
        }
        r.push_str(&format!("}} {};\n", name));
        if !c.fields.is_empty() {
            let params: Vec<String> = c.fields.iter()
                .map(|f| format!("{} {}", self.type_str(&f.ty), Self::safe_name(&f.name)))
                .collect();
            r.push_str(&format!("\nstatic inline {} {}_new({}) {{\n    {} result;\n", name, name, params.join(", "), name));
            for field in &c.fields {
                let fn_ = Self::safe_name(&field.name);
                r.push_str(&format!("    result.{} = {};\n", fn_, fn_));
            }
            r.push_str("    return result;\n}\n");
        }
        let saved = self.current_struct.take();
        self.current_struct = Some(c.name.clone());
        for method in &c.methods { r.push('\n'); r.push_str(&self.gen_function_str(method)); }
        self.current_struct = saved;
        r
    }

    fn gen_enum_str(&mut self, e: &EnumDef) -> String {
        let mut r = String::new();
        let name = Self::safe_name(&e.name);
        let has_data = e.variants.iter().any(|v| !v.fields.is_empty());
        if has_data {
            r.push_str(&format!("typedef enum {}_tag {{\n", name));
            for v in &e.variants { r.push_str(&format!("    {}_tag_{},\n", name, Self::safe_name(&v.name))); }
            r.push_str(&format!("}} {}_tag;\n\ntypedef struct {} {{\n    {}_tag tag;\n    union {{\n", name, name, name));
            for v in &e.variants {
                if !v.fields.is_empty() {
                    let fs: Vec<String> = v.fields.iter().enumerate()
                        .map(|(i, t)| format!("{} field_{};", self.type_str(t), i)).collect();
                    r.push_str(&format!("        struct {{ {} }} {};\n", fs.join(" "), Self::safe_name(&v.name)));
                }
            }
            r.push_str("    } data;\n");
            r.push_str(&format!("}} {};\n", name));
            for v in &e.variants {
                if !v.fields.is_empty() {
                    let vn = Self::safe_name(&v.name);
                    let params: Vec<String> = v.fields.iter().enumerate()
                        .map(|(i, t)| format!("{} field_{}", self.type_str(t), i)).collect();
                    r.push_str(&format!("\nstatic inline {} {}_{}({}) {{\n    {} result;\n    result.tag = {}_tag_{};\n", name, name, vn, params.join(", "), name, name, vn));
                    for i in 0..v.fields.len() { r.push_str(&format!("    result.data.{}.field_{} = field_{};\n", vn, i, i)); }
                    r.push_str("    return result;\n}\n");
                }
            }
        } else {
            r.push_str(&format!("typedef enum {} {{\n", name));
            for v in &e.variants { r.push_str(&format!("    {}_{},\n", name, Self::safe_name(&v.name))); }
            r.push_str(&format!("}} {};\n", name));
        }
        r
    }

    fn gen_impl_items(&mut self, imp: &ImplBlock) {
        let self_type = match &imp.self_type {
            Type::Name(n) => n.clone(), Type::Path(p) => p.join("_"), _ => "unknown".into(),
        };
        let saved = self.current_struct.take();
        self.current_struct = Some(self_type);
        for method in &imp.methods {
            let s = self.gen_function_str(method);
            self.func_defs.push(s);
        }
        self.current_struct = saved;
    }

    fn gen_trait_as_struct(&mut self, t: &TraitDef) {
        let name = Self::safe_name(&t.name);
        let mut r = format!("typedef struct {}_vtable {{\n", name);
        for method in &t.methods {
            let params: Vec<String> = method.params.iter().map(|p| {
                if matches!(p.ty, Type::SelfType) { "void* self".into() }
                else { format!("{} {}", self.type_str(&p.ty), Self::safe_name(&p.name)) }
            }).collect();
            let ret = match &method.return_type { Some(ty) => self.type_str(ty), None => "void".into() };
            r.push_str(&format!("    {} (*{})({});\n", ret, Self::safe_name(&method.name), params.join(", ")));
        }
        r.push_str(&format!("}} {}_vtable;\n", name));
        self.struct_defs.push(r);
    }

    fn gen_extern_block_items(&mut self, eb: &ExternBlock) {
        let mut r = format!("extern \"{}\" {{\n", eb.abi);
        for item in &eb.items {
            match item {
                ExternItem::Function { is_pub: _, name, params, return_type } => {
                    let ps: Vec<String> = params.iter().map(|p| format!("{} {}", self.type_str(&p.ty), Self::safe_name(&p.name))).collect();
                    let ret = match return_type { Some(t) => self.type_str(t), None => "void".into() };
                    r.push_str(&format!("    {} {}({});\n", ret, Self::safe_name(name), ps.join(", ")));
                }
                ExternItem::Static { is_pub: _, is_mut, name, ty } => {
                    let m = if *is_mut { "" } else { "const " };
                    r.push_str(&format!("    extern {}{} {};\n", m, self.type_str(ty), Self::safe_name(name)));
                }
                ExternItem::Const { is_pub: _, name, ty, value: _ } => {
                    r.push_str(&format!("    extern const {} {};\n", self.type_str(ty), Self::safe_name(name)));
                }
            }
        }
        r.push_str("}\n");
        self.func_defs.push(r);
    }

    // ─── Blocks ────────────────────────────────────────────────────────────

    fn gen_block_to(&mut self, block: &Block, out: &mut String) {
        for stmt in &block.stmts { self.gen_stmt_to(stmt, out); }
        if let Some(ref expr) = block.expr {
            self.write_indent(out);
            self.gen_expr_to(expr, out);
            out.push_str(";\n");
        }
    }

    fn gen_block_braces(&mut self, block: &Block, out: &mut String) {
        out.push_str("{\n");
        self.indent += 1;
        self.gen_block_to(block, out);
        self.indent -= 1;
        self.write_indent(out);
        out.push('}');
    }

    fn write_indent_to(&self, out: &mut String) {
        for _ in 0..self.indent { out.push_str("    "); }
    }

    // ─── Statements ────────────────────────────────────────────────────────

    fn gen_stmt_to(&mut self, stmt: &Stmt, out: &mut String) {
        match stmt {
            Stmt::Let { pattern, ty, is_mut, value } => {
                let pat = self.pattern_str(pattern);
                let ty_s = match ty { Some(t) => self.type_str(t), None => "auto".into() };
                let ck = if *is_mut { "" } else { "const " };
                self.write_indent(out);
                out.push_str(&format!("{}{} {} = ", ck, ty_s, pat));
                self.gen_expr_to(value, out);
                out.push_str(";\n");
            }
            Stmt::Expr(expr) => { self.write_indent(out); self.gen_expr_to(expr, out); out.push_str(";\n"); }
            Stmt::Return(expr) => {
                self.write_indent(out);
                if let Some(e) = expr { out.push_str("return "); self.gen_expr_to(e, out); out.push_str(";\n"); }
                else { out.push_str("return;\n"); }
            }
            Stmt::If { condition, then_body, else_body } => {
                self.write_indent(out);
                out.push_str("if ("); self.gen_expr_to(condition, out); out.push_str(") ");
                self.gen_block_braces(then_body, out);
                if let Some(ek) = else_body {
                    match ek {
                        ElseKind::If(cond, body) => {
                            out.push_str(" else if ("); self.gen_expr_to(cond, out); out.push_str(") ");
                            self.gen_block_braces(body, out);
                        }
                        ElseKind::Else(body) => { out.push_str(" else "); self.gen_block_braces(body, out); }
                    }
                }
                out.push('\n');
            }
            Stmt::While { condition, body } => {
                self.write_indent(out); out.push_str("while (");
                self.gen_expr_to(condition, out); out.push_str(") ");
                self.gen_block_braces(body, out); out.push('\n');
            }
            Stmt::Loop(body) => {
                self.write_indent(out); out.push_str("while (1) ");
                self.gen_block_braces(body, out); out.push('\n');
            }
            Stmt::Break(_) => { self.write_indent(out); out.push_str("break;\n"); }
            Stmt::Continue => { self.write_indent(out); out.push_str("continue;\n"); }
            Stmt::Match { expr, arms } => {
                self.write_indent(out); out.push_str("{\n"); self.indent += 1;
                self.write_indent(out); out.push_str("auto _mv = "); self.gen_expr_to(expr, out); out.push_str(";\n");
                for (i, arm) in arms.iter().enumerate() {
                    self.write_indent(out);
                    if i == 0 { out.push_str("if ("); } else { out.push_str("} else if ("); }
                    let ps = self.pattern_str(&arm.pattern);
                    if ps == "_" { out.push_str("1"); } else { out.push_str(&format!("_mv == {}", ps)); }
                    out.push_str(") {\n"); self.indent += 1;
                    self.write_indent(out); self.gen_expr_to(&arm.body, out); out.push_str(";\n");
                    self.indent -= 1;
                }
                self.write_indent(out); out.push_str("}\n"); self.indent -= 1;
                self.write_indent(out); out.push_str("}\n");
            }
            Stmt::TryCatch { try_body, catch_param, catch_body } => {
                self.write_indent(out); out.push_str("/* try */ {\n"); self.indent += 1;
                self.gen_block_to(try_body, out); self.indent -= 1;
                self.write_indent(out); out.push_str("}\n");
                self.write_indent(out); out.push_str(&format!("/* catch({}) */\n", catch_param));
                self.gen_block_to(catch_body, out);
            }
            Stmt::Block(block) => { self.gen_block_braces(block, out); out.push('\n'); }
            Stmt::Unsafe(block) => { self.gen_block_braces(block, out); out.push('\n'); }
            Stmt::For { pattern, iterable, body } => {
                let ps = self.pattern_str(pattern);
                self.write_indent(out); out.push_str(&format!("/* for {} */\n", ps));
                self.write_indent(out); out.push_str("for (size_t _i = 0; _i < ");
                self.gen_expr_to(iterable, out); out.push_str("; _i++) ");
                self.gen_block_braces(body, out); out.push('\n');
            }
            Stmt::WhileLet { pattern, value: _, body } => {
                let ps = self.pattern_str(pattern);
                self.write_indent(out); out.push_str(&format!("/* while let {} */\n", ps));
                self.write_indent(out); out.push_str("while (1) ");
                self.gen_block_braces(body, out); out.push('\n');
            }
        }
    }

    // ─── Expressions ───────────────────────────────────────────────────────

    fn gen_expr_to(&mut self, expr: &Expr, out: &mut String) {
        match expr {
            Expr::Int(n) => out.push_str(&format!("{}", n)),
            Expr::Float(f) => {
                let s = format!("{}", f);
                if !s.contains('.') && !s.contains('e') && !s.contains('E') { out.push_str(&format!("{}.0", s)); }
                else { out.push_str(&s); }
            }
            Expr::Str(s) => {
                out.push('"');
                for c in s.chars() { match c { '\n' => out.push_str("\\n"), '\t' => out.push_str("\\t"), '\r' => out.push_str("\\r"), '\\' => out.push_str("\\\\"), '"' => out.push_str("\\\""), '\0' => out.push_str("\\0"), _ => out.push(c) } }
                out.push('"');
            }
            Expr::Char(c) => { out.push('\''); out.push(*c); out.push('\''); }
            Expr::Bool(b) => { out.push_str(if *b { "true" } else { "false" }); }
            Expr::Null => out.push_str("NULL"),
            Expr::Self_ => out.push_str("self"),
            Expr::Ident(name) => out.push_str(&Self::safe_name(name)),
            Expr::Path(path) => out.push_str(&Self::safe_name(&path.join("_"))),
            Expr::Array(elements) => {
                out.push('{');
                for (i, el) in elements.iter().enumerate() { if i > 0 { out.push_str(", "); } self.gen_expr_to(el, out); }
                out.push('}');
            }
            Expr::Tuple(elements) => {
                out.push('{');
                for (i, el) in elements.iter().enumerate() { if i > 0 { out.push_str(", "); } self.gen_expr_to(el, out); }
                out.push('}');
            }
            Expr::Binary { op, left, right } => {
                out.push('('); self.gen_expr_to(left, out); out.push_str(&format!(" {} ", op)); self.gen_expr_to(right, out); out.push(')');
            }
            Expr::Unary { op, expr } => {
                match op { UnaryOp::Neg => out.push('-'), UnaryOp::Not => out.push('!'), UnaryOp::Deref => out.push('*') }
                self.gen_expr_to(expr, out);
            }
            Expr::Assign { target, value } => { self.gen_expr_to(target, out); out.push_str(" = "); self.gen_expr_to(value, out); }
            Expr::CompoundAssign { op, target, value } => { self.gen_expr_to(target, out); out.push_str(&format!(" {}= ", op)); self.gen_expr_to(value, out); }
            Expr::Call { function, args } => {
                self.gen_expr_to(function, out); out.push('(');
                for (i, arg) in args.iter().enumerate() { if i > 0 { out.push_str(", "); } self.gen_expr_to(arg, out); }
                out.push(')');
            }
            Expr::MethodCall { object, method, args } => {
                self.gen_expr_to(object, out); out.push('_'); out.push_str(&Self::safe_name(method)); out.push('(');
                self.gen_expr_to(object, out);
                for arg in args { out.push_str(", "); self.gen_expr_to(arg, out); }
                out.push(')');
            }
            Expr::Field { object, field } => { self.gen_expr_to(object, out); out.push('.'); out.push_str(&Self::safe_name(field)); }
            Expr::Index { object, index } => { self.gen_expr_to(object, out); out.push('['); self.gen_expr_to(index, out); out.push(']'); }
            Expr::Block(block) => { self.gen_block_braces(block, out); }
            Expr::Loop(body) => { out.push_str("while(1) "); self.gen_block_braces(body, out); }
            Expr::UnsafeBlock(body) => { self.gen_block_braces(body, out); }
            Expr::Sizeof(ty) => { out.push_str(&format!("sizeof({})", self.type_str(ty))); }
            Expr::NullPtr => out.push_str("NULL"),
            Expr::Macro { name, args } => {
                match name.as_str() {
                    "println!" | "eprintln!" => {
                        out.push_str("printf(\"%s\\n\"");
                        for arg in args { out.push_str(", "); self.gen_expr_to(arg, out); }
                        out.push(')');
                    }
                    "print!" => {
                        out.push_str("printf(\"%s\"");
                        for arg in args { out.push_str(", "); self.gen_expr_to(arg, out); }
                        out.push(')');
                    }
                    "panic!" => { out.push_str("fprintf(stderr, \"PANIC\"); exit(1)"); }
                    "vec!" => {
                        out.push('{');
                        for (i, arg) in args.iter().enumerate() { if i > 0 { out.push_str(", "); } self.gen_expr_to(arg, out); }
                        out.push('}');
                    }
                    _ => {
                        out.push_str(&Self::safe_name(name)); out.push('(');
                        for (i, arg) in args.iter().enumerate() { if i > 0 { out.push_str(", "); } self.gen_expr_to(arg, out); }
                        out.push(')');
                    }
                }
            }
            Expr::Reference { expr, is_mut: _ } => { out.push('&'); self.gen_expr_to(expr, out); }
            Expr::Deref(expr) => { out.push('*'); self.gen_expr_to(expr, out); }
            Expr::Move(expr) => { self.gen_expr_to(expr, out); }
            Expr::Cast { expr, ty } => { out.push_str(&format!("(({})", self.type_str(ty))); self.gen_expr_to(expr, out); out.push(')'); }
            Expr::VecLit(elements) => {
                out.push('{');
                for (i, el) in elements.iter().enumerate() { if i > 0 { out.push_str(", "); } self.gen_expr_to(el, out); }
                out.push('}');
            }
            Expr::StructLiteral { name, fields } => {
                if matches!(name.as_ref(), Expr::Self_) { out.push_str("Self"); } else { self.gen_expr_to(name, out); }
                out.push_str(" {");
                for (i, (fn_, val)) in fields.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(&Self::safe_name(fn_)); out.push_str(" = "); self.gen_expr_to(val, out);
                }
                out.push('}');
            }
            Expr::Assert { condition, message } => {
                out.push_str("assert("); self.gen_expr_to(condition, out);
                if let Some(ref msg) = message { out.push_str(" /* "); self.gen_expr_to(msg, out); out.push_str(" */"); }
                out.push(')');
            }
            Expr::AssertEq { left, right, message } => {
                out.push_str("assert("); self.gen_expr_to(left, out); out.push_str(" == "); self.gen_expr_to(right, out);
                if let Some(ref msg) = message { out.push_str(" /* "); self.gen_expr_to(msg, out); out.push_str(" */"); }
                out.push(')');
            }
            Expr::AssertNe { left, right, message } => {
                out.push_str("assert("); self.gen_expr_to(left, out); out.push_str(" != "); self.gen_expr_to(right, out);
                if let Some(ref msg) = message { out.push_str(" /* "); self.gen_expr_to(msg, out); out.push_str(" */"); }
                out.push(')');
            }
            Expr::If { condition, then_body, else_body } => {
                out.push('('); self.gen_expr_to(condition, out); out.push_str(" ? (");
                if let Some(ref expr) = then_body.expr { self.gen_expr_to(expr, out); } else { out.push_str("0"); }
                out.push_str(") : (");
                if let Some(ref e) = else_body { self.gen_expr_to(e, out); } else { out.push_str("0"); }
                out.push_str("))");
            }
            Expr::Match { arms, .. } => { if let Some(arm) = arms.first() { self.gen_expr_to(&arm.body, out); } }
            Expr::Try(expr) => { self.gen_expr_to(expr, out); }
            Expr::OptionalChaining { object, field } => {
                out.push('('); self.gen_expr_to(object, out); out.push_str(" ? ");
                self.gen_expr_to(object, out); out.push('.'); out.push_str(&Self::safe_name(field));
                out.push_str(" : NULL)");
            }
            Expr::NullCoalesce { left, right } => {
                self.gen_expr_to(left, out); out.push_str(" ? "); self.gen_expr_to(left, out); out.push_str(" : "); self.gen_expr_to(right, out);
            }
            Expr::ArrayRepeat { value, size } => {
                out.push_str("/* ["); self.gen_expr_to(value, out); out.push_str("; "); self.gen_expr_to(size, out); out.push_str("] */");
            }
            Expr::Range { start, end, inclusive } => {
                self.gen_expr_to(start, out); if *inclusive { out.push_str("..="); } else { out.push_str(".."); } self.gen_expr_to(end, out);
            }
            Expr::Closure { params, return_type: _, body } => {
                out.push_str("((void*)(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    let ty = p.ty.as_ref().map(|t| self.type_str(t)).unwrap_or_else(|| "void*".into());
                    out.push_str(&format!("{} {}", ty, p.name));
                }
                out.push_str("))("); self.gen_expr_to(body, out); out.push_str("))");
            }
            Expr::FString(parts) => {
                out.push_str("ruva_fstring(\"");
                for part in parts {
                    match part {
                        FStringPart::Text(text) => { out.push_str(&text.replace('"', "\\\"")); }
                        FStringPart::Expr(expr) => {
                            out.push_str("\"), "); self.gen_expr_to(expr, out); out.push_str(", ruva_fstring(\"");
                        }
                    }
                }
                out.push_str("\")");
            }
            Expr::Offsetof { struct_type, field } => {
                out.push_str(&format!("offsetof({}, {})", Self::safe_name(struct_type), Self::safe_name(field)));
            }
            Expr::FString(_) => {}
        }
    }

    fn pattern_str(&mut self, pat: &Pattern) -> String {
        match pat {
            Pattern::Wildcard => "_".into(),
            Pattern::Ident(name) => Self::safe_name(name),
            Pattern::Literal(expr) => self.gen_expr_str(expr),
            Pattern::Tuple(pats) => { let inner: Vec<String> = pats.iter().map(|p| self.pattern_str(p)).collect(); format!("({})", inner.join(", ")) }
            Pattern::Enum { path, fields } => {
                let ps = Self::safe_name(&path.join("_"));
                if fields.is_empty() { ps } else { let fs: Vec<String> = fields.iter().map(|f| self.pattern_str(f)).collect(); format!("{}({})", ps, fs.join(", ")) }
            }
            Pattern::Or(pats) => { let inner: Vec<String> = pats.iter().map(|p| self.pattern_str(p)).collect(); inner.join(" | ") }
            Pattern::Reference(inner) => format!("&{}", self.pattern_str(inner)),
            Pattern::Mut(name) => format!("mut {}", Self::safe_name(name)),
            Pattern::Struct { path, fields } => {
                let ps = Self::safe_name(&path.join("_"));
                let fs: Vec<String> = fields.iter().map(|(n, pat)| {
                    let p = self.pattern_str(pat);
                    if n == &p { n.clone() } else { format!("{}: {}", n, p) }
                }).collect();
                format!("{} {{ {} }}", ps, fs.join(", "))
            }
            Pattern::Range { start, end, inclusive } => {
                let s = self.gen_expr_str(start); let e = self.gen_expr_str(end);
                if *inclusive { format!("{}..={}", s, e) } else { format!("{}..{}", s, e) }
            }
        }
    }
}

impl CodeGenerator for CCodeGen {
    fn generate(&mut self, program: &Program) -> String { self.generate_c(program) }
    fn target_name(&self) -> &str { "c" }
    fn file_extension(&self) -> &str { ".c" }
}

// ─── C++ Code Generator ──────────────────────────────────────────────────────

pub struct CppCodeGen { c_gen: CCodeGen }

impl CppCodeGen {
    pub fn new() -> Self { Self { c_gen: CCodeGen::new() } }
}

impl CodeGenerator for CppCodeGen {
    fn generate(&mut self, program: &Program) -> String {
        let c_code = self.c_gen.generate_c(program);
        let mut r = String::with_capacity(c_code.len() + 512);
        r.push_str("// Generated by the Ruva transpiler (C++ backend) — do not edit\n");
        r.push_str("#include <iostream>\n#include <string>\n#include <vector>\n#include <memory>\n#include <functional>\n#include <optional>\n#include <cstdint>\n#include <cassert>\n\nnamespace ruva {\n\n");
        r.push_str(&c_code.replace("const char*", "std::string_view").replace("NULL", "nullptr"));
        r.push_str("\n} // namespace ruva\n");
        r
    }
    fn target_name(&self) -> &str { "cpp" }
    fn file_extension(&self) -> &str { ".cpp" }
}

// ─── WASM Code Generator ─────────────────────────────────────────────────────

pub struct WasmCodeGen { output: String, indent: usize }

impl WasmCodeGen {
    pub fn new() -> Self { Self { output: String::with_capacity(4096), indent: 0 } }

    fn write_indent(&mut self) { for _ in 0..self.indent { self.output.push_str("  "); } }
    fn writeln(&mut self, s: &str) { self.write_indent(); self.output.push_str(s); self.output.push('\n'); }

    fn type_str(&self, ty: &Type) -> String {
        match ty {
            Type::Name(name) => match name.as_str() {
                "i32" | "i64" | "f32" | "f64" => name.clone(),
                "i8" | "i16" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => if name.contains("64") { "i64".into() } else { "i32".into() },
                "bool" | "string" => "i32".into(),
                "void" => String::new(),
                _ => "i32".into(),
            },
            Type::Reference { .. } | Type::RawPointer { .. } => "i32".into(),
            Type::Unit => String::new(),
            _ => "i32".into(),
        }
    }

    pub fn generate_wat(&mut self, program: &Program) -> String {
        self.writeln("(module"); self.indent += 1;
        self.writeln("(memory (export \"memory\") 1)");
        for item in &program.items { self.gen_item(item); }
        self.indent -= 1; self.writeln(")");
        std::mem::take(&mut self.output)
    }

    fn gen_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.gen_function(f),
            Item::Impl(imp) => { for m in &imp.methods { self.gen_function(m); } }
            Item::Module(m) => { if let Some(ref body) = m.body { for i in body { self.gen_item(i); } } }
            Item::Attribute(a) => self.gen_item(&a.item),
            _ => {}
        }
    }

    fn gen_function(&mut self, f: &FunctionDef) {
        let params: Vec<String> = f.params.iter().map(|p| {
            let ty = self.type_str(&p.ty);
            if ty.is_empty() { format!("${}", p.name) } else { format!("${}: {}", p.name, ty) }
        }).collect();
        let ret = match &f.return_type { Some(ty) => { let ts = self.type_str(ty); if ts.is_empty() { String::new() } else { format!(" (result {})", ts) } }, None => String::new() };
        self.writeln(&format!("(func ${} (param {}){}", f.name, params.join(" "), ret));
        self.indent += 1;
        for stmt in &f.body.stmts { self.gen_stmt(stmt); }
        if let Some(ref expr) = f.body.expr { self.gen_expr(expr); }
        self.indent -= 1;
        self.writeln(")");
        if f.is_pub { self.writeln(&format!("(export \"{}\" (func ${}))", f.name, f.name)); }
    }

    fn gen_block(&mut self, block: &Block) {
        for stmt in &block.stmts { self.gen_stmt(stmt); }
        if let Some(ref expr) = block.expr { self.gen_expr(expr); }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(e) => { if let Some(e) = e { self.gen_expr(e); } self.writeln("return"); }
            Stmt::Expr(e) => { self.gen_expr(e); }
            Stmt::While { condition, body } => {
                self.writeln("(block $break"); self.indent += 1;
                self.writeln("(loop $continue"); self.indent += 1;
                self.gen_expr(condition); self.writeln("i32.eqz"); self.writeln("br_if $break");
                self.gen_block(body); self.writeln("br $continue");
                self.indent -= 1; self.writeln(")"); self.indent -= 1; self.writeln(")");
            }
            Stmt::Loop(body) => {
                self.writeln("(block $break"); self.indent += 1;
                self.writeln("(loop $continue"); self.indent += 1;
                self.gen_block(body); self.writeln("br $continue");
                self.indent -= 1; self.writeln(")"); self.indent -= 1; self.writeln(")");
            }
            Stmt::Break(_) => { self.writeln("br $break"); }
            Stmt::Continue => { self.writeln("br $continue"); }
            Stmt::Let { pattern, value, .. } => { self.gen_expr(value); self.writeln(&format!("(local.get ${})", self.pattern_str(pattern))); }
            Stmt::If { condition, then_body, else_body } => {
                self.gen_expr(condition); self.writeln("(if"); self.indent += 1;
                self.writeln("(then"); self.indent += 1; self.gen_block(then_body); self.indent -= 1; self.writeln(")");
                if let Some(ElseKind::Else(body)) = else_body { self.writeln("(else"); self.indent += 1; self.gen_block(body); self.indent -= 1; self.writeln(")"); }
                self.indent -= 1; self.writeln(")");
            }
            _ => { self.writeln(";; unhandled"); }
        }
    }

    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(n) => self.writeln(&format!("i64.const {}", n)),
            Expr::Float(f) => self.writeln(&format!("f64.const {}", f)),
            Expr::Bool(b) => { if *b { self.writeln("i32.const 1") } else { self.writeln("i32.const 0") } }
            Expr::Ident(name) => { self.writeln(&format!("local.get ${}", name)); }
            Expr::Binary { op, left, right } => {
                self.gen_expr(left); self.gen_expr(right);
                let op_str = match op { BinOp::Add => "i64.add", BinOp::Sub => "i64.sub", BinOp::Mul => "i64.mul", BinOp::Div => "i64.div_s", BinOp::Rem => "i64.rem_s", BinOp::Eq => "i64.eq", BinOp::Ne => "i64.ne", BinOp::Lt => "i64.lt_s", BinOp::Gt => "i64.gt_s", BinOp::Le => "i64.le_s", BinOp::Ge => "i64.ge_s", _ => ";; binop" };
                self.writeln(op_str);
            }
            Expr::Unary { op, expr } => {
                match op {
                    UnaryOp::Neg => { self.writeln("i64.const 0"); self.gen_expr(expr); self.writeln("i64.sub"); }
                    UnaryOp::Not => { self.gen_expr(expr); self.writeln("i64.eqz"); }
                    UnaryOp::Deref => { self.gen_expr(expr); self.writeln("i64.load"); }
                }
            }
            _ => { self.writeln(";; unhandled expr"); }
        }
    }

    fn pattern_str(&self, pat: &Pattern) -> String {
        match pat { Pattern::Ident(name) => name.clone(), _ => "_".into() }
    }
}

impl CodeGenerator for WasmCodeGen {
    fn generate(&mut self, program: &Program) -> String { self.generate_wat(program) }
    fn target_name(&self) -> &str { "wasm" }
    fn file_extension(&self) -> &str { ".wat" }
}
