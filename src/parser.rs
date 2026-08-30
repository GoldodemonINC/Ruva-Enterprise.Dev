use crate::ast::*;
use crate::lexer::Lexer;
use anyhow::{bail, Result};

pub struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    /// When false, `Self { ... }` is NOT parsed as a struct literal
    /// (e.g. inside match/if/while discriminant where `{` is the body)
    can_construct: bool,
    /// Whether the most recently parsed expression statement consumed a
    /// semicolon. Used to distinguish a tail expression from `expr;`.
    last_expr_had_semicolon: bool,
    /// When true, the next `peek()` returns `Gt` and `advance()` returns `Gt`
    /// without consuming a real token. This handles `>>` splitting for nested
    /// generics like `Vec<Vec<f64>>` — the lexer produces one `Shr` token but
    /// the parser needs two `Gt` tokens.
    split_gt_pending: bool,
}

impl Parser {
    pub fn new(source: &str) -> Result<Self> {
        let tokens = Lexer::new(source).tokenize()?;
        Ok(Self {
            tokens,
            pos: 0,
            can_construct: true,
            split_gt_pending: false,
            last_expr_had_semicolon: false,
        })
    }

    fn peek(&self) -> &Token {
        if self.split_gt_pending {
            return &Token::Gt;
        }
        &self.tokens.get(self.pos).map(|(t, _)| t).unwrap_or(&Token::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens.get(self.pos).map(|(_, s)| *s).unwrap_or(Span { line: 0, col: 0 })
    }

    fn advance(&mut self) -> (Token, Span) {
        if self.split_gt_pending {
            self.split_gt_pending = false;
            let span = self.tokens.get(self.pos).map(|(_, s)| *s).unwrap_or(Span { line: 0, col: 0 });
            return (Token::Gt, span);
        }
        if self.pos < self.tokens.len() {
            let default = (Token::Eof, Span { line: 0, col: 0 });
            let result = std::mem::replace(&mut self.tokens[self.pos], default);
            self.pos += 1;
            result
        } else {
            (Token::Eof, Span { line: 0, col: 0 })
        }
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

    /// Expect a `Gt` token to close a generic parameter list.
    /// If we see `Shr` (i.e. `>>`), consume it and leave `split_gt_pending = true`
    /// so the next `peek()`/`advance()` returns the second `Gt` without consuming
    /// a real token. This handles nested generics like `Vec<Vec<f64>>`.
    fn expect_close_generic(&mut self) -> Result<Span> {
        if self.at(&Token::Shr) {
            // `>>` needs to be split into two `Gt` tokens.
            let span = self.peek_span();
            self.advance(); // consume the Shr token
            self.split_gt_pending = true; // next peek/advance returns Gt
            Ok(span)
        } else {
            self.expect(&Token::Gt)
        }
    }

    fn at(&self, token: &Token) -> bool {
        self.peek() == token
    }

    // Program

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

    // Imports

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

    // Attributes

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

    // Functions

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

    // Struct

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

    // Enum

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

    // Class

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

    // Impl Block

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

    // Trait

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

    // Type Alias

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

    // Module

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

    // Extern Blocks

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

    // Use Declarations

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

    // Generic Parameters

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

    // Types

    fn parse_type(&mut self) -> Result<Type> {
        let base = match self.peek() {
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
                    while !self.at(&Token::Gt) && !self.at(&Token::Shr) && !self.at(&Token::Eof) {
                        args.push(self.parse_type()?);
                        if self.at(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect_close_generic()?;
                    Type::Generic { name, args }
                } else if self.at(&Token::DoubleColon) {
                    // Path: std::io::Error
                    let mut path = vec![name];
                    while self.at(&Token::DoubleColon) {
                        self.advance();
                        let (seg, _) = self.expect_ident()?;
                        path.push(seg);
                        if self.at(&Token::Lt) {
                            self.advance();
                            let mut args = Vec::new();
                            while !self.at(&Token::Gt) && !self.at(&Token::Shr) && !self.at(&Token::Eof) {
                                args.push(self.parse_type()?);
                                if self.at(&Token::Comma) {
                                    self.advance();
                                }
                            }
                            self.expect_close_generic()?;
                            let full_name = path.join("::");
                            return Ok(Type::Generic { name: full_name, args });
                        }
                    }
                    Type::Path(path)
                } else {
                    Type::Name(name)
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

    // Blocks

    fn parse_block(&mut self) -> Result<Block> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        let mut tail_expr = None;

        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            let stmt = self.parse_stmt()?;
            // If this is an expression stmt followed by RBrace (no semicolon),
            // it's a tail expression — the block's return value.
            match stmt {
                Stmt::Expr(e) if !self.last_expr_had_semicolon && self.at(&Token::RBrace) => {
                    tail_expr = Some(Box::new(e));
                }
                stmt => stmts.push(stmt),
            }
        }

        self.expect(&Token::RBrace)?;

        Ok(Block { stmts, expr: tail_expr })
    }

    fn is_expr_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Int(_) | Token::Float(_) | Token::Str(_) | Token::Char(_)
            | Token::Bool(_) | Token::Null | Token::Ident(_) | Token::Self_ | Token::SelfType
            | Token::LParen | Token::LBracket | Token::LBrace
            |            Token::Not | Token::Amp | Token::Star | Token::Minus | Token::Pipe
            | Token::If | Token::Match | Token::Loop
            | Token::Fn | Token::Move
            // A leading `||` in operand-start position is a zero-argument closure
            // header (logical-OR needs a left operand, so it cannot occur here).
            | Token::Or
        )
    }

    // Statements

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek() {
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
                // Parsing a nested block expression may update this flag;
                // only the outer statement's terminator matters here.
                self.last_expr_had_semicolon = false;
                match self.peek() {
                    Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq | Token::PercentEq
                    | Token::AmpEq | Token::PipeEq | Token::CaretEq => {
                        let op = match self.peek() {
                            Token::PlusEq => BinOp::Add,
                            Token::MinusEq => BinOp::Sub,
                            Token::StarEq => BinOp::Mul,
                            Token::SlashEq => BinOp::Div,
                            Token::PercentEq => BinOp::Rem,
                            Token::AmpEq => BinOp::BitAnd,
                            Token::PipeEq => BinOp::BitOr,
                            Token::CaretEq => BinOp::BitXor,
                            _ => unreachable!(),
                        };
                        self.advance();
                        let value = self.parse_expr()?;
                        if self.at(&Token::Semicolon) {
                            self.advance();
                            self.last_expr_had_semicolon = true;
                        }
                        Ok(Stmt::Expr(Expr::CompoundAssign { op, target: Box::new(expr), value: Box::new(value) }))
                    }
                    Token::Eq => {
                        self.advance();
                        let value = self.parse_expr()?;
                        if self.at(&Token::Semicolon) {
                            self.advance();
                            self.last_expr_had_semicolon = true;
                        }
                        Ok(Stmt::Expr(Expr::Assign { target: Box::new(expr), value: Box::new(value) }))
                    }
                    _ => {
                        if self.at(&Token::Semicolon) {
                            self.advance();
                            self.last_expr_had_semicolon = true;
                        }
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

    // Patterns

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.peek() {
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

    // Expressions

    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_expr_bp(0)
    }

    /// Consume a closure parameter's leading `&` or `&&` patterns.
    fn parse_closure_ref_count(&mut self) -> usize {
        let mut ref_count = 0;
        while self.at(&Token::And) || self.at(&Token::Amp) {
            if self.at(&Token::And) {
                ref_count += 2;
            } else {
                ref_count += 1;
            }
            self.advance();
        }
        ref_count
    }

    /// Pratt parser with precedence climbing
    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr> {
        let mut lhs = self.parse_unary()?;

        // Range expressions: `start..end` and `start..=end`
        // Lowest precedence — parsed before all binary operators.
        if min_bp == 0 && (self.at(&Token::DotDot) || self.at(&Token::DotDotEq)) {
            let inclusive = self.at(&Token::DotDotEq);
            self.advance();
            let rhs = self.parse_expr_bp(1)?;
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

            // Single match: resolve operator and its precedence in one step.
            let (op, bp) = match self.peek() {
                Token::Or => (BinOp::Or, (1, 2)),
                Token::And => (BinOp::And, (3, 4)),
                Token::EqEq => (BinOp::Eq, (5, 6)),
                Token::Ne => (BinOp::Ne, (5, 6)),
                Token::Lt => (BinOp::Lt, (7, 8)),
                Token::Gt => (BinOp::Gt, (7, 8)),
                Token::Le => (BinOp::Le, (7, 8)),
                Token::Ge => (BinOp::Ge, (7, 8)),
                Token::Pipe => (BinOp::BitOr, (9, 10)),
                Token::Caret => (BinOp::BitXor, (11, 12)),
                Token::Amp => (BinOp::BitAnd, (13, 14)),
                Token::Shl => (BinOp::Shl, (15, 16)),
                Token::Shr => (BinOp::Shr, (15, 16)),
                Token::Plus => (BinOp::Add, (17, 18)),
                Token::Minus => (BinOp::Sub, (17, 18)),
                Token::Star => (BinOp::Mul, (19, 20)),
                Token::Slash => (BinOp::Div, (19, 20)),
                Token::Percent => (BinOp::Rem, (19, 20)),
                _ => break,
            };

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

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
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
                    let (field_name, _) = self.expect_ident()?;

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
        match self.peek() {
            Token::Int(_) => {
                let (tok, _) = self.advance();
                if let Token::Int(n) = tok { Ok(Expr::Int(n)) } else { unreachable!() }
            }
            Token::Float(_) => {
                let (tok, _) = self.advance();
                if let Token::Float(f) = tok { Ok(Expr::Float(f)) } else { unreachable!() }
            }
            Token::Str(_) => {
                let (tok, _) = self.advance();
                if let Token::Str(s) = tok { Ok(Expr::Str(s)) } else { unreachable!() }
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
            Token::Char(_) => {
                let (tok, _) = self.advance();
                if let Token::Char(c) = tok { Ok(Expr::Char(c)) } else { unreachable!() }
            }
            Token::Bool(_) => {
                let (tok, _) = self.advance();
                if let Token::Bool(b) = tok { Ok(Expr::Bool(b)) } else { unreachable!() }
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
            Token::Ident(_) => {
                let (tok, _) = self.advance();
                let name = if let Token::Ident(s) = tok { s } else { unreachable!() };

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
                    let (macro_args, separator) = if self.at(&Token::LParen) || self.at(&Token::LBrace) || self.at(&Token::LBracket) {
                        let delim = self.peek().clone();
                        match delim {
                            Token::LParen => {
                                self.advance();
                                let args = self.parse_args()?;
                                self.expect(&Token::RParen)?;
                                (args, ',')
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
                                (args, ',')
                            }
                            Token::LBracket => {
                                self.advance();
                                let mut args = Vec::new();
                                let mut sep = ',';
                                if !self.at(&Token::RBracket) {
                                    args.push(self.parse_expr()?);
                                    while self.at(&Token::Comma) || self.at(&Token::Semicolon) {
                                        if self.at(&Token::Semicolon) { sep = ';'; }
                                        self.advance();
                                        if self.at(&Token::RBracket) { break; }
                                        args.push(self.parse_expr()?);
                                    }
                                }
                                self.expect(&Token::RBracket)?;
                                (args, sep)
                            }
                            _ => (Vec::new(), ','),
                        }
                    } else {
                        (Vec::new(), ',')
                    };
                    return Ok(Expr::Macro { name, args: macro_args, separator });
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
                    let ref_count = self.parse_closure_ref_count();
                    let is_ref = ref_count > 0;
                    let is_mut = if self.at(&Token::Mut) { self.advance(); true } else { false };
                    let (name, _) = self.expect_ident()?;
                    let ty = if self.at(&Token::Colon) {
                        self.advance();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    params.push(ClosureParam { name, ty, is_ref, is_mut, ref_count });
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
                    let ref_count = self.parse_closure_ref_count();
                    let is_ref = ref_count > 0;
                    let is_mut = if self.at(&Token::Mut) { self.advance(); true } else { false };
                    let (name, _) = self.expect_ident()?;
                    let ty = if self.at(&Token::Colon) {
                        self.advance();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    params.push(ClosureParam { name, ty, is_ref, is_mut, ref_count });
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
            Token::Or => {
                // Zero-argument closure: `|| -> T { ... }`. In primary (operand-start)
                // position the lexical `Or` emitted by `||` cannot be the logical-OR
                // operator (it would have no left operand), so it is unambiguously an
                // empty parameter list. In ordinary infix position this token is still
                // consumed as logical OR by the precedence loop, so both uses coexist.
                self.advance();
                let return_type = if self.at(&Token::Arrow) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                let body = Box::new(self.parse_expr()?);
                Ok(Expr::Closure { params: Vec::new(), return_type, body })
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

    // Java Features

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

    // Zig Features

    fn parse_comptime(&mut self) -> Result<ComptimeBlock> {
        self.expect(&Token::Comptime)?;
        let body = self.parse_block()?;
        Ok(ComptimeBlock { body })
    }

    // Python Features

    fn parse_decorators(&mut self) -> Result<Vec<Expr>> {
        let mut decorators = Vec::new();
        while self.at(&Token::At) {
            self.advance();
            let expr = self.parse_expr()?;
            decorators.push(expr);
        }
        Ok(decorators)
    }

    #[allow(dead_code)]
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
    fn test_semicolon_prevents_tail_expression() {
        let src = "fn f() -> i32 { 1; }\nfn g() -> i32 { 1 }";
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();

        let Item::Function(f) = &program.items[0] else { panic!("Expected function") };
        assert!(f.body.expr.is_none());
        assert_eq!(f.body.stmts.len(), 1);

        let Item::Function(g) = &program.items[1] else { panic!("Expected function") };
        assert!(g.body.expr.is_some());
        assert!(g.body.stmts.is_empty());
    }

    #[test]
    fn test_nested_block_does_not_leak_semicolon_state() {
        let src = "fn f() -> i32 { ({ 1; }) }";
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();

        let Item::Function(f) = &program.items[0] else { panic!("Expected function") };
        assert!(f.body.expr.is_some());
        assert!(f.body.stmts.is_empty());
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
    fn test_parse_closure_ref_pattern() {
        // Test |&x| — single ref pattern
        let src = r#"
            fn main() {
                let f = |&x| x
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Function(f) => {
                if let Stmt::Let { value, .. } = &f.body.stmts[0] {
                    if let Expr::Closure { params, .. } = value {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].name, "x");
                        assert!(params[0].is_ref);
                        assert_eq!(params[0].ref_count, 1);
                    } else { panic!("Expected Closure") }
                } else { panic!("Expected Let") }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_parse_closure_double_ref_pattern() {
        // Test |&&x| — double ref pattern (used in filter on iter of &T)
        let src = r#"
            fn main() {
                let f = |&&x| x
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Function(f) => {
                if let Stmt::Let { value, .. } = &f.body.stmts[0] {
                    if let Expr::Closure { params, .. } = value {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].name, "x");
                        assert!(params[0].is_ref);
                        assert_eq!(params[0].ref_count, 2);
                    } else { panic!("Expected Closure") }
                } else { panic!("Expected Let") }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_parse_closure_mixed_ref_and_nonref() {
        // Test |&&x, y, &z| — mixed patterns
        let src = r#"
            fn main() {
                let f = |&&x, y, &z| x + y + z
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Function(f) => {
                if let Stmt::Let { value, .. } = &f.body.stmts[0] {
                    if let Expr::Closure { params, .. } = value {
                        assert_eq!(params.len(), 3);
                        assert_eq!(params[0].name, "x");
                        assert_eq!(params[0].ref_count, 2);
                        assert_eq!(params[1].name, "y");
                        assert_eq!(params[1].ref_count, 0);
                        assert!(!params[1].is_ref);
                        assert_eq!(params[2].name, "z");
                        assert_eq!(params[2].ref_count, 1);
                    } else { panic!("Expected Closure") }
                } else { panic!("Expected Let") }
            }
            _ => panic!("Expected Function"),
        }
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

    // Use / Module Tests

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

    #[test]
    fn test_parse_nested_generics_gt_split() {
        // The lexer produces a single Shr token for >>, but the parser
        // needs two Gt tokens for nested generics like Vec<Vec<f64>>.
        let src = r#"
            fn process(a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
                return a
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Function(f) => {
                // Check return type is Vec<Vec<f64>>
                let ret = f.return_type.as_ref().unwrap();
                match ret {
                    Type::Generic { name, args } => {
                        assert_eq!(name, "Vec");
                        assert_eq!(args.len(), 1);
                        match &args[0] {
                            Type::Generic { name: inner_name, args: inner_args } => {
                                assert_eq!(inner_name, "Vec");
                                assert_eq!(inner_args.len(), 1);
                            }
                            _ => panic!("Expected inner Generic type"),
                        }
                    }
                    _ => panic!("Expected Generic return type"),
                }
                // Check param types
                for param in &f.params {
                    match &param.ty {
                        Type::Reference { inner, .. } => {
                            match inner.as_ref() {
                                Type::Generic { name, args } => {
                                    assert_eq!(name, "Vec");
                                    assert_eq!(args.len(), 1);
                                }
                                _ => panic!("Expected Generic in reference"),
                            }
                        }
                        _ => panic!("Expected Reference type for param"),
                    }
                }
            }
            _ => panic!("Expected Function item"),
        }
    }

    #[test]
    fn test_parse_triple_nested_generics() {
        // Triple-nested: Vec<Vec<Vec<u8>>> produces two Shr tokens
        let src = r#"
            fn get_data() -> Vec<Vec<Vec<u8>>> {
                return Vec::new()
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Function(f) => {
                let ret = f.return_type.as_ref().unwrap();
                match ret {
                    Type::Generic { name, args } => {
                        assert_eq!(name, "Vec");
                        assert_eq!(args.len(), 1);
                        match &args[0] {
                            Type::Generic { name: n2, args: a2 } => {
                                assert_eq!(n2, "Vec");
                                assert_eq!(a2.len(), 1);
                                match &a2[0] {
                                    Type::Generic { name: n3, args: a3 } => {
                                        assert_eq!(n3, "Vec");
                                        assert_eq!(a3.len(), 1);
                                    }
                                    _ => panic!("Expected innermost Generic"),
                                }
                            }
                            _ => panic!("Expected middle Generic"),
                        }
                    }
                    _ => panic!("Expected Generic return type"),
                }
            }
            _ => panic!("Expected Function item"),
        }
    }

    #[test]
    fn test_parse_option_vec_generic() {
        // Option<Vec<Vec<i32>>> — generic with Option wrapping
        let src = r#"
            fn get_nested() -> Option<Vec<Vec<i32>>> {
                return Option::None
            }
        "#;
        let mut parser = Parser::new(src).unwrap();
        let program = parser.parse_program().unwrap();
        match &program.items[0] {
            Item::Function(f) => {
                let ret = f.return_type.as_ref().unwrap();
                match ret {
                    Type::Generic { name, args } => {
                        assert_eq!(name, "Option");
                        assert_eq!(args.len(), 1);
                        match &args[0] {
                            Type::Generic { name: n2, args: a2 } => {
                                assert_eq!(n2, "Vec");
                                assert_eq!(a2.len(), 1);
                            }
                            _ => panic!("Expected inner Generic"),
                        }
                    }
                    _ => panic!("Expected Generic return type"),
                }
            }
            _ => panic!("Expected Function item"),
        }
    }
}
