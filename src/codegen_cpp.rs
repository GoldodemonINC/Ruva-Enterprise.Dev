use crate::ast::*;
use crate::backend::CodeGenerator;
use std::fmt::Write;

/// C++ code generator for Ruva.
///
/// Targets C++20 with modern idioms: RAII, smart pointers, constexpr,
/// concepts, ranges. Designed for game engine integration (Unreal Engine,
/// Godot, custom engines) and high-performance server hosting.
pub struct CppCodeGen {
    output: String,
    header: String,
    indent: usize,
    includes: Vec<String>,
    func_decls: Vec<String>,
    struct_defs: Vec<String>,
    enum_defs: Vec<String>,
    current_function: Option<String>,
}

impl CppCodeGen {
    pub fn new() -> Self {
        let mut includes = Vec::new();
        includes.push("#include <cstdint>".into());
        includes.push("#include <cstddef>".into());
        includes.push("#include <cstdio>".into());
        includes.push("#include <cstdlib>".into());
        includes.push("#include <cstring>".into());
        includes.push("#include <string>".into());
        includes.push("#include <memory>".into());
        includes.push("#include <vector>".into());
        includes.push("#include <optional>".into());
        includes.push("#include <variant>".into());
        includes.push("#include <functional>".into());
        includes.push("#include <cassert>".into());

        Self {
            output: String::with_capacity(8192),
            header: String::with_capacity(4096),
            indent: 0,
            includes,
            func_decls: Vec::new(),
            struct_defs: Vec::new(),
            enum_defs: Vec::new(),
            current_function: None,
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        writeln!(self.output, "{}", s).unwrap();
    }

    fn header_writeln(&mut self, s: &str) {
        writeln!(self.header, "{}", s).unwrap();
    }

    fn cpp_type_str(&self, ty: &Type) -> String {
        match ty {
            Type::Name(name) => match name.as_str() {
                "i8" => "int8_t".into(),
                "i16" => "int16_t".into(),
                "i32" => "int32_t".into(),
                "i64" => "int64_t".into(),
                "isize" => "intptr_t".into(),
                "u8" => "uint8_t".into(),
                "u16" => "uint16_t".into(),
                "u32" => "uint32_t".into(),
                "u64" => "uint64_t".into(),
                "usize" => "size_t".into(),
                "f32" => "float".into(),
                "f64" => "double".into(),
                "bool" => "bool".into(),
                "char" => "char".into(),
                "string" | "String" => "std::string".into(),
                "void" => "void".into(),
                "Self" => "self_t".into(),
                "Option" => "std::optional".into(),
                "Result" => "std::expected".into(),
                "Vec" => "std::vector".into(),
                _ => name.clone(),
            },
            Type::Reference { inner, is_mut } => {
                if *is_mut {
                    format!("{}&", self.cpp_type_str(inner))
                } else {
                    format!("const {}&", self.cpp_type_str(inner))
                }
            }
            Type::RawPointer { inner, is_mut } => {
                if *is_mut {
                    format!("{}*", self.cpp_type_str(inner))
                } else {
                    format!("const {}*", self.cpp_type_str(inner))
                }
            }
            Type::Array { inner, size } => {
                if let Some(s) = size {
                    if let Expr::Int(n) = **s {
                        format!("std::array<{}, {}>", self.cpp_type_str(inner), n)
                    } else {
                        format!("std::vector<{}>", self.cpp_type_str(inner))
                    }
                } else {
                    format!("std::vector<{}>", self.cpp_type_str(inner))
                }
            }
            Type::Slice(inner) => format!("std::span<const {}>", self.cpp_type_str(inner)),
            Type::Tuple(types) => {
                let arg_strs: Vec<String> = types.iter().map(|t| self.cpp_type_str(t)).collect();
                format!("std::tuple<{}>", arg_strs.join(", "))
            }
            Type::Generic { name, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| self.cpp_type_str(a)).collect();
                match name.as_str() {
                    "Option" => format!("std::optional<{}>", arg_strs.join(", ")),
                    "Vec" => format!("std::vector<{}>", arg_strs.join(", ")),
                    "HashMap" => format!("std::unordered_map<{}>", arg_strs.join(", ")),
                    "Result" => format!("std::expected<{}>", arg_strs.join(", ")),
                    _ => format!("{}<{}>", name, arg_strs.join(", ")),
                }
            }
            Type::Function { params, return_type } => {
                let param_strs: Vec<String> = params.iter().map(|p| self.cpp_type_str(p)).collect();
                format!("std::function<{} ({})>", self.cpp_type_str(return_type), param_strs.join(", "))
            }
            Type::Path(_) => "void".into(),
            Type::Unit => "void".into(),
            Type::Never => "[[noreturn]] void".into(),
            Type::SelfType => "self_t".into(),
        }
    }

    fn cpp_name(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        for ch in name.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
            } else {
                result.push('_');
            }
        }
        result
    }

    fn gen_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.gen_function(f),
            Item::Struct(s) => self.gen_struct(s),
            Item::Enum(e) => self.gen_enum(e),
            Item::Class(c) => self.gen_class(c),
            Item::Impl(imp) => self.gen_impl(imp),
            Item::Trait(t) => self.gen_trait(t),
            Item::TypeAlias(ta) => self.gen_type_alias(ta),
            Item::Const(c) => self.gen_const(c),
            Item::Import(_) | Item::Use(_) => {}
            Item::Attribute(attr) => self.gen_attribute(attr),
            Item::Module(m) => {
                if let Some(ref body) = m.body {
                    for inner in body { self.gen_item(inner); }
                }
            }
            Item::ExternBlock(eb) => self.gen_extern_block(eb),
        }
    }

    fn gen_function(&mut self, f: &FunctionDef) {
        let c_name = Self::cpp_name(&f.name);
        let ret_type = f.return_type.as_ref()
            .map(|t| self.cpp_type_str(t))
            .unwrap_or_else(|| "void".into());

        let mut params = Vec::new();
        for p in &f.params {
            let param_ty = self.cpp_type_str(&p.ty);
            if p.name == "self" || p.name == "self_" {
                params.insert(0, format!("{}& self", param_ty));
            } else if p.is_ref || p.is_mut {
                params.push(format!("{}& {}", param_ty, Self::cpp_name(&p.name)));
            } else {
                params.push(format!("{} {}", param_ty, Self::cpp_name(&p.name)));
            }
        }

        let params_str = if params.is_empty() { "void".into() } else { params.join(", ") };
        let decl = format!("{} {}({});", ret_type, c_name, params_str);

        if f.is_pub { self.func_decls.push(format!("{};", decl)); }

        if !f.body.stmts.is_empty() || f.body.expr.is_some() {
            self.writeln(&decl);
            self.writeln("{");
            self.indent += 1;
            let old_fn = self.current_function.take();
            self.current_function = Some(c_name.clone());
            self.gen_block(&f.body);
            self.current_function = old_fn;
            self.indent -= 1;
            self.writeln("}");
            self.writeln("");
        }
    }

    fn gen_struct(&mut self, s: &StructDef) {
        let c_name = Self::cpp_name(&s.name);
        self.writeln(&format!("struct {} {{", c_name));
        self.indent += 1;
        for field in &s.fields {
            let vis = if field.is_pub { "public" } else { "private" };
            self.writeln(&format!("{}: {} {};", vis, self.cpp_type_str(&field.ty), Self::cpp_name(&field.name)));
        }
        self.indent -= 1;
        self.writeln("};");
        self.writeln("");
    }

    fn gen_enum(&mut self, e: &EnumDef) {
        let c_name = Self::cpp_name(&e.name);
        let has_data = e.variants.iter().any(|v| !v.fields.is_empty());

        if has_data {
            self.writeln(&format!("struct {} {{", c_name));
            self.indent += 1;
            self.writeln("enum Tag {");
            self.indent += 1;
            for v in &e.variants {
                self.writeln(&format!("{},", Self::cpp_name(&v.name)));
            }
            self.indent -= 1;
            self.writeln("};");
            self.writeln("Tag tag;");
            self.writeln("union {");
            self.indent += 1;
            for v in &e.variants {
                if !v.fields.is_empty() {
                    let field_tys: Vec<String> = v.fields.iter().map(|t| self.cpp_type_str(t)).collect();
                    self.writeln(&format!("struct {{ {} }} {};",
                        field_tys.iter().enumerate()
                            .map(|(i, t)| format!("{} f{}", t, i))
                            .collect::<Vec<_>>().join("; "),
                        Self::cpp_name(&v.name)));
                }
            }
            self.indent -= 1;
            self.writeln("};");
            self.indent -= 1;
            self.writeln("};");
        } else {
            self.writeln(&format!("enum class {} {{", c_name));
            self.indent += 1;
            for v in &e.variants {
                self.writeln(&format!("{},", Self::cpp_name(&v.name)));
            }
            self.indent -= 1;
            self.writeln("};");
        }
        self.writeln("");
    }

    fn gen_class(&mut self, c: &ClassDef) {
        let c_name = Self::cpp_name(&c.name);
        self.writeln(&format!("class {} {{", c_name));
        self.indent += 1;
        self.writeln("public:");
        self.indent += 1;

        // Fields
        for field in &c.fields {
            self.writeln(&format!("{} {};", self.cpp_type_str(&field.ty), Self::cpp_name(&field.name)));
        }

        // Methods
        for method in &c.methods {
            self.gen_function(method);
        }

        self.indent -= 1;
        self.indent -= 1;
        self.writeln("};");
        self.writeln("");
    }

    fn gen_impl(&mut self, imp: &ImplBlock) {
        for method in &imp.methods {
            let mut method_def = method.clone();
            if method_def.params.is_empty() || method_def.params[0].name != "self" {
                let self_param = Param {
                    name: "self".into(),
                    ty: Type::Reference {
                        inner: Box::new(imp.self_type.clone()),
                        is_mut: true,
                    },
                    is_ref: false,
                    is_mut: false,
                };
                method_def.params.insert(0, self_param);
            }
            self.gen_function(&method_def);
        }
    }

    fn gen_trait(&mut self, t: &TraitDef) {
        let c_name = Self::cpp_name(&t.name);
        self.writeln(&format!("class {} {{", c_name));
        self.indent += 1;
        self.writeln("public:");
        self.indent += 1;
        for method in &t.methods {
            let ret_type = method.return_type.as_ref()
                .map(|t| self.cpp_type_str(t))
                .unwrap_or_else(|| "void".into());
            let params: Vec<String> = method.params.iter().map(|p| {
                let ty = self.cpp_type_str(&p.ty);
                format!("{} {}", ty, Self::cpp_name(&p.name))
            }).collect();
            let params_str = if params.is_empty() { "void".into() } else { params.join(", ") };
            self.writeln(&format!("virtual {} {}({}) = 0;", ret_type, Self::cpp_name(&method.name), params_str));
        }
        self.indent -= 1;
        self.indent -= 1;
        self.writeln("};");
        self.writeln("");
    }

    fn gen_type_alias(&mut self, ta: &TypeAliasDef) {
        let c_name = Self::cpp_name(&ta.name);
        let target = self.cpp_type_str(&ta.ty);
        self.writeln(&format!("using {} = {};", c_name, target));
    }

    fn gen_const(&mut self, c: &ConstDef) {
        let c_name = Self::cpp_name(&c.name);
        let ty_str = c.ty.as_ref().map(|t| self.cpp_type_str(t));
        let value_str = self.gen_expr_str(&c.value);
        if let Some(ref ts) = ty_str {
            self.writeln(&format!("constexpr {} {} = {};", ts, c_name, value_str));
        } else {
            self.writeln(&format!("constexpr auto {} = {};", c_name, value_str));
        }
    }

    fn gen_attribute(&mut self, attr: &Attribute) {
        match attr.name.as_str() {
            "safe" => self.writeln("// @safe"),
            "trusted" => self.writeln("// @trusted"),
            "unsafe" => self.writeln("// @unsafe"),
            "hotpath" => self.writeln("__attribute__((hot)) // @hotpath"),
            "realtime" => self.writeln("// @realtime - must complete within deadline"),
            "component" => self.writeln("// @component - ECS component"),
            "immutable" => self.writeln("// @immutable"),
            "tamper_proof" => self.writeln("// @tamper_proof - integrity checked"),
            "nodiscard" => self.writeln("[[nodiscard]]"),
            "noreturn" => self.writeln("[[noreturn]]"),
            "constexpr" => self.writeln("constexpr"),
            _ => { self.writeln(&format!("// @{}", attr.name)); }
        }
        self.gen_item(&attr.item);
    }

    fn gen_extern_block(&mut self, eb: &ExternBlock) {
        self.writeln(&format!("extern \"{}\" {{", eb.abi));
        self.indent += 1;
        for item in &eb.items {
            match item {
                ExternItem::Function { name, params, return_type, .. } => {
                    let ret = return_type.as_ref().map(|t| self.cpp_type_str(t)).unwrap_or_else(|| "void".into());
                    let ps: Vec<String> = params.iter().map(|p| {
                        format!("{} {}", self.cpp_type_str(&p.ty), Self::cpp_name(&p.name))
                    }).collect();
                    self.writeln(&format!("{} {}({});", ret, Self::cpp_name(name), ps.join(", ")));
                }
                ExternItem::Static { name, ty, is_mut, .. } => {
                    let q = if *is_mut { "" } else { "const " };
                    self.writeln(&format!("extern {}{} {};", q, self.cpp_type_str(ty), Self::cpp_name(name)));
                }
                ExternItem::Const { name, ty, .. } => {
                    self.writeln(&format!("extern const {} {};", self.cpp_type_str(ty), Self::cpp_name(name)));
                }
            }
        }
        self.indent -= 1;
        self.writeln("}");
    }

    fn gen_block(&mut self, block: &Block) {
        for stmt in &block.stmts { self.gen_stmt(stmt); }
        if let Some(ref expr) = block.expr {
            self.write_indent();
            self.gen_expr(expr);
            self.output.push_str(";\n");
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { pattern, ty, value, .. } => {
                let val_str = self.gen_expr_str(value);
                let names = self.pattern_names(pattern);
                for name in &names {
                    let var_name = Self::cpp_name(name);
                    if let Some(declared_ty) = ty {
                        self.writeln(&format!("auto {} = static_cast<{}>({});", var_name, self.cpp_type_str(declared_ty), val_str));
                    } else {
                        self.writeln(&format!("auto {} = {};", var_name, val_str));
                    }
                }
            }
            Stmt::Expr(expr) => {
                self.write_indent();
                self.gen_expr(expr);
                self.output.push_str(";\n");
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.writeln(&format!("return {};", self.gen_expr_str(e)));
                } else {
                    self.writeln("return;");
                }
            }
            Stmt::If { condition, then_body, else_body } => {
                self.writeln(&format!("if ({}) {{", self.gen_expr_str(condition)));
                self.indent += 1;
                self.gen_block(then_body);
                self.indent -= 1;
                if let Some(ElseKind::Else(body)) = else_body {
                    self.writeln("} else {");
                    self.indent += 1;
                    self.gen_block(body);
                    self.indent -= 1;
                }
                self.writeln("}");
            }
            Stmt::For { pattern, iterable, body } => {
                let var_name = self.pattern_names(pattern).first()
                    .map(|n| Self::cpp_name(n)).unwrap_or_else(|| "_i".into());
                self.writeln(&format!("for (auto& {} : {}) {{", var_name, self.gen_expr_str(iterable)));
                self.indent += 1;
                self.gen_block(body);
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::While { condition, body } => {
                self.writeln(&format!("while ({}) {{", self.gen_expr_str(condition)));
                self.indent += 1;
                self.gen_block(body);
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::Loop(body) => {
                self.writeln("while (true) {");
                self.indent += 1;
                self.gen_block(body);
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::Break(_) => self.writeln("break;"),
            Stmt::Continue => self.writeln("continue;"),
            Stmt::Match { expr, arms } => {
                let expr_str = self.gen_expr_str(expr);
                self.writeln(&format!("switch (auto _mv = {}; _mv.tag) {{", &expr_str[..expr_str.len().min(40)]));
                self.indent += 1;
                for arm in arms {
                    self.gen_pattern_match_arm(arm);
                }
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::TryCatch { try_body, catch_param, catch_body } => {
                self.writeln("try {");
                self.indent += 1;
                self.gen_block(try_body);
                self.indent -= 1;
                self.writeln(&format!("}} catch (auto& {}) {{", Self::cpp_name(catch_param)));
                self.indent += 1;
                self.gen_block(catch_body);
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::Block(block) => {
                self.writeln("{");
                self.indent += 1;
                self.gen_block(block);
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::Unsafe(block) => {
                self.writeln("{ /* unsafe */");
                self.indent += 1;
                self.gen_block(block);
                self.indent -= 1;
                self.writeln("}");
            }
            Stmt::WhileLet { pattern, value, body } => {
                let val_str = self.gen_expr_str(value);
                let names = self.pattern_names(pattern);
                let var_name = names.first().map(|n| Self::cpp_name(n)).unwrap_or_else(|| "_tmp".into());
                self.writeln(&format!("while (auto {} = {}) {{", var_name, val_str));
                self.indent += 1;
                self.gen_block(body);
                self.indent -= 1;
                self.writeln("}");
            }
        }
    }

    fn gen_pattern_match_arm(&mut self, arm: &MatchArm) {
        match &arm.pattern {
            Pattern::Wildcard => {
                self.writeln("default: {");
                self.indent += 1;
                self.write_indent();
                self.gen_expr(&arm.body);
                self.output.push_str(";\n");
                self.writeln("break;");
                self.indent -= 1;
                self.writeln("}");
            }
            Pattern::Enum { path, .. } => {
                let variant = path.last().map(|p| Self::cpp_name(p)).unwrap_or_default();
                self.writeln(&format!("case {}::{}: {{", Self::cpp_name(&path.join("::")), variant));
                self.indent += 1;
                self.write_indent();
                self.gen_expr(&arm.body);
                self.output.push_str(";\n");
                self.writeln("break;");
                self.indent -= 1;
                self.writeln("}");
            }
            _ => {
                self.writeln("default: {");
                self.indent += 1;
                self.write_indent();
                self.gen_expr(&arm.body);
                self.output.push_str(";\n");
                self.writeln("break;");
                self.indent -= 1;
                self.writeln("}");
            }
        }
    }

    fn gen_expr_str(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Float(f) => format!("{:?}", f),
            Expr::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Expr::Char(c) => format!("'{}'", c),
            Expr::Bool(b) => if *b { "true" } else { "false" }.into(),
            Expr::Null => "nullptr".into(),
            Expr::NullPtr => "nullptr".into(),
            Expr::Self_ => "self".into(),
            Expr::Ident(name) => Self::cpp_name(name),
            Expr::Path(parts) => Self::cpp_name(&parts.join("::")),
            Expr::Binary { op, left, right } => {
                format!("({} {} {})", self.gen_expr_str(left), op, self.gen_expr_str(right))
            }
            Expr::Unary { op, expr } => {
                match op {
                    UnaryOp::Not => format!("!({})", self.gen_expr_str(expr)),
                    UnaryOp::Neg => format!("-({})", self.gen_expr_str(expr)),
                    UnaryOp::Deref => format!("*({})", self.gen_expr_str(expr)),
                }
            }
            Expr::Assign { target, value } => format!("{} = {}", self.gen_expr_str(target), self.gen_expr_str(value)),
            Expr::CompoundAssign { op, target, value } => format!("{} {}= {}", self.gen_expr_str(target), op, self.gen_expr_str(value)),
            Expr::Call { function, args } => {
                let as_: Vec<String> = args.iter().map(|a| self.gen_expr_str(a)).collect();
                format!("{}({})", self.gen_expr_str(function), as_.join(", "))
            }
            Expr::MethodCall { object, method, args } => {
                let as_: Vec<String> = args.iter().map(|a| self.gen_expr_str(a)).collect();
                format!("{}.{}({})", self.gen_expr_str(object), Self::cpp_name(method), as_.join(", "))
            }
            Expr::Field { object, field } => format!("({}).{}", self.gen_expr_str(object), Self::cpp_name(field)),
            Expr::Index { object, index } => format!("({})[{}]", self.gen_expr_str(object), self.gen_expr_str(index)),
            Expr::Reference { expr, .. } => format!("&({})", self.gen_expr_str(expr)),
            Expr::Deref(expr) => format!("*({})", self.gen_expr_str(expr)),
            Expr::Move(expr) => self.gen_expr_str(expr),
            Expr::Array(elements) => {
                let es: Vec<String> = elements.iter().map(|e| self.gen_expr_str(e)).collect();
                format!("std::vector<auto>{{ {} }}", es.join(", "))
            }
            Expr::ArrayRepeat { value, size } => {
                format!("std::vector<auto>({}, {})", self.gen_expr_str(size), self.gen_expr_str(value))
            }
            Expr::Tuple(elements) => {
                let es: Vec<String> = elements.iter().map(|e| self.gen_expr_str(e)).collect();
                format!("std::make_tuple({})", es.join(", "))
            }
            Expr::Range { start, end, inclusive } => {
                let op = if *inclusive { "<=" } else { "<" };
                format!("/* range */ 0")
            }
            Expr::Cast { expr, ty } => format!("static_cast<{}>({})", self.cpp_type_str(ty), self.gen_expr_str(expr)),
            Expr::Macro { name, args } => {
                let as_: Vec<String> = args.iter().map(|a| self.gen_expr_str(a)).collect();
                match name.as_str() {
                    "println" => format!("std::cout << \"{}\" << std::endl", as_.join(" << \" \" << ")),
                    "eprintln" => format!("std::cerr << \"{}\" << std::endl", as_.join(" << \" \" << ")),
                    "format" => format!("fmt::format(\"{}\", {})", "{}", as_.join(", ")),
                    "sizeof" => {
                        if let Some(a) = args.first() {
                            match a {
                                Expr::Ident(n) => format!("sizeof({})", Self::cpp_name(n)),
                                _ => format!("sizeof({})", self.gen_expr_str(a)),
                            }
                        } else { "sizeof(void)".into() }
                    }
                    _ => format!("{}({})", Self::cpp_name(name), as_.join(", ")),
                }
            }
            Expr::StructLiteral { name, fields } => {
                let n = match name.as_ref() {
                    Expr::Ident(n) => Self::cpp_name(n),
                    _ => self.gen_expr_str(name),
                };
                let fs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!(".{} = {}", Self::cpp_name(k), self.gen_expr_str(v)))
                    .collect();
                format!("{}{{ {} }}", n, fs.join(", "))
            }
            Expr::If { condition, then_body, else_body } => {
                let tv = then_body.expr.as_ref().map(|e| self.gen_expr_str(e)).unwrap_or_else(|| "0".into());
                let ev = else_body.as_ref().map(|e| self.gen_expr_str(e)).unwrap_or_else(|| "0".into());
                format!("({} ? {} : {})", self.gen_expr_str(condition), tv, ev)
            }
            Expr::Block(block) => {
                let mut r: String = "([&]() {{ ".into();
                for stmt in &block.stmts { r.push_str(&self.gen_expr_str_stmt(stmt)); r.push_str("; "); }
                if let Some(ref e) = block.expr { r.push_str(&self.gen_expr_str(e)); } else { r.push_str("0"); }
                r.push_str(" }})()");
                r
            }
            Expr::Loop(_) => "/* loop */ 0".into(),
            Expr::Match { arms, .. } => {
                if let Some(arm) = arms.first() { self.gen_expr_str(&arm.body) } else { "0".into() }
            }
            Expr::Closure { params, body, .. } => {
                let ps: Vec<String> = params.iter().map(|p| {
                    let ty = p.ty.as_ref().map(|t| self.cpp_type_str(t)).unwrap_or_else(|| "auto".into());
                    format!("{} {}", ty, Self::cpp_name(&p.name))
                }).collect();
                format!("[=]({}) {{ return {}; }}", ps.join(", "), self.gen_expr_str(body))
            }
            Expr::Try(inner) => format!("({}).value()", self.gen_expr_str(inner)),
            Expr::OptionalChaining { object, field } => {
                format!("({} ? {}->{} : std::nullopt)", self.gen_expr_str(object), self.gen_expr_str(object), Self::cpp_name(field))
            }
            Expr::NullCoalesce { left, right } => {
                format!("({}).value_or({})", self.gen_expr_str(left), self.gen_expr_str(right))
            }
            Expr::Assert { condition, message } => {
                format!("assert({})", self.gen_expr_str(condition))
            }
            Expr::AssertEq { left, right, .. } => {
                format!("assert(({}) == ({}))", self.gen_expr_str(left), self.gen_expr_str(right))
            }
            Expr::AssertNe { left, right, .. } => {
                format!("assert(({}) != ({}))", self.gen_expr_str(left), self.gen_expr_str(right))
            }
            Expr::Sizeof(ty) => format!("sizeof({})", self.cpp_type_str(ty)),
            Expr::Offsetof { struct_type, field } => {
                format!("offsetof({}, {})", Self::cpp_name(struct_type), Self::cpp_name(field))
            }
            Expr::FString(parts) => {
                let mut fmt = String::new();
                let mut args = Vec::new();
                for part in parts {
                    match part {
                        FStringPart::Text(t) => fmt.push_str(t),
                        FStringPart::Expr(e) => { fmt.push_str("{}"); args.push(self.gen_expr_str(e)); }
                    }
                }
                if args.is_empty() { format!("\"{}\"", fmt) }
                else { format!("fmt::format(\"{}\", {})", fmt, args.join(", ")) }
            }
            Expr::VecLit(elements) => {
                let es: Vec<String> = elements.iter().map(|e| self.gen_expr_str(e)).collect();
                format!("std::vector<auto>{{ {} }}", es.join(", "))
            }
            Expr::UnsafeBlock(block) => {
                let mut r = String::from("([&]() {{ ");
                for stmt in &block.stmts { r.push_str(&self.gen_expr_str_stmt(stmt)); r.push_str("; "); }
                if let Some(ref e) = block.expr { r.push_str(&self.gen_expr_str(e)); } else { r.push_str("0"); }
                r.push_str(" }})()");
                r
            }
        }
    }

    fn gen_expr_str_stmt(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Expr(expr) => self.gen_expr_str(expr),
            Stmt::Return(Some(expr)) => format!("return {}", self.gen_expr_str(expr)),
            Stmt::Return(None) => "return".into(),
            _ => "0".into(),
        }
    }

    fn gen_expr(&mut self, expr: &Expr) {
        self.output.push_str(&self.gen_expr_str(expr));
    }

    fn pattern_names(&self, pattern: &Pattern) -> Vec<String> {
        match pattern {
            Pattern::Ident(name) => vec![name.clone()],
            Pattern::Tuple(pats) => {
                let mut names = Vec::new();
                for p in pats { names.extend(self.pattern_names(p)); }
                names
            }
            Pattern::Enum { fields, .. } => {
                let mut names = Vec::new();
                for f in fields { names.extend(self.pattern_names(f)); }
                names
            }
            Pattern::Struct { fields, .. } => {
                let mut names = Vec::new();
                for (_, p) in fields { names.extend(self.pattern_names(p)); }
                names
            }
            Pattern::Mut(name) => vec![name.clone()],
            Pattern::Reference(inner) => self.pattern_names(inner),
            _ => vec![],
        }
    }
}

impl CodeGenerator for CppCodeGen {
    fn generate(&mut self, program: &Program) -> String {
        self.header_writeln("// Generated by the Ruva transpiler — C++ backend — do not edit");
        self.header_writeln("#pragma once");
        self.header_writeln("");

        self.writeln("// Generated by the Ruva transpiler — C++ backend — do not edit");
        self.writeln("");

        self.header_writeln("// Includes");
        { let includes = self.includes.clone(); for inc in &includes { self.header_writeln(inc); } }
        self.header_writeln("");

        for item in &program.items { self.gen_item(item); }

        let mut h = String::with_capacity(4096);
        h.push_str(&self.header);
        if !self.func_decls.is_empty() {
            h.push_str("// Function declarations\n");
            for d in &self.func_decls { h.push_str(d); h.push('\n'); }
        }
        format!("{}\n{}", h, self.output)
    }

    fn target_name(&self) -> &str { "cpp" }
    fn file_extension(&self) -> &str { ".cpp" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn gen_cpp(source: &str) -> String {
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();
        let mut gen = CppCodeGen::new();
        gen.generate(&program)
    }

    #[test]
    fn test_simple_function() {
        let code = gen_cpp("fn add(a: i32, b: i32) -> i32 { return a + b }");
        assert!(code.contains("int32_t"));
        assert!(code.contains("add"));
    }

    #[test]
    fn test_class() {
        let code = gen_cpp(r#"
            class Dog {
                pub let name: string
                pub fn bark(&self) { println!("woof") }
            }
        "#);
        assert!(code.contains("class Dog"));
        assert!(code.contains("std::string name"));
    }

    #[test]
    fn test_trait() {
        let code = gen_cpp("trait Drawable { fn draw(&self) }");
        assert!(code.contains("class Drawable"));
        assert!(code.contains("virtual"));
        assert!(code.contains("= 0"));
    }

    #[test]
    fn test_enum_class() {
        let code = gen_cpp("enum Color { Red, Green, Blue }");
        assert!(code.contains("enum class Color"));
    }

    #[test]
    fn test_constexpr() {
        let code = gen_cpp("const MAX: i32 = 42");
        assert!(code.contains("constexpr"));
    }
}
