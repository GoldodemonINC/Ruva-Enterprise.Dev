use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::ast::*;

//  Values

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Nil,
    Array(Vec<Value>),
    Object(Rc<RefCell<Object>>),
    Function(Rc<RefCell<VmFunction>>),
    Closure(Rc<RefCell<VmClosure>>),
    /// A reference to a mutable cell (a captured upvalue slot). Nested closures
    /// share the same cell so reads/writes through either closure see the same
    /// state. Deref'd on `GetUpvalue`, wrapped by `MakeClosure`.
    Ref(Rc<RefCell<Value>>),
    Native(fn(&[Value]) -> Value),
}

#[derive(Debug, Clone)]
pub struct Object {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct VmFunction {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
    pub max_locals: usize,
}

/// A closure: a function body plus a heap-allocated environment holding by-value
/// copies of the enclosing locals it references (its free / captured variables).
/// The environment outlives the closure's creating invocation because it lives on
/// the heap and is shared (`Rc`), so a closure stays valid after the outer function
/// returns. Mutations a closure makes to a captured cell (`SetUpvalue`) persist
/// across calls of the same closure instance.
#[derive(Debug, Clone)]
pub struct VmClosure {
    pub function: Rc<RefCell<VmFunction>>,
    pub env: Rc<RefCell<Vec<Value>>>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Int(b)) => *a == *b as f64,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if *n == (*n as i64) as f64 { write!(f, "{}.0", *n as i64) } else { write!(f, "{}", n) }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "nil"),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() { if i > 0 { write!(f, ", ")?; } write!(f, "{}", v)?; }
                write!(f, "]")
            }
            Value::Object(obj) => {
                let o = obj.borrow();
                write!(f, "{} {{ ", o.class_name)?;
                for (i, (k, v)) in o.fields.iter().enumerate() { if i > 0 { write!(f, ", ")?; } write!(f, "{}: {}", k, v)?; }
                write!(f, " }}")
            }
            Value::Function(func) => write!(f, "<fn {}>", func.borrow().name),
            Value::Closure(c) => write!(f, "<fn {}>", c.borrow().function.borrow().name),
            Value::Ref(r) => write!(f, "{}", r.borrow()),
            Value::Native(_) => write!(f, "<native fn>"),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil | Value::Bool(false) => false,
            Value::Bool(true) => true,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            _ => true,
        }
    }
}

//  Opcodes

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    LoadConst, LoadInt, LoadFloat,    LoadStr, LoadTrue, LoadFalse, LoadNil, LoadArrayLen,
    Pop, Dup, Swap,
    GetLocal, SetLocal, GetGlobal, SetGlobal,
    Add, Sub, Mul, Div, Mod, Negate,
    Eq, Ne, Lt, Le, Gt, Ge,
    Not, And, Or,
    Jmp, JmpIfFalse, Loop,
    Call, Return,
    MakeArray, GetIndex, SetIndex,
    GetField, SetField, NewObject,
    Print, Halt, PrintN,
    MakeClosure, GetUpvalue, SetUpvalue, GetUpvalueRef,
}

impl Opcode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0=>Some(Self::LoadConst),1=>Some(Self::LoadInt),2=>Some(Self::LoadFloat),
            3=>Some(Self::LoadStr),4=>Some(Self::LoadTrue),5=>Some(Self::LoadFalse),
            6=>Some(Self::LoadNil),7=>Some(Self::LoadArrayLen),
            8=>Some(Self::Pop),9=>Some(Self::Dup),10=>Some(Self::Swap),
            11=>Some(Self::GetLocal),12=>Some(Self::SetLocal),
            13=>Some(Self::GetGlobal),14=>Some(Self::SetGlobal),
            15=>Some(Self::Add),16=>Some(Self::Sub),17=>Some(Self::Mul),
            18=>Some(Self::Div),19=>Some(Self::Mod),20=>Some(Self::Negate),
            21=>Some(Self::Eq),22=>Some(Self::Ne),23=>Some(Self::Lt),
            24=>Some(Self::Le),25=>Some(Self::Gt),26=>Some(Self::Ge),
            27=>Some(Self::Not),28=>Some(Self::And),29=>Some(Self::Or),
            30=>Some(Self::Jmp),31=>Some(Self::JmpIfFalse),32=>Some(Self::Loop),
            33=>Some(Self::Call),34=>Some(Self::Return),
            35=>Some(Self::MakeArray),36=>Some(Self::GetIndex),37=>Some(Self::SetIndex),
            38=>Some(Self::GetField),39=>Some(Self::SetField),40=>Some(Self::NewObject),
            41=>Some(Self::Print),            42=>Some(Self::Halt),43=>Some(Self::PrintN),
            44=>Some(Self::MakeClosure),45=>Some(Self::GetUpvalue),46=>Some(Self::SetUpvalue),
            47=>Some(Self::GetUpvalueRef),
            _=>None,
        }
    }
}

//  Chunk

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<u32>,
    pub constants: Vec<Value>,
    pub strings: Vec<String>,
}

impl Chunk {
    pub fn new() -> Self { Self { code: Vec::new(), lines: Vec::new(), constants: Vec::new(), strings: Vec::new() } }
    pub fn emit_byte(&mut self, op: Opcode, line: u32) { self.code.push(op as u8); self.lines.push(line); }
    pub fn emit_bytes(&mut self, op: Opcode, arg: u8, line: u32) { self.code.push(op as u8); self.code.push(arg); self.lines.push(line); self.lines.push(line); }
    pub fn emit_u16(&mut self, op: Opcode, val: u16, line: u32) { self.code.push(op as u8); self.code.push((val>>8) as u8); self.code.push(val as u8); self.lines.extend([line;3]); }
    pub fn emit_i16(&mut self, op: Opcode, val: i16, line: u32) { self.code.push(op as u8); self.code.push((val>>8) as u8); self.code.push(val as u8); self.lines.extend([line;3]); }
    pub fn emit_i64(&mut self, op: Opcode, val: i64, line: u32) {
        self.code.push(op as u8); self.lines.push(line);
        for b in val.to_be_bytes() { self.code.push(b); self.lines.push(line); }
    }
    pub fn add_constant(&mut self, val: Value) -> u16 { let i = self.constants.len() as u16; self.constants.push(val); i }
    pub fn add_string(&mut self, s: String) -> u16 { let i = self.strings.len() as u16; self.strings.push(s); i }
    pub fn read_u16(&self, ip: usize) -> u16 { ((self.code[ip] as u16) << 8) | (self.code[ip+1] as u16) }
    pub fn read_i16(&self, ip: usize) -> i16 { (((self.code[ip] as u16) << 8) | (self.code[ip+1] as u16)) as i16 }
    pub fn read_i64(&self, ip: usize) -> i64 { let mut b=[0u8;8]; for i in 0..8{b[i]=self.code[ip+i];} i64::from_be_bytes(b) }
    pub fn disassemble(&self, name: &str) {
        eprintln!("=== {} ===", name);
        eprintln!("  constants: {}, strings: {}, code: {} bytes", self.constants.len(), self.strings.len(), self.code.len());
        let mut ip = 0; while ip < self.code.len() { ip = self.disassemble_instruction(ip); }
    }
    pub fn disassemble_instruction(&self, mut ip: usize) -> usize {
        let off = ip; let ob = self.code[ip]; ip += 1;
        match Opcode::from_u8(ob) {
            Some(Opcode::LoadConst) => { let i=self.read_u16(ip); eprintln!("  {:4}  LOAD_CONST  {} ({:?})", off, i, self.constants[i as usize]); ip+2 }
            Some(Opcode::LoadInt) => { eprintln!("  {:4}  LOAD_INT    {}", off, self.read_i64(ip)); ip+8 }
            Some(Opcode::LoadFloat) => { let mut b=[0u8;8]; for i in 0..8{b[i]=self.code[ip+i];} eprintln!("  {:4}  LOAD_FLOAT  {}", off, f64::from_be_bytes(b)); ip+8 }
            Some(Opcode::LoadStr) => { let i=self.read_u16(ip); eprintln!("  {:4}  LOAD_STR    \"{}\"", off, self.strings[i as usize]); ip+2 }
            Some(Opcode::LoadTrue)=>{eprintln!("  {:4}  LOAD_TRUE",off);ip}
            Some(Opcode::LoadFalse)=>{eprintln!("  {:4}  LOAD_FALSE",off);ip}
            Some(Opcode::LoadNil)=>{eprintln!("  {:4}  LOAD_NIL",off);ip}
            Some(Opcode::LoadArrayLen)=>{eprintln!("  {:4}  LOAD_ARRAY_LEN",off);ip}
            Some(Opcode::Pop)=>{eprintln!("  {:4}  POP",off);ip}
            Some(Opcode::Dup)=>{eprintln!("  {:4}  DUP",off);ip}
            Some(Opcode::Swap)=>{eprintln!("  {:4}  SWAP",off);ip}
            Some(Opcode::GetLocal)=>{eprintln!("  {:4}  GET_LOCAL   {}",off,self.code[ip]);ip+1}
            Some(Opcode::SetLocal)=>{eprintln!("  {:4}  SET_LOCAL   {}",off,self.code[ip]);ip+1}
            Some(Opcode::GetGlobal)=>{let i=self.read_u16(ip);eprintln!("  {:4}  GET_GLOBAL  \"{}\"",off,self.strings[i as usize]);ip+2}
            Some(Opcode::SetGlobal)=>{let i=self.read_u16(ip);eprintln!("  {:4}  SET_GLOBAL  \"{}\"",off,self.strings[i as usize]);ip+2}
            Some(Opcode::Add)=>{eprintln!("  {:4}  ADD",off);ip}
            Some(Opcode::Sub)=>{eprintln!("  {:4}  SUB",off);ip}
            Some(Opcode::Mul)=>{eprintln!("  {:4}  MUL",off);ip}
            Some(Opcode::Div)=>{eprintln!("  {:4}  DIV",off);ip}
            Some(Opcode::Mod)=>{eprintln!("  {:4}  MOD",off);ip}
            Some(Opcode::Negate)=>{eprintln!("  {:4}  NEGATE",off);ip}
            Some(Opcode::Eq)=>{eprintln!("  {:4}  EQ",off);ip}
            Some(Opcode::Ne)=>{eprintln!("  {:4}  NE",off);ip}
            Some(Opcode::Lt)=>{eprintln!("  {:4}  LT",off);ip}
            Some(Opcode::Le)=>{eprintln!("  {:4}  LE",off);ip}
            Some(Opcode::Gt)=>{eprintln!("  {:4}  GT",off);ip}
            Some(Opcode::Ge)=>{eprintln!("  {:4}  GE",off);ip}
            Some(Opcode::Not)=>{eprintln!("  {:4}  NOT",off);ip}
            Some(Opcode::And)=>{eprintln!("  {:4}  AND",off);ip}
            Some(Opcode::Or)=>{eprintln!("  {:4}  OR",off);ip}
            Some(Opcode::Jmp)=>{let o=self.read_i16(ip);eprintln!("  {:4}  JMP         +{}",off,o);ip+2}
            Some(Opcode::JmpIfFalse)=>{let o=self.read_i16(ip);eprintln!("  {:4}  JMP_IF_FALSE +{}",off,o);ip+2}
            Some(Opcode::Loop)=>{let o=self.read_i16(ip);eprintln!("  {:4}  LOOP        -{}",off,o);ip+2}
            Some(Opcode::Call)=>{eprintln!("  {:4}  CALL        argc={}",off,self.code[ip]);ip+1}
            Some(Opcode::Return)=>{eprintln!("  {:4}  RETURN",off);ip}
            Some(Opcode::MakeArray)=>{eprintln!("  {:4}  MAKE_ARRAY  count={}",off,self.read_u16(ip));ip+2}
            Some(Opcode::GetIndex)=>{eprintln!("  {:4}  GET_INDEX",off);ip}
            Some(Opcode::SetIndex)=>{eprintln!("  {:4}  SET_INDEX",off);ip}
            Some(Opcode::GetField)=>{let i=self.read_u16(ip);eprintln!("  {:4}  GET_FIELD   \"{}\"",off,self.strings[i as usize]);ip+2}
            Some(Opcode::SetField)=>{let i=self.read_u16(ip);eprintln!("  {:4}  SET_FIELD   \"{}\"",off,self.strings[i as usize]);ip+2}
            Some(Opcode::NewObject)=>{let n=self.read_u16(ip);let fc=self.read_u16(ip+2);eprintln!("  {:4}  NEW_OBJECT  \"{}\" fields={}",off,self.strings[n as usize],fc);ip+4}
            Some(Opcode::Print)=>{eprintln!("  {:4}  PRINT",off);ip}
            Some(Opcode::PrintN)=>{eprintln!("  {:4}  PRINTN      argc={}",off,self.code[ip]);ip+1}
            Some(Opcode::MakeClosure)=>{eprintln!("  {:4}  MAKE_CLOSURE n={}",off,self.read_u16(ip));ip+2}
            Some(Opcode::GetUpvalue)=>{eprintln!("  {:4}  GET_UPVALUE {}",off,self.code[ip]);ip+1}
            Some(Opcode::SetUpvalue)=>{eprintln!("  {:4}  SET_UPVALUE {}",off,self.code[ip]);ip+1}
            Some(Opcode::GetUpvalueRef)=>{eprintln!("  {:4}  GET_UPVALUE_REF {}",off,self.code[ip]);ip+1}
            Some(Opcode::Halt)=>{eprintln!("  {:4}  HALT",off);ip}
            None=>{eprintln!("  {:4}  ?? ({})",off,ob);ip}
        }
    }
}
//  Bytecode Compiler — walks the AST and emits instructions

#[derive(Debug, Clone)]
struct Local { name: String, slot: usize, depth: usize }

/// Compile-time bookkeeping for an enclosing loop, so `break`/`continue` can be
/// resolved into proper forward (exit / for-in increment) and backward
/// (while / `loop` re-entry) jumps after the loop is fully emitted.
#[derive(Debug, Clone)]
struct LoopCtx {
    /// Loop start: the condition for `while`, body top for `loop`/`Expr::Loop`, and
    /// the `i < len` check for `for-in`. `continue` in non-for loops jumps back here.
    start: usize,
    /// True for `for-in`: `continue` must jump forward to the increment step, not back
    /// to the condition, otherwise the same element is reprocessed forever.
    is_for: bool,
    /// `scope_depth` when the loop was entered. Locals the loop pushes (the loop
    /// variable, and for `for-in` the internal iterator/len/index) live in the loop's
    /// body scope (`base_scope + 1`), so a nested block's `retain()` must not prune
    /// them while the loop is still active.
    base_scope: usize,
    /// Byte offsets of candidate `Jmp`s emitted by `break` (forward escapes out of the loop).
    breaks: Vec<usize>,
    /// Byte offsets of candidate `Jmp`s emitted by `continue` that still need patching to
    /// the for-in increment step (only used when `is_for`).
    continues: Vec<usize>,
}

pub struct Compiler {
    pub chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    loop_stack: Vec<LoopCtx>,
    max_locals: usize,
    /// Snapshot of the *enclosing* compiler's locals at a closure's creation point, so
    /// the closure body can resolve references to outer locals as captured upvalues
    /// (by name). Only populated while compiling a closure body.
    outer_locals: Vec<Local>,
    /// Map from captured local name to upvalue index for the closure body currently
    /// being compiled.
    upvalues: HashMap<String, usize>,
    /// Upvalue index -> captured local name, in capture (env slot) order.
    upvalue_names: Vec<String>,
    /// Names a nested closure may reference as upvalue-of-upvalue (a variable that
    /// lives in an enclosing *closure's* environment rather than in a plain local of
    /// this compiler's immediate scope). Passed down transitively so deep chains
    /// resolve; the parent resolves each such capture as a cell reference.
    chain_upvalue_names: Vec<String>,
}

struct CompileError(String);

impl Compiler {
    pub fn new() -> Self {
        Self { chunk: Chunk::new(), locals: Vec::new(), scope_depth: 0, loop_stack: Vec::new(), max_locals: 0, outer_locals: Vec::new(), upvalues: HashMap::new(), upvalue_names: Vec::new(), chain_upvalue_names: Vec::new() }
    }

    /// Register a local at `slot` in the current scope, keeping `max_locals` as the
    /// high-water mark of slots used. This stays correct even when end-of-block
    /// prunes the name before `max_locals` is read, so frames reserve enough slots for
    /// all locals their bytecode references during execution.
    fn push_local(&mut self, name: String, slot: usize, depth: usize) {
        self.locals.push(Local { name, slot, depth });
        self.max_locals = self.max_locals.max(slot + 1);
    }

    /// Lower bound (as a `depth`) that nested blocks must retain down to while an
    /// enclosing loop is active: the loop body scope is `base_scope + 1`, so locals
    /// there (the loop variable and, for `for-in`, the iterator setup) survive a nested
    /// block ending. Returns 0 when no loop is active, so scoping is unaffected.
    fn loop_body_floor(&self) -> usize {
        self.loop_stack.last().map(|l| l.base_scope + 1).unwrap_or(0)
    }

    /// Prune locals from scopes that have exited, but never prune locals belonging to
    /// an active enclosing loop body (see [`Self::loop_body_floor`]). The depth of the
    /// scope being left is `leaving_scope`.
    fn retain_locals(&mut self, leaving_scope: usize) {
        let threshold = leaving_scope.max(self.loop_body_floor());
        self.locals.retain(|l| l.depth <= threshold);
    }

    fn current_line(program: &Program) -> u32 {
        program.items.first().map(|i| match i {
            Item::Function(f) => f.span.line as u32,
            _ => 1,
        }).unwrap_or(1)
    }

    pub fn compile(&mut self, program: &Program) -> Result<(), CompileError> {
        let line = Self::current_line(program);
        for item in &program.items { self.compile_item(item, line)?; }
        // Auto-invoke main() if defined
        let main_idx = self.chunk.add_string("main".to_string());
        self.chunk.emit_u16(Opcode::GetGlobal, main_idx, line);
        self.chunk.emit_bytes(Opcode::Call, 0, line);
        self.chunk.emit_byte(Opcode::Pop, line);
        self.chunk.emit_byte(Opcode::Halt, line);
        Ok(())
    }

    fn compile_item(&mut self, item: &Item, line: u32) -> Result<(), CompileError> {
        match item {
            Item::Function(func) => self.compile_function(func),
            Item::Impl(imp) => {
                for method in &imp.methods { self.compile_function(method)?; }
                Ok(())
            }
            Item::Module(m) => {
                if let Some(body) = &m.body {
                    for item in body { self.compile_item(item, line)?; }
                }
                Ok(())
            }
            Item::Comptime(c) => {
                for stmt in &c.body.stmts { self.compile_stmt(stmt, line)?; }
                Ok(())
            }
            Item::Decorated(d) => self.compile_item(&d.definition, line),
            _ => Ok(()),
        }
    }

    fn compile_function(&mut self, func: &FunctionDef) -> Result<(), CompileError> {
        let func_line = func.span.line as u32;
        let mut sub = Compiler::new();
        sub.scope_depth = 1;
        for param in &func.params {
            sub.push_local(param.name.clone(), sub.locals.len(), 1);
        }
        for stmt in &func.body.stmts { sub.compile_stmt(stmt, func_line)?; }
        if let Some(tail) = &func.body.expr {
            sub.compile_expr(tail, func_line)?;
            sub.chunk.emit_byte(Opcode::Return, func_line);
        } else {
            match func.body.stmts.last() {
                Some(Stmt::Return(_)) => {}
                _ => {
                    sub.chunk.emit_byte(Opcode::LoadNil, func_line);
                    sub.chunk.emit_byte(Opcode::Return, func_line);
                }
            }
        }
        let func_obj = Rc::new(RefCell::new(VmFunction {
            name: func.name.clone(), arity: func.params.len(), chunk: sub.chunk, max_locals: sub.max_locals,
        }));
        let const_idx = self.chunk.add_constant(Value::Function(func_obj));
        self.chunk.emit_u16(Opcode::LoadConst, const_idx, func_line);
        let name_idx = self.chunk.add_string(func.name.clone());
        self.chunk.emit_u16(Opcode::SetGlobal, name_idx, func_line);
        self.chunk.emit_byte(Opcode::Pop, func_line);
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt, line: u32) -> Result<(), CompileError> {
        match stmt {
            Stmt::Let { pattern, ty: _, is_mut: _, value } => {
                self.compile_expr(value, line)?;
                let name = match pattern { Pattern::Ident(n) => n.clone(), _ => "_".into() };
                let slot = self.locals.len();
                self.push_local(name, slot, self.scope_depth);
                self.chunk.emit_bytes(Opcode::SetLocal, slot as u8, line);
                self.chunk.emit_byte(Opcode::Pop, line);
            }
            Stmt::Expr(expr) => { self.compile_expr(expr, line)?; self.chunk.emit_byte(Opcode::Pop, line); }
            Stmt::Return(val) => {
                match val { Some(e) => self.compile_expr(e, line)?, None => self.chunk.emit_byte(Opcode::LoadNil, line) }
                self.chunk.emit_byte(Opcode::Return, line);
            }
            Stmt::If { condition, then_body, else_body } => {
                self.compile_if(condition, then_body, else_body.as_ref(), line)?;
            }
            Stmt::While { condition, body } => {
                let ls = self.chunk.code.len();
                self.compile_expr(condition, line)?;
                let ej = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);
                let base_scope = self.scope_depth;
                self.loop_stack.push(LoopCtx { start: ls, is_for: false, base_scope, breaks: Vec::new(), continues: Vec::new() });
                for s in &body.stmts { self.compile_stmt(s, line)?; }
                if let Some(tail) = &body.expr { self.compile_expr(tail, line)?; }
                self.chunk.emit_byte(Opcode::Pop, line);
                self.chunk.emit_i16(Opcode::Loop, (self.chunk.code.len() - ls + 3) as i16, line);
                let exit = self.chunk.code.len();
                let eo = (exit - ej - 3) as i16;
                self.chunk.code[ej+1] = (eo >> 8) as u8; self.chunk.code[ej+2] = eo as u8;
                if let Some(ctx) = self.loop_stack.last() {
                    for &b in &ctx.breaks { let off = (exit - b - 3) as i16; self.chunk.code[b+1] = (off >> 8) as u8; self.chunk.code[b+2] = off as u8; }
                }
                self.loop_stack.pop();
            }
            Stmt::For { pattern, iterable, body } => {
                self.compile_for(pattern, iterable, body, line)?;
            }
            Stmt::Loop(body) => {
                let ls = self.chunk.code.len();
                let base_scope = self.scope_depth;
                self.loop_stack.push(LoopCtx { start: ls, is_for: false, base_scope, breaks: Vec::new(), continues: Vec::new() });
                for s in &body.stmts { self.compile_stmt(s, line)?; }
                if let Some(tail) = &body.expr { self.compile_expr(tail, line)?; }
                self.chunk.emit_byte(Opcode::Pop, line);
                self.chunk.emit_i16(Opcode::Loop, (self.chunk.code.len() - ls + 3) as i16, line);
                let exit = self.chunk.code.len();
                if let Some(ctx) = self.loop_stack.last() {
                    for &b in &ctx.breaks { let off = (exit - b - 3) as i16; self.chunk.code[b+1] = (off >> 8) as u8; self.chunk.code[b+2] = off as u8; }
                }
                self.loop_stack.pop();
            }
            Stmt::Break(val) => {
                if let Some(e) = val {
                    self.compile_expr(e, line)?;
                    // Statement loops (while/for/loop) discard a carried break value to keep
                    // the stack balanced; only the jump escapes the loop.
                    self.chunk.emit_byte(Opcode::Pop, line);
                }
                let jmp = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::Jmp, 0, line);
                match self.loop_stack.last_mut() {
                    Some(ctx) => ctx.breaks.push(jmp),
                    None => return Err(CompileError("break outside loop".into())),
                }
            }
            Stmt::Continue => {
                if self.loop_stack.last().map_or(false, |c| c.is_for) {
                    // for-in: jump forward to the increment step (patched once known).
                    let jmp = self.chunk.code.len();
                    self.chunk.emit_i16(Opcode::Jmp, 0, line);
                    match self.loop_stack.last_mut() {
                        Some(ctx) => ctx.continues.push(jmp),
                        None => return Err(CompileError("continue outside loop".into())),
                    }
                } else {
                    // while / loop: jump back to the condition or body top.
                    let start = self.loop_stack.last()
                        .ok_or_else(|| CompileError("continue outside loop".into()))?;
                    let start = start.start;
                    self.chunk.emit_i16(Opcode::Loop, (self.chunk.code.len() - start + 3) as i16, line);
                }
            }
            Stmt::Match { expr, arms } => { self.compile_match(expr, arms, line)?; }
            Stmt::Block(block) => {
                self.scope_depth += 1;
                for s in &block.stmts { self.compile_stmt(s, line)?; }
                if let Some(tail) = &block.expr { self.compile_expr(tail, line)?; }
                self.scope_depth -= 1;
                self.retain_locals(self.scope_depth);
            }
            Stmt::Unsafe(block) => {
                for s in &block.stmts { self.compile_stmt(s, line)?; }
                if let Some(tail) = &block.expr { self.compile_expr(tail, line)?; self.chunk.emit_byte(Opcode::Pop, line); }
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_for(&mut self, pattern: &Pattern, iterable: &Expr, body: &Block, line: u32) -> Result<(), CompileError> {
        self.compile_expr(iterable, line)?;
        let iter_slot = self.locals.len();
        self.push_local("_iter".into(), iter_slot, self.scope_depth);
        self.chunk.emit_bytes(Opcode::SetLocal, iter_slot as u8, line);
        self.chunk.emit_byte(Opcode::Pop, line);

        let len_slot = self.locals.len();
        self.push_local("_len".into(), len_slot, self.scope_depth);
        self.chunk.emit_bytes(Opcode::GetLocal, iter_slot as u8, line);
        self.chunk.emit_byte(Opcode::LoadArrayLen, line);
        self.chunk.emit_bytes(Opcode::SetLocal, len_slot as u8, line);
        self.chunk.emit_byte(Opcode::Pop, line);

        let i_slot = self.locals.len();
        self.push_local("_i".into(), i_slot, self.scope_depth);
        self.chunk.emit_i64(Opcode::LoadInt, 0, line);
        self.chunk.emit_bytes(Opcode::SetLocal, i_slot as u8, line);
        self.chunk.emit_byte(Opcode::Pop, line);

        let ls = self.chunk.code.len();
        let base_scope = self.scope_depth;
        self.loop_stack.push(LoopCtx { start: ls, is_for: true, base_scope, breaks: Vec::new(), continues: Vec::new() });
        self.chunk.emit_bytes(Opcode::GetLocal, i_slot as u8, line);
        self.chunk.emit_bytes(Opcode::GetLocal, len_slot as u8, line);
        self.chunk.emit_byte(Opcode::Lt, line);
        let ej = self.chunk.code.len();
        self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);

        self.chunk.emit_bytes(Opcode::GetLocal, iter_slot as u8, line);
        self.chunk.emit_bytes(Opcode::GetLocal, i_slot as u8, line);
        self.chunk.emit_byte(Opcode::GetIndex, line);
        let var_slot = self.locals.len();
        let var_name = match pattern { Pattern::Ident(n) => n.clone(), _ => "_".into() };
        self.push_local(var_name, var_slot, self.scope_depth + 1);
        self.chunk.emit_bytes(Opcode::SetLocal, var_slot as u8, line);
        self.chunk.emit_byte(Opcode::Pop, line);

        for s in &body.stmts { self.compile_stmt(s, line)?; }
        if let Some(tail) = &body.expr { self.compile_expr(tail, line)?; }
        self.chunk.emit_byte(Opcode::Pop, line);

        // `continue` inside the for body jumps forward to this increment step.
        let continue_target = self.chunk.code.len();
        if let Some(ctx) = self.loop_stack.last() {
            for &c in &ctx.continues { let off = (continue_target - c - 3) as i16; self.chunk.code[c+1] = (off >> 8) as u8; self.chunk.code[c+2] = off as u8; }
        }
        self.chunk.emit_bytes(Opcode::GetLocal, i_slot as u8, line);
        self.chunk.emit_i64(Opcode::LoadInt, 1, line);
        self.chunk.emit_byte(Opcode::Add, line);
        self.chunk.emit_bytes(Opcode::SetLocal, i_slot as u8, line);
        self.chunk.emit_byte(Opcode::Pop, line);

        self.chunk.emit_i16(Opcode::Loop, (self.chunk.code.len() - ls + 3) as i16, line);
        let exit = self.chunk.code.len();
        let eo = (exit - ej - 3) as i16;
        self.chunk.code[ej+1] = (eo >> 8) as u8; self.chunk.code[ej+2] = eo as u8;
        if let Some(ctx) = self.loop_stack.last() {
            for &b in &ctx.breaks { let off = (exit - b - 3) as i16; self.chunk.code[b+1] = (off >> 8) as u8; self.chunk.code[b+2] = off as u8; }
        }        self.loop_stack.pop();
        // Prune the loop variable (declared in the loop body scope) from name
        // resolution, but *keep* the loop's internal locals (_iter/_len/_i) so that
        // `max_locals` still reserves the slots they occupy during execution.
        self.locals.retain(|l| l.depth <= self.scope_depth);
        Ok(())
    }


    fn compile_if(&mut self, condition: &Expr, then_body: &Block, else_body: Option<&ElseKind>, line: u32) -> Result<(), CompileError> {
        self.compile_expr(condition, line)?;
        let tj = self.chunk.code.len();
        self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);
        self.scope_depth += 1;
        for s in &then_body.stmts { self.compile_stmt(s, line)?; }
        if let Some(tail) = &then_body.expr { self.compile_expr(tail, line)?; }
        self.chunk.emit_byte(Opcode::Pop, line);
        self.scope_depth -= 1;
        self.retain_locals(self.scope_depth);
        if let Some(ek) = else_body {
            let ej = self.chunk.code.len();
            self.chunk.emit_i16(Opcode::Jmp, 0, line);
            let to = (self.chunk.code.len() - tj - 3) as i16;
            self.chunk.code[tj+1] = (to >> 8) as u8; self.chunk.code[tj+2] = to as u8;
            match ek {
                ElseKind::If(ec, eb) => self.compile_if(ec, eb, None, line)?,
                ElseKind::Else(eb) => {
                    self.scope_depth += 1;
                    for s in &eb.stmts { self.compile_stmt(s, line)?; }
                    if let Some(tail) = &eb.expr { self.compile_expr(tail, line)?; }
                    self.chunk.emit_byte(Opcode::Pop, line);
                    self.scope_depth -= 1;
                    self.retain_locals(self.scope_depth);
                }
            }
            let eo = (self.chunk.code.len() - ej - 3) as i16;
            self.chunk.code[ej+1] = (eo >> 8) as u8; self.chunk.code[ej+2] = eo as u8;
        } else {
            let to = (self.chunk.code.len() - tj - 3) as i16;
            self.chunk.code[tj+1] = (to >> 8) as u8; self.chunk.code[tj+2] = to as u8;
        }
        Ok(())
    }

    fn compile_match(&mut self, expr: &Expr, arms: &[MatchArm], line: u32) -> Result<(), CompileError> {
        self.compile_expr(expr, line)?;
        let mut patches: Vec<usize> = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            match &arm.pattern {
                Pattern::Wildcard => {
                    self.chunk.emit_byte(Opcode::Pop, line);
                    self.compile_expr(&arm.body, line)?;
                }
                Pattern::Literal(lit) => {
                    if i > 0 { self.chunk.emit_byte(Opcode::Dup, line); }
                    self.compile_expr(lit, line)?;
                    self.chunk.emit_byte(Opcode::Eq, line);
                    let jmp = self.chunk.code.len();
                    self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);
                    self.chunk.emit_byte(Opcode::Pop, line);
                    self.compile_expr(&arm.body, line)?;
                    let end = self.chunk.code.len();
                    self.chunk.emit_i16(Opcode::Jmp, 0, line);
                    patches.push(end);
                    let off = (self.chunk.code.len() - jmp - 3) as i16;
                    self.chunk.code[jmp+1] = (off >> 8) as u8; self.chunk.code[jmp+2] = off as u8;
                }
                Pattern::Ident(name) => {
                    self.chunk.emit_byte(Opcode::Pop, line);
                    let slot = self.locals.len();
                    self.push_local(name.clone(), slot, self.scope_depth + 1);
                    self.compile_expr(&arm.body, line)?;
                }
                _ => { self.chunk.emit_byte(Opcode::Pop, line); }
            }
        }
        self.chunk.emit_byte(Opcode::Pop, line);
        for jmp in patches {
            let off = (self.chunk.code.len() - jmp - 3) as i16;
            self.chunk.code[jmp+1] = (off >> 8) as u8; self.chunk.code[jmp+2] = off as u8;
        }
        Ok(())
    }

    pub fn compile_expr(&mut self, expr: &Expr, line: u32) -> Result<(), CompileError> {
        match expr {
            Expr::Int(n) => { self.chunk.emit_i64(Opcode::LoadInt, *n, line); }
            Expr::Float(n) => {
                self.chunk.emit_byte(Opcode::LoadFloat, line);
                for b in n.to_be_bytes() { self.chunk.code.push(b); self.chunk.lines.push(line); }
            }
            Expr::Str(s) => { let i = self.chunk.add_string(s.clone()); self.chunk.emit_u16(Opcode::LoadStr, i, line); }
            Expr::Char(c) => { let i = self.chunk.add_string(c.to_string()); self.chunk.emit_u16(Opcode::LoadStr, i, line); }
            Expr::Bool(b) => { self.chunk.emit_byte(if *b { Opcode::LoadTrue } else { Opcode::LoadFalse }, line); }
            Expr::Null => { self.chunk.emit_byte(Opcode::LoadNil, line); }
            Expr::Ident(name) => self.emit_read_name(name, line)?,
            Expr::Self_ => {
                if let Some(local) = self.locals.iter().rev().find(|l| l.name == "self") {
                    self.chunk.emit_bytes(Opcode::GetLocal, local.slot as u8, line);
                } else { return Err(CompileError("'self' outside method".into())); }
            }
            Expr::Closure { params, return_type: _, body } => self.compile_closure(params, body, line)?,
            Expr::Binary { op, left, right } => self.compile_binary(op, left, right, line)?,
            Expr::Unary { op, expr } => {
                self.compile_expr(expr, line)?;
                match op { UnaryOp::Neg => self.chunk.emit_byte(Opcode::Negate, line), UnaryOp::Not => self.chunk.emit_byte(Opcode::Not, line), _ => {} }
            }
            Expr::Assign { target, value } => self.compile_assign(target, value, line)?,
            Expr::CompoundAssign { op, target, value } => {
                self.compile_compound_assign(op, target, value, line)?;
            }
            Expr::Call { function, args } => {
                for a in args { self.compile_expr(a, line)?; }
                self.compile_expr(function, line)?;
                self.chunk.emit_bytes(Opcode::Call, args.len() as u8, line);
            }
            Expr::MethodCall { object, method, args } => {
                for a in args { self.compile_expr(a, line)?; }
                self.compile_expr(object, line)?;
                let i = self.chunk.add_string(method.clone());
                self.chunk.emit_u16(Opcode::GetField, i, line);
                self.chunk.emit_bytes(Opcode::Call, (args.len() + 1) as u8, line);
            }
            Expr::Field { object, field } => {
                self.compile_expr(object, line)?;
                let i = self.chunk.add_string(field.clone());
                self.chunk.emit_u16(Opcode::GetField, i, line);
            }
            Expr::Array(elems) | Expr::Tuple(elems) | Expr::VecLit(elems) => {
                for e in elems { self.compile_expr(e, line)?; }
                self.chunk.emit_u16(Opcode::MakeArray, elems.len() as u16, line);
            }
            Expr::Index { object, index } => {
                self.compile_expr(object, line)?;
                self.compile_expr(index, line)?;
                self.chunk.emit_byte(Opcode::GetIndex, line);
            }
            Expr::If { condition, then_body, else_body } => {
                self.compile_expr(condition, line)?;
                let tj = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);
                // True branch: pop condition, evaluate then body
                self.chunk.emit_byte(Opcode::Pop, line);
                for s in &then_body.stmts { self.compile_stmt(s, line)?; }
                if let Some(tail) = &then_body.expr { self.compile_expr(tail, line)?; }
                if let Some(else_e) = else_body {
                    let ej = self.chunk.code.len();
                    self.chunk.emit_i16(Opcode::Jmp, 0, line);
                    // Patch JmpIfFalse to jump to false branch
                    let to = (self.chunk.code.len() - tj - 3) as i16;
                    self.chunk.code[tj+1] = (to >> 8) as u8; self.chunk.code[tj+2] = to as u8;
                    // False branch: pop condition, evaluate else
                    self.chunk.emit_byte(Opcode::Pop, line);
                    self.compile_expr(else_e, line)?;
                    // Patch Jmp to end
                    let eo = (self.chunk.code.len() - ej - 3) as i16;
                    self.chunk.code[ej+1] = (eo >> 8) as u8; self.chunk.code[ej+2] = eo as u8;
                } else {
                    // Patch JmpIfFalse to jump to false branch
                    let to = (self.chunk.code.len() - tj - 3) as i16;
                    self.chunk.code[tj+1] = (to >> 8) as u8; self.chunk.code[tj+2] = to as u8;
                    // False branch: pop condition, push nil
                    self.chunk.emit_byte(Opcode::Pop, line);
                    self.chunk.emit_byte(Opcode::LoadNil, line);
                }
            }
            Expr::Block(block) => {
                self.scope_depth += 1;
                for s in &block.stmts { self.compile_stmt(s, line)?; }
                match &block.expr {
                    Some(tail) => self.compile_expr(tail, line)?,
                    None => { if block.stmts.is_empty() { self.chunk.emit_byte(Opcode::LoadNil, line); } }
                }
                self.scope_depth -= 1;
                self.retain_locals(self.scope_depth);
            }
            Expr::Macro { name, args, separator: _ } => self.compile_macro(name, args, line)?,
            Expr::StructLiteral { name, fields } => {
                let cn = match name.as_ref() { Expr::Ident(n) => n.clone(), Expr::Path(p) => p.join("::"), _ => "Object".into() };
                let ni = self.chunk.add_string(cn);
                for (_, v) in fields { self.compile_expr(v, line)?; }
                for (n, _) in fields { let i = self.chunk.add_string(n.clone()); self.chunk.emit_u16(Opcode::LoadStr, i, line); }
                self.chunk.emit_u16(Opcode::NewObject, ni, line);
                self.chunk.emit_u16(Opcode::LoadStr, fields.len() as u16, line);
            }
            Expr::Reference { expr: inner, .. } | Expr::Deref(inner) | Expr::Cast { expr: inner, .. }
            | Expr::Move(inner) | Expr::Try(inner) => self.compile_expr(inner, line)?,
            Expr::Range { start, end, .. } => {
                self.compile_expr(start, line)?;
                self.compile_expr(end, line)?;
                self.chunk.emit_u16(Opcode::MakeArray, 2, line);
            }
            Expr::FString(parts) => {
                for part in parts { match part {
                    FStringPart::Text(s) => { let i = self.chunk.add_string(s.clone()); self.chunk.emit_u16(Opcode::LoadStr, i, line); }
                    FStringPart::Expr(e) => { self.compile_expr(e, line)?; }
                }}
            }
            Expr::NullCoalesce { left, right } => {
                self.compile_expr(left, line)?;
                self.chunk.emit_byte(Opcode::Dup, line);
                let ej = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);
                self.chunk.emit_byte(Opcode::Pop, line);
                let ej2 = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::Jmp, 0, line);
                let off = (self.chunk.code.len() - ej - 3) as i16;
                self.chunk.code[ej+1] = (off >> 8) as u8; self.chunk.code[ej+2] = off as u8;
                self.chunk.emit_byte(Opcode::Pop, line);
                self.compile_expr(right, line)?;
                let off2 = (self.chunk.code.len() - ej2 - 3) as i16;
                self.chunk.code[ej2+1] = (off2 >> 8) as u8; self.chunk.code[ej2+2] = off2 as u8;
            }
            Expr::Loop(block) => {
                let ls = self.chunk.code.len();
                let base_scope = self.scope_depth;
                self.loop_stack.push(LoopCtx { start: ls, is_for: false, base_scope, breaks: Vec::new(), continues: Vec::new() });
                for s in &block.stmts { self.compile_stmt(s, line)?; }
                self.chunk.emit_i16(Opcode::Loop, (self.chunk.code.len() - ls + 3) as i16, line);
                let exit = self.chunk.code.len();
                if let Some(ctx) = self.loop_stack.last() {
                    for &b in &ctx.breaks { let off = (exit - b - 3) as i16; self.chunk.code[b+1] = (off >> 8) as u8; self.chunk.code[b+2] = off as u8; }
                }
                self.loop_stack.pop();
            }
            Expr::Path(parts) => {
                let name = parts.join("::");
                let i = self.chunk.add_string(name);
                self.chunk.emit_u16(Opcode::GetGlobal, i, line);
            }
            Expr::OptionalChaining { object, field } => {
                self.compile_expr(object, line)?;
                let i = self.chunk.add_string(field.clone());
                self.chunk.emit_u16(Opcode::GetField, i, line);
            }
            _ => { self.chunk.emit_byte(Opcode::LoadNil, line); }
        }
        Ok(())
    }

    fn compile_binary(&mut self, op: &BinOp, left: &Expr, right: &Expr, line: u32) -> Result<(), CompileError> {
        match op {
            BinOp::And => {
                self.compile_expr(left, line)?;
                let end = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);
                self.chunk.emit_byte(Opcode::Pop, line);
                self.compile_expr(right, line)?;
                let off = (self.chunk.code.len() - end - 3) as i16;
                self.chunk.code[end+1] = (off >> 8) as u8; self.chunk.code[end+2] = off as u8;
            }
            BinOp::Or => {
                // Short-circuit OR: if left is truthy, return left; otherwise evaluate right
                self.compile_expr(left, line)?;
                let right_label = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::JmpIfFalse, 0, line);
                let end_label = self.chunk.code.len();
                self.chunk.emit_i16(Opcode::Jmp, 0, line);
                // Patch JmpIfFalse to jump here (evaluate right side)
                let right_off = (self.chunk.code.len() - right_label - 3) as i16;
                self.chunk.code[right_label+1] = (right_off >> 8) as u8; self.chunk.code[right_label+2] = right_off as u8;
                self.chunk.emit_byte(Opcode::Pop, line); // remove false left value
                self.compile_expr(right, line)?;
                // Patch Jmp to jump here (end, left was truthy)
                let end_off = (self.chunk.code.len() - end_label - 3) as i16;
                self.chunk.code[end_label+1] = (end_off >> 8) as u8; self.chunk.code[end_label+2] = end_off as u8;
            }
            _ => {
                self.compile_expr(left, line)?;
                self.compile_expr(right, line)?;
                self.chunk.emit_byte(match op {
                    BinOp::Add => Opcode::Add, BinOp::Sub => Opcode::Sub, BinOp::Mul => Opcode::Mul,
                    BinOp::Div => Opcode::Div, BinOp::Rem => Opcode::Mod, BinOp::Eq => Opcode::Eq,
                    BinOp::Ne => Opcode::Ne, BinOp::Lt => Opcode::Lt, BinOp::Le => Opcode::Le,
                    BinOp::Gt => Opcode::Gt, BinOp::Ge => Opcode::Ge, _ => Opcode::Add,
                }, line);
            }
        }
        Ok(())
    }

    /// Resolve a name to a mutable-cell upvalue slot if it is capturable as an
    /// upvalue here (either from an immediate outer local or from a chain upvalue of an
    /// enclosing closure). Returns the upvalue index, allocating it on first use.
    fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        if self.outer_locals.iter().rev().any(|l| l.name == name)
            || self.chain_upvalue_names.iter().any(|n| n == name)
        {
            Some(self.upvalue_index(name))
        } else {
            None
        }
    }

    /// Emit a read of a named variable: local slot, captured upvalue, or global.
    fn emit_read_name(&mut self, name: &str, line: u32) -> Result<(), CompileError> {
        if let Some(local) = self.locals.iter().rev().find(|l| l.name == name) {
            self.chunk.emit_bytes(Opcode::GetLocal, local.slot as u8, line);
        } else if self.upvalues.contains_key(name) {
            let idx = self.upvalues[name];
            self.chunk.emit_bytes(Opcode::GetUpvalue, idx as u8, line);
        } else if let Some(idx) = self.resolve_upvalue(name) {
            self.chunk.emit_bytes(Opcode::GetUpvalue, idx as u8, line);
        } else {
            let i = self.chunk.add_string(name.to_string());
            self.chunk.emit_u16(Opcode::GetGlobal, i, line);
        }
        Ok(())
    }

    /// Emit a write of a named variable (leaving its value on the stack).
    fn emit_write_name(&mut self, name: &str, line: u32) -> Result<(), CompileError> {
        if let Some(local) = self.locals.iter().rev().find(|l| l.name == name) {
            self.chunk.emit_bytes(Opcode::SetLocal, local.slot as u8, line);
        } else if self.upvalues.contains_key(name) {
            let idx = self.upvalues[name];
            self.chunk.emit_bytes(Opcode::SetUpvalue, idx as u8, line);
        } else if let Some(idx) = self.resolve_upvalue(name) {
            self.chunk.emit_bytes(Opcode::SetUpvalue, idx as u8, line);
        } else {
            let i = self.chunk.add_string(name.to_string());
            self.chunk.emit_u16(Opcode::SetGlobal, i, line);
        }
        Ok(())
    }

    /// Get or allocate the upvalue index for a captured outer local name. Captures are
    /// recorded in first-reference order, which is the order they land in the heap
    /// environment built at closure creation.
    fn upvalue_index(&mut self, name: &str) -> usize {
        if let Some(&i) = self.upvalues.get(name) {
            i
        } else {
            let i = self.upvalue_names.len();
            self.upvalue_names.push(name.to_string());
            self.upvalues.insert(name.to_string(), i);
            i
        }
    }

    /// Compile a closure literal, capturing (by value) the enclosing variables it
    /// references. Produces: LoadConst(<fn>), then one capture arg per upvalue, then
    /// MakeClosure <count>.
    ///
    /// A capture arg is either a `GetLocal` (the variable is a plain local of the
    /// parent) or a `GetUpvalueRef` / `GetUpvalue` (the variable is already a captured
    /// upvalue of an enclosing *closure*, so the nested closure receives the same heap
    /// cell and mutations propagate through the chain).
    fn compile_closure(&mut self, params: &[ClosureParam], body: &Expr, line: u32) -> Result<(), CompileError> {
        let mut sub = Compiler::new();
        sub.outer_locals = self.locals.clone();
        // Everything this compiler could capture (its immediate outer locals, its own
        // captures, and the transitive chain above it) is visible to the nested closure
        // as a possible upvalue-of-upvalue.
        let mut chain: Vec<String> = Vec::new();
        for l in &self.locals { chain.push(l.name.clone()); }
        chain.extend(self.upvalue_names.iter().cloned());
        chain.extend(self.chain_upvalue_names.iter().cloned());
        sub.chain_upvalue_names = chain;
        sub.scope_depth = 1;
        for p in params {
            sub.push_local(p.name.clone(), sub.locals.len(), 1);
        }
        sub.compile_expr(body, line)?;
        // Force an implicit return of the body value.
        sub.chunk.emit_byte(Opcode::Return, line);
        let func_obj = Rc::new(RefCell::new(VmFunction {
            name: "<closure>".into(), arity: params.len(), chunk: sub.chunk, max_locals: sub.max_locals,
        }));
        let const_idx = self.chunk.add_constant(Value::Function(func_obj));
        self.chunk.emit_u16(Opcode::LoadConst, const_idx, line);
        for name in &sub.upvalue_names {
            self.emit_closure_capture_arg(name, line)?;
        }
        self.chunk.emit_u16(Opcode::MakeClosure, sub.upvalue_names.len() as u16, line);
        Ok(())
    }

    /// Emit a single capture argument for the nested closure currently being built:
    /// its current value if the name is a plain local here, or its shared cell if the
    /// name is (or can become) one of this closure's own captured upvalues.
    fn emit_closure_capture_arg(&mut self, name: &str, line: u32) -> Result<(), CompileError> {
        if let Some(local) = self.locals.iter().rev().find(|l| &l.name == name) {
            self.chunk.emit_bytes(Opcode::GetLocal, local.slot as u8, line);
        } else if self.upvalues.contains_key(name) {
            let idx = self.upvalues[name];
            self.chunk.emit_bytes(Opcode::GetUpvalueRef, idx as u8, line);
        } else if self.resolve_upvalue(name).is_some() {
            // The name is an enclosing-closure upvalue this closure has not referenced
            // itself yet; capture it so we can pass the cell down.
            let idx = self.upvalue_index(name);
            self.chunk.emit_bytes(Opcode::GetUpvalueRef, idx as u8, line);
        } else {
            return Err(CompileError(format!("captured variable `{}` not in scope at closure", name)));
        }
        Ok(())
    }

    fn compile_assign(&mut self, target: &Expr, value: &Expr, line: u32) -> Result<(), CompileError> {
        self.compile_expr(value, line)?;
        match target {
            Expr::Ident(name) => self.emit_write_name(name, line)?,
            Expr::Field { object, field } => {
                self.compile_expr(object, line)?;
                self.chunk.emit_byte(Opcode::Swap, line);
                let i = self.chunk.add_string(field.clone());
                self.chunk.emit_u16(Opcode::SetField, i, line);
            }            _ => {}
        }
        Ok(())
    }

    fn compile_compound_assign(&mut self, op: &BinOp, target: &Expr, value: &Expr, line: u32) -> Result<(), CompileError> {
        match target {
            Expr::Ident(name) => {
                self.emit_read_name(name, line)?;
                self.compile_expr(value, line)?;
                self.compile_binary_op(op, line);
                self.emit_write_name(name, line)?;
            }
            Expr::Index { object, index } => {
                // arr[i] += val  →  load arr[i], apply op, store back
                self.compile_expr(object, line)?;
                self.compile_expr(index, line)?;
                self.chunk.emit_byte(Opcode::GetIndex, line);
                self.compile_expr(value, line)?;
                self.compile_binary_op(op, line);
                // store: value is on top, need arr and index below
                // re-load arr and index, then SetIndex
                self.compile_expr(object, line)?;
                self.compile_expr(index, line)?;
                self.chunk.emit_byte(Opcode::SetIndex, line);
            }
            _ => {
                // Fallback: treat as regular assign with binary op
                self.compile_expr(target, line)?;
                self.compile_expr(value, line)?;
                self.compile_binary_op(op, line);
                self.compile_assign_store(target, line)?;
            }
        }
        Ok(())
    }

    fn compile_binary_op(&mut self, op: &BinOp, line: u32) {
        self.chunk.emit_byte(match op {
            BinOp::Add => Opcode::Add, BinOp::Sub => Opcode::Sub,
            BinOp::Mul => Opcode::Mul, BinOp::Div => Opcode::Div,
            BinOp::Rem => Opcode::Mod, BinOp::BitAnd => Opcode::And,
            BinOp::BitOr => Opcode::Or, BinOp::BitXor => Opcode::Add,
            _ => Opcode::Add,
        }, line);
    }

    fn compile_assign_store(&mut self, target: &Expr, line: u32) -> Result<(), CompileError> {
        match target {
            Expr::Ident(name) => self.emit_write_name(name, line)?,
            _ => {}
        }
        Ok(())
    }


    fn compile_macro(&mut self, name: &str, args: &[Expr], line: u32) -> Result<(), CompileError> {
        match name {
            "println" | "print" => {
                for a in args { self.compile_expr(a, line)?; }
                self.chunk.emit_bytes(Opcode::PrintN, args.len() as u8, line);
                self.chunk.emit_byte(Opcode::LoadNil, line);
            }
            "panic" => {
                for a in args { self.compile_expr(a, line)?; }
                self.chunk.emit_bytes(Opcode::PrintN, args.len() as u8, line);
                self.chunk.emit_byte(Opcode::Halt, line);
            }
            _ => {
                let i = self.chunk.add_string(name.to_string());
                self.chunk.emit_u16(Opcode::GetGlobal, i, line);
                for a in args { self.compile_expr(a, line)?; }
                self.chunk.emit_bytes(Opcode::Call, args.len() as u8, line);
            }
        }
        Ok(())
    }

    pub fn into_chunk(self) -> Chunk { self.chunk }
}

//  Virtual Machine — stack-based bytecode interpreter

/// Upper bound on a single string-repeat allocation to avoid capacity-overflow
/// panics and absurd allocations from hostile programs (e.g. `"ab" * 10^18`).
const MAX_STRING_ALLOC: usize = 1 << 30; // 1 GiB

struct Frame { ip: usize, slot: usize }

pub struct Vm {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    /// Stack of closure environments, aligned with active `run_loop` invocations.
    /// Each time a closure is called its environment is pushed; `GetUpvalue` /
    /// `SetUpvalue` in the closure body operate on `env_stack.last()`.
    env_stack: Vec<Rc<RefCell<Vec<Value>>>>,
    globals: HashMap<String, Value>,
    debug: bool,
    current_chunk: Chunk,
}

impl Vm {
    pub fn new(debug: bool) -> Self {
        Self { stack: Vec::with_capacity(512), frames: Vec::with_capacity(64), env_stack: Vec::new(), globals: HashMap::new(), debug, current_chunk: Chunk::new() }
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<Value, String> {
        self.current_chunk = chunk.clone();
        self.frames.push(Frame { ip: 0, slot: 0 });
        self.run_loop(chunk)
    }

    fn run_loop(&mut self, chunk: &Chunk) -> Result<Value, String> {
        let mut ip = 0;
        let code = &chunk.code;
        let base = self.frames.last().map(|f| f.slot).unwrap_or(0);

        loop {
            if ip >= code.len() { break; }
            let op_byte = code[ip]; ip += 1;
            let op = Opcode::from_u8(op_byte).ok_or_else(|| format!("Unknown opcode: {}", op_byte))?;

            if self.debug { chunk.disassemble_instruction(ip - 1); }

            match op {
                Opcode::LoadConst => { let i = chunk.read_u16(ip) as usize; ip += 2; if i >= chunk.constants.len() { return Err(format!("LoadConst {} out of bounds (len={})", i, chunk.constants.len())); } self.stack.push(chunk.constants[i].clone()); }
                Opcode::LoadInt => { let v = chunk.read_i64(ip); ip += 8; self.stack.push(Value::Int(v)); }
                Opcode::LoadFloat => {
                    let mut b = [0u8; 8]; for i in 0..8 { b[i] = code[ip+i]; } ip += 8;
                    self.stack.push(Value::Float(f64::from_be_bytes(b)));
                }
                Opcode::LoadStr => { let i = chunk.read_u16(ip); ip += 2; self.stack.push(Value::Str(chunk.strings[i as usize].clone())); }
                Opcode::LoadTrue => { self.stack.push(Value::Bool(true)); }
                Opcode::LoadFalse => { self.stack.push(Value::Bool(false)); }
                Opcode::LoadNil => { self.stack.push(Value::Nil); }
                Opcode::LoadArrayLen => {
                    let v = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(match v { Value::Array(a) => Value::Int(a.len() as i64), _ => Value::Int(0) });
                }
                Opcode::Pop => { self.stack.pop(); }
                Opcode::Dup => { let v = self.stack.last().cloned().unwrap_or(Value::Nil); self.stack.push(v); }
                Opcode::Swap => { let a = self.stack.pop().unwrap_or(Value::Nil); let b = self.stack.pop().unwrap_or(Value::Nil); self.stack.push(a); self.stack.push(b); }
                Opcode::GetLocal => { let s = code[ip] as usize; ip += 1; let target = base + s; self.stack.push(if target < self.stack.len() { self.stack[target].clone() } else { Value::Nil }); }
                Opcode::SetLocal => { let s = code[ip] as usize; ip += 1; let v = self.stack.last().cloned().unwrap_or(Value::Nil); let target = base + s; if target >= self.stack.len() { self.stack.resize(target + 1, Value::Nil); } self.stack[target] = v; }
                Opcode::GetGlobal => { let i = chunk.read_u16(ip); ip += 2; self.stack.push(self.globals.get(&chunk.strings[i as usize]).cloned().unwrap_or(Value::Nil)); }
                Opcode::SetGlobal => { let i = chunk.read_u16(ip); ip += 2; let n = chunk.strings[i as usize].clone(); let v = self.stack.last().cloned().unwrap_or(Value::Nil); self.globals.insert(n, v); }
                Opcode::Add => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => { let v = a.checked_add(*b).ok_or_else(|| format!("Integer overflow: {} + {}", a, b))?; self.stack.push(Value::Int(v)); }
                        (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a + b)),
                        (Value::Int(a), Value::Float(b)) => self.stack.push(Value::Float(*a as f64 + b)),
                        (Value::Float(a), Value::Int(b)) => self.stack.push(Value::Float(a + *b as f64)),
                        (Value::Str(a), Value::Str(b)) => self.stack.push(Value::Str(format!("{}{}", a, b))),
                        (Value::Array(a), Value::Array(b)) => { let mut r = a.clone(); r.extend(b.iter().cloned()); self.stack.push(Value::Array(r)); }
                        _ => return Err(format!("Cannot add {:?} and {:?}", l, r)),
                    }
                }
                Opcode::Sub => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => { let v = a.checked_sub(*b).ok_or_else(|| format!("Integer overflow: {} - {}", a, b))?; Value::Int(v) }
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 - b),
                        (Value::Float(a), Value::Int(b)) => Value::Float(a - *b as f64),
                        _ => return Err(format!("Cannot subtract {:?} from {:?}", r, l)),
                    });
                }
                Opcode::Mul => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => { let v = a.checked_mul(*b).ok_or_else(|| format!("Integer overflow: {} * {}", a, b))?; self.stack.push(Value::Int(v)); }
                        (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a * b)),
                        (Value::Int(a), Value::Float(b)) => self.stack.push(Value::Float(*a as f64 * b)),
                        (Value::Float(a), Value::Int(b)) => self.stack.push(Value::Float(a * *b as f64)),
                        (Value::Str(a), Value::Int(b)) => {
                            if *b < 0 { return Err(format!("Cannot repeat string by negative count {}", b)); }
                            let count = *b as usize;
                            let total = a.len().checked_mul(count)
                                .ok_or_else(|| format!("String repeat size overflow: len={} count={}", a.len(), count))?;
                            if total > MAX_STRING_ALLOC {
                                return Err(format!("String repeat too large: {} bytes (limit {})", total, MAX_STRING_ALLOC));
                            }
                            self.stack.push(Value::Str(a.repeat(count)));
                        }
                        _ => return Err(format!("Cannot multiply {:?} and {:?}", l, r)),
                    }
                }
                Opcode::Div => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => { let v = a.checked_div(*b).ok_or_else(|| format!("Integer division error: {} / {}", a, b))?; self.stack.push(Value::Int(v)); }
                        (Value::Float(a), Value::Float(b)) => self.stack.push(Value::Float(a / b)),
                        (Value::Int(a), Value::Float(b)) => self.stack.push(Value::Float(*a as f64 / b)),
                        (Value::Float(a), Value::Int(b)) => self.stack.push(Value::Float(a / *b as f64)),
                        _ => return Err(format!("Cannot divide {:?} by {:?}", l, r)),
                    }
                }
                Opcode::Mod => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => { let v = a.checked_rem(*b).ok_or_else(|| format!("Integer modulo error: {} % {}", a, b))?; self.stack.push(Value::Int(v)); }
                        _ => return Err(format!("Cannot modulo {:?} by {:?}", l, r)),
                    }
                }
                Opcode::Negate => {
                    let v = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(match v { Value::Int(n) => { let r = n.checked_neg().ok_or_else(|| format!("Integer overflow in negation: {}", n))?; Value::Int(r) }, Value::Float(n) => Value::Float(-n), _ => return Err(format!("Cannot negate {:?}", v)) });
                }
                Opcode::Eq => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(Value::Bool(l == r));
                }
                Opcode::Ne => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(Value::Bool(l != r));
                }
                Opcode::Lt => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(Value::Bool(match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => a < b, (Value::Float(a), Value::Float(b)) => a < b,
                        (Value::Int(a), Value::Float(b)) => (*a as f64) < *b, (Value::Float(a), Value::Int(b)) => *a < (*b as f64),
                        (Value::Str(a), Value::Str(b)) => a < b, _ => return Err(format!("Cannot compare {:?} < {:?}", l, r)),
                    }));
                }
                Opcode::Le => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(Value::Bool(match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => a <= b, (Value::Float(a), Value::Float(b)) => a <= b,
                        (Value::Int(a), Value::Float(b)) => (*a as f64) <= *b, (Value::Float(a), Value::Int(b)) => *a <= (*b as f64),
                        _ => return Err(format!("Cannot compare {:?} <= {:?}", l, r)),
                    }));
                }
                Opcode::Gt => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(Value::Bool(match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => a > b, (Value::Float(a), Value::Float(b)) => a > b,
                        (Value::Int(a), Value::Float(b)) => (*a as f64) > *b, (Value::Float(a), Value::Int(b)) => *a > (*b as f64),
                        _ => return Err(format!("Cannot compare {:?} > {:?}", l, r)),
                    }));
                }
                Opcode::Ge => {
                    let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(Value::Bool(match (&l, &r) {
                        (Value::Int(a), Value::Int(b)) => a >= b, (Value::Float(a), Value::Float(b)) => a >= b,
                        (Value::Int(a), Value::Float(b)) => (*a as f64) >= *b, (Value::Float(a), Value::Int(b)) => *a >= (*b as f64),
                        _ => return Err(format!("Cannot compare {:?} >= {:?}", l, r)),
                    }));
                }
                Opcode::Not => { let v = self.stack.pop().unwrap_or(Value::Nil); self.stack.push(Value::Bool(!v.is_truthy())); }
                Opcode::And => { let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil); self.stack.push(Value::Bool(l.is_truthy() && r.is_truthy())); }
                Opcode::Or => { let r = self.stack.pop().unwrap_or(Value::Nil); let l = self.stack.pop().unwrap_or(Value::Nil); self.stack.push(Value::Bool(l.is_truthy() || r.is_truthy())); }
                Opcode::Jmp => { let o = chunk.read_i16(ip); ip += 2; ip = (ip as i64 + o as i64) as usize; }
                Opcode::JmpIfFalse => {
                    let o = chunk.read_i16(ip); ip += 2;
                    let v = self.stack.last().cloned().unwrap_or(Value::Nil);
                    if !v.is_truthy() { ip = (ip as i64 + o as i64) as usize; }
                }
                Opcode::Loop => { let o = chunk.read_i16(ip); ip += 2; ip = (ip as i64 - o as i64) as usize; }
                Opcode::Call => {
                    let argc = code[ip] as usize; ip += 1;
                    let fv = self.stack.pop().unwrap_or(Value::Nil);
                    // Native functions are Rust callbacks stored in `globals`.
                    if let Value::Native(nf) = &fv {
                        let mut args: Vec<Value> = (0..argc).map(|_| self.stack.pop().unwrap_or(Value::Nil)).collect();
                        args.reverse();
                        self.stack.push(nf(&args));
                    } else {
                    // Resolve the callable to its function object plus (for closures) a
                    // heap environment holding captured variables.
                    let (f, env): (Rc<RefCell<VmFunction>>, Option<Rc<RefCell<Vec<Value>>>>) =
                        match &fv {
                            Value::Function(f) => (f.clone(), None),
                            Value::Closure(c) => { let c = c.borrow().clone(); (c.function, Some(c.env)) }
                            _ => return Err(format!("Cannot call {:?}", fv)),
                        };
                    let fref = f.borrow();
                    if argc != fref.arity { return Err(format!("Expected {} args for `{}`, got {}", fref.arity, fref.name, argc)); }
                    if self.frames.len() >= 64 { return Err("Stack overflow".into()); }
                    let new_base = self.stack.len() - argc;
                    let fc = fref.chunk.clone();
                    let ml = fref.max_locals;
                    drop(fref);
                    // Pre-fill local slots with nil
                    for _ in argc..ml { self.stack.push(Value::Nil); }
                    if let Some(env) = &env { self.env_stack.push(env.clone()); }
                    self.frames.push(Frame { ip: 0, slot: new_base });
                    let result = self.run_loop(&fc)?;
                    self.frames.pop();
                    if let Some(_) = &env { self.env_stack.pop(); }
                    self.stack.truncate(new_base);
                    self.stack.push(result);
                    }
                }
                Opcode::MakeClosure => {
                    let n = chunk.read_u16(ip) as usize; ip += 2;
                    let mut captured: Vec<Value> = (0..n).map(|_| self.stack.pop().unwrap_or(Value::Nil)).collect();
                    captured.reverse(); // capture order: index 0 was pushed first
                    let f = self.stack.pop().unwrap_or(Value::Nil);
                    match f {
                        Value::Function(func) => {
                            // Every captured slot becomes a shared cell. If the value is
                            // already a `Ref` (a `GetUpvalueRef` from an enclosing
                            // closure), reuse the same cell so the chain aliases it;
                            // otherwise wrap the plain value in a fresh cell.
                            let cells: Vec<Value> = captured.into_iter().map(|c| match c {
                                Value::Ref(_) => c,
                                other => Value::Ref(Rc::new(RefCell::new(other))),
                            }).collect();
                            let env = Rc::new(RefCell::new(cells));
                            self.stack.push(Value::Closure(Rc::new(RefCell::new(VmClosure { function: func, env }))));
                        }
                        _ => return Err(format!("MakeClosure: expected a function object on the stack, found {:?}", f)),
                    }
                }
                Opcode::GetUpvalue => {
                    let idx = code[ip] as usize; ip += 1;
                    let env = self.env_stack.last().ok_or_else(|| "GetUpvalue with no closure environment".to_string())?;
                    let item = env.borrow().get(idx).cloned().unwrap_or(Value::Nil);
                    match item {
                        Value::Ref(cell) => self.stack.push(cell.borrow().clone()),
                        other => self.stack.push(other),
                    }
                }
                Opcode::SetUpvalue => {
                    let idx = code[ip] as usize; ip += 1;
                    let v = self.stack.last().cloned().unwrap_or(Value::Nil);
                    let env = self.env_stack.last().ok_or_else(|| "SetUpvalue with no closure environment".to_string())?;
                    let item = env.borrow().get(idx).cloned();
                    if let Some(Value::Ref(cell)) = item {
                        *cell.borrow_mut() = v;
                    } else {
                        let mut b = env.borrow_mut();
                        if idx >= b.len() { b.resize(idx + 1, Value::Nil); }
                        b[idx] = v;
                    }
                }
                Opcode::GetUpvalueRef => {
                    let idx = code[ip] as usize; ip += 1;
                    let env = self.env_stack.last().ok_or_else(|| "GetUpvalueRef with no closure environment".to_string())?;
                    let item = env.borrow().get(idx).cloned().unwrap_or(Value::Nil);
                    // The shared cell (a `Ref`) is what gets captured by MakeClosure so
                    // a nested closure aliases this same slot. If it is somehow a plain
                    // value, wrap it so sharing still works.
                    match item {
                        Value::Ref(cell) => self.stack.push(Value::Ref(cell)),
                        other => self.stack.push(Value::Ref(Rc::new(RefCell::new(other)))),
                    }
                }
                Opcode::Return => { return Ok(self.stack.pop().unwrap_or(Value::Nil)); }
                Opcode::MakeArray => {
                    let c = chunk.read_u16(ip); ip += 2;
                    let mut elems: Vec<Value> = (0..c).map(|_| self.stack.pop().unwrap_or(Value::Nil)).collect();
                    elems.reverse(); self.stack.push(Value::Array(elems));
                }
                Opcode::GetIndex => {
                    let idx = self.stack.pop().unwrap_or(Value::Nil);
                    let obj = self.stack.pop().unwrap_or(Value::Nil);
                    match (&obj, &idx) {
                        (Value::Array(arr), Value::Int(i)) => {
                            let ix = if *i < 0 { (arr.len() as i64 + i) as usize } else { *i as usize };
                            self.stack.push(arr.get(ix).cloned().unwrap_or(Value::Nil));
                        }
                        _ => return Err(format!("Cannot index {:?} with {:?}", obj, idx)),
                    }
                }
                Opcode::SetIndex => {
                    let val = self.stack.pop().unwrap_or(Value::Nil);
                    let idx = self.stack.pop().unwrap_or(Value::Nil);
                    let mut obj = self.stack.pop().unwrap_or(Value::Nil);
                    if let (Value::Array(arr), Value::Int(i)) = (&mut obj, &idx) {
                        let ix = *i as usize; if ix < arr.len() { arr[ix] = val.clone(); }
                    }
                    self.stack.push(val);
                }
                Opcode::GetField => {
                    let i = chunk.read_u16(ip); ip += 2;
                    let fname = &chunk.strings[i as usize];
                    let obj = self.stack.pop().unwrap_or(Value::Nil);
                    self.stack.push(match obj {
                        Value::Object(o) => o.borrow().fields.get(fname).cloned().unwrap_or(Value::Nil),
                        _ => Value::Nil,
                    });
                }
                Opcode::SetField => {
                    let i = chunk.read_u16(ip); ip += 2;
                    let fname = chunk.strings[i as usize].clone();
                    let val = self.stack.pop().unwrap_or(Value::Nil);
                    let mut obj = self.stack.pop().unwrap_or(Value::Nil);
                    if let Value::Object(o) = &mut obj { o.borrow_mut().fields.insert(fname, val.clone()); }
                    self.stack.push(val);
                }
                Opcode::NewObject => {
                    let ni = chunk.read_u16(ip); let fc = chunk.read_u16(ip + 2); ip += 4;
                    let cname = chunk.strings[ni as usize].clone();
                    let mut fields = HashMap::new();
                    let mut names: Vec<Value> = (0..fc).map(|_| self.stack.pop().unwrap_or(Value::Nil)).collect();
                    let vals: Vec<Value> = (0..fc).map(|_| self.stack.pop().unwrap_or(Value::Nil)).collect();
                    for (n, v) in names.drain(..).zip(vals.iter()) {
                        if let Value::Str(s) = n { fields.insert(s, v.clone()); }
                    }
                    self.stack.push(Value::Object(Rc::new(RefCell::new(Object { class_name: cname, fields }))));
                }
                Opcode::Print => {
                    let v = self.stack.pop().unwrap_or(Value::Nil);
                    match &v { Value::Str(s) => println!("{}", s), _ => println!("{}", v) }
                    self.stack.push(Value::Nil);
                }
                Opcode::PrintN => {
                    let argc = code[ip] as usize; ip += 1;
                    let mut parts: Vec<String> = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        let v = self.stack.pop().unwrap_or(Value::Nil);
                        parts.push(format!("{}", v));
                    }
                    parts.reverse();
                    println!("{}", parts.join(""));
                    self.stack.push(Value::Nil);
                }
                Opcode::Halt => { return Ok(self.stack.pop().unwrap_or(Value::Nil)); }
            }
        }
        Ok(self.stack.pop().unwrap_or(Value::Nil))
    }
}

//  Public API

pub fn compile_and_run(program: &Program, debug: bool) -> Result<Value, String> {
    let mut compiler = Compiler::new();
    compiler.compile(program).map_err(|e| e.0)?;
    if debug { compiler.chunk.disassemble("main"); }
    let mut vm = Vm::new(debug);
    vm.run(&compiler.chunk)
}

pub fn compile_program(program: &Program, debug: bool) -> Result<Chunk, String> {
    let mut compiler = Compiler::new();
    compiler.compile(program).map_err(|e| e.0)?;
    if debug { compiler.chunk.disassemble("main"); }
    Ok(compiler.chunk)
}

pub fn run_chunk(chunk: &Chunk, debug: bool) -> Result<Value, String> {
    let mut vm = Vm::new(debug);
    vm.run(chunk)
}
