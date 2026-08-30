use std::fmt;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}



#[derive(Debug, Clone, PartialEq)]
pub enum Token {

    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),


    Ident(String),


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

    Interface,
    Abstract,
    Synchronized,
    Package,
    Try,
    Finally,
    Throw,

    Comptime,

    Decorator,


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



    FStringStart,

    FStringPart(String),

    FStringExpr,

    FStringEnd,



    Eof,
}




#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}


#[derive(Debug, Clone)]
pub enum Item {
    Function(FunctionDef),
    Struct(StructDef),
    Class(ClassDef),
    Enum(EnumDef),
    Impl(ImplBlock),
    Trait(TraitDef),
    TypeAlias(TypeAliasDef),
    Import(ImportDef),
    Use(UseDef),
    Attribute(Attribute),
    Module(ModDef),

    ExternBlock(ExternBlock),

    Interface(InterfaceDef),
    TryCatch(TryCatchExpr),
    Throw(ThrowExpr),
    Package(PackageDef),

    Comptime(ComptimeBlock),

    Decorated(DecoratedDef),
    ListComp(ListCompExpr),
}



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
    #[allow(dead_code)]
    pub is_mut: bool,
    pub name: String,
    pub ty: Type,
}



#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImplBlock {
    pub generics: Vec<GenericParam>,
    pub self_type: Type,
    pub trait_name: Option<Type>,
    pub methods: Vec<FunctionDef>,
    pub span: Span,
}



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
pub struct ImportDef {
    pub path: String,
    pub alias: Option<String>,
    pub items: Option<Vec<String>>,
}




#[derive(Debug, Clone)]
pub struct UseDef {

    pub path: Vec<String>,

    pub alias: Option<String>,

    pub selective: Vec<UseItem>,

    pub wildcard: bool,
}


#[derive(Debug, Clone)]
pub struct UseItem {
    pub name: String,
    pub alias: Option<String>,
}




#[derive(Debug, Clone)]
pub struct ModDef {
    pub is_pub: bool,
    pub name: String,

    pub body: Option<Vec<Item>>,
}



#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<String>,
    pub item: Box<Item>,
}




#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub abi: String,

    pub items: Vec<ExternItem>,
}


#[derive(Debug, Clone)]
pub enum ExternItem {

    Function {
        is_pub: bool,
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
    },

    Static {
        is_pub: bool,
        is_mut: bool,
        name: String,
        ty: Type,
    },

    Const {
        is_pub: bool,
        name: String,
        ty: Type,
        #[allow(dead_code)]
        value: Option<Expr>,
    },
}




#[derive(Debug, Clone)]
pub struct InterfaceDef {
    pub is_pub: bool,
    pub name: String,
    #[allow(dead_code)]
    pub generics: Vec<GenericParam>,
    pub methods: Vec<InterfaceMethod>,
    #[allow(dead_code)]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub default_body: Option<Block>,
}


#[derive(Debug, Clone)]
pub struct TryCatchExpr {
    pub try_body: Block,
    pub catch_clauses: Vec<CatchClause>,
    #[allow(dead_code)]
    pub finally_body: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub var_name: Option<String>,
    #[allow(dead_code)]
    pub var_type: Option<Type>,
    pub body: Block,
}


#[derive(Debug, Clone)]
pub struct ThrowExpr {
    pub value: Box<Expr>,
}


#[derive(Debug, Clone)]
pub struct PackageDef {
    pub path: Vec<String>,
}




#[derive(Debug, Clone)]
pub struct ComptimeBlock {
    pub body: Block,
}




#[derive(Debug, Clone)]
pub struct DecoratedDef {
    pub decorators: Vec<Expr>,
    pub definition: Box<Item>,
}


#[derive(Debug, Clone)]
pub struct ListCompExpr {
    pub element: Box<Expr>,
    pub variable: String,
    pub iterable: Box<Expr>,
    pub condition: Option<Box<Expr>>,
}



#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Type {

    Name(String),

    Path(Vec<String>),

    Reference {
        inner: Box<Type>,
        is_mut: bool,
    },

    Slice(Box<Type>),

    Array {
        inner: Box<Type>,
        size: Option<Box<Expr>>,
    },

    Tuple(Vec<Type>),

    Generic {
        name: String,
        args: Vec<Type>,
    },

    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    Unit,

    Never,

    SelfType,

    RawPointer {
        inner: Box<Type>,
        is_mut: bool,
    },
}



#[derive(Debug, Clone)]
pub enum Stmt {

    Let {
        pattern: Pattern,
        ty: Option<Type>,
        is_mut: bool,
        value: Expr,
    },

    Expr(Expr),

    Return(Option<Expr>),

    If {
        condition: Expr,
        then_body: Block,
        else_body: Option<ElseKind>,
    },

    For {
        pattern: Pattern,
        iterable: Expr,
        body: Block,
    },

    While {
        condition: Expr,
        body: Block,
    },

    WhileLet {
        pattern: Pattern,
        value: Expr,
        body: Block,
    },

    Loop(Block),

    Break(Option<Expr>),

    Continue,

    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
    },

    TryCatch {
        try_body: Block,
        catch_param: String,
        catch_body: Block,
    },

    Block(Block),

    Unsafe(Block),
}

#[derive(Debug, Clone)]
pub enum ElseKind {
    If(Expr, Block),
    Else(Block),
}



#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr {

    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),
    Null,

    Array(Vec<Expr>),

    Tuple(Vec<Expr>),

    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },


    Ident(String),

    Path(Vec<String>),

    Self_,


    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },


    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },


    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },


    CompoundAssign {
        op: BinOp,
        target: Box<Expr>,
        value: Box<Expr>,
    },


    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },


    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },


    Field {
        object: Box<Expr>,
        field: String,
    },


    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },


    Closure {
        params: Vec<ClosureParam>,
        return_type: Option<Type>,
        body: Box<Expr>,
    },


    Block(Block),


    Loop(Block),


    If {
        condition: Box<Expr>,
        then_body: Block,
        else_body: Option<Box<Expr>>,
    },


    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },


    Try(Box<Expr>),


    Cast {
        expr: Box<Expr>,
        ty: Type,
    },


    Macro {
        name: String,
        args: Vec<Expr>,

        separator: char,
    },


    Reference {
        expr: Box<Expr>,
        is_mut: bool,
    },


    Deref(Box<Expr>),


    Move(Box<Expr>),


    VecLit(Vec<Expr>),


    StructLiteral {
        name: Box<Expr>,
        fields: Vec<(String, Expr)>,
    },

    UnsafeBlock(Block),

    Sizeof(Type),

    Offsetof {
        struct_type: String,
        field: String,
    },

    NullPtr,

    FString(Vec<FStringPart>),

    OptionalChaining {
        object: Box<Expr>,
        field: String,
    },

    NullCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
    },

    TryCatch(TryCatchExpr),

    Throw(ThrowExpr),

    Comptime(ComptimeBlock),

    ListComp(ListCompExpr),

    Assert {
        condition: Box<Expr>,
        message: Option<Box<Expr>>,
    },

    AssertEq {
        left: Box<Expr>,
        right: Box<Expr>,
        message: Option<Box<Expr>>,
    },

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

    pub ref_count: usize,
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



#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Pattern {

    Wildcard,

    Ident(String),

    Literal(Expr),

    Tuple(Vec<Pattern>),

    Enum {
        path: Vec<String>,
        fields: Vec<Pattern>,
    },

    Struct {
        path: Vec<String>,
        fields: Vec<(String, Pattern)>,
    },

    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },

    Or(Vec<Pattern>),

    Reference(Box<Pattern>),

    Mut(String),
}



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

