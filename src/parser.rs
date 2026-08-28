#[allow(dead_code)]
use crate::ast::*;
use crate::lexer::Lexer;
use anyhow::{bail, Result};

pub struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    /// When false, `Self { ... }` is NOT parsed as a struct literal
    /// (e.g. inside match/if/while discriminant where `{` is the body)
    can_construct: bool,
}

impl Parser {
    pub fn new(source: &str) -> Result<Self> {
        let tokens = Lexer::new(source).tokenize()?;
        Ok(Self { tokens, pos: 0, can_construct: true })
    }

    fn peek(&self) -> &Token {
        &self.tokens.get(self.pos).map(|(t, _)| t).unwrap_or(&Token::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens.get(self.pos).map(|(_, s)| *s).unwrap_or(Span { line: 0, col: 0 })
    }

    fn advance(&mut self) -> (Token, Span) {
        let result = self.tokens.get(self.pos).cloned().unwrap_or((Token::Eof, Span { line: 0, col: 0 }));
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        result
    }

    fn _advance_ref(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos).map(|(t, _)| t);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<Span> {
        let (tok, span) = self.advance();
        if &tok == expected {
            Ok(span)
        } else {
            bail!("Expected {:?}, got {:?} at {}:{}", expected, tok, span.line, span.col);
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span)> {
        let (tok, span) = self.advance();
        match tok {
            Token::Ident(s) => Ok((s, span)),
            _ => bail!("Expected identifier, got {:?} at {}:{}", tok, span.line, span.col),
        }
    }

    fn at(&self, token: &Token) -> bool {
        self.peek() == token
    }

    // ─── Program ─────────────────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.at(&Token::Eof) {
            items.push(self.parse_item()?);
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item> {
        // Check for attributes first
        if self.at(&Token::Hash) {
            return self.parse_attribute();
        }

        // Check for imports
        if self.at(&Token::Import) {
            return Ok(Item::Import(self.parse_import()?));
        }

        // Check for pub
        let is_pub = if self.at(&Token::Pub) {
            self.advance();
            true
        } else {
            false
        };

        // Use declarations (after pub so `pub use` works)
        if self.at(&Token::Use) {
            return Ok(Item::Use(self.parse_use()?));
        }

        // Handle unsafe fn, unsafe impl, extern block
        let is_unsafe = if self.at(&Token::Unsafe) {
            self.advance();
            true
        } else {
            false
        };

        match self.peek() {
            Token::Fn => {
                let mut f = self.parse_function(is_pub)?;
                f.is_unsafe = is_unsafe;
                Ok(Item::Function(f))
            }
            Token::Struct => Ok(Item::Struct(self.parse_struct(is_pub)?)),
            Token::Enum => Ok(Item::Enum(self.parse_enum(is_pub)?)),
            Token::Class => Ok(Item::Class(self.parse_class(is_pub)?)),
            Token::Impl => Ok(Item::Impl(self.parse_impl_block()?)),
            Token::Trait => Ok(Item::Trait(self.parse_trait(is_pub)?)),
            Token::Type => Ok(Item::TypeAlias(self.parse_type_alias(is_pub)?)),
            Token::Const => Ok(Item::Const(self.parse_const_item(is_pub)?)),
            Token::Mod => Ok(Item::Module(self.parse_mod(is_pub)?)),
            Token::Extern => Ok(Item::ExternBlock(self.parse_extern_block()?)),
            Token::Interface => Ok(Item::Interface(self.parse_interface_def(is_pub)?)),
            Token::Package => Ok(Item::Package(self.parse_package()?)),
            Token::Comptime => Ok(Item::Comptime(self.parse_comptime()?)),
            Token::At => {
                // Python-style decorator: @decorator
                let decorators = self.parse_decorators()?;
                let definition = self.parse_item()?;
                Ok(Item::Decorated(DecoratedDef { decorators, definition: Box::new(definition) }))
            }
            Token::Try => {
                // Try/catch as a statement expression
                let try_expr = self.parse_try_catch_expr()?;
                Ok(Item::Function(FunctionDef {
                    is_pub: false, is_test: false, is_unsafe: false,
                    name: "__try_expr__".into(), generics: vec![],
                    params: vec![], return_type: None,
                    body: Block { stmts: vec![], expr: Some(Box::new(try_expr)) },
                    span: self.peek_span(),
                }))
            }
            Token::Throw => {
                let throw_expr = self.parse_throw()?;
                Ok(Item::Function(FunctionDef {
                    is_pub: false, is_test: false, is_unsafe: false,
                    name: "__throw_expr__".into(), generics: vec![],
                    params: vec![], return_type: None,
                    body: Block { stmts: vec![], expr: Some(Box::new(throw_expr)) },
                    span: self.peek_span(),
                }))
            }
            Token::LBrace if is_unsafe => {
                // unsafe { ... } as a top-level item? Not standard, but let's handle gracefully
                bail!("unsafe blocks cannot be top-level items at {}:{}", self.peek_span().line, self.peek_span().col)
            }
            _ if is_unsafe => {
                bail!("Expected function or impl after 'unsafe' at {}:{}", self.peek_span().line, self.peek_span().col)
            }
            _ => {
                let span = self.peek_span();
                bail!("Unexpected token {:?} at {}:{}", self.peek(), span.line, span.col);
            }
        }
    }

    // ─── Imports ─────────────────────────────────────────────────────────

    fn parse_import(&mut self) -> Result<ImportDef> {
        self.expect(&Token::Import)?;

        let mut path = String::new();
        loop {
            let (name, _) = self.expect_ident()?;
            path.push_str(&name);
            if self.at(&Token::DoubleColon) {
                self.advance();
                path.push_str("::");
            } else if self.at(&Token::Dot) {
                self.advance();
                path.push('.');
            } else {
                break;
            }
        }

        // Check for `as alias`
        let alias = if self.at(&Token::As) {
            self.advance();
            Some(self.expect_ident()?.0)
        } else {
            None
        };

        // Optional semicolon
        if self.at(&Token::Semicolon) {
            self.advance();
        }

        Ok(ImportDef { path, alias, items: None })
    }

    // ─── Attributes ──────────────────────────────────────────────────────

    fn parse_attribute(&mut self) -> Result<Item> {
        self.expect(&Token::Hash)?;
        self.expect(&Token::LBracket)?;

        let (name, _) = self.expect_ident()?;

        let mut args = Vec::new();
        if self.at(&Token::LParen) {
            self.advance();
            while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                if let Token::Ident(s) = self.advance().0 {
                    args.push(s);
                }
                if self.at(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(&Token::RParen)?;
        }

        self.expect(&Token::RBracket)?;

        // Parse the item this attribute applies to
        let item = Box::new(self.parse_item()?);

        Ok(Item::Attribute(Attribute { name, args, item }))
    }

    // ─── Functions ───────────────────────────────────────────────────────

    fn parse_function(&mut self, is_pub: bool) -> Result<FunctionDef> {
        self.expect(&Token::Fn)?;

        let is_test = if self.at(&Token::Test) {
            self.advance();
            true
        } else {
            false
        };

        let (name, _) = self.expect_ident()?;

        // Generic parameters
        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        // Parameters
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
            params.push(self.parse_param()?);
            if self.at(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RParen)?;

        // Return type
        let return_type = if self.at(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Where clause
        if self.at(&Token::Where) {
            self.skip_where_clause()?;
        }

        // Body
        let body = self.parse_block()?;

        Ok(FunctionDef {
            is_pub,
            is_test,
            is_unsafe: false,
            name,
            generics,
            params,
            return_type,
            body,
            span: self.peek_span(),
        })
    }

    fn parse_param(&mut self) -> Result<Param> {
        let mut is_ref = false;
        let mut is_mut = false;

        // Check for &self or &mut self
        if self.at(&Token::Amp) {
            is_ref = true;
            self.advance();
        }

        if self.at(&Token::Mut) {
            is_mut = true;
            self.advance();
        }

        // Handle self / Self as a parameter name
        if self.at(&Token::Self_) {
            self.advance();
            return Ok(Param {
                name: "self".into(),
                ty: Type::SelfType,
                is_ref,
                is_mut,
            });
        }
        if self.at(&Token::SelfType) {
            self.advance();
            return Ok(Param {
                name: "self".into(),
                ty: Type::SelfType,
                is_ref,
                is_mut,
            });
        }

        let (name, _) = self.expect_ident()?;

        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;

        Ok(Param {
            name,
            ty,
            is_ref,
            is_mut,
        })
    }

    // ─── Struct ──────────────────────────────────────────────────────────

    fn parse_struct(&mut self, is_pub: bool) -> Result<StructDef> {
        self.expect(&Token::Struct)?;
        let (name, _) = self.expect_ident()?;

        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let derives = Vec::new();

        // Unit struct: struct Foo;
        if self.at(&Token::Semicolon) {
            self.advance();
            return Ok(StructDef {
                is_pub,
                name,
                generics,
                fields: Vec::new(),
                derives,
                span: self.peek_span(),
            });
        }

        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            // Check for pub
            let field_pub = if self.at(&Token::Pub) {
                self.advance();
                true
            } else {
                false
            };

            let (field_name, _) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;

            // Optional comma
            if self.at(&Token::Comma) {
                self.advance();
            }

            fields.push(FieldDef {
                is_pub: field_pub,
                name: field_name,
                ty,
            });
        }

        self.expect(&Token::RBrace)?;

        Ok(StructDef {
            is_pub,
            name,
            generics,
            fields,
            derives,
            span: self.peek_span(),
        })
    }

    // ─── Enum ────────────────────────────────────────────────────────────

    fn parse_enum(&mut self, is_pub: bool) -> Result<EnumDef> {
        self.expect(&Token::Enum)?;
        let (name, _) = self.expect_ident()?;
        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        self.expect(&Token::LBrace)?;
        let mut variants = Vec::new();

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            let (variant_name, _) = self.expect_ident()?;
            let fields = if self.at(&Token::LParen) {
                self.advance();
                let mut fields = Vec::new();
                while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                    fields.push(self.parse_type()?);
                    if self.at(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::RParen)?;
                fields
            } else if self.at(&Token::LBrace) {
                // Struct variant — parse field types for proper codegen
                self.advance();
                let mut fields = Vec::new();
                while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
                    let (_field_name, _) = self.expect_ident()?;
                    self.expect(&Token::Colon)?;
                    let ty = self.parse_type()?;
                    fields.push(ty);
                    if self.at(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::RBrace)?;
                fields
            } else {
                Vec::new()
            };
            if self.at(&Token::Comma) { self.advance(); }
            variants.push(EnumVariant { name: variant_name, fields });
        }
        self.expect(&Token::RBrace)?;

        Ok(EnumDef {
            is_pub,
            name,
            generics,
            variants,
            span: self.peek_span(),
        })
    }

    // ─── Class ───────────────────────────────────────────────────────────

    fn parse_class(&mut self, is_pub: bool) -> Result<ClassDef> {
        self.expect(&Token::Class)?;
        let (name, _) = self.expect_ident()?;

        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let derives = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        self.expect(&Token::LBrace)?;

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            // Check for pub
            let field_pub = if self.at(&Token::Pub) {
                self.advance();
                true
            } else {
                false
            };

            // Check for let (field declaration)
            if self.at(&Token::Let) {
                self.advance();

                let field_mut = if self.at(&Token::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };

                let (field_name, _) = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let ty = self.parse_type()?;

                if self.at(&Token::Comma) {
                    self.advance();
                }

                fields.push(ClassField {
                    is_pub: field_pub,
                    is_mut: field_mut,
                    name: field_name,
                    ty,
                });
            }
            // Check for fn (method)
            else if self.at(&Token::Fn) || (field_pub && self.at(&Token::Fn)) {
                // We already consumed `pub` if present
                let method = self.parse_function(true)?;
                methods.push(method);
            }
            // Handle bare pub fn
            else if field_pub && !self.at(&Token::Let) {
                // We consumed `pub` but next is neither `let` nor `fn`
                // Push it as a method if fn follows
                if self.at(&Token::Fn) {
                    let method = self.parse_function(true)?;
                    methods.push(method);
                } else {
                    bail!("Expected 'let' or 'fn' after 'pub' in class at {}", self.peek_span().line);
                }
            }
            else {
                let span = self.peek_span();
                bail!("Unexpected token {:?} in class body at {}:{}", self.peek(), span.line, span.col);
            }
        }

        self.expect(&Token::RBrace)?;

        Ok(ClassDef {
            is_pub,
            name,
            generics,
            fields,
            methods,
            derives,
            span: self.peek_span(),
        })
    }

    // ─── Impl Block ──────────────────────────────────────────────────────

    fn parse_impl_block(&mut self) -> Result<ImplBlock> {
        self.expect(&Token::Impl)?;

        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        // Parse the type being implemented
        let first_type = self.parse_type()?;

        // Check for `impl Trait for Type` syntax
        let (self_type, trait_name) = if self.at(&Token::For) {
            // first_type is the trait, next type is the self type
            self.advance();
            let self_type = self.parse_type()?;
            (self_type, Some(first_type))
        } else {
            (first_type, None)
        };

        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            let is_pub = if self.at(&Token::Pub) {
                self.advance();
                true
            } else {
                false
            };
            methods.push(self.parse_function(is_pub)?);
        }

        self.expect(&Token::RBrace)?;

        Ok(ImplBlock {
            generics,
            self_type,
            trait_name,
            methods,
            span: self.peek_span(),
        })
    }

    // ─── Trait ───────────────────────────────────────────────────────────

    fn parse_trait(&mut self, is_pub: bool) -> Result<TraitDef> {
        self.expect(&Token::Trait)?;
        let (name, _) = self.expect_ident()?;

        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            self.expect(&Token::Fn)?;
            let (method_name, _) = self.expect_ident()?;

            self.expect(&Token::LParen)?;
            let mut params = Vec::new();
            while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                params.push(self.parse_param()?);
                if self.at(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(&Token::RParen)?;

            let return_type = if self.at(&Token::Arrow) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };

            // Optional default body
            let default_body = if self.at(&Token::LBrace) {
                Some(self.parse_block()?)
            } else {
                // Expect semicolon for required methods
                if self.at(&Token::Semicolon) {
                    self.advance();
                }
                None
            };

            methods.push(TraitMethod {
                name: method_name,
                params,
                return_type,
                default_body,
            });
        }

        self.expect(&Token::RBrace)?;

        Ok(TraitDef {
            is_pub,
            name,
            generics,
            methods,
            span: self.peek_span(),
        })
    }

    // ─── Type Alias ──────────────────────────────────────────────────────

    fn parse_const_item(&mut self, is_pub: bool) -> Result<ConstDef> {
        self.expect(&Token::Const)?;
        let (name, span) = self.expect_ident()?;
        let ty = if self.at(&Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        if self.at(&Token::Semicolon) { self.advance(); }
        Ok(ConstDef { is_pub, name, ty, value, span })
    }

    fn parse_type_alias(&mut self, is_pub: bool) -> Result<TypeAliasDef> {
        self.expect(&Token::Type)?;
        let (name, _) = self.expect_ident()?;

        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(&Token::Eq)?;
        let ty = self.parse_type()?;

        if self.at(&Token::Semicolon) {
            self.advance();
        }

        Ok(TypeAliasDef {
            is_pub,
            name,
            generics,
            ty,
        })
    }

    // ─── Module ──────────────────────────────────────────────────────────

    fn parse_mod(&mut self, is_pub: bool) -> Result<ModDef> {
        self.expect(&Token::Mod)?;
        let (name, _) = self.expect_ident()?;

        // Inline module: mod name { ... }
        if self.at(&Token::LBrace) {
            self.advance();
            let mut items = Vec::new();
            while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
                items.push(self.parse_item()?);
            }
            self.expect(&Token::RBrace)?;
            return Ok(ModDef { is_pub, name, body: Some(items) });
        }

        // File-based module: mod name;
        if self.at(&Token::Semicolon) {
            self.advance();
        }
        Ok(ModDef { is_pub, name, body: None })
    }

    // ─── Extern Blocks ──────────────────────────────────────────────────────

    fn parse_extern_block(&mut self) -> Result<ExternBlock> {
        self.expect(&Token::Extern)?;

        // Parse ABI string: extern "C" or just extern
        let abi = if matches!(self.peek(), Token::Str(_)) {
            if let Token::Str(s) = self.advance().0 {
                s
            } else {
                "C".to_string()
            }
        } else {
            "C".to_string()
        };

        self.expect(&Token::LBrace)?;

        let mut items = Vec::new();
        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            items.push(self.parse_extern_item()?);
        }

        self.expect(&Token::RBrace)?;
        Ok(ExternBlock { abi, items })
    }

    fn parse_extern_item(&mut self) -> Result<ExternItem> {
        let is_pub = if self.at(&Token::Pub) {
            self.advance();
            true
        } else {
            false
        };

        if self.at(&Token::Fn) {
            self.advance();
            let (name, _) = self.expect_ident()?;
            self.expect(&Token::LParen)?;
            let mut params = Vec::new();
            while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                params.push(self.parse_param()?);
                if self.at(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(&Token::RParen)?;
            let return_type = if self.at(&Token::Arrow) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            if self.at(&Token::Semicolon) { self.advance(); }
            return Ok(ExternItem::Function { is_pub, name, params, return_type });
        }

        if self.at(&Token::Static) || self.at(&Token::Const) {
            let is_const = self.at(&Token::Const);
            self.advance();
            let is_mut_static = if !is_const && self.at(&Token::Mut) {
                self.advance();
                true
            } else {
                false
            };
            let (name, _) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            let value = if self.at(&Token::Eq) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            if self.at(&Token::Semicolon) { self.advance(); }
            if is_const {
                return Ok(ExternItem::Const { is_pub, name, ty, value });
            } else {
                return Ok(ExternItem::Static { is_pub, is_mut: is_mut_static, name, ty });
            }
        }

        bail!("Expected function or static in extern block at {}:{}", self.peek_span().line, self.peek_span().col)
    }

    // ─── Use Declarations ──────────────────────────────────────────────────

    fn parse_use(&mut self) -> Result<UseDef> {
        self.expect(&Token::Use)?;

        // Parse the path segments
        let mut path = Vec::new();
        let first = self.expect_ident()?.0;
        path.push(first);

        while self.at(&Token::DoubleColon) {
            self.advance();
            // Check for wildcard: use foo::*
            if self.at(&Token::Star) {
                self.advance();
                if self.at(&Token::Semicolon) { self.advance(); }
                return Ok(UseDef {
                    path,
                    alias: None,
                    selective: Vec::new(),
                    wildcard: true,
                });
            }
            // Check for selective: use foo::{A, B, C as D}
            if self.at(&Token::LBrace) {
                self.advance();
                let mut selective = Vec::new();
                while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
                    let (item_name, _) = self.expect_ident()?;
                    let item_alias = if self.at(&Token::As) {
                        self.advance();
                        Some(self.expect_ident()?.0)
                    } else {
                        None
                    };
                    selective.push(UseItem { name: item_name, alias: item_alias });
                    if self.at(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::RBrace)?;
                if self.at(&Token::Semicolon) { self.advance(); }
                return Ok(UseDef {
                    path,
                    alias: None,
                    selective,
                    wildcard: false,
                });
            }
            // Normal path segment
            let (seg, _) = self.expect_ident()?;
            path.push(seg);
        }

        // Check for alias: use foo as bar
        let alias = if self.at(&Token::As) {
            self.advance();
            Some(self.expect_ident()?.0)
        } else {
            None
        };

        if self.at(&Token::Semicolon) { self.advance(); }

        Ok(UseDef {
            path,
            alias,
            selective: Vec::new(),
            wildcard: false,
        })
    }

    // ─── Generic Parameters ──────────────────────────────────────────────

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>> {
        self.expect(&Token::Lt)?;
        let mut params = Vec::new();

        while !self.at(&Token::Gt) && !self.at(&Token::Eof) {
            let (name, _) = self.expect_ident()?;

            let bounds = if self.at(&Token::Colon) {
                self.advance();
                let mut bounds = Vec::new();
                bounds.push(self.parse_type()?);
                while self.at(&Token::Plus) {
                    self.advance();
                    bounds.push(self.parse_type()?);
                }
                bounds
            } else {
                Vec::new()
            };

            if self.at(&Token::Comma) {
                self.advance();
            }

            params.push(GenericParam { name, bounds });
        }

        self.expect(&Token::Gt)?;
        Ok(params)
    }

    fn skip_where_clause(&mut self) -> Result<()> {
        self.expect(&Token::Where)?;
        loop {
            // Skip type + bounds
            self.parse_type()?;
            if self.at(&Token::Colon) {
                self.advance();
                loop {
                    self.parse_type()?;
                    if self.at(&Token::Plus) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            if self.at(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(())
    }

    // ─── Types ───────────────────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<Type> {
        let base = match self.peek().clone() {
            Token::Star => {
                // Raw pointer: *const T or *mut T
                self.advance();
                let is_mut = if self.at(&Token::Mut) {
                    self.advance();
                    true
                } else {
                    if self.at(&Token::Const) {
                        self.advance();
                    }
                    false
                };
                let inner = self.parse_type()?;
                Type::RawPointer {
                    inner: Box::new(inner),
                    is_mut,
                }
            }
            Token::Amp => {
                self.advance();
                let is_mut = if self.at(&Token::Mut) { self.advance(); true } else { false };
                let inner = self.parse_type()?;
                Type::Reference {
                    inner: Box::new(inner),
                    is_mut,
                }
            }
            Token::LBracket => {
                self.advance();
                if self.at(&Token::RBracket) {
                    // &[T] slice
                    self.advance();
                    let inner = self.parse_type()?;
                    Type::Slice(Box::new(inner))
                } else {
                    let inner = self.parse_type()?;
                    if self.at(&Token::Semicolon) {
                        self.advance();
                        let size = self.parse_expr()?;
                        self.expect(&Token::RBracket)?;
                        Type::Array {
                            inner: Box::new(inner),
                            size: Some(Box::new(size)),
                        }
                    } else {
                        self.expect(&Token::RBracket)?;
                        Type::Slice(Box::new(inner))
                    }
                }
            }
            Token::LParen => {
                self.advance();
                let mut types = Vec::new();
                while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                    types.push(self.parse_type()?);
                    if self.at(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(&Token::RParen)?;
                if types.len() == 1 {
                    types.remove(0) // Just (T), not a tuple
                } else {
                    Type::Tuple(types)
                }
            }
            Token::Fn => {
                self.advance();
                self.expect(&Token::LParen)?;
                let mut params = Vec::new();
                while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                    params.push(self.parse_type()?);
                    if self.at(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(&Token::RParen)?;
                self.expect(&Token::Arrow)?;
                let ret = self.parse_type()?;
                Type::Function {
                    params,
                    return_type: Box::new(ret),
                }
            }
            Token::SelfType => {
                self.advance();
                Type::SelfType
            }
            Token::Impl => {
                // impl Trait in argument/return position
                self.advance();
                let (trait_name, _) = self.expect_ident()?;
                Type::Name(format!("impl {}", trait_name))
            }
            Token::Ident(_) => {
                let (name, _) = self.expect_ident()?;

                // Check for generic arguments: Vec<T>, Option<T>, etc.
                if self.at(&Token::Lt) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.at(&Token::Gt) && !self.at(&Token::Eof) {
                        args.push(self.parse_type()?);
                        if self.at(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(&Token::Gt)?;
                    Type::Generic { name, args }
                } else if self.at(&Token::DoubleColon) {
                    // Path: std::io::Error
                    let mut path = vec![name];
                    while self.at(&Token::DoubleColon) {
                        self.advance();
                        let (seg, _) = self.expect_ident()?;
                        path.push(seg);
                        // Check for generic args on the last segment
                        if self.at(&Token::Lt) {
                            self.advance();
                            let mut args = Vec::new();
                            while !self.at(&Token::Gt) && !self.at(&Token::Eof) {
                                args.push(self.parse_type()?);
                                if self.at(&Token::Comma) {
                                    self.advance();
                                }
                            }
                            self.expect(&Token::Gt)?;
                            // Return Generic type with full path as name
                            let full_name = path.join("::");
                            return Ok(Type::Generic { name: full_name, args });
                        }
                    }
                    // If we didn't break with generic args
                    Type::Path(path)
                } else {
                    // Check for special types
                    match name.as_str() {
                        "string" => Type::Name("String".into()),
                        "bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                        | "f32" | "f64" | "char" => Type::Name(name),
                        _ => Type::Name(name),
                    }
                }
            }
            _ => {
                let span = self.peek_span();
                bail!("Expected type, got {:?} at {}:{}", self.peek(), span.line, span.col);
            }
        };

        // Check for function type: fn(A, B) -> C
        // Already handled above in the match

        Ok(base)
    }

    // ─── Blocks ──────────────────────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Block> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        self.expect(&Token::RBrace)?;

        Ok(Block { stmts, expr: None })
    }

    fn is_expr_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Int(_) | Token::Float(_) | Token::Str(_) | Token::Char(_)
            | Token::Bool(_) | Token::Null | Token::Ident(_) | Token::Self_ | Token::SelfType
            | Token::FStringStart
            | Token::LParen | Token::LBracket | Token::LBrace
            |            Token::Not | Token::Amp | Token::Star | Token::Minus | Token::Pipe
            | Token::If | Token::Match | Token::Loop
            | Token::Fn | Token::Move
        )
    }

    // ─── Statements ──────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek().clone() {
            Token::Let => self.parse_let_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::If => self.parse_if_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::Loop => self.parse_loop_stmt(),
            Token::Break => {
                self.advance();
                let expr = if self.is_expr_start() {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                if self.at(&Token::Semicolon) { self.advance(); }
                Ok(Stmt::Break(expr))
            }
            Token::Continue => {
                self.advance();
                if self.at(&Token::Semicolon) { self.advance(); }
                Ok(Stmt::Continue)
            }
            Token::Match => self.parse_match_stmt(),
            Token::LBrace => Ok(Stmt::Block(self.parse_block()?)),
            Token::Unsafe => {
                self.advance();
                Ok(Stmt::Unsafe(self.parse_block()?))
            }
            _ => {
                // Try-catch
                if let Token::Ident(ref s) = self.peek() {
                    if s == "try" {
                        self.advance();
                        return self.parse_try_catch();
                    }
                }
                // Expression statement or compound assignment
                let expr = self.parse_expr()?;
                match self.peek() {
                    Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq
                    | Token::AmpEq | Token::PipeEq | Token::CaretEq => {
                        let op = match self.peek() {
                            Token::PlusEq => BinOp::Add,
                            Token::MinusEq => BinOp::Sub,
                            Token::StarEq => BinOp::Mul,
                            Token::SlashEq => BinOp::Div,
                            Token::AmpEq => BinOp::BitAnd,
                            Token::PipeEq => BinOp::BitOr,
                            Token::CaretEq => BinOp::BitXor,
                            _ => unreachable!(),
                        };
                        self.advance();
                        let value = self.parse_expr()?;
                        if self.at(&Token::Semicolon) { self.advance(); }
                        Ok(Stmt::Expr(Expr::CompoundAssign { op, target: Box::new(expr), value: Box::new(value) }))
                    }
                    Token::Eq => {
                        self.advance();
                        let value = self.parse_expr()?;
                        if self.at(&Token::Semicolon) { self.advance(); }
                        Ok(Stmt::Expr(Expr::Assign { target: Box::new(expr), value: Box::new(value) }))
                    }
                    _ => {
                        if self.at(&Token::Semicolon) { self.advance(); }
                        Ok(Stmt::Expr(expr))
                    }
                }
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::Let)?;
        let is_mut = if self.at(&Token::Mut) { self.advance(); true } else { false };
        // Parse pattern: x, (a, b), _, etc.
        let pattern = self.parse_pattern()?;
        let ty = if self.at(&Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        if self.at(&Token::Semicolon) { self.advance(); }
        Ok(Stmt::Let { pattern, ty, is_mut, value })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::Return)?;
        let expr = if self.is_expr_start() {
            Some(self.parse_expr()?)
        } else {
            None
        };
        if self.at(&Token::Semicolon) { self.advance(); }
        Ok(Stmt::Return(expr))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::If)?;

        // Check for `if let` pattern
        if self.at(&Token::Let) {
            self.advance();
            let pattern = self.parse_pattern()?;
            self.expect(&Token::Eq)?;
            let saved = self.can_construct;
            self.can_construct = false;
            let value = self.parse_expr()?;
            self.can_construct = saved;
            let then_body = self.parse_block()?;
            let else_body = if self.at(&Token::Else) {
                self.advance();
                if self.at(&Token::If) {
                    let else_if = self.parse_if_stmt()?;
                    match else_if {
                        Stmt::If { condition, then_body, else_body: _ } => {
                            Some(ElseKind::If(condition, then_body))
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Some(ElseKind::Else(self.parse_block()?))
                }
            } else {
                None
            };
            // Generate match { pattern => then_body, _ => () }
            let mut arms = vec![
                MatchArm::new(pattern, None, Expr::Block(then_body)),
            ];
            // Add else branch or wildcard
            if let Some(ElseKind::Else(else_block)) = else_body {
                arms.push(MatchArm::new(
                    Pattern::Wildcard,
                    None,
                    Expr::Block(else_block),
                ));
            } else {
                arms.push(MatchArm::new(
                    Pattern::Wildcard,
                    None,
                    Expr::Tuple(Vec::new()),
                ));
            }
            let match_expr = Expr::Match {
                expr: Box::new(value),
                arms,
            };
            return Ok(Stmt::Expr(match_expr));
        }

        let saved = self.can_construct;
        self.can_construct = false;
        let condition = self.parse_expr()?;
        self.can_construct = saved;
        let then_body = self.parse_block()?;
        let else_body = if self.at(&Token::Else) {
            self.advance();
            if self.at(&Token::If) {
                let else_if = self.parse_if_stmt()?;
                match else_if {
                    Stmt::If { condition, then_body, else_body: _ } => {
                        Some(ElseKind::If(condition, then_body))
                    }
                    _ => unreachable!(),
                }
            } else {
                Some(ElseKind::Else(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(Stmt::If { condition, then_body, else_body })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::For)?;
        let pattern = self.parse_pattern()?;
        self.expect(&Token::In)?;
        let saved = self.can_construct;
        self.can_construct = false;
        let iterable = self.parse_expr()?;
        self.can_construct = saved;
        let body = self.parse_block()?;
        Ok(Stmt::For { pattern, iterable, body })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::While)?;

        // while let pattern = expr { body }
        if self.at(&Token::Let) {
            self.advance();
            let pattern = self.parse_pattern()?;
            self.expect(&Token::Eq)?;
            let saved = self.can_construct;
            self.can_construct = false;
            let value = self.parse_expr()?;
            self.can_construct = saved;
            let body = self.parse_block()?;
            return Ok(Stmt::WhileLet { pattern, value, body });
        }

        // while expr { body }
        let saved = self.can_construct;
        self.can_construct = false;
        let condition = self.parse_expr()?;
        self.can_construct = saved;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_loop_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::Loop)?;
        let body = self.parse_block()?;
        Ok(Stmt::Loop(body))
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt> {
        self.expect(&Token::Match)?;
        let saved = self.can_construct;
        self.can_construct = false;
        let expr = self.parse_expr()?;
        self.can_construct = saved;
        self.expect(&Token::LBrace)?;
        let mut arms = Vec::new();

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            // Parse or-pattern alternatives: A | B | C
            let mut alternatives = vec![self.parse_pattern()?];
            while self.at(&Token::Pipe) {
                self.advance();
                alternatives.push(self.parse_pattern()?);
            }
            let pattern = if alternatives.len() > 1 {
                Pattern::Or(alternatives)
            } else {
                alternatives.into_iter().next().unwrap()
            };
            let guard = if self.at(&Token::If) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&Token::FatArrow)?;
            let body = self.parse_expr()?;
            if self.at(&Token::Comma) { self.advance(); }
            arms.push(MatchArm::new(pattern, guard, body));
        }

        self.expect(&Token::RBrace)?;
        Ok(Stmt::Match { expr, arms })
    }

    fn parse_try_catch(&mut self) -> Result<Stmt> {
        let try_body = self.parse_block()?;
        self.expect(&Token::Catch)?;
        self.expect(&Token::LParen)?;
        let (catch_param, _) = self.expect_ident()?;
        self.expect(&Token::RParen)?;
        let catch_body = self.parse_block()?;
        Ok(Stmt::TryCatch { try_body, catch_param, catch_body })
    }

    // ─── Patterns ────────────────────────────────────────────────────────

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.peek().clone() {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::Mut => {
                self.advance();
                let (name, _) = self.expect_ident()?;
                Ok(Pattern::Mut(name))
            }
            Token::Int(_) | Token::Float(_) | Token::Str(_) | Token::Char(_) | Token::Bool(_) => {
                Ok(Pattern::Literal(self.parse_expr()?))
            }
            Token::Ident(_) => {
                // Could be a binding or an enum variant
                let (name, _) = self.expect_ident()?;
                if self.at(&Token::DoubleColon) {
                    // Enum variant pattern
                    let mut path = vec![name];
                    while self.at(&Token::DoubleColon) {
                        self.advance();
                        let (seg, _) = self.expect_ident()?;
                        path.push(seg);
                    }
                    let fields = if self.at(&Token::LParen) {
                        self.advance();
                        let mut fields = Vec::new();
                        while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                            fields.push(self.parse_pattern()?);
                            if self.at(&Token::Comma) { self.advance(); }
                        }
                        self.expect(&Token::RParen)?;
                        fields
                    } else if self.at(&Token::LBrace) {
                        // Struct variant: Variant { field: pattern, .. }
                        self.advance();
                        let mut struct_fields = Vec::new();
                        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
                            if self.at(&Token::DotDot) {
                                self.advance(); // skip rest pattern
                            } else {
                                let (fname, _) = self.expect_ident()?;
                                if self.at(&Token::Colon) {
                                    self.advance();
                                    let fp = self.parse_pattern()?;
                                    struct_fields.push((fname, fp));
                                } else {
                                    struct_fields.push((fname.clone(), Pattern::Ident(fname)));
                                }
                            }
                            if self.at(&Token::Comma) { self.advance(); }
                        }
                        self.expect(&Token::RBrace)?;
                        // Convert struct fields to positional patterns for enum representation
                        struct_fields.into_iter().map(|(_, p)| p).collect()
                    } else {
                        Vec::new()
                    };
                    Ok(Pattern::Enum { path, fields })
                } else if self.at(&Token::LParen) {
                    // Enum variant without path: Some(val), Ok(x), etc.
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                        fields.push(self.parse_pattern()?);
                        if self.at(&Token::Comma) { self.advance(); }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Pattern::Enum { path: vec![name], fields })
                } else {
                    // Binding
                    Ok(Pattern::Ident(name))
                }
            }
            Token::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                    patterns.push(self.parse_pattern()?);
                    if self.at(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::RParen)?;
                Ok(Pattern::Tuple(patterns))
            }
            Token::Amp => {
                self.advance();
                let inner = self.parse_pattern()?;
                Ok(Pattern::Reference(Box::new(inner)))
            }
            _ => {
                let span = self.peek_span();
                bail!("Expected pattern, got {:?} at {}:{}", self.peek(), span.line, span.col);
            }
        }
    }

    // ─── Expressions ─────────────────────────────────────────────────────

    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_expr_bp(0)
    }

    /// Pratt parser with precedence climbing
    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr> {
        let mut lhs = self.parse_unary()?;

        // Range expressions: `start..end` and `start..=end`
        // Lowest precedence — parsed before all binary operators.
        if min_bp == 0 && (self.at(&Token::DotDot) || self.at(&Token::DotDotEq)) {
            let inclusive = self.at(&Token::DotDotEq);
            self.advance();
            let rhs = self.parse_expr_bp(1)?; // right side binds higher than range itself
            lhs = Expr::Range {
                start: Box::new(lhs),
                end: Box::new(rhs),
                inclusive,
            };
        }

        loop {
            // Check for `as` type cast (higher precedence than binary ops)
            if self.at(&Token::As) {
                self.advance();
                let ty = self.parse_type()?;
                lhs = Expr::Cast {
                    expr: Box::new(lhs),
                    ty,
                };
                continue;
            }

            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Rem,
                Token::EqEq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::Le => BinOp::Le,
                Token::Ge => BinOp::Ge,
                Token::And => BinOp::And,
                Token::Or => BinOp::Or,
                Token::Amp => BinOp::BitAnd,
                Token::Pipe => BinOp::BitOr,
                Token::Caret => BinOp::BitXor,
                Token::Shl => BinOp::Shl,
                Token::Shr => BinOp::Shr,
                _ => break,
            };

            let bp = self.binop_bp(&op);
            if bp.0 < min_bp {
                break;
            }

            self.advance();
            let rhs = self.parse_expr_bp(bp.1)?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn binop_bp(&self, op: &BinOp) -> (u8, u8) {
        match op {
            BinOp::Or => (1, 2),
            BinOp::And => (3, 4),
            BinOp::Eq | BinOp::Ne => (5, 6),
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => (7, 8),
            BinOp::BitOr => (9, 10),
            BinOp::BitXor => (11, 12),
            BinOp::BitAnd => (13, 14),
            BinOp::Shl | BinOp::Shr => (15, 16),
            BinOp::Add | BinOp::Sub => (17, 18),
            BinOp::Mul | BinOp::Div | BinOp::Rem => (19, 20),
        }
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnaryOp::Not, expr: Box::new(expr) })
            }
            Token::Amp => {
                self.advance();
                let is_mut = if self.at(&Token::Mut) { self.advance(); true } else { false };
                let expr = self.parse_unary()?;
                Ok(Expr::Reference { expr: Box::new(expr), is_mut })
            }
            Token::Star => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Deref(Box::new(expr)))
            }
            Token::Move => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Move(Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_atom()?;

        loop {
            match self.peek() {
                Token::Dot => {
                    self.advance();
                    // Numeric tuple field access: point.0, point.1
                    let field_name = if let Token::Int(n) = self.peek().clone() {
                        self.advance();
                        n.to_string()
                    } else {
                        let (name, _) = self.expect_ident()?;
                        name
                    };

                    // Check if this is a method call: obj.method(args)
                    if self.at(&Token::LParen) {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: field_name,
                            args,
                        };
                    } else {
                        expr = Expr::Field {
                            object: Box::new(expr),
                            field: field_name,
                        };
                    }
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::LBrace if (matches!(&expr, Expr::Path(_))
                    || matches!(&expr, Expr::Ident(ref n) if n.chars().next().map_or(false, |c| c.is_uppercase())))
                    && self.can_construct => {
                    // Qualified struct literal: Shape::Variant { fields }
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
                        let (field_name, _) = self.expect_ident()?;
                        if self.at(&Token::Colon) {
                            self.advance();
                            let value = self.parse_expr()?;
                            fields.push((field_name, value));
                        } else {
                            let name = field_name.clone();
                            fields.push((field_name, Expr::Ident(name)));
                        }
                        if self.at(&Token::Comma) { self.advance(); }
                    }
                    self.expect(&Token::RBrace)?;
                    expr = Expr::StructLiteral {
                        name: Box::new(expr),
                        fields,
                    };
                }
                Token::DoubleColon => {
                    // Static call / associated method: Self::new(...), Type::method(...)
                    // Fold into a Path so the normal call handling takes over.
                    self.advance();
                    let (seg, _) = self.expect_ident()?;
                    let mut parts = match &expr {
                        Expr::Self_ => vec!["Self".to_string()],
                        Expr::Ident(n) => vec![n.clone()],
                        Expr::Path(p) => p.clone(),
                        _ => bail!("Unexpected '::' after expression at {}:{}",
                            self.peek_span().line, self.peek_span().col),
                    };
                    parts.push(seg);
                    expr = Expr::Path(parts);
                }
                Token::LParen => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    expr = Expr::Call {
                        function: Box::new(expr),
                        args,
                    };
                }
                Token::Question => {
                    self.advance();
                    expr = Expr::Try(Box::new(expr));
                }
                Token::QuestionDot => {
                    self.advance();
                    let (field, _) = self.expect_ident()?;
                    expr = Expr::OptionalChaining {
                        object: Box::new(expr),
                        field,
                    };
                }
                Token::NullCoalesce => {
                    self.advance();
                    let right = self.parse_expr()?;
                    expr = Expr::NullCoalesce {
                        left: Box::new(expr),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        if !self.at(&Token::RParen) {
            args.push(self.parse_expr()?);
            while self.at(&Token::Comma) {
                self.advance();
                if self.at(&Token::RParen) { break; }
                args.push(self.parse_expr()?);
            }
        }
        Ok(args)
    }

    fn parse_atom(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Expr::Float(f))
            }
            Token::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            Token::FStringStart => {
                self.advance(); // skip FStringStart
                let mut parts = Vec::new();
                while !self.at(&Token::FStringEnd) && !self.at(&Token::Eof) {
                    match self.peek().clone() {
                        Token::FStringPart(text) => {
                            self.advance();
                            parts.push(FStringPart::Text(text));
                        }
                        Token::FStringExpr => {
                            self.advance(); // skip FStringExpr marker
                            let expr = self.parse_expr()?;
                            parts.push(FStringPart::Expr(expr));
                            self.expect(&Token::RBrace)?; // consume closing brace
                        }
                        _ => break,
                    }
                }
                self.expect(&Token::FStringEnd)?;
                Ok(Expr::FString(parts))
            }
            Token::Char(c) => {
                self.advance();
                Ok(Expr::Char(c))
            }
            Token::Bool(b) => {
                self.advance();
                Ok(Expr::Bool(b))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            Token::Self_ | Token::SelfType => {
                self.advance();
                // Parse Self { x, y } as struct literal ONLY when constructing
                if self.can_construct && self.at(&Token::LBrace) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
                        let (field_name, _) = self.expect_ident()?;
                        if self.at(&Token::Colon) {
                            self.advance();
                            let value = self.parse_expr()?;
                            fields.push((field_name, value));
                        } else {
                            let name = field_name.clone();
                            fields.push((field_name, Expr::Ident(name)));
                        }
                        if self.at(&Token::Comma) { self.advance(); }
                    }
                    self.expect(&Token::RBrace)?;
                    return Ok(Expr::StructLiteral {
                        name: Box::new(Expr::Self_),
                        fields,
                    });
                }
                Ok(Expr::Self_)
            }
            Token::Ident(name) => {
                self.advance();

                // Check for :: path continuation
                if self.at(&Token::DoubleColon) {
                    let mut path = vec![name];
                    while self.at(&Token::DoubleColon) {
                        self.advance();
                        let (seg, _) = self.expect_ident()?;
                        path.push(seg);
                    }
                    return Ok(Expr::Path(path));
                }

                // Check for closure: |x, y| x + y
                // Already handled by match — not in ident

                // Macro invocation: println!, vec!, etc.
                if self.at(&Token::Not) {
                    self.advance();
                    let macro_args = if self.at(&Token::LParen) || self.at(&Token::LBrace) || self.at(&Token::LBracket) {
                        let delim = self.peek().clone();
                        match delim {
                            Token::LParen => {
                                self.advance();
                                let args = self.parse_args()?;
                                self.expect(&Token::RParen)?;
                                args
                            }
                            Token::LBrace => {
                                self.advance();
                                let mut args = Vec::new();
                                if !self.at(&Token::RBrace) {
                                    args.push(self.parse_expr()?);
                                    while self.at(&Token::Comma) {
                                        self.advance();
                                        if self.at(&Token::RBrace) { break; }
                                        args.push(self.parse_expr()?);
                                    }
                                }
                                self.expect(&Token::RBrace)?;
                                args
                            }
                            Token::LBracket => {
                                self.advance();
                                let mut args = Vec::new();
                                if !self.at(&Token::RBracket) {
                                    args.push(self.parse_expr()?);
                                    while self.at(&Token::Comma) {
                                        self.advance();
                                        if self.at(&Token::RBracket) { break; }
                                        args.push(self.parse_expr()?);
                                    }
                                }
                                self.expect(&Token::RBracket)?;
                                args
                            }
                            _ => Vec::new(),
                        }
                    } else {
                        Vec::new()
                    };
                    return Ok(Expr::Macro { name, args: macro_args });
                }

                // Special built-in expressions
                match name.as_str() {
                    "sizeof" => {
                        self.expect(&Token::LParen)?;
                        let ty = self.parse_type()?;
                        self.expect(&Token::RParen)?;
                        return Ok(Expr::Sizeof(ty));
                    }
                    "offsetof" => {
                        self.expect(&Token::LParen)?;
                        let (struct_type, _) = self.expect_ident()?;
                        self.expect(&Token::Comma)?;
                        let (field, _) = self.expect_ident()?;
                        self.expect(&Token::RParen)?;
                        return Ok(Expr::Offsetof { struct_type, field });
                    }
                    "null_mut" => {
                        if self.at(&Token::LParen) {
                            self.advance();
                            self.expect(&Token::RParen)?;
                        }
                        return Ok(Expr::NullPtr);
                    }
                    "assert" => {
                        self.expect(&Token::LParen)?;
                        let condition = self.parse_expr()?;
                        let message = if self.at(&Token::Comma) {
                            self.advance();
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        self.expect(&Token::RParen)?;
                        return Ok(Expr::Assert { condition: Box::new(condition), message });
                    }
                    "assert_eq" => {
                        self.expect(&Token::LParen)?;
                        let left = self.parse_expr()?;
                        self.expect(&Token::Comma)?;
                        let right = self.parse_expr()?;
                        let message = if self.at(&Token::Comma) {
                            self.advance();
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        self.expect(&Token::RParen)?;
                        return Ok(Expr::AssertEq { left: Box::new(left), right: Box::new(right), message });
                    }
                    "assert_ne" => {
                        self.expect(&Token::LParen)?;
                        let left = self.parse_expr()?;
                        self.expect(&Token::Comma)?;
                        let right = self.parse_expr()?;
                        let message = if self.at(&Token::Comma) {
                            self.advance();
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        self.expect(&Token::RParen)?;
                        return Ok(Expr::AssertNe { left: Box::new(left), right: Box::new(right), message });
                    }
                    _ => {}
                }

                Ok(Expr::Ident(name))
            }
            Token::LParen => {
                self.advance();

                // Check for closure: |params| body
                // Not in parens — closures use |

                // Tuple or grouped expression
                if self.at(&Token::RParen) {
                    self.advance();
                    return Ok(Expr::Tuple(Vec::new()));
                }

                let first = self.parse_expr()?;

                if self.at(&Token::Comma) {
                    // Tuple
                    let mut elements = vec![first];
                    while self.at(&Token::Comma) {
                        self.advance();
                        if self.at(&Token::RParen) { break; }
                        elements.push(self.parse_expr()?);
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Tuple(elements))
                } else {
                    self.expect(&Token::RParen)?;
                    Ok(first) // Grouped expression
                }
            }
            Token::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !self.at(&Token::RBracket) && !self.at(&Token::Eof) {
                    elements.push(self.parse_expr()?);
                    if self.at(&Token::Semicolon) {
                        // Repeat array literal: [value; size]
                        self.advance();
                        let size = self.parse_expr()?;
                        self.expect(&Token::RBracket)?;
                        if elements.len() != 1 {
                            bail!("Repeat array literal must have exactly one value at {}:{}",
                                self.peek_span().line, self.peek_span().col);
                        }
                        return Ok(Expr::ArrayRepeat {
                            value: Box::new(elements.remove(0)),
                            size: Box::new(size),
                        });
                    }
                    if self.at(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::Array(elements))
            }
            Token::LBrace => {
                let block = self.parse_block()?;
                Ok(Expr::Block(block))
            }
            Token::If => {
                self.advance();
                let saved = self.can_construct;
                self.can_construct = false;
                let condition = Box::new(self.parse_expr()?);
                self.can_construct = saved;
                let then_body = self.parse_block()?;
                let else_body = if self.at(&Token::Else) {
                    self.advance();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Ok(Expr::If { condition, then_body, else_body })
            }
            Token::Match => {
                self.advance();
                let saved = self.can_construct;
                self.can_construct = false;
                let expr = Box::new(self.parse_expr()?);
                self.can_construct = saved;
                self.expect(&Token::LBrace)?;
                let mut arms = Vec::new();
                while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
                    let pattern = self.parse_pattern()?;
                    let guard = if self.at(&Token::If) {
                        self.advance();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(&Token::FatArrow)?;
                    let body = self.parse_expr()?;
                    if self.at(&Token::Comma) { self.advance(); }
                    arms.push(MatchArm::new(pattern, guard, body));
                }
                self.expect(&Token::RBrace)?;
                Ok(Expr::Match { expr, arms })
            }
            Token::Loop => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Expr::Loop(body))
            }
            Token::Unsafe => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Expr::UnsafeBlock(body))
            }
            Token::Fn => {
                // Closure
                self.advance();
                self.expect(&Token::LParen)?;
                let mut params = Vec::new();
                while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                    let is_ref = if self.at(&Token::Amp) { self.advance(); true } else { false };
                    let is_mut = if self.at(&Token::Mut) { self.advance(); true } else { false };
                    let (name, _) = self.expect_ident()?;
                    let ty = if self.at(&Token::Colon) {
                        self.advance();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    params.push(ClosureParam { name, ty, is_ref, is_mut });
                    if self.at(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::RParen)?;
                let return_type = if self.at(&Token::Arrow) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                let body = Box::new(self.parse_expr()?);
                Ok(Expr::Closure { params, return_type, body })
            }
            Token::Pipe => {
                // Closure: |params| body
                self.advance(); // consume first |
                let mut params = Vec::new();
                while !self.at(&Token::Pipe) && !self.at(&Token::Eof) {
                    let is_ref = if self.at(&Token::Amp) { self.advance(); true } else { false };
                    let is_mut = if self.at(&Token::Mut) { self.advance(); true } else { false };
                    let (name, _) = self.expect_ident()?;
                    let ty = if self.at(&Token::Colon) {
                        self.advance();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    params.push(ClosureParam { name, ty, is_ref, is_mut });
                    if self.at(&Token::Comma) { self.advance(); }
                }
                self.expect(&Token::Pipe)?; // consume closing |
                let return_type = if self.at(&Token::Arrow) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                let body = Box::new(self.parse_expr()?);
                Ok(Expr::Closure { params, return_type, body })
            }
            Token::Dot => {
                // Method chain starting with .method()
                self.advance();
                let (_method, _) = self.expect_ident()?;
                self.expect(&Token::LParen)?;
                let _args = self.parse_args()?;
                self.expect(&Token::RParen)?;
                // This shouldn't happen in isolation — it's a postfix operation
                // But we handle it gracefully
                bail!("Unexpected method call at start of expression at {}", self.peek_span().line);
            }
            _ => {
                let span = self.peek_span();
                bail!("Unexpected token {:?} in expression at {}:{}", self.peek(), span.line, span.col);
            }
        }
    }

    // ─── Java Features ────────────────────────────────────────────────

    fn parse_interface_def(&mut self, is_pub: bool) -> Result<InterfaceDef> {
        self.expect(&Token::Interface)?;
        let (name, _) = self.expect_ident()?;
        let generics = if self.at(&Token::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            let _is_pub_method = self.at(&Token::Pub);
            if _is_pub_method { self.advance(); }
            let (method_name, _) = self.expect_ident()?;
            self.expect(&Token::LParen)?;
            let mut params = Vec::new();
            while !self.at(&Token::RParen) && !self.at(&Token::Eof) {
                params.push(self.parse_param()?);
                if self.at(&Token::Comma) { self.advance(); }
            }
            self.expect(&Token::RParen)?;
            let return_type = if self.at(&Token::Arrow) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            let default_body = if self.at(&Token::LBrace) {
                Some(self.parse_block()?)
            } else {
                self.expect(&Token::Semicolon)?;
                None
            };
            methods.push(InterfaceMethod {
                name: method_name,
                params,
                return_type,
                default_body,
            });
        }
        self.expect(&Token::RBrace)?;
        Ok(InterfaceDef { is_pub, name, generics, methods, span: self.peek_span() })
    }

    fn parse_try_catch_expr(&mut self) -> Result<Expr> {
        self.expect(&Token::Try)?;
        let try_body = self.parse_block()?;
        let mut catch_clauses = Vec::new();
        while self.at(&Token::Catch) {
            self.advance();
            let (var_name, var_type) = if self.at(&Token::LParen) {
                self.advance();
                let (name, _) = self.expect_ident()?;
                let ty = if self.at(&Token::Colon) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&Token::RParen)?;
                (Some(name), ty)
            } else {
                (None, None)
            };
            let body = self.parse_block()?;
            catch_clauses.push(CatchClause { var_name, var_type, body });
        }
        let finally_body = if self.at(&Token::Finally) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Expr::TryCatch(TryCatchExpr {
            try_body,
            catch_clauses,
            finally_body,
        }))
    }

    fn parse_throw(&mut self) -> Result<Expr> {
        self.expect(&Token::Throw)?;
        let value = self.parse_expr()?;
        Ok(Expr::Throw(ThrowExpr { value: Box::new(value) }))
    }

    fn parse_package(&mut self) -> Result<PackageDef> {
        self.expect(&Token::Package)?;
        let mut path = Vec::new();
        let (first, _) = self.expect_ident()?;
        path.push(first);
        while self.at(&Token::DoubleColon) {
            self.advance();
            let (part, _) = self.expect_ident()?;
            path.push(part);
        }
        self.expect(&Token::Semicolon)?;
        Ok(PackageDef { path })
    }

    // ─── Zig Features ─────────────────────────────────────────────────

    fn parse_comptime(&mut self) -> Result<ComptimeBlock> {
        self.expect(&Token::Comptime)?;
        let body = self.parse_block()?;
        Ok(ComptimeBlock { body })
    }

    // ─── Python Features ──────────────────────────────────────────────

    fn parse_decorators(&mut self) -> Result<Vec<Expr>> {
        let mut decorators = Vec::new();
        while self.at(&Token::At) {
            self.advance();
            let expr = self.parse_expr()?;
            decorators.push(expr);
        }
        Ok(decorators)
    }

    fn parse_list_comp(&mut self) -> Result<ListCompExpr> {
        self.expect(&Token::LBracket)?;
        let element = self.parse_expr()?;
        self.expect(&Token::For)?;
        let (var_name, _) = self.expect_ident()?;
        self.expect(&Token::In)?;
        let iterable = self.parse_expr()?;
        let condition = if self.at(&Token::If) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(&Token::RBracket)?;
        Ok(ListCompExpr {
            element: Box::new(element),
            variable: var_name,
            iterable: Box::new(iterable),
            condition,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let() {
        let src = r#"
            fn main() {
                let x = 42
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_function() {
        let src = r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_struct() {
        let src = r#"
            struct Point {
                pub x: f64,
                pub y: f64,
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_as_cast() {
        let src = r#"
            fn main() {
                let x = 42 as f64
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_closure() {
        let src = r#"
            fn main() {
                let add = |a: i32, b: i32| a + b
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_if_let() {
        let src = r#"
            fn main() {
                if let Some(x) = value {
                    println!("{}", x)
                }
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    // ─── Use / Module Tests ────────────────────────────────────────────

    #[test]
    fn test_parse_use_simple() {
        let src = r#"
            use std::io
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Use(u) => {
                assert_eq!(u.path, vec!["std", "io"]);
                assert!(u.alias.is_none());
                assert!(u.selective.is_empty());
                assert!(!u.wildcard);
            }
            _ => panic!("Expected Use item"),
        }
    }

    #[test]
    fn test_parse_use_selective() {
        let src = r#"
            use std::io::{Read, Write}
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Use(u) => {
                assert_eq!(u.path, vec!["std", "io"]);
                assert_eq!(u.selective.len(), 2);
                assert_eq!(u.selective[0].name, "Read");
                assert_eq!(u.selective[1].name, "Write");
            }
            _ => panic!("Expected Use item"),
        }
    }

    #[test]
    fn test_parse_use_selective_with_alias() {
        let src = r#"
            use std::io::{Read as R, Write}
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Use(u) => {
                assert_eq!(u.selective[0].name, "Read");
                assert_eq!(u.selective[0].alias.as_deref(), Some("R"));
                assert_eq!(u.selective[1].name, "Write");
                assert!(u.selective[1].alias.is_none());
            }
            _ => panic!("Expected Use item"),
        }
    }

    #[test]
    fn test_parse_use_alias() {
        let src = r#"
            use std::io as io
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Use(u) => {
                assert_eq!(u.path, vec!["std", "io"]);
                assert_eq!(u.alias.as_deref(), Some("io"));
            }
            _ => panic!("Expected Use item"),
        }
    }

    #[test]
    fn test_parse_use_wildcard() {
        let src = r#"
            use std::io::*
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Use(u) => {
                assert_eq!(u.path, vec!["std", "io"]);
                assert!(u.wildcard);
            }
            _ => panic!("Expected Use item"),
        }
    }

    #[test]
    fn test_parse_mod_inline() {
        let src = r#"
            mod math {
                pub fn add(a: i32, b: i32) -> i32 {
                    return a + b
                }
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "math");
                assert!(m.body.is_some());
                assert_eq!(m.body.as_ref().unwrap().len(), 1);
            }
            _ => panic!("Expected Module item"),
        }
    }

    #[test]
    fn test_parse_mod_file() {
        let src = r#"
            mod geometry;
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "geometry");
                assert!(m.body.is_none());
            }
            _ => panic!("Expected Module item"),
        }
    }

    #[test]
    fn test_parse_mod_nested() {
        let src = r#"
            mod utils {
                pub mod strings {
                    pub fn reverse(s: string) -> string {
                        return s
                    }
                }
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "utils");
                let body = m.body.as_ref().unwrap();
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Item::Module(inner) => {
                        assert_eq!(inner.name, "strings");
                        assert!(inner.body.is_some());
                    }
                    _ => panic!("Expected nested Module"),
                }
            }
            _ => panic!("Expected Module item"),
        }
    }

    #[test]
    fn test_parse_mixed_items() {
        let src = r#"
            use std::io
            mod math {
                pub fn add(a: i32, b: i32) -> i32 {
                    return a + b
                }
            }
            fn main() {
                let x = add(1, 2)
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 3);
        assert!(matches!(&program.items[0], Item::Use(_)));
        assert!(matches!(&program.items[1], Item::Module(_)));
        assert!(matches!(&program.items[2], Item::Function(_)));
    }
}