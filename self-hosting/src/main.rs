// Self-hosted Ruva Parser — compiled from Ruva!
#![allow(unused, dead_code, non_snake_case, non_camel_case_types)]
type string = String;

#[derive(Debug, PartialEq, Clone, Copy)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum UnaryOp {
    Neg,
    Not,
    Deref,
}

#[derive(Debug, Clone)]
struct Program {
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
enum Item {
    Function(FunctionDef),
    Struct(StructDef),
    Enum(EnumDef),
    Impl(ImplBlock),
    Trait(TraitDef),
    Import(ImportDef),
    ExternBlock(ExternBlock),
}

#[derive(Debug, Clone)]
struct FunctionDef {
    is_pub: bool,
    name: string,
    params: Vec<Param>,
    return_type: Option<Type>,
    body: Block,
}

#[derive(Debug, Clone)]
struct Param {
    name: string,
    ty: Type,
}

#[derive(Debug, Clone)]
struct StructDef {
    is_pub: bool,
    name: string,
    fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
struct FieldDef {
    is_pub: bool,
    name: string,
    ty: Type,
}

#[derive(Debug, Clone)]
struct EnumDef {
    is_pub: bool,
    name: string,
    variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
struct EnumVariant {
    name: string,
    fields: Vec<Type>,
}

#[derive(Debug, Clone)]
struct ImplBlock {
    self_type: Type,
    trait_name: Option<Type>,
    methods: Vec<FunctionDef>,
}

#[derive(Debug, Clone)]
struct TraitDef {
    is_pub: bool,
    name: string,
    methods: Vec<TraitMethod>,
}

#[derive(Debug, Clone)]
struct TraitMethod {
    name: string,
    params: Vec<Param>,
    return_type: Option<Type>,
    default_body: Option<Block>,
}

#[derive(Debug, Clone)]
struct ImportDef {
    path: string,
}

#[derive(Debug, Clone)]
struct ExternBlock {
    abi: string,
    items: Vec<ExternItem>,
}

#[derive(Debug, Clone)]
enum ExternItem {
    Function(bool, string, Vec<Param>, Option<Type>),
    Static(bool, string, Type),
}

#[derive(Debug, Clone)]
enum Type {
    Name(string),
    Generic(string, Vec<Type>),
    Reference(Box<Type>, bool),
    Slice(Box<Type>),
    Array(Box<Type>, Option<Box<Expr>>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Unit,
    RawPointer(Box<Type>, bool),
}

#[derive(Debug, Clone)]
struct Block {
    stmts: Vec<Stmt>,
    expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
enum Stmt {
    Let(string, Option<Type>, bool, Expr),
    Expr(Expr),
    Return(Option<Expr>),
    If(Expr, Block, Option<ElseKind>),
    For(string, Expr, Block),
    While(Expr, Block),
    Loop(Block),
    Break(Option<Expr>),
    Continue,
    Match(Expr, Vec<MatchArm>),
    Unsafe(Block),
}

#[derive(Debug, Clone)]
enum ElseKind {
    If(Expr, Block),
    Else(Block),
}

#[derive(Debug, Clone)]
struct MatchArm {
    pattern: Pattern,
    body: Expr,
}

#[derive(Debug, Clone)]
enum Pattern {
    Wildcard,
    Ident(string),
    Literal(Expr),
    Enum(Vec<string>, Vec<Pattern>),
    Or(Vec<Pattern>),
}

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    Float(f64),
    Str(string),
    Bool(bool),
    Null,
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),
    Range(Box<Expr>, Box<Expr>, bool),
    Ident(string),
    Path(Vec<string>),
    Self_,
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Assign { target: Box<Expr>, value: Box<Expr> },
    CompoundAssign { op: BinOp, target: Box<Expr>, value: Box<Expr> },
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, string, Vec<Expr>),
    Field(Box<Expr>, string),
    Index(Box<Expr>, Box<Expr>),
    Closure(Vec<ClosureParam>, Box<Expr>),
    Block(Block),
    If(Box<Expr>, Block, Option<Box<Expr>>),
    Match(Box<Expr>, Vec<MatchArm>),
    Loop(Block),
    Macro(string, Vec<Expr>),
    Reference(Box<Expr>, bool),
    Deref(Box<Expr>),
    Cast(Box<Expr>, Type),
    StructLiteral(Box<Expr>, Vec<(string, Expr)>),
    UnsafeBlock(Block),
    Sizeof(Type),
    Try(Box<Expr>),
}

#[derive(Debug, Clone)]
struct ClosureParam {
    name: string,
    is_ref: bool,
    is_mut: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TokenKind {
    IntLit,
    FloatLit,
    StrLit,
    CharLit,
    BoolLit,
    Ident,
    KwFn,
    KwLet,
    KwMut,
    KwPub,
    KwStruct,
    KwClass,
    KwImpl,
    KwTrait,
    KwEnum,
    KwType,
    KwIf,
    KwElse,
    KwFor,
    KwWhile,
    KwLoop,
    KwBreak,
    KwContinue,
    KwReturn,
    KwMatch,
    KwSelf,
    KwSelfType,
    KwNull,
    KwTrue,
    KwFalse,
    KwImport,
    KwUse,
    KwAs,
    KwMove,
    KwUnsafe,
    KwExtern,
    KwStatic,
    KwConst,
    KwMod,
    KwInterface,
    KwTry,
    KwCatch,
    KwFinally,
    KwThrow,
    KwComptime,
    KwPackage,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Not,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Shl,
    Shr,
    Arrow,
    FatArrow,
    DoubleColon,
    Dot,
    DotDot,
    DotDotEq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    AmpEq,
    PipeEq,
    PercentEq,
    CaretEq,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Colon,
    Comma,
    Hash,
    At,
    Question,
    QuestionDot,
    NullCoalesce,
    Underscore,
    Eof,
}

#[derive(Debug, Clone, Copy)]
struct Span {
    line: u32,
    col: u32,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    value: string,
    span: Span,
}

#[derive(Debug, Clone)]
struct Lexer {
    source: string,
    pos: usize,
    line: u32,
    col: u32,
}

impl Lexer {
    pub fn new(source: string) -> Self
    {
        return Self {source: source, pos: 0, line: 1, col: 1};
    }
    
    fn source_len(&self) -> usize
    {
        return self.source.len();
    }
    
    fn peek_byte(&self) -> Option<u8>
    {
if (self.pos >= self.source_len())         {
            return None;
        }
        let bytes = self.source.as_bytes();
        return Some(bytes[self.pos]);
    }
    
    fn peek_at(&self, offset: usize) -> Option<u8>
    {
        let idx = (self.pos + offset);
if (idx >= self.source_len())         {
            return None;
        }
        return Some(self.source.as_bytes()[idx]);
    }
    
    fn advance(&mut self) -> Option<u8>
    {
        let ch = self.peek_byte();
if ch.is_some()         {
            let c = ch.unwrap();
            self.pos += 1;
if (c == 10)             {
                self.line += 1;
                self.col = 1
            } else             {
                self.col += 1
            }
            return Some(c);
        }
        return None;
    }
    
    fn skip_whitespace(&mut self)
    {
        loop         {
            let c = self.peek_byte();
if c.is_none()             {
                break;
            }
            let ch = c.unwrap();
            let is_ws = ((((ch == 32) || (ch == 9)) || (ch == 10)) || (ch == 13));
if is_ws             {
                self.advance();
            } else             {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self)
    {
        loop         {
            let c = self.peek_byte();
if c.is_none()             {
                break;
            }
            let ch = c.unwrap();
if (ch == 10)             {
                break;
            }
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self)
    {
        let mut depth: u32 = 1;
        while (depth > 0)         {
            let c1 = self.peek_byte();
            let c2 = self.peek_at(1);
if c1.is_none()             {
                break;
            }
            let a = c1.unwrap();
if c2.is_some()             {
                let b = c2.unwrap();
if ((a == 47) && (b == 42))                 {
                    self.advance();
                    self.advance();
                    depth += 1
                } else if ((a == 42) && (b == 47))                 {
                    self.advance();
                    self.advance();
                    depth -= 1
                }
            } else             {
                self.advance();
            }
        }
    }
    
    fn read_string(&mut self, quote: u8) -> string
    {
        let mut s = String::new();
        loop         {
            let ch = self.advance();
if ch.is_none()             {
                break;
            }
            let c = ch.unwrap();
if (c == 92)             {
                let esc = self.advance();
if esc.is_some()                 {
                    let e = esc.unwrap();
                    s.push(e as char)
                }
            } else if (c == quote)             {
                break;
            }
        }
        return s;
    }
    
    fn read_number(&mut self, first: u8) -> Token
    {
        let mut num = String::new();
        num.push(first as char);
        let mut is_float = false;
if ((first == 48) && (self.peek_byte() == Some(120)))         {
            num.push(self.advance().unwrap() as char);
            loop             {
                let c = self.peek_byte();
if c.is_none()                 {
                    break;
                }
                let ch = c.unwrap();
if (((((ch >= 48) && (ch <= 57)) || ((ch >= 65) && (ch <= 70))) || ((ch >= 97) && (ch <= 102))) || (ch == 95))                 {
                    num.push(self.advance().unwrap() as char)
                } else                 {
                    break;
                }
            }
            return Token {kind: TokenKind::IntLit, value: num, span: Span {line: self.line, col: self.col}};
        }
if ((first == 48) && (self.peek_byte() == Some(98)))         {
            num.push(self.advance().unwrap() as char);
            loop             {
                let c = self.peek_byte();
if c.is_none()                 {
                    break;
                }
                let ch = c.unwrap();
if (((ch == 48) || (ch == 49)) || (ch == 95))                 {
                    num.push(self.advance().unwrap() as char)
                } else                 {
                    break;
                }
            }
            return Token {kind: TokenKind::IntLit, value: num, span: Span {line: self.line, col: self.col}};
        }
        loop         {
            let c = self.peek_byte();
if c.is_none()             {
                break;
            }
            let ch = c.unwrap();            if ((ch >= 48) && (ch <= 57))             {
                num.push(self.advance().unwrap() as char)
            } else if (((ch == 46) && !is_float) && (self.peek_at(1) != Some(46)))             {
                is_float = true;
                num.push(self.advance().unwrap() as char)
            } else {
                break;
            }
        }
if is_float         {
            return Token {kind: TokenKind::FloatLit, value: num, span: Span {line: self.line, col: self.col}};
        }
        return Token {kind: TokenKind::IntLit, value: num, span: Span {line: self.line, col: self.col}};
    }
    
    fn keyword_or_ident(&self, word: &str) -> TokenKind
    {
if (word == "fn")         {
            return TokenKind::KwFn;
        }
if (word == "let")         {
            return TokenKind::KwLet;
        }
if (word == "mut")         {
            return TokenKind::KwMut;
        }
if (word == "pub")         {
            return TokenKind::KwPub;
        }
if (word == "struct")         {
            return TokenKind::KwStruct;
        }
if (word == "class")         {
            return TokenKind::KwClass;
        }
if (word == "impl")         {
            return TokenKind::KwImpl;
        }
if (word == "trait")         {
            return TokenKind::KwTrait;
        }
if (word == "enum")         {
            return TokenKind::KwEnum;
        }
if (word == "type")         {
            return TokenKind::KwType;
        }
if (word == "if")         {
            return TokenKind::KwIf;
        }
if (word == "else")         {
            return TokenKind::KwElse;
        }
if (word == "for")         {
            return TokenKind::KwFor;
        }
if (word == "while")         {
            return TokenKind::KwWhile;
        }
if (word == "loop")         {
            return TokenKind::KwLoop;
        }
if (word == "break")         {
            return TokenKind::KwBreak;
        }
if (word == "continue")         {
            return TokenKind::KwContinue;
        }
if (word == "return")         {
            return TokenKind::KwReturn;
        }
if (word == "match")         {
            return TokenKind::KwMatch;
        }
if (word == "self")         {
            return TokenKind::KwSelf;
        }
if (word == "Self")         {
            return TokenKind::KwSelfType;
        }
if (word == "null")         {
            return TokenKind::KwNull;
        }
if (word == "true")         {
            return TokenKind::KwTrue;
        }
if (word == "false")         {
            return TokenKind::KwFalse;
        }
if (word == "import")         {
            return TokenKind::KwImport;
        }
if (word == "use")         {
            return TokenKind::KwUse;
        }
if (word == "as")         {
            return TokenKind::KwAs;
        }
if (word == "move")         {
            return TokenKind::KwMove;
        }
if (word == "unsafe")         {
            return TokenKind::KwUnsafe;
        }
if (word == "extern")         {
            return TokenKind::KwExtern;
        }
if (word == "static")         {
            return TokenKind::KwStatic;
        }
if (word == "const")         {
            return TokenKind::KwConst;
        }
if (word == "mod")         {
            return TokenKind::KwMod;
        }
if (word == "interface")         {
            return TokenKind::KwInterface;
        }
if (word == "try")         {
            return TokenKind::KwTry;
        }
if (word == "catch")         {
            return TokenKind::KwCatch;
        }
if (word == "finally")         {
            return TokenKind::KwFinally;
        }
if (word == "throw")         {
            return TokenKind::KwThrow;
        }
if (word == "comptime")         {
            return TokenKind::KwComptime;
        }
if (word == "package")         {
            return TokenKind::KwPackage;
        }
        return TokenKind::Ident;
    }
    
    fn read_ident(&mut self, first: u8) -> Token
    {
        let mut ident = String::new();
        ident.push(first as char);
        loop         {
            let c = self.peek_byte();
if c.is_none()             {
                break;
            }
            let ch = c.unwrap();
if (((((ch >= 48) && (ch <= 57)) || ((ch >= 65) && (ch <= 90))) || ((ch >= 97) && (ch <= 122))) || (ch == 95))             {
                ident.push(self.advance().unwrap() as char)
            } else             {
                break;
            }
        }
        let kind = self.keyword_or_ident(&ident);
        return Token {kind: kind, value: ident, span: Span {line: self.line, col: self.col}};
    }
    
    fn read_char_lit(&mut self) -> Token
    {
        let span = Span {line: self.line, col: self.col};
        let c = self.advance();
if c.is_some()         {
            let ch = c.unwrap();
if (ch == 92)             {
                let esc = self.advance();
if esc.is_some()                 {
                    let _ = self.advance();
                }
            } else             {
                let _ = self.advance();
            }
        }
        return Token {kind: TokenKind::CharLit, value: String::from("chr"), span: span};
    }
    
    fn next_token(&mut self) -> Token
    {
        self.skip_whitespace();
        let c = self.peek_byte();
if c.is_none()         {
            return Token {kind: TokenKind::Eof, value: String::new(), span: Span {line: self.line, col: self.col}};
        }
        let ch = c.unwrap();
        let span = Span {line: self.line, col: self.col};
if (ch == 34)         {
            let _ = self.advance();
            let s = self.read_string(34);
            return Token {kind: TokenKind::StrLit, value: s, span: span};
        }
if (ch == 39)         {
            let _ = self.advance();
            return self.read_char_lit();
        }
if ((ch >= 48) && (ch <= 57))         {
            let first = self.advance().unwrap(); return self.read_number(first);
        }
if ((((ch >= 65) && (ch <= 90)) || ((ch >= 97) && (ch <= 122))) || (ch == 95))         {
            let first = self.advance().unwrap(); return self.read_ident(first);
        }
if (ch == 47)         {
            self.advance();
            let next = self.peek_byte();
if (next == Some(47))             {
                self.skip_line_comment();
                return self.next_token();
            }
if (next == Some(42))             {
                self.skip_block_comment();
                return self.next_token();
            }
            return Token {kind: TokenKind::Slash, value: String::from("/"), span: span};
        }
if (ch == 43)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::PlusEq, value: String::from("+="), span: span};
            }
            return Token {kind: TokenKind::Plus, value: String::from("+"), span: span};
        }
if (ch == 45)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(62))             {
                let _ = self.advance();
                return Token {kind: TokenKind::Arrow, value: String::from("->"), span: span};
            }
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::MinusEq, value: String::from("-="), span: span};
            }
            return Token {kind: TokenKind::Minus, value: String::from("-"), span: span};
        }
if (ch == 42)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::StarEq, value: String::from("*="), span: span};
            }
            return Token {kind: TokenKind::Star, value: String::from("*"), span: span};
        }
if (ch == 37)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::PercentEq, value: String::from("%="), span: span};
            }
            return Token {kind: TokenKind::Percent, value: String::from("%"), span: span};
        }
if (ch == 61)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::EqEq, value: String::from("=="), span: span};
            }
            return Token {kind: TokenKind::Eq, value: String::from("="), span: span};
        }
if (ch == 33)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::Ne, value: String::from("!="), span: span};
            }
            return Token {kind: TokenKind::Not, value: String::from("!"), span: span};
        }
if (ch == 60)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::Le, value: String::from("<="), span: span};
            }
if (n == Some(60))             {
                let _ = self.advance();
                return Token {kind: TokenKind::Shl, value: String::from("<<"), span: span};
            }
            return Token {kind: TokenKind::Lt, value: String::from("<"), span: span};
        }
if (ch == 62)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::Ge, value: String::from(">="), span: span};
            }
if (n == Some(62))             {
                let _ = self.advance();
                return Token {kind: TokenKind::Shr, value: String::from(">>"), span: span};
            }
            return Token {kind: TokenKind::Gt, value: String::from(">"), span: span};
        }
if (ch == 38)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(38))             {
                let _ = self.advance();
                return Token {kind: TokenKind::And, value: String::from("&&"), span: span};
            }
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::AmpEq, value: String::from("&="), span: span};
            }
            return Token {kind: TokenKind::Amp, value: String::from("&"), span: span};
        }
if (ch == 124)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(124))             {
                let _ = self.advance();
                return Token {kind: TokenKind::Or, value: String::from("||"), span: span};
            }
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::PipeEq, value: String::from("|="), span: span};
            }
            return Token {kind: TokenKind::Pipe, value: String::from("|"), span: span};
        }
if (ch == 94)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(61))             {
                let _ = self.advance();
                return Token {kind: TokenKind::CaretEq, value: String::from("^="), span: span};
            }
            return Token {kind: TokenKind::Caret, value: String::from("^"), span: span};
        }
if (ch == 126)         {
            self.advance();
            return Token {kind: TokenKind::Tilde, value: String::from("~"), span: span};
        }
if (ch == 46)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(46))             {
                let _ = self.advance();
                let n2 = self.peek_byte();
if (n2 == Some(61))                 {
                    let _ = self.advance();
                    return Token {kind: TokenKind::DotDotEq, value: String::from("..="), span: span};
                }
                return Token {kind: TokenKind::DotDot, value: String::from(".."), span: span};
            }
            return Token {kind: TokenKind::Dot, value: String::from("."), span: span};
        }
if (ch == 58)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(58))             {
                let _ = self.advance();
                return Token {kind: TokenKind::DoubleColon, value: String::from("::"), span: span};
            }
            return Token {kind: TokenKind::Colon, value: String::from(":"), span: span};
        }
if (ch == 63)         {
            self.advance();
            let n = self.peek_byte();
if (n == Some(46))             {
                let _ = self.advance();
                return Token {kind: TokenKind::QuestionDot, value: String::from("?."), span: span};
            }
if (n == Some(63))             {
                let _ = self.advance();
                return Token {kind: TokenKind::NullCoalesce, value: String::from("??"), span: span};
            }
            return Token {kind: TokenKind::Question, value: String::from("?"), span: span};
        }
if (ch == 40)         {
            self.advance();
            return Token {kind: TokenKind::LParen, value: String::from("("), span: span};
        }
if (ch == 41)         {
            self.advance();
            return Token {kind: TokenKind::RParen, value: String::from(")"), span: span};
        }
if (ch == 123)         {
            self.advance();
            return Token {kind: TokenKind::LBrace, value: String::from("{"), span: span};
        }
if (ch == 125)         {
            self.advance();
            return Token {kind: TokenKind::RBrace, value: String::from("}"), span: span};
        }
if (ch == 91)         {
            self.advance();
            return Token {kind: TokenKind::LBracket, value: String::from("["), span: span};
        }
if (ch == 93)         {
            self.advance();
            return Token {kind: TokenKind::RBracket, value: String::from("]"), span: span};
        }
if (ch == 59)         {
            self.advance();
            return Token {kind: TokenKind::Semicolon, value: String::from(";"), span: span};
        }
if (ch == 44)         {
            self.advance();
            return Token {kind: TokenKind::Comma, value: String::from(","), span: span};
        }
if (ch == 35)         {
            self.advance();
            return Token {kind: TokenKind::Hash, value: String::from("#"), span: span};
        }
if (ch == 64)         {
            self.advance();
            return Token {kind: TokenKind::At, value: String::from("@"), span: span};
        }
if (ch == 95)         {
            self.advance();
            return Token {kind: TokenKind::Underscore, value: String::from("_"), span: span};
        }
        self.advance();
        return Token {kind: TokenKind::Ident, value: String::from("?"), span: span};
    }
    
    pub fn tokenize(&mut self) -> Vec<Token>
    {
        let mut tokens = Vec::new();
        loop         {
            let tok = self.next_token();
if (tok.kind == TokenKind::Eof)             {
                tokens.push(tok);
                break;
            }
            tokens.push(tok)
        }
        return tokens;
    }
    
}

#[derive(Debug, Clone)]
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    last_expr_had_semicolon: bool,
}

impl Parser {
    pub fn new(source: string) -> Self
    {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        return Parser {tokens: tokens, pos: 0, last_expr_had_semicolon: false};
    }
    
    fn peek(&self) -> TokenKind
    {
if (self.pos >= self.tokens.len())         {
            return TokenKind::Eof;
        }
        return self.tokens[self.pos].kind;
    }
    
    fn peek_value(&self) -> string
    {
if (self.pos >= self.tokens.len())         {
            return String::new();
        }
        return self.tokens[self.pos].value.clone();
    }
    
    fn advance(&mut self) -> Token
    {
if (self.pos < self.tokens.len())         {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            return tok;
        }
        return Token {kind: TokenKind::Eof, value: String::new(), span: Span {line: 0, col: 0}};
    }
    
    fn expect_kind(&mut self, expected: TokenKind) -> Token
    {
        let tok = self.advance();
if (tok.kind == expected)         {
            return tok;
        }
        return Token {kind: TokenKind::Eof, value: String::new(), span: tok.span};
    }
    
    fn at(&self, kind: TokenKind) -> bool
    {
        return (self.peek() == kind);
    }
    
    fn at_any(&self, a: TokenKind, b: TokenKind) -> bool
    {
        return ((self.peek() == a) || (self.peek() == b));
    }
    
    pub fn parse_program(&mut self) -> Program
    {
        let mut items = Vec::new();
        while !self.at(TokenKind::Eof)         {
            items.push(self.parse_item())
        }
        return Program {items: items};
    }
    
    fn parse_item(&mut self) -> Item
    {
if self.at(TokenKind::KwImport)         {
            return Item::Import(self.parse_import());
        }
        let is_pub = self.at(TokenKind::KwPub);
if is_pub         {
            self.advance();
        }
if self.at(TokenKind::KwFn)         {
            return Item::Function(self.parse_function(is_pub));
        }
if self.at(TokenKind::KwStruct)         {
            return Item::Struct(self.parse_struct(is_pub));
        }
if self.at(TokenKind::KwEnum)         {
            return Item::Enum(self.parse_enum(is_pub));
        }
if self.at(TokenKind::KwImpl)         {
            return Item::Impl(self.parse_impl());
        }
if self.at(TokenKind::KwTrait)         {
            return Item::Trait(self.parse_trait(is_pub));
        }
if self.at(TokenKind::KwExtern)         {
            return Item::ExternBlock(self.parse_extern_block());
        }
        while !self.at_any(TokenKind::Eof, TokenKind::RBrace)         {
            self.advance();
        }
        if self.at(TokenKind::RBrace)         {
            self.advance();
        }
        return Item::Import(ImportDef {path: String::from("__skip__")});
    }
    
    fn parse_import(&mut self) -> ImportDef
    {
        self.expect_kind(TokenKind::KwImport);
        let mut path = String::new();
        loop         {
if self.at_any(TokenKind::Eof, TokenKind::Semicolon)             {
                break;
            }
            let tok = self.advance();
if ((tok.kind == TokenKind::Dot) || (tok.kind == TokenKind::DoubleColon))             {
                path.push_str("::")
            } else             {
                path.push_str(&tok.value)
            }
        }
if self.at(TokenKind::Semicolon)         {
            self.advance();
        }
        return ImportDef {path: path};
    }
    
    fn parse_function(&mut self, is_pub: bool) -> FunctionDef
    {
        self.expect_kind(TokenKind::KwFn);
        let name = self.advance().value;
        let params = self.parse_params();
        let return_type = if self.at(TokenKind::Arrow)         {
            self.advance();
            Some(self.parse_type())
        }
 else         {
            None
        }
;
        let body = self.parse_block();
        return FunctionDef {is_pub: is_pub, name: name, params: params, return_type: return_type, body: body};
    }
    
    fn parse_params(&mut self) -> Vec<Param>
    {
        let mut params = Vec::new();
        self.expect_kind(TokenKind::LParen);
        while !self.at(TokenKind::RParen)         {
            let name = self.advance().value;
            self.expect_kind(TokenKind::Colon);
            let ty = self.parse_type();
            params.push(Param {name: name, ty: ty});
if self.at(TokenKind::Comma)             {
                self.advance();
            }
        }
        self.expect_kind(TokenKind::RParen);
        return params;
    }
    
    fn parse_struct(&mut self, is_pub: bool) -> StructDef
    {
        self.expect_kind(TokenKind::KwStruct);
        let name = self.advance().value;
        self.expect_kind(TokenKind::LBrace);
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace)         {
            let fpub = self.at(TokenKind::KwPub);
if fpub             {
                self.advance();
            }
            let fname = self.advance().value;
            self.expect_kind(TokenKind::Colon);
            let fty = self.parse_type();
            fields.push(FieldDef {is_pub: fpub, name: fname, ty: fty});
if self.at(TokenKind::Comma)             {
                self.advance();
            }
        }
        self.expect_kind(TokenKind::RBrace);
        return StructDef {is_pub: is_pub, name: name, fields: fields};
    }
    
    fn parse_enum(&mut self, is_pub: bool) -> EnumDef
    {
        self.expect_kind(TokenKind::KwEnum);
        let name = self.advance().value;
        self.expect_kind(TokenKind::LBrace);
        let mut variants = Vec::new();
        while !self.at(TokenKind::RBrace)         {
            let vname = self.advance().value;
            let fields = if self.at(TokenKind::LParen)             {
                self.advance();
                let mut f = Vec::new();
                while !self.at(TokenKind::RParen)                 {
                    f.push(self.parse_type());
if self.at(TokenKind::Comma)                     {
                        self.advance();
                    }
                }
                self.expect_kind(TokenKind::RParen);
                f
            }
 else             {
                Vec::new()
            }
;
            variants.push(EnumVariant {name: vname, fields: fields});
if self.at(TokenKind::Comma)             {
                self.advance();
            }
        }
        self.expect_kind(TokenKind::RBrace);
        return EnumDef {is_pub: is_pub, name: name, variants: variants};
    }
    
    fn parse_impl(&mut self) -> ImplBlock
    {
        self.expect_kind(TokenKind::KwImpl);
        let trait_name = if !self.at(TokenKind::KwFor)         {
            let t = self.parse_type();
if self.at(TokenKind::KwFor)             {
                self.advance();
                Some(t)
            } else             {
                Some(t)
            }
        }
 else         {
            None
        }
;
        let self_type = self.parse_type();
        self.expect_kind(TokenKind::LBrace);
        let mut methods = Vec::new();
        while !self.at(TokenKind::RBrace)         {
            let mpub = self.at(TokenKind::KwPub);
if mpub             {
                self.advance();
            }
            methods.push(self.parse_function(mpub))
        }
        self.expect_kind(TokenKind::RBrace);
        return ImplBlock {self_type: self_type, trait_name: trait_name, methods: methods};
    }
    
    fn parse_trait(&mut self, is_pub: bool) -> TraitDef
    {
        self.expect_kind(TokenKind::KwTrait);
        let name = self.advance().value;
        self.expect_kind(TokenKind::LBrace);
        let mut methods = Vec::new();
        while !self.at(TokenKind::RBrace)         {
            self.expect_kind(TokenKind::KwFn);
            let mname = self.advance().value;
            let params = self.parse_params();
            let return_type = if self.at(TokenKind::Arrow)             {
                self.advance();
                Some(self.parse_type())
            }
 else             {
                None
            }
;
            let default_body = if self.at(TokenKind::LBrace)             {
                Some(self.parse_block())
            }
 else             {
                None
            }
;
            methods.push(TraitMethod {name: mname, params: params, return_type: return_type, default_body: default_body})
        }
        self.expect_kind(TokenKind::RBrace);
        return TraitDef {is_pub: is_pub, name: name, methods: methods};
    }
    
    fn parse_extern_block(&mut self) -> ExternBlock
    {
        self.expect_kind(TokenKind::KwExtern);
        let abi = if self.at(TokenKind::StrLit)         {
            self.advance().value
        }
 else         {
            String::from("C")
        }
;
        self.expect_kind(TokenKind::LBrace);
        let mut items = Vec::new();
        while !self.at(TokenKind::RBrace)         {
if self.at(TokenKind::KwFn)             {
                self.advance();
                let name = self.advance().value;
                let params = self.parse_params();
                let ret = if self.at(TokenKind::Arrow)                 {
                    self.advance();
                    Some(self.parse_type())
                }
 else                 {
                    None
                }
;
if self.at(TokenKind::Semicolon)                 {
                    self.advance();
                }
                items.push(ExternItem::Function(true, name, params, ret))
            } else if self.at(TokenKind::KwStatic)             {
                self.advance();
                let name = self.advance().value;
                self.expect_kind(TokenKind::Colon);
                let ty = self.parse_type();
if self.at(TokenKind::Semicolon)                 {
                    self.advance();
                }
                items.push(ExternItem::Static(true, name, ty))
            }
        }
        self.expect_kind(TokenKind::RBrace);
        return ExternBlock {abi: abi, items: items};
    }
    
    fn parse_type(&mut self) -> Type
    {
if self.at(TokenKind::KwSelfType)         {
            self.advance();
            return Type::Name(String::from("Self"));
        }
if self.at(TokenKind::Amp)         {
            self.advance();
            let is_mut = self.at(TokenKind::KwMut);
if is_mut             {
                self.advance();
            }
            let inner = Box::new(self.parse_type());
            return Type::Reference(inner, is_mut);
        }
if self.at(TokenKind::Star)         {
            self.advance();
            let is_mut = self.at(TokenKind::KwMut);
if is_mut             {
                self.advance();
            }
            self.expect_kind(TokenKind::KwSelfType);
            let inner = Box::new(Type::Name(String::from("u8")));
            return Type::RawPointer(inner, is_mut);
        }
        let name = self.advance().value;
if self.at(TokenKind::Lt)         {
            self.advance();
            let mut args = Vec::new();
            while !self.at(TokenKind::Gt)             {
                args.push(self.parse_type());
if self.at(TokenKind::Comma)                 {
                    self.advance();
                }
            }
            self.expect_kind(TokenKind::Gt);
            return Type::Generic(name, args);
        }
        return Type::Name(name);
    }
    
    fn parse_block(&mut self) -> Block
    {
        self.expect_kind(TokenKind::LBrace);
        let mut stmts = Vec::new();
        let mut tail_expr = None;
        while (!self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof))         {
            let stmt = self.parse_stmt();
            match stmt {
                Stmt::Expr(e) if !self.last_expr_had_semicolon && self.at(TokenKind::RBrace) => {
                    tail_expr = Some(Box::new(e));
                }
                stmt => stmts.push(stmt),
            }
        }
        self.expect_kind(TokenKind::RBrace);
        return Block {stmts: stmts, expr: tail_expr};
    }
    
    fn parse_stmt(&mut self) -> Stmt
    {
if self.at(TokenKind::KwLet)         {
            return self.parse_let();
        }
if self.at(TokenKind::KwIf)         {
            return self.parse_if_stmt();
        }
if self.at(TokenKind::KwWhile)         {
            return self.parse_while();
        }
if self.at(TokenKind::KwFor)         {
            return self.parse_for();
        }
if self.at(TokenKind::KwLoop)         {
            return self.parse_loop_stmt();
        }
if self.at(TokenKind::KwReturn)         {
            return self.parse_return();
        }
if self.at(TokenKind::KwBreak)         {
            self.advance();
            let v = self.parse_break_value();
if self.at(TokenKind::Semicolon)             {
                self.advance();
            }
            return Stmt::Break(v);
        }
if self.at(TokenKind::KwContinue)         {
            self.advance();
if self.at(TokenKind::Semicolon)             {
                self.advance();
            }
            return Stmt::Continue;
        }
if self.at(TokenKind::KwMatch)         {
            return self.parse_match_stmt();
        }
if self.at(TokenKind::KwUnsafe)         {
            self.advance();
            return Stmt::Unsafe(self.parse_block());
        }
if self.at(TokenKind::LBrace)         {
            return Stmt::Expr(Expr::Block(self.parse_block()));
        }
        let expr = self.parse_expr();
        self.last_expr_had_semicolon = false;
        if self.at(TokenKind::PlusEq) || self.at(TokenKind::MinusEq) || self.at(TokenKind::StarEq) || self.at(TokenKind::SlashEq) || self.at(TokenKind::PercentEq) || self.at(TokenKind::AmpEq) || self.at(TokenKind::PipeEq) || self.at(TokenKind::CaretEq)         {
            let op = match self.peek() {
                TokenKind::PlusEq => BinOp::Add,
                TokenKind::MinusEq => BinOp::Sub,
                TokenKind::StarEq => BinOp::Mul,
                TokenKind::SlashEq => BinOp::Div,
                TokenKind::PercentEq => BinOp::Rem,
                TokenKind::AmpEq => BinOp::BitAnd,
                TokenKind::PipeEq => BinOp::BitOr,
                TokenKind::CaretEq => BinOp::BitXor,
                _ => unreachable!(),
            };
            self.advance();
            let value = self.parse_expr();
            if self.at(TokenKind::Semicolon)             {
                self.advance();
                self.last_expr_had_semicolon = true;
            }
            return Stmt::Expr(Expr::CompoundAssign { op: op, target: Box::new(expr), value: Box::new(value) });
        }
        if self.at(TokenKind::Eq)         {
            self.advance();
            let value = self.parse_expr();
            if self.at(TokenKind::Semicolon)             {
                self.advance();
                self.last_expr_had_semicolon = true;
            }
            return Stmt::Expr(Expr::Assign { target: Box::new(expr), value: Box::new(value) });
        }
        if self.at(TokenKind::Semicolon)         {
            self.advance();
            self.last_expr_had_semicolon = true;
        }
        return Stmt::Expr(expr);
    }
    
    fn parse_let(&mut self) -> Stmt
    {
        self.expect_kind(TokenKind::KwLet);
        let is_mut = self.at(TokenKind::KwMut);
if is_mut         {
            self.advance();
        }
        let name = self.advance().value;
        let ty = if self.at(TokenKind::Colon)         {
            self.advance();
            Some(self.parse_type())
        }
 else         {
            None
        }
;
        self.expect_kind(TokenKind::Eq);
        let value = self.parse_expr();
if self.at(TokenKind::Semicolon)         {
            self.advance();
        }
        return Stmt::Let(name, ty, is_mut, value);
    }
    
    fn parse_if_stmt(&mut self) -> Stmt
    {
        self.expect_kind(TokenKind::KwIf);
        let condition = self.parse_expr();
        let then_body = self.parse_block();
        let else_body = if self.at(TokenKind::KwElse)         {
            self.advance();
if self.at(TokenKind::KwIf)             {
                Some(ElseKind::If(self.parse_if_condition(), self.parse_block()))
            } else             {
                Some(ElseKind::Else(self.parse_block()))
            }
        }
 else         {
            None
        }
;
        return Stmt::If(condition, then_body, else_body);
    }
    
    fn parse_if_condition(&mut self) -> Expr
    {
        let e = self.parse_expr();
        return e;
    }
    
    fn parse_while(&mut self) -> Stmt
    {
        self.expect_kind(TokenKind::KwWhile);
        let condition = self.parse_expr();
        let body = self.parse_block();
        return Stmt::While(condition, body);
    }
    
    fn parse_for(&mut self) -> Stmt
    {
        self.expect_kind(TokenKind::KwFor);
        let var = self.advance().value;
        self.advance();
        let iterable = self.parse_expr();
        let body = self.parse_block();
        return Stmt::For(var, iterable, body);
    }
    
    fn parse_loop_stmt(&mut self) -> Stmt
    {
        self.expect_kind(TokenKind::KwLoop);
        return Stmt::Loop(self.parse_block());
    }
    
    fn parse_return(&mut self) -> Stmt
    {
        self.expect_kind(TokenKind::KwReturn);
        let value = if self.at(TokenKind::Semicolon)         {
            None
        }
 else if self.at_any(TokenKind::RBrace, TokenKind::Eof)         {
            None
        }
 else         {
            Some(self.parse_expr())
        }
;
if self.at(TokenKind::Semicolon)         {
            self.advance();
        }
        return Stmt::Return(value);
    }
    
    fn parse_match_stmt(&mut self) -> Stmt
    {
        self.expect_kind(TokenKind::KwMatch);
        let expr = self.parse_expr();
        self.expect_kind(TokenKind::LBrace);
        let mut arms = Vec::new();
        while !self.at(TokenKind::RBrace)         {
            let pattern = self.parse_pattern();
            self.expect_kind(TokenKind::FatArrow);
            let body = self.parse_expr();
if self.at(TokenKind::Comma)             {
                self.advance();
            }
            arms.push(MatchArm {pattern: pattern, body: body})
        }
        self.expect_kind(TokenKind::RBrace);
        return Stmt::Match(expr, arms);
    }
    
    fn parse_pattern(&mut self) -> Pattern
    {
if self.at(TokenKind::Underscore)         {
            self.advance();
            return Pattern::Wildcard;
        }
if self.at(TokenKind::IntLit)         {
            let v = self.advance().value.parse().unwrap_or(0);
            return Pattern::Literal(Expr::Int(v));
        }
if self.at(TokenKind::StrLit)         {
            let v = self.advance().value;
            return Pattern::Literal(Expr::Str(v));
        }
if self.at(TokenKind::KwTrue)         {
            self.advance();
            return Pattern::Literal(Expr::Bool(true));
        }
if self.at(TokenKind::KwFalse)         {
            self.advance();
            return Pattern::Literal(Expr::Bool(false));
        }
        let name = self.advance().value;
        return Pattern::Ident(name);
    }
    
    fn parse_expr(&mut self) -> Expr
    {
        return self.parse_expr_prec(0);
    }
    
    fn parse_expr_prec(&mut self, min_prec: u32) -> Expr
    {
        let mut left = self.parse_unary();
        loop         {
            let op = self.peek();
            let prec = self.binop_prec(op);
if ((prec == 0) || (prec < min_prec))             {
                break;
            }
            let binop = self.binop_from_kind(op);
            self.advance();
            let right = self.parse_expr_prec((prec + 1));
            left = Expr::Binary(binop, Box::new(left), Box::new(right))
        }
if self.at(TokenKind::Eq)         {
            self.advance();
            let value = self.parse_expr();
            return Expr::Assign { target: Box::new(left), value: Box::new(value) };
        }
        return left;
    }
    
    fn binop_prec(&self, kind: TokenKind) -> u32
    {
if (kind == TokenKind::Or)         {
            return 1;
        }
if (kind == TokenKind::And)         {
            return 2;
        }
if ((kind == TokenKind::EqEq) || (kind == TokenKind::Ne))         {
            return 3;
        }
if ((((kind == TokenKind::Lt) || (kind == TokenKind::Gt)) || (kind == TokenKind::Le)) || (kind == TokenKind::Ge))         {
            return 4;
        }
if (kind == TokenKind::Pipe)         {
            return 5;
        }
if (kind == TokenKind::Caret)         {
            return 6;
        }
if (kind == TokenKind::Amp)         {
            return 7;
        }
if ((kind == TokenKind::Shl) || (kind == TokenKind::Shr))         {
            return 8;
        }
if ((kind == TokenKind::Plus) || (kind == TokenKind::Minus))         {
            return 9;
        }
if (((kind == TokenKind::Star) || (kind == TokenKind::Slash)) || (kind == TokenKind::Percent))         {
            return 10;
        }
        return 0;
    }
    
    fn binop_from_kind(&self, kind: TokenKind) -> BinOp
    {
if (kind == TokenKind::Plus)         {
            return BinOp::Add;
        }
if (kind == TokenKind::Minus)         {
            return BinOp::Sub;
        }
if (kind == TokenKind::Star)         {
            return BinOp::Mul;
        }
if (kind == TokenKind::Slash)         {
            return BinOp::Div;
        }
if (kind == TokenKind::Percent)         {
            return BinOp::Rem;
        }
if (kind == TokenKind::EqEq)         {
            return BinOp::Eq;
        }
if (kind == TokenKind::Ne)         {
            return BinOp::Ne;
        }
if (kind == TokenKind::Lt)         {
            return BinOp::Lt;
        }
if (kind == TokenKind::Gt)         {
            return BinOp::Gt;
        }
if (kind == TokenKind::Le)         {
            return BinOp::Le;
        }
if (kind == TokenKind::Ge)         {
            return BinOp::Ge;
        }
if (kind == TokenKind::And)         {
            return BinOp::And;
        }
if (kind == TokenKind::Or)         {
            return BinOp::Or;
        }
if (kind == TokenKind::Amp)         {
            return BinOp::BitAnd;
        }
if (kind == TokenKind::Pipe)         {
            return BinOp::BitOr;
        }
if (kind == TokenKind::Caret)         {
            return BinOp::BitXor;
        }
if (kind == TokenKind::Shl)         {
            return BinOp::Shl;
        }
if (kind == TokenKind::Shr)         {
            return BinOp::Shr;
        }
        return BinOp::Add;
    }
    
    fn parse_unary(&mut self) -> Expr
    {
if self.at(TokenKind::Minus)         {
            self.advance();
            return Expr::Unary(UnaryOp::Neg, Box::new(self.parse_unary()));
        }
if self.at(TokenKind::Not)         {
            self.advance();
            return Expr::Unary(UnaryOp::Not, Box::new(self.parse_unary()));
        }
if self.at(TokenKind::Star)         {
            self.advance();
            return Expr::Unary(UnaryOp::Deref, Box::new(self.parse_unary()));
        }
if self.at(TokenKind::Amp)         {
            self.advance();
            let is_mut = self.at(TokenKind::KwMut);
if is_mut             {
                self.advance();
            }
            return Expr::Reference(Box::new(self.parse_unary()), is_mut);
        }
        return self.parse_postfix();
    }
    
    fn parse_postfix(&mut self) -> Expr
    {
        let mut expr = self.parse_primary();
        loop         {
if self.at(TokenKind::LParen)             {
                self.advance();
                let mut args = Vec::new();
                while !self.at(TokenKind::RParen)                 {
                    args.push(self.parse_expr());
if self.at(TokenKind::Comma)                     {
                        self.advance();
                    }
                }
                self.expect_kind(TokenKind::RParen);
                expr = Expr::Call(Box::new(expr), args)
            } else if self.at(TokenKind::Dot)             {
                self.advance();
                let field = self.advance().value;
if self.at(TokenKind::LParen)                 {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.at(TokenKind::RParen)                     {
                        args.push(self.parse_expr());
if self.at(TokenKind::Comma)                         {
                            self.advance();
                        }
                    }
                    self.expect_kind(TokenKind::RParen);
                    expr = Expr::MethodCall(Box::new(expr), field, args)
                } else                 {
                    expr = Expr::Field(Box::new(expr), field)
                }
            } else if self.at(TokenKind::LBracket)             {
                self.advance();
                let index = self.parse_expr();
                self.expect_kind(TokenKind::RBracket);
                expr = Expr::Index(Box::new(expr), Box::new(index))
            } else if self.at(TokenKind::KwAs)             {
                self.advance();
                let ty = self.parse_type();
                expr = Expr::Cast(Box::new(expr), ty)
            } else if self.at(TokenKind::Question)             {
                self.advance();
                expr = Expr::Try(Box::new(expr))
            } else {
                break;
            }
        }
        return expr;
    }
    
    fn parse_primary(&mut self) -> Expr
    {
if self.at(TokenKind::IntLit)         {
            let v = self.advance().value.parse().unwrap_or(0);
            return Expr::Int(v);
        }
if self.at(TokenKind::FloatLit)         {
            let v = self.advance().value.parse().unwrap_or(0.0);
            return Expr::Float(v);
        }
if self.at(TokenKind::StrLit)         {
            let v = self.advance().value;
            return Expr::Str(v);
        }
if self.at(TokenKind::KwTrue)         {
            self.advance();
            return Expr::Bool(true);
        }
if self.at(TokenKind::KwFalse)         {
            self.advance();
            return Expr::Bool(false);
        }
if self.at(TokenKind::KwNull)         {
            self.advance();
            return Expr::Null;
        }
if self.at(TokenKind::KwSelf)         {
            self.advance();
            return Expr::Self_;
        }
if self.at(TokenKind::LParen)         {
            self.advance();
if self.at(TokenKind::RParen)             {
                self.advance();
                return Expr::Tuple(Vec::new());
            }
            let first = self.parse_expr();
if self.at(TokenKind::Comma)             {
                let mut elems = vec!(first);
                while !self.at(TokenKind::RParen)                 {
if self.at(TokenKind::Comma)                     {
                        self.advance();
                    }
if self.at(TokenKind::RParen)                     {
                        break;
                    }
                    elems.push(self.parse_expr())
                }
                self.expect_kind(TokenKind::RParen);
                return Expr::Tuple(elems);
            }
            self.expect_kind(TokenKind::RParen);
            return first;
        }
if self.at(TokenKind::LBracket)         {
            self.advance();
            let mut elems = Vec::new();
            while !self.at(TokenKind::RBracket)             {
                elems.push(self.parse_expr());
if self.at(TokenKind::Comma)                 {
                    self.advance();
                }
            }
            self.expect_kind(TokenKind::RBracket);
            return Expr::Array(elems);
        }
if self.at(TokenKind::LBrace)         {
            return Expr::Block(self.parse_block());
        }
if self.at(TokenKind::KwIf)         {
            return self.parse_if_expr();
        }
if self.at(TokenKind::KwLoop)         {
            self.advance();
            return Expr::Loop(self.parse_block());
        }
if self.at(TokenKind::KwMatch)         {
            return self.parse_match_expr();
        }
if self.at(TokenKind::KwUnsafe)         {
            self.advance();
            return Expr::UnsafeBlock(self.parse_block());
        }
if self.at(TokenKind::Pipe)         {
            return self.parse_closure();
        }
if (((self.peek() == TokenKind::Ident) && ((self.pos + 1) < self.tokens.len())) && (self.tokens[(self.pos + 1)].kind == TokenKind::Not))         {
            let name = self.advance().value;
            self.advance();
            self.expect_kind(TokenKind::LParen);
            let mut args = Vec::new();
            while !self.at(TokenKind::RParen)             {
                args.push(self.parse_expr());
if self.at(TokenKind::Comma)                 {
                    self.advance();
                }
            }
            self.expect_kind(TokenKind::RParen);
            return Expr::Macro(name, args);
        }
if self.at(TokenKind::Ident)         {
            let name = self.advance().value;
            return Expr::Ident(name);
        }
        self.advance();
        return Expr::Null;
    }
    
    fn parse_if_expr(&mut self) -> Expr
    {
        self.expect_kind(TokenKind::KwIf);
        let condition = Box::new(self.parse_expr());
        let then_body = self.parse_block();
        let else_body = if self.at(TokenKind::KwElse)         {
            self.advance();
if self.at(TokenKind::KwIf)             {
                Some(Box::new(self.parse_if_expr()))
            } else             {
                Some(Box::new(Expr::Block(self.parse_block())))
            }
        }
 else         {
            None
        }
;
        return Expr::If(condition, then_body, else_body);
    }
    
    fn parse_match_expr(&mut self) -> Expr
    {
        self.expect_kind(TokenKind::KwMatch);
        let expr = Box::new(self.parse_expr());
        self.expect_kind(TokenKind::LBrace);
        let mut arms = Vec::new();
        while !self.at(TokenKind::RBrace)         {
            let pattern = self.parse_pattern();
            self.expect_kind(TokenKind::FatArrow);
            let body = self.parse_expr();
if self.at(TokenKind::Comma)             {
                self.advance();
            }
            arms.push(MatchArm {pattern: pattern, body: body})
        }
        self.expect_kind(TokenKind::RBrace);
        return Expr::Match(expr, arms);
    }
    
    fn parse_closure(&mut self) -> Expr
    {
        self.expect_kind(TokenKind::Pipe);
        let mut params = Vec::new();
        while !self.at(TokenKind::Pipe)         {
            let is_ref = self.at(TokenKind::Amp);
if is_ref             {
                self.advance();
            }
            let is_mut = self.at(TokenKind::KwMut);
if is_mut             {
                self.advance();
            }
            let name = self.advance().value;
            params.push(ClosureParam {name: name, is_ref: is_ref, is_mut: is_mut});
if self.at(TokenKind::Comma)             {
                self.advance();
            }
        }
        self.expect_kind(TokenKind::Pipe);
        let body = if self.at(TokenKind::LBrace)         {
            Box::new(Expr::Block(self.parse_block()))
        }
 else         {
            Box::new(self.parse_expr())
        }
;
        return Expr::Closure(params, body);
    }
    
    fn parse_break_value(&mut self) -> Option<Expr>
    {
if self.at(TokenKind::Semicolon)         {
            return None;
        }
if self.at_any(TokenKind::RBrace, TokenKind::Eof)         {
            return None;
        }
        return Some(self.parse_expr());
    }
    
}

fn main()
{
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file.ruva>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];
    let source = std::fs::read_to_string(path).expect(&format!("Failed to read {}", path));
    let mut parser = Parser::new(source);
    let program = parser.parse_program();
    println!("Parsed {} items", program.items.len())
}

