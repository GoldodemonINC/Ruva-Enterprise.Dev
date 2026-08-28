use std::fmt;

// ─── Source Location ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

// ─── Tokens ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),

    // Identifiers
    Ident(String),

    // Keywords
    Fn,
    Let,
    Mut,
    Pub,
    Struct,
    Class,
    Impl,
    Trait,
    Enum,
    Type,
    If,
    Else,
    For,
    While,
    Loop,
    Break,
    Continue,
    Return,
    Match,
    Self_,
    SelfType,
    Null,
    Import,
    Use,
    As,
    Move,
    Where,
    Test,
    In,
    Catch,
    Mod,
    Unsafe,
    Extern,
    Static,
    Const,
    // Java features
    Interface,
    Abstract,
    Synchronized,
    Package,
    Try,
    Finally,
    Throw,
    // Zig features
    Comptime,
    // Python features
    Decorator,

    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Eq,         // =
    EqEq,       // ==
    Ne,         // !=
    Lt,         // <
    Gt,         // >
    Le,         // <=
    Ge,         // >=
    And,        // &&
    Or,         // //
    Not,        // !
    Amp,        // &
    Pipe,       // |
    Caret,      // ^
    Tilde,      // ~
    Shl,        // <<
    Shr,        // >>
    Arrow,      // ->
    FatArrow,   // =>
    DoubleColon, // ::
    Dot,        // .
    DotDot,     // ..
    DotDotEq,   // ..=
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    AmpEq,      // &=
    PipeEq,     // |=
    CaretEq,    // ^=

    // Delimiters
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Semicolon,  // ;
    Colon,      // :
    Comma,      // ,
    Hash,       // #
    At,         // @
    Question,   // ?
    QuestionDot, // ?.
    NullCoalesce, // ??
    Underscore, // _

    // FString interpolation
    FStringStart, // f"
    FStringPart(String), // text between braces
    FStringExpr, // {expr} inside f-string
    FStringEnd, // closing "

    // Special
    Eof,
}

// ─── AST Nodes ───────────────────────────────────────────────────────────────

/// A complete Ruva source file
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Item {
    Function(FunctionDef),
    Struct(StructDef),
    Class(ClassDef),
    Enum(EnumDef),
    Impl(ImplBlock),
    Trait(TraitDef),
    TypeAlias(TypeAliasDef),
    Const(ConstDef),
    Import(ImportDef),
    Use(UseDef),
    Attribute(Attribute),
    Module(ModDef),
    /// extern "C" { fn name(...) -> ...; ... }
    ExternBlock(ExternBlock),
    // Java features
    Interface(InterfaceDef),
    TryCatch(TryCatchExpr),
    Throw(ThrowExpr),
    Package(PackageDef),
    // Zig features
    Comptime(ComptimeBlock),
    // Python features
    Decorated(DecoratedDef),
    ListComp(ListCompExpr),
}

// ─── Functions ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FunctionDef {
    pub is_pub: bool,
    pub is_test: bool,
    pub is_unsafe: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub is_ref: bool,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<Type>,
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructDef {
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<FieldDef>,
    pub derives: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub is_pub: bool,
    pub name: String,
    pub ty: Type,
}

// ─── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnumDef {
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
}

// ─── Classes ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClassDef {
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<ClassField>,
    pub methods: Vec<FunctionDef>,
    pub derives: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ClassField {
    pub is_pub: bool,
    pub is_mut: bool,
    pub name: String,
    pub ty: Type,
}

// ─── Impl Blocks ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImplBlock {
    pub generics: Vec<GenericParam>,
    pub self_type: Type,
    pub trait_name: Option<Type>,
    pub methods: Vec<FunctionDef>,
    pub span: Span,
}

// ─── Traits ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TraitDef {
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<TraitMethod>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub default_body: Option<Block>,
}

// ─── Type Alias ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TypeAliasDef {
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub ty: Type,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConstDef {
    pub is_pub: bool,
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

// ─── Legacy Imports ──────────────────────────────────────────────────────────

/// Old-style `import ruva::foo` (kept for backward compat)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImportDef {
    pub path: String,
    pub alias: Option<String>,
    pub items: Option<Vec<String>>,
}

// ─── Use Declarations ─────────────────────────────────────────────────────

/// `use path::to::Item` or `use path::to::{A, B, C}` or `use path as alias`
#[derive(Debug, Clone)]
pub struct UseDef {
    /// The full path segments (e.g. ["std", "io", "Read"])
    pub path: Vec<String>,
    /// Optional alias: `use foo as bar`
    pub alias: Option<String>,
    /// Selective imports: `use foo::{A, B, C as D}`
    pub selective: Vec<UseItem>,
    /// Is this a wildcard import? `use foo::*`
    pub wildcard: bool,
}

/// A single item inside a `use path::{ ... }` block
#[derive(Debug, Clone)]
pub struct UseItem {
    pub name: String,
    pub alias: Option<String>,
}

// ─── Module ──────────────────────────────────────────────────────────────────

/// `mod name;` loads from file, `mod name { ... }` is inline
#[derive(Debug, Clone)]
pub struct ModDef {
    pub is_pub: bool,
    pub name: String,
    /// Inline module body (None = file-based module `mod name;`)
    pub body: Option<Vec<Item>>,
}

// ─── Attributes ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<String>,
    pub item: Box<Item>,
}

// ─── Extern Blocks ─────────────────────────────────────────────────────────

/// `extern "C" { fn name(...) -> ...; ... }` or `extern "C" { static NAME: type; }`
#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub abi: String, // "C", "system", etc.
    pub items: Vec<ExternItem>,
}

/// A single item inside an extern block
#[derive(Debug, Clone)]
pub enum ExternItem {
    /// extern fn name(params) -> ret;
    Function {
        is_pub: bool,
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
    },
    /// extern static NAME: type;
    Static {
        is_pub: bool,
        is_mut: bool,
        name: String,
        ty: Type,
    },
    /// extern const NAME: type = value;
    Const {
        is_pub: bool,
        name: String,
        ty: Type,
        value: Option<Expr>,
    },
}

// ─── Java Features ─────────────────────────────────────────────────────────

/// Interface definition (Java-style)
#[derive(Debug, Clone)]
pub struct InterfaceDef {
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<InterfaceMethod>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub default_body: Option<Block>,
}

/// Try/catch expression (Java-style)
#[derive(Debug, Clone)]
pub struct TryCatchExpr {
    pub try_body: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub finally_body: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub var_name: Option<String>,
    pub var_type: Option<Type>,
    pub body: Block,
}

/// Throw expression (Java-style)
#[derive(Debug, Clone)]
pub struct ThrowExpr {
    pub value: Box<Expr>,
}

/// Package declaration (Java-style)
#[derive(Debug, Clone)]
pub struct PackageDef {
    pub path: Vec<String>,
}

// ─── Zig Features ──────────────────────────────────────────────────────────

/// Comptime block (Zig-style) — evaluated at compile time
#[derive(Debug, Clone)]
pub struct ComptimeBlock {
    pub body: Block,
}

// ─── Python Features ───────────────────────────────────────────────────────

/// Decorated definition (Python-style @decorator)
#[derive(Debug, Clone)]
pub struct DecoratedDef {
    pub decorators: Vec<Expr>,
    pub definition: Box<Item>,
}

/// List comprehension (Python-style)
#[derive(Debug, Clone)]
pub struct ListCompExpr {
    pub element: Box<Expr>,
    pub variable: String,
    pub iterable: Box<Expr>,
    pub condition: Option<Box<Expr>>,
}

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Type {
    /// Simple named type: i32, string, bool, etc.
    Name(String),
    /// Qualified path: std::io::Error
    Path(Vec<String>),
    /// Reference: &Type, &mut Type
    Reference {
        inner: Box<Type>,
        is_mut: bool,
    },
    /// Slice: &[T]
    Slice(Box<Type>),
    /// Array: [T; N]
    Array {
        inner: Box<Type>,
        size: Option<Box<Expr>>,
    },
    /// Tuple: (A, B, C)
    Tuple(Vec<Type>),
    /// Generic: Vec<T>, Option<T>
    Generic {
        name: String,
        args: Vec<Type>,
    },
    /// Function pointer: fn(A, B) -> C
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    /// Unit type ()
    Unit,
    /// Never type !
    Never,
    /// Self type
    SelfType,
    /// Raw pointer: *const T or *mut T
    RawPointer {
        inner: Box<Type>,
        is_mut: bool,
    },
}

// ─── Statements ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    /// let [mut] name [: type] = expr;
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        is_mut: bool,
        value: Expr,
    },
    /// expr;
    Expr(Expr),
    /// return expr;
    Return(Option<Expr>),
    /// if expr { block } [else if ...] [else { block }]
    If {
        condition: Expr,
        then_body: Block,
        else_body: Option<ElseKind>,
    },
    /// for pattern in expr { block }
    For {
        pattern: Pattern,
        iterable: Expr,
        body: Block,
    },
    /// while expr { block }
    While {
        condition: Expr,
        body: Block,
    },
    /// while let pattern = expr { block }
    WhileLet {
        pattern: Pattern,
        value: Expr,
        body: Block,
    },
    /// loop { block }
    Loop(Block),
    /// break [expr];
    Break(Option<Expr>),
    /// continue;
    Continue,
    /// match expr { arms }
    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
    },
    /// try { block } catch (err) { block }
    TryCatch {
        try_body: Block,
        catch_param: String,
        catch_body: Block,
    },
    /// A block expression as a statement
    Block(Block),
    /// unsafe { ... }
    Unsafe(Block),
}

#[derive(Debug, Clone)]
pub enum ElseKind {
    If(Expr, Block),
    Else(Block),
}

// ─── Expressions ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr {
    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),
    Null,
    /// Array literal [1, 2, 3]
    Array(Vec<Expr>),
    /// Repeat array literal [0; 100]
    ArrayRepeat {
        value: Box<Expr>,
        size: Box<Expr>,
    },
    /// Tuple literal (1, "two", 3.0)
    Tuple(Vec<Expr>),
    /// Range 1..10 or 1..=10
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },

    // Identifiers & paths
    Ident(String),
    /// Qualified path: Vec::new, std::io::println
    Path(Vec<String>),
    /// Self
    Self_,

    // Binary operations
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    // Unary operations
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    // Assignment
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    // Compound assignment
    CompoundAssign {
        op: BinOp,
        target: Box<Expr>,
        value: Box<Expr>,
    },

    // Function call
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },

    // Method call
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    // Field access
    Field {
        object: Box<Expr>,
        field: String,
    },

    // Index access
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    // Closure
    Closure {
        params: Vec<ClosureParam>,
        return_type: Option<Type>,
        body: Box<Expr>,
    },

    // Block expression
    Block(Block),

    // Loop expression (loop { break value })
    Loop(Block),

    // If expression
    If {
        condition: Box<Expr>,
        then_body: Block,
        else_body: Option<Box<Expr>>,
    },

    // Match expression
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    // Try operator ?
    Try(Box<Expr>),

    // Type cast
    Cast {
        expr: Box<Expr>,
        ty: Type,
    },

    // Macro invocation (println!, format!, etc.)
    Macro {
        name: String,
        args: Vec<Expr>,
    },

    // Reference
    Reference {
        expr: Box<Expr>,
        is_mut: bool,
    },

    // Dereference
    Deref(Box<Expr>),

    // Move expression
    Move(Box<Expr>),

    // Vec! macro
    VecLit(Vec<Expr>),

    // Struct literal: Self { x, y } or Point { x, y }
    StructLiteral {
        name: Box<Expr>,
        fields: Vec<(String, Expr)>,
    },
    // Unsafe block expression: unsafe { ... }
    UnsafeBlock(Block),
    // Sizeof expression: sizeof(Type)
    Sizeof(Type),
    // Offsetof expression: offsetof(StructType, field)
    Offsetof {
        struct_type: String,
        field: String,
    },
    // Null pointer literal: null_mut()
    NullPtr,
    // FString interpolation: f"Hello {name}"
    FString(Vec<FStringPart>),
    // Optional chaining: expr?.field
    OptionalChaining {
        object: Box<Expr>,
        field: String,
    },
    // Null coalescing: expr ?? default
    NullCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // Java-style try/catch
    TryCatch(TryCatchExpr),
    // Java-style throw
    Throw(ThrowExpr),
    // Zig comptime block
    Comptime(ComptimeBlock),
    // Python list comprehension
    ListComp(ListCompExpr),
    // Assert: assert!(condition, msg)
    Assert {
        condition: Box<Expr>,
        message: Option<Box<Expr>>,
    },
    // AssertEq: assert_eq!(a, b, msg)
    AssertEq {
        left: Box<Expr>,
        right: Box<Expr>,
        message: Option<Box<Expr>>,
    },
    // AssertNe: assert_ne!(a, b, msg)
    AssertNe {
        left: Box<Expr>,
        right: Box<Expr>,
        message: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub name: String,
    pub ty: Option<Type>,
    pub is_ref: bool,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub enum FStringPart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

impl MatchArm {
    pub fn new(pattern: Pattern, guard: Option<Expr>, body: Expr) -> Self {
        Self { pattern, guard, body }
    }
}

// ─── Patterns ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Pattern {
    /// Wildcard _
    Wildcard,
    /// Identifier binding x
    Ident(String),
    /// Literal pattern 42, "hello", true
    Literal(Expr),
    /// Tuple pattern (a, b, c)
    Tuple(Vec<Pattern>),
    /// Enum variant Path::Variant or Path::Variant(fields)
    Enum {
        path: Vec<String>,
        fields: Vec<Pattern>,
    },
    /// Struct pattern { x, y: z }
    Struct {
        path: Vec<String>,
        fields: Vec<(String, Pattern)>,
    },
    /// Range pattern 1..=9
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    /// Or pattern A | B | C
    Or(Vec<Pattern>),
    /// Reference pattern &x
    Reference(Box<Pattern>),
    /// Mutable binding mut x
    Mut(String),
}

// ─── Binary Operators ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
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

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Rem => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
}

// ─── Block ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
}

#[allow(dead_code)]
impl Block {
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            expr: None,
        }
    }
}
