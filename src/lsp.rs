// ─── Ruva LSP Server ────────────────────────────────────────────────────────
// Language Server Protocol implementation for the Ruva language.
// Provides: text document sync, hover info, go-to-definition, completion.

use crate::ast::*;
use crate::json_protocol::*;
use crate::parser::Parser;
use crate::typecheck::TypeChecker;
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};

// ─── LSP Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Clone)]
pub struct TextRange {
    pub start: Position,
    pub end: Position,
}

impl TextRange {
    #[allow(dead_code)]
    pub fn end_pos(&self) -> Position {
        self.end.clone()
    }
}

#[derive(Debug, Clone)]
pub struct TextDocumentItem {
    pub uri: String,
    #[allow(dead_code)]
    pub language_id: String,
    pub version: i64,
    pub text: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i64,
}

#[derive(Debug, Clone)]
pub struct TextDocumentContentChangeEvent {
    pub range: Option<TextRange>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: i64, // CompletionItemKind
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HoverContent {
    pub contents: String,
    pub range: Option<TextRange>,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub uri: String,
    pub range: TextRange,
}

#[derive(Debug, Clone)]
pub struct DiagnosticInfo {
    pub range: TextRange,
    pub severity: i64,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub params: Vec<(String, String)>,
    pub return_type: Option<String>,
}

// ─── Symbol Index ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SymbolLocation {
    pub uri: String,
    pub line: usize,
    pub character: usize,
    pub length: usize,
    #[allow(dead_code)]
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Definition,
    Usage,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolIndex {
    /// symbol name -> locations where it is defined
    pub definitions: HashMap<String, Vec<SymbolLocation>>,
    /// symbol name -> locations where it is used
    pub usages: HashMap<String, Vec<SymbolLocation>>,
}

// ─── Document Store ─────────────────────────────────────────────────────────

pub struct DocumentStore {
    documents: HashMap<String, String>,
    versions: HashMap<String, i64>,
    parsed: HashMap<String, Program>,
    symbol_index: HashMap<String, SymbolIndex>,
    /// Parse/lexer errors per document, for diagnostic reporting
    parse_errors: HashMap<String, DiagnosticInfo>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            versions: HashMap::new(),
            parsed: HashMap::new(),
            symbol_index: HashMap::new(),
            parse_errors: HashMap::new(),
        }
    }

    /// Maximum document size to prevent DoS (1MB)
    const MAX_DOCUMENT_SIZE: usize = 1024 * 1024;

    pub fn open(&mut self, item: TextDocumentItem) {
        // Security: reject documents that are too large
        if item.text.len() > Self::MAX_DOCUMENT_SIZE {
            eprintln!("Warning: Document {} too large ({} bytes), limiting to {} bytes",
                item.uri, item.text.len(), Self::MAX_DOCUMENT_SIZE);
            let truncated: String = item.text.chars().take(Self::MAX_DOCUMENT_SIZE).collect();
            self.versions.insert(item.uri.clone(), item.version);
            self.documents.insert(item.uri.clone(), truncated);
        } else {
            self.versions.insert(item.uri.clone(), item.version);
            self.documents.insert(item.uri.clone(), item.text.clone());
        }
        self.parse_document(&item.uri);
    }

    pub fn change(&mut self, uri: &str, changes: Vec<TextDocumentContentChangeEvent>) {
        if let Some(doc) = self.documents.get_mut(uri) {
            for change in changes {
                match change.range {
                    Some(range) => {
                        // Incremental update: apply range-based edit
                        Self::apply_incremental_change(doc, &range, &change.text);
                    }
                    None => {
                        // Full document replacement (no range = entire document)
                        *doc = change.text;
                    }
                }
            }
            self.parse_document(uri);
        }
    }

    /// Apply an incremental text change by replacing the text at the given range.
    /// This converts the range to byte offsets, replaces the text, and updates the document.
    fn apply_incremental_change(doc: &mut String, range: &TextRange, new_text: &str) {
        let lines: Vec<&str> = doc.lines().collect();
        let line_count = lines.len();

        // Clamp range to document bounds
        let start_line = range.start.line.min(line_count.saturating_sub(1));
        let start_char = range.start.character;
        let end_line = range.end.line.min(line_count.saturating_sub(1));
        let end_char = range.end.character;

        // Convert line/col to byte offset for start
        let start_byte = Self::position_to_byte_offset(&lines, start_line, start_char);
        // Convert line/col to byte offset for end
        let end_byte = Self::position_to_byte_offset(&lines, end_line, end_char);

        // Replace the range with new text
        let before: String = doc.chars().take(start_byte).collect();
        let after: String = doc.chars().skip(end_byte).collect();
        *doc = format!("{}{}{}", before, new_text, after);
    }

    /// Convert a (line, character) position to a byte offset in the document.
    fn position_to_byte_offset(lines: &[&str], target_line: usize, target_char: usize) -> usize {
        let mut offset = 0;
        for (i, line) in lines.iter().enumerate() {
            if i == target_line {
                let char_count = line.chars().count();
                let clamped_char = target_char.min(char_count);
                // Walk chars to find byte position
                for (ci, ch) in line.chars().enumerate() {
                    if ci >= clamped_char {
                        break;
                    }
                    offset += ch.len_utf8();
                }
                return offset;
            }
            // +1 for the newline character (use 1 for \n)
            offset += line.len() + 1;
        }
        offset // past end of document
    }

    #[allow(dead_code)]
    pub fn get_version(&self, uri: &str) -> Option<i64> {
        self.versions.get(uri).copied()
    }

    pub fn set_version(&mut self, uri: &str, version: i64) {
        self.versions.insert(uri.to_string(), version);
    }

    pub fn close(&mut self, uri: &str) {
        self.documents.remove(uri);
        self.parsed.remove(uri);
        self.symbol_index.remove(uri);
        self.parse_errors.remove(uri);
    }

    pub fn get_text(&self, uri: &str) -> Option<&str> {
        self.documents.get(uri).map(|s| s.as_str())
    }

    pub fn get_parsed(&self, uri: &str) -> Option<&Program> {
        self.parsed.get(uri)
    }

    fn parse_document(&mut self, uri: &str) {
        if let Some(text) = self.documents.get(uri) {
            let text_clone = text.clone();
            let uri_clone = uri.to_string();
            match Parser::new(&text_clone) {
                Ok(mut parser) => match parser.parse_program() {
                    Ok(program) => {
                        let index = Self::build_symbol_index(&program, &uri_clone, &text_clone);
                        self.symbol_index.insert(uri_clone, index);
                        self.parsed.insert(uri.to_string(), program);
                        self.parse_errors.remove(uri);
                    }
                    Err(e) => {
                        // Parse error — store it as a diagnostic so LSP reports it
                        let msg = e.to_string();
                        let (line, col) = parse_error_location(&msg);
                        self.parsed.remove(uri);
                        self.symbol_index.remove(uri);
                        // Store parse error for diagnostics
                        self.parse_errors.insert(uri.to_string(), DiagnosticInfo {
                            range: TextRange {
                                start: Position { line: line.saturating_sub(1), character: col.saturating_sub(1) },
                                end: Position { line: line.saturating_sub(1), character: col },
                            },
                            severity: 1,
                            message: msg,
                            source: "ruva".to_string(),
                        });
                    }
                },
                Err(e) => {
                    // Lexer error — store it as a diagnostic
                    let msg = e.to_string();
                    self.parsed.remove(uri);
                    self.symbol_index.remove(uri);
                    self.parse_errors.insert(uri.to_string(), DiagnosticInfo {
                        range: TextRange {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 0 },
                        },
                        severity: 1,
                        message: msg,
                        source: "ruva".to_string(),
                    });
                }
            }
        }
    }

    fn build_symbol_index(program: &Program, uri: &str, source: &str) -> SymbolIndex {
        let mut index = SymbolIndex::default();
        let lines: Vec<&str> = source.lines().collect();

        for item in &program.items {
            Self::collect_index_from_item(item, uri, &lines, &mut index);
        }

        index
    }

    fn collect_index_from_item(item: &Item, uri: &str, lines: &[&str], index: &mut SymbolIndex) {
        match item {
            Item::Function(f) => {
                // Register the function name as a definition
                let def = SymbolLocation {
                    uri: uri.to_string(),
                    line: f.span.line.saturating_sub(1),
                    character: f.span.col.saturating_sub(1),
                    length: f.name.len(),
                    kind: SymbolKind::Definition,
                };
                index.definitions.entry(f.name.clone()).or_default().push(def);

                // Scan body for usages
                Self::collect_usages_from_block(&f.body, uri, lines, index);

                // Also scan parameter types for usages of type names
                for param in &f.params {
                    Self::collect_usages_from_type(&param.ty, uri, lines, index);
                }
                if let Some(ref ret) = f.return_type {
                    Self::collect_usages_from_type(ret, uri, lines, index);
                }
            }
            Item::Struct(s) => {
                let def = SymbolLocation {
                    uri: uri.to_string(),
                    line: s.span.line.saturating_sub(1),
                    character: s.span.col.saturating_sub(1),
                    length: s.name.len(),
                    kind: SymbolKind::Definition,
                };
                index.definitions.entry(s.name.clone()).or_default().push(def);
                for field in &s.fields {
                    Self::collect_usages_from_type(&field.ty, uri, lines, index);
                }
            }
            Item::Class(c) => {
                let def = SymbolLocation {
                    uri: uri.to_string(),
                    line: c.span.line.saturating_sub(1),
                    character: c.span.col.saturating_sub(1),
                    length: c.name.len(),
                    kind: SymbolKind::Definition,
                };
                index.definitions.entry(c.name.clone()).or_default().push(def);
                for field in &c.fields {
                    Self::collect_usages_from_type(&field.ty, uri, lines, index);
                }
                for method in &c.methods {
                    Self::collect_usages_from_block(&method.body, uri, lines, index);
                }
            }
            Item::Enum(e) => {
                let def = SymbolLocation {
                    uri: uri.to_string(),
                    line: e.span.line.saturating_sub(1),
                    character: e.span.col.saturating_sub(1),
                    length: e.name.len(),
                    kind: SymbolKind::Definition,
                };
                index.definitions.entry(e.name.clone()).or_default().push(def);
            }
            Item::Trait(t) => {
                let def = SymbolLocation {
                    uri: uri.to_string(),
                    line: t.span.line.saturating_sub(1),
                    character: t.span.col.saturating_sub(1),
                    length: t.name.len(),
                    kind: SymbolKind::Definition,
                };
                index.definitions.entry(t.name.clone()).or_default().push(def);
                for method in &t.methods {
                    if let Some(ref body) = method.default_body {
                        Self::collect_usages_from_block(body, uri, lines, index);
                    }
                }
            }
            Item::TypeAlias(ta) => {
                let def = SymbolLocation {
                    uri: uri.to_string(),
                    line: 0,
                    character: 0,
                    length: ta.name.len(),
                    kind: SymbolKind::Definition,
                };
                index.definitions.entry(ta.name.clone()).or_default().push(def);
                Self::collect_usages_from_type(&ta.ty, uri, lines, index);
            }
            Item::Module(m) => {
                let def = SymbolLocation {
                    uri: uri.to_string(),
                    line: 0,
                    character: 0,
                    length: m.name.len(),
                    kind: SymbolKind::Definition,
                };
                index.definitions.entry(m.name.clone()).or_default().push(def);
                if let Some(ref body) = m.body {
                    for inner in body {
                        Self::collect_index_from_item(inner, uri, lines, index);
                    }
                }
            }
            Item::Impl(imp) => {
                for method in &imp.methods {
                    Self::collect_usages_from_block(&method.body, uri, lines, index);
                }
            }
            Item::Import(imp) => {
                // Register import path components as usages
                let parts: Vec<&str> = imp.path.split("::").collect();
                for part in &parts {
                    if !part.is_empty() {
                        Self::scan_line_usages(part, uri, lines, index);
                    }
                }
                if let Some(ref items) = imp.items {
                    for item_name in items {
                        Self::scan_line_usages(item_name, uri, lines, index);
                    }
                }
            }
            Item::Use(u) => {
                for part in &u.path {
                    Self::scan_line_usages(part, uri, lines, index);
                }
                for item in &u.selective {
                    Self::scan_line_usages(&item.name, uri, lines, index);
                }
            }
            _ => {}
        }
    }

    fn collect_usages_from_expr(expr: &Expr, uri: &str, lines: &[&str], index: &mut SymbolIndex) {
        match expr {
            Expr::Ident(name) => {
                Self::scan_line_usages(name, uri, lines, index);
            }
            Expr::Path(parts) => {
                for part in parts {
                    Self::scan_line_usages(part, uri, lines, index);
                }
            }
            Expr::Binary { left, right, .. } => {
                Self::collect_usages_from_expr(left, uri, lines, index);
                Self::collect_usages_from_expr(right, uri, lines, index);
            }
            Expr::Unary { expr, .. } => {
                Self::collect_usages_from_expr(expr, uri, lines, index);
            }
            Expr::Call { function, args } => {
                Self::collect_usages_from_expr(function, uri, lines, index);
                for arg in args {
                    Self::collect_usages_from_expr(arg, uri, lines, index);
                }
            }
            Expr::MethodCall { object, args, .. } => {
                Self::collect_usages_from_expr(object, uri, lines, index);
                for arg in args {
                    Self::collect_usages_from_expr(arg, uri, lines, index);
                }
            }
            Expr::Field { object, .. } => {
                Self::collect_usages_from_expr(object, uri, lines, index);
            }
            Expr::Index { object, index: idx } => {
                Self::collect_usages_from_expr(object, uri, lines, index);
                Self::collect_usages_from_expr(idx, uri, lines, index);
            }
            Expr::Assign { target, value } => {
                Self::collect_usages_from_expr(target, uri, lines, index);
                Self::collect_usages_from_expr(value, uri, lines, index);
            }
            Expr::CompoundAssign { target, value, .. } => {
                Self::collect_usages_from_expr(target, uri, lines, index);
                Self::collect_usages_from_expr(value, uri, lines, index);
            }
            Expr::Block(block) => {
                Self::collect_usages_from_block(block, uri, lines, index);
            }
            Expr::If { condition, then_body, else_body } => {
                Self::collect_usages_from_expr(condition, uri, lines, index);
                Self::collect_usages_from_block(then_body, uri, lines, index);
                if let Some(ref else_b) = else_body {
                    Self::collect_usages_from_expr(else_b, uri, lines, index);
                }
            }
            Expr::Match { expr: scrutinee, arms } => {
                Self::collect_usages_from_expr(scrutinee, uri, lines, index);
                for arm in arms {
                    Self::collect_usages_from_expr(&arm.body, uri, lines, index);
                    if let Some(ref guard) = arm.guard {
                        Self::collect_usages_from_expr(guard, uri, lines, index);
                    }
                }
            }
            Expr::Closure { body, .. } => {
                Self::collect_usages_from_expr(body, uri, lines, index);
            }
            Expr::Reference { expr, .. } => {
                Self::collect_usages_from_expr(expr, uri, lines, index);
            }
            Expr::Try(inner) => {
                Self::collect_usages_from_expr(inner, uri, lines, index);
            }
            Expr::Cast { expr, .. } => {
                Self::collect_usages_from_expr(expr, uri, lines, index);
            }
            Expr::Macro { args, .. } => {
                for arg in args {
                    Self::collect_usages_from_expr(arg, uri, lines, index);
                }
            }
            Expr::StructLiteral { name, fields } => {
                Self::collect_usages_from_expr(name, uri, lines, index);
                for (_, val) in fields {
                    Self::collect_usages_from_expr(val, uri, lines, index);
                }
            }
            Expr::Array(items) => {
                for item in items {
                    Self::collect_usages_from_expr(item, uri, lines, index);
                }
            }
            Expr::Tuple(items) => {
                for item in items {
                    Self::collect_usages_from_expr(item, uri, lines, index);
                }
            }
            Expr::Range { start, end, .. } => {
                Self::collect_usages_from_expr(start, uri, lines, index);
                Self::collect_usages_from_expr(end, uri, lines, index);
            }
            Expr::Loop(block) => {
                Self::collect_usages_from_block(block, uri, lines, index);
            }

            Expr::Deref(inner) => {
                Self::collect_usages_from_expr(inner, uri, lines, index);
            }
            Expr::Move(inner) => {
                Self::collect_usages_from_expr(inner, uri, lines, index);
            }
            Expr::VecLit(items) => {
                for item in items {
                    Self::collect_usages_from_expr(item, uri, lines, index);
                }
            }
            _ => {} // Literals, Self_ etc. don't reference named symbols
        }
    }

    fn collect_usages_from_block(block: &Block, uri: &str, lines: &[&str], index: &mut SymbolIndex) {
        for stmt in &block.stmts {
            Self::collect_usages_from_stmt(stmt, uri, lines, index);
        }
        if let Some(ref expr) = block.expr {
            Self::collect_usages_from_expr(expr, uri, lines, index);
        }
    }

    fn collect_usages_from_stmt(stmt: &Stmt, uri: &str, lines: &[&str], index: &mut SymbolIndex) {
        match stmt {
            Stmt::Let { value, .. } => {
                Self::collect_usages_from_expr(value, uri, lines, index);
            }
            Stmt::Expr(expr) => {
                Self::collect_usages_from_expr(expr, uri, lines, index);
            }
            Stmt::Return(Some(expr)) => {
                Self::collect_usages_from_expr(expr, uri, lines, index);
            }
            Stmt::If { condition, then_body, else_body } => {
                Self::collect_usages_from_expr(condition, uri, lines, index);
                Self::collect_usages_from_block(then_body, uri, lines, index);
                if let Some(ref eb) = else_body {
                    match eb {
                        ElseKind::If(e, b) => {
                            Self::collect_usages_from_expr(e, uri, lines, index);
                            Self::collect_usages_from_block(b, uri, lines, index);
                        }
                        ElseKind::Else(b) => {
                            Self::collect_usages_from_block(b, uri, lines, index);
                        }
                    }
                }
            }
            Stmt::For { iterable, body, .. } => {
                Self::collect_usages_from_expr(iterable, uri, lines, index);
                Self::collect_usages_from_block(body, uri, lines, index);
            }
            Stmt::While { condition, body } => {
                Self::collect_usages_from_expr(condition, uri, lines, index);
                Self::collect_usages_from_block(body, uri, lines, index);
            }
            Stmt::WhileLet { value, body, .. } => {
                Self::collect_usages_from_expr(value, uri, lines, index);
                Self::collect_usages_from_block(body, uri, lines, index);
            }
            Stmt::Loop(body) => {
                Self::collect_usages_from_block(body, uri, lines, index);
            }
            Stmt::Break(Some(expr)) => {
                Self::collect_usages_from_expr(expr, uri, lines, index);
            }
            Stmt::Match { expr, arms } => {
                Self::collect_usages_from_expr(expr, uri, lines, index);
                for arm in arms {
                    Self::collect_usages_from_expr(&arm.body, uri, lines, index);
                    if let Some(ref guard) = arm.guard {
                        Self::collect_usages_from_expr(guard, uri, lines, index);
                    }
                }
            }
            Stmt::TryCatch { try_body, catch_body, .. } => {
                Self::collect_usages_from_block(try_body, uri, lines, index);
                Self::collect_usages_from_block(catch_body, uri, lines, index);
            }
            Stmt::Block(block) => {
                Self::collect_usages_from_block(block, uri, lines, index);
            }
            _ => {}
        }
    }

    fn collect_usages_from_type(ty: &Type, uri: &str, lines: &[&str], index: &mut SymbolIndex) {
        match ty {
            Type::Name(name) => {
                Self::scan_line_usages(name, uri, lines, index);
            }
            Type::Path(parts) => {
                for part in parts {
                    Self::scan_line_usages(part, uri, lines, index);
                }
            }
            Type::Reference { inner, .. } => {
                Self::collect_usages_from_type(inner, uri, lines, index);
            }
            Type::Generic { name, args } => {
                Self::scan_line_usages(name, uri, lines, index);
                for arg in args {
                    Self::collect_usages_from_type(arg, uri, lines, index);
                }
            }
            Type::Function { params, return_type } => {
                for param in params {
                    Self::collect_usages_from_type(param, uri, lines, index);
                }
                Self::collect_usages_from_type(return_type, uri, lines, index);
            }
            Type::Tuple(types) => {
                for ty in types {
                    Self::collect_usages_from_type(ty, uri, lines, index);
                }
            }
            _ => {}
        }
    }

    /// Scan all lines in source for occurrences of `name` as a whole word and record usages.
    fn scan_line_usages(name: &str, uri: &str, lines: &[&str], index: &mut SymbolIndex) {
        if name.is_empty() || name.len() < 2 {
            return; // Skip single-char and empty names to avoid noise
        }
        for (line_idx, line) in lines.iter().enumerate() {
            let mut col = 0;
            for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                if word == name {
                    let usage = SymbolLocation {
                        uri: uri.to_string(),
                        line: line_idx,
                        character: col,
                        length: name.len(),
                        kind: SymbolKind::Usage,
                    };
                    index.usages.entry(name.to_string()).or_default().push(usage);
                }
                col += word.len() + 1; // +1 for the separator
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_symbol_index(&self, uri: &str) -> Option<&SymbolIndex> {
        self.symbol_index.get(uri)
    }

    /// Find all references to the symbol at the given position across all open documents.
    pub fn find_references(&self, uri: &str, word: &str) -> Vec<SymbolLocation> {
        let mut refs = Vec::new();

        // Search in all open documents
        for (doc_uri, index) in &self.symbol_index {
            // Add definitions
            if let Some(defs) = index.definitions.get(word) {
                for def in defs {
                    if def.uri == *uri || self.documents.contains_key(doc_uri) {
                        refs.push(def.clone());
                    }
                }
            }
            // Add usages
            if let Some(usages) = index.usages.get(word) {
                for usage in usages {
                    if self.documents.contains_key(doc_uri) {
                        refs.push(usage.clone());
                    }
                }
            }
        }

        refs
    }

    /// Find all locations that need to be renamed (definition + usages) for the symbol at position.
    pub fn find_rename_locations(&self, uri: &str, word: &str) -> Vec<SymbolLocation> {
        self.find_references(uri, word)
    }

    /// Apply a rename to all documents and return the new text for each document.
    #[allow(dead_code)]
    pub fn apply_rename(&mut self, uri: &str, word: &str, new_name: &str) -> HashMap<String, String> {
        let locations = self.find_rename_locations(uri, word);
        let mut edits: HashMap<String, Vec<(usize, usize, String)>> = HashMap::new();

        // Group edits by URI, sort by position (descending so we apply from end to start)
        for loc in &locations {
            edits.entry(loc.uri.clone()).or_default().push(
                (loc.line, loc.character, new_name.to_string())
            );
        }

        let mut results = HashMap::new();
        for (edit_uri, mut file_edits) in edits {
            // Sort descending by line then character so we can apply from bottom to top
            file_edits.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));

            if let Some(text) = self.documents.get(&edit_uri) {
                let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
                for (line, col, replacement) in &file_edits {
                    if let Some(line_text) = lines.get_mut(*line) {
                        let chars: Vec<char> = line_text.chars().collect();
                        if *col < chars.len() {
                            let end = (*col + word.len()).min(chars.len());
                            let before: String = chars[..*col].iter().collect();
                            let after: String = chars[end..].iter().collect();
                            *line_text = format!("{}{}{}", before, replacement, after);
                        }
                    }
                }
                let new_text = lines.join("\n");
                results.insert(edit_uri, new_text);
            }
        }

        results
    }

    #[allow(dead_code)]
    pub fn get_all_symbols(&self, uri: &str) -> Vec<(String, String, usize, usize)> {
        // Returns (name, kind, line, col) for all symbols in the document
        let mut symbols = Vec::new();
        if let Some(program) = self.parsed.get(uri) {
            for item in &program.items {
                match item {
                    Item::Function(f) => {
                        symbols.push((f.name.clone(), "function".to_string(), f.span.line, f.span.col));
                    }
                    Item::Struct(s) => {
                        symbols.push((s.name.clone(), "struct".to_string(), s.span.line, s.span.col));
                    }
                    Item::Class(c) => {
                        symbols.push((c.name.clone(), "class".to_string(), c.span.line, c.span.col));
                    }
                    Item::Enum(e) => {
                        symbols.push((e.name.clone(), "enum".to_string(), e.span.line, e.span.col));
                    }
                    Item::Trait(t) => {
                        symbols.push((t.name.clone(), "trait".to_string(), t.span.line, t.span.col));
                    }
                    Item::TypeAlias(ta) => {
                        symbols.push((ta.name.clone(), "typeAlias".to_string(), 0, 0));
                    }
                    Item::Module(m) => {
                        symbols.push((m.name.clone(), "module".to_string(), 0, 0));
                    }
                    _ => {}
                }
            }
        }
        symbols
    }

    pub fn get_document_symbols(&self, uri: &str) -> Vec<(String, String, Position)> {
        let mut symbols = Vec::new();
        if let Some(program) = self.parsed.get(uri) {
            for item in &program.items {
                Self::collect_item_symbols(item, &mut symbols);
            }
        }
        symbols
    }

    fn collect_item_symbols(item: &Item, symbols: &mut Vec<(String, String, Position)>) {
        match item {
            Item::Function(f) => {
                symbols.push((
                    f.name.clone(),
                    "function".to_string(),
                    Position { line: f.span.line.saturating_sub(1), character: f.span.col.saturating_sub(1) },
                ));
            }
            Item::Struct(s) => {
                symbols.push((
                    s.name.clone(),
                    "struct".to_string(),
                    Position { line: s.span.line.saturating_sub(1), character: s.span.col.saturating_sub(1) },
                ));
                let _ = &s.fields;
            }
            Item::Class(c) => {
                symbols.push((
                    c.name.clone(),
                    "class".to_string(),
                    Position { line: c.span.line.saturating_sub(1), character: c.span.col.saturating_sub(1) },
                ));
                for method in &c.methods {
                    symbols.push((
                        method.name.clone(),
                        "method".to_string(),
                        Position { line: method.span.line.saturating_sub(1), character: method.span.col.saturating_sub(1) },
                    ));
                }
            }
            Item::Enum(e) => {
                symbols.push((
                    e.name.clone(),
                    "enum".to_string(),
                    Position { line: e.span.line.saturating_sub(1), character: e.span.col.saturating_sub(1) },
                ));
            }
            Item::Impl(imp) => {
                for method in &imp.methods {
                    symbols.push((
                        method.name.clone(),
                        "method".to_string(),
                        Position { line: method.span.line.saturating_sub(1), character: method.span.col.saturating_sub(1) },
                    ));
                }
            }
            Item::Trait(t) => {
                symbols.push((
                    t.name.clone(),
                    "trait".to_string(),
                    Position { line: t.span.line.saturating_sub(1), character: t.span.col.saturating_sub(1) },
                ));
                for method in &t.methods {
                    symbols.push((
                        method.name.clone(),
                        "method".to_string(),
                        Position { line: 0, character: 0 },
                    ));
                }
            }
            Item::Module(m) => {
                symbols.push((
                    m.name.clone(),
                    "module".to_string(),
                    Position { line: 0, character: 0 },
                ));
                if let Some(ref body) = m.body {
                    for inner_item in body {
                        Self::collect_item_symbols(inner_item, symbols);
                    }
                }
            }
            _ => {}
        }
    }
}

// ─── LSP Server ─────────────────────────────────────────────────────────────

pub struct LspServer {
    store: DocumentStore,
    root_uri: Option<String>,
    initialized: bool,
    #[allow(dead_code)]
    request_id: i64,
}

impl LspServer {
    pub fn new() -> Self {
        Self {
            store: DocumentStore::new(),
            root_uri: None,
            initialized: false,
            request_id: 0,
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut buffer = String::new();

        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = buffer.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Handle Content-Length header
                    if line.starts_with("Content-Length:") {
                        // Security: limit message size to 5MB to prevent DoS
                        const MAX_MESSAGE_SIZE: usize = 5 * 1024 * 1024;
                        let len: usize = line[15..].trim().parse().unwrap_or(0);
                        if len == 0 || len > MAX_MESSAGE_SIZE {
                            if len > MAX_MESSAGE_SIZE {
                                eprintln!("Warning: LSP message too large ({} bytes), skipping", len);
                            }
                            // Skip to next message
                            continue;
                        }

                        // Read the blank line
                        let mut blank = String::new();
                        let _ = reader.read_line(&mut blank);

                        // Read the JSON body
                        let mut body = vec![0u8; len];
                        if let Err(_) = reader.read_exact(&mut body) {
                            break;
                        }
                        let body_str = match String::from_utf8(body) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        // Parse and handle
                        if let Some(msg) = json_parse(&body_str) {
                            let response = self.handle_message(&msg);
                            if let Some(resp) = response {
                                self.send_response(&resp);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn handle_message(&mut self, msg: &JsonValue) -> Option<JsonValue> {
        let method = msg.get("method")?.as_str()?;
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(JsonValue::Object(Vec::new()));

        let result = match method {
            "initialize" => self.handle_initialize(params),
            "initialized" => {
                self.initialized = true;
                None
            }
            "shutdown" => Some(JsonValue::Null),
            "exit" => std::process::exit(0),
            "textDocument/didOpen" => {
                self.handle_did_open(params);
                None
            }
            "textDocument/didChange" => {
                self.handle_did_change(params);
                None
            }
            "textDocument/didClose" => {
                self.handle_did_close(params);
                None
            }
            "textDocument/hover" => self.handle_hover(params),
            "textDocument/definition" => self.handle_definition(params),
            "textDocument/completion" => self.handle_completion(params),
            "textDocument/documentSymbol" => self.handle_document_symbol(params),
            "textDocument/diagnostic" | "textDocument/publishDiagnostics" => {
                self.handle_diagnostics(params)
            }
            "textDocument/references" => self.handle_references(params),
            "textDocument/rename" => self.handle_rename(params),
            "textDocument/prepareRename" => self.handle_prepare_rename(params),
            "textDocument/signatureHelp" => self.handle_signature_help(params),
            "textDocument/codeAction" => self.handle_code_action(params),
            "workspace/symbol" => self.handle_workspace_symbol(params),
            _ => None,
        };

        match id {
            Some(id_val @ JsonValue::Number(_)) | Some(id_val @ JsonValue::Str(_)) => {
                let mut response = vec![
                    ("jsonrpc".to_string(), JsonValue::Str("2.0".to_string())),
                    ("id".to_string(), id_val),
                ];
                match result {
                    Some(r) => response.push(("result".to_string(), r)),
                    None => response.push(("result".to_string(), JsonValue::Null)),
                }
                Some(JsonValue::Object(response))
            }
            _ => None, // Notification — no response
        }
    }

    fn send_response(&mut self, response: &JsonValue) {
        let body = json_stringify(response);
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body);
        let _ = out.flush();
    }

    #[allow(dead_code)]
    fn send_notification(&mut self, method: &str, params: JsonValue) {
        let notification = JsonValue::Object(vec![
            ("jsonrpc".to_string(), JsonValue::Str("2.0".to_string())),
            ("method".to_string(), JsonValue::Str(method.to_string())),
            ("params".to_string(), params),
        ]);
        self.send_response(&notification);
    }

    // ─── Initialize ─────────────────────────────────────────────────

    fn handle_initialize(&mut self, params: JsonValue) -> Option<JsonValue> {
        self.root_uri = params
            .get("rootUri")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let capabilities = JsonValue::Object(vec![
            ("textDocumentSync".to_string(), JsonValue::Object(vec![
                ("openClose".to_string(), JsonValue::Bool(true)),
                ("change".to_string(), JsonValue::Number(2.0)), // Incremental sync
                ("save".to_string(), JsonValue::Object(vec![
                    ("includeText".to_string(), JsonValue::Bool(true)),
                ])),
            ])),
            ("hoverProvider".to_string(), JsonValue::Bool(true)),
            ("definitionProvider".to_string(), JsonValue::Bool(true)),
            ("completionProvider".to_string(), JsonValue::Object(vec![
                ("triggerCharacters".to_string(), JsonValue::Array(vec![
                    JsonValue::Str(".".to_string()),
                    JsonValue::Str(":".to_string()),
                    JsonValue::Str("::".to_string()),
                ])),
            ])),
            ("documentSymbolProvider".to_string(), JsonValue::Bool(true)),
            ("referencesProvider".to_string(), JsonValue::Bool(true)),
            ("renameProvider".to_string(), JsonValue::Object(vec![
                ("prepareProvider".to_string(), JsonValue::Bool(true)),
            ])),
            ("signatureHelpProvider".to_string(), JsonValue::Object(vec![
                ("triggerCharacters".to_string(), JsonValue::Array(vec![
                    JsonValue::Str("(".to_string()),
                    JsonValue::Str(",".to_string()),
                ])),
            ])),
            ("codeActionProvider".to_string(), JsonValue::Object(vec![
                ("codeActionKinds".to_string(), JsonValue::Array(vec![
                    JsonValue::Str("quickfix".to_string()),
                ])),
            ])),
            ("workspaceSymbolProvider".to_string(), JsonValue::Bool(true)),
        ]);

        Some(JsonValue::Object(vec![
            ("capabilities".to_string(), capabilities),
        ]))
    }

    // ─── Text Document Sync ─────────────────────────────────────────

    fn handle_did_open(&mut self, params: JsonValue) {
        let text_doc = match params.get("textDocument") {
            Some(v) => v,
            None => return,
        };
        let uri = match text_doc.get("uri").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => return,
        };
        let language_id = text_doc.get("languageId")
            .and_then(|v| v.as_str())
            .unwrap_or("ruva")
            .to_string();
        let version = text_doc.get("version")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let text = match text_doc.get("text").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => return,
        };
        let item = TextDocumentItem { uri, language_id, version, text };
        self.store.open(item);
    }

    fn handle_did_change(&mut self, params: JsonValue) {
        let text_doc = match params.get("textDocument") {
            Some(v) => v,
            None => return,
        };
        let uri = match text_doc.get("uri").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => return,
        };
        // Track version for incremental sync
        let version = text_doc.get("version")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        self.store.set_version(&uri, version);

        let changes_raw = match params.get("contentChanges").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => return,
        };
        let mut changes = Vec::new();
        for change in changes_raw {
            let range = change.get("range").and_then(|r| {
                let start = r.get("start")?;
                let end = r.get("end")?;
                Some(TextRange {
                    start: Position {
                        line: start.get("line")?.as_i64()? as usize,
                        character: start.get("character")?.as_i64()? as usize,
                    },
                    end: Position {
                        line: end.get("line")?.as_i64()? as usize,
                        character: end.get("character")?.as_i64()? as usize,
                    },
                })
            });
            let text = match change.get("text").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            changes.push(TextDocumentContentChangeEvent { range, text });
        }
        self.store.change(&uri, changes);
    }

    fn handle_did_close(&mut self, params: JsonValue) {
        let text_doc = match params.get("textDocument") {
            Some(v) => v,
            None => return,
        };
        let uri = match text_doc.get("uri").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => return,
        };
        self.store.close(&uri);
    }

    // ─── Hover ──────────────────────────────────────────────────────

    fn handle_hover(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let pos = params.get("position")?;
        let line = pos.get("line")?.as_i64()? as usize;
        let character = pos.get("character")?.as_i64()? as usize;

        let text = self.store.get_text(uri)?;
        let program = self.store.get_parsed(uri)?;

        // Get the word at the cursor position
        let word = self.get_word_at_position(text, line, character);

        // Find the symbol and its type information
        let hover_text = self.find_hover_for_word(&word, program, uri);

        if let Some(content) = hover_text {
            Some(JsonValue::Object(vec![
                ("contents".to_string(), JsonValue::Object(vec![
                    ("kind".to_string(), JsonValue::Str("markdown".to_string())),
                    ("value".to_string(), JsonValue::Str(content)),
                ])),
            ]))
        } else {
            None
        }
    }

    fn get_word_at_position(&self, text: &str, line: usize, character: usize) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if line >= lines.len() {
            return String::new();
        }
        let line_text = lines[line];
        let chars: Vec<char> = line_text.chars().collect();
        if character >= chars.len() {
            return String::new();
        }

        // Find word boundaries
        let mut start = character;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        let mut end = character;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        chars[start..end].iter().collect()
    }

    fn find_hover_for_word(&self, word: &str, program: &Program, _uri: &str) -> Option<String> {
        if word.is_empty() {
            return None;
        }

        // Check built-in keywords
        match word {
            "fn" => return Some("Function definition".to_string()),
            "let" => return Some("Variable binding".to_string()),
            "mut" => return Some("Mutable binding".to_string()),
            "pub" => return Some("Public visibility".to_string()),
            "struct" => return Some("Struct definition".to_string()),
            "class" => return Some("Class definition".to_string()),
            "enum" => return Some("Enum definition".to_string()),
            "impl" => return Some("Implementation block".to_string()),
            "trait" => return Some("Trait definition".to_string()),
            "type" => return Some("Type alias".to_string()),
            "if" => return Some("Conditional expression".to_string()),
            "else" => return Some("Else branch".to_string()),
            "for" => return Some("For loop".to_string()),
            "while" => return Some("While loop".to_string()),
            "loop" => return Some("Loop expression".to_string()),
            "return" => return Some("Return statement".to_string()),
            "break" => return Some("Break statement".to_string()),
            "continue" => return Some("Continue statement".to_string()),
            "match" => return Some("Pattern matching".to_string()),
            "self" => return Some("Self reference".to_string()),
            "Self" => return Some("Self type".to_string()),
            "true" | "false" => return Some(format!("Boolean literal: {}", word)),
            "null" => return Some("Null literal".to_string()),
            "mod" => return Some("Module declaration".to_string()),
            "use" => return Some("Use declaration".to_string()),
            "import" => return Some("Import declaration".to_string()),
            "test" => return Some("Test function marker".to_string()),
            "where" => return Some("Where clause".to_string()),
            "as" => return Some("Type cast / alias".to_string()),
            "move" => return Some("Move expression".to_string()),
            _ => {}
        }

        // Check program symbols
        for item in &program.items {
            match item {
                Item::Function(f) if f.name == word => {
                    let params: Vec<String> = f.params.iter().map(|p| {
                        format!("{}: {}", p.name, self.type_to_string(&p.ty))
                    }).collect();
                    let ret = f.return_type.as_ref().map(|t| format!(" -> {}", self.type_to_string(t))).unwrap_or_default();
                    let vis = if f.is_pub { "pub " } else { "" };
                    let test = if f.is_test { "test " } else { "" };
                    return Some(format!("```ruva\n{}{}fn {}({}){}\n```\nFunction definition", vis, test, f.name, params.join(", "), ret));
                }
                Item::Struct(s) if s.name == word => {
                    let field_count = s.fields.len();
                    let vis = if s.is_pub { "pub " } else { "" };
                    return Some(format!("```ruva\n{}struct {}\n```\nStruct with {} fields", vis, s.name, field_count));
                }
                Item::Class(c) if c.name == word => {
                    let field_count = c.fields.len();
                    let method_count = c.methods.len();
                    let vis = if c.is_pub { "pub " } else { "" };
                    return Some(format!("```ruva\n{}class {}\n```\nClass with {} fields, {} methods", vis, c.name, field_count, method_count));
                }
                Item::Enum(e) if e.name == word => {
                    let variant_count = e.variants.len();
                    let vis = if e.is_pub { "pub " } else { "" };
                    return Some(format!("```ruva\n{}enum {}\n```\nEnum with {} variants", vis, e.name, variant_count));
                }
                Item::Trait(t) if t.name == word => {
                    let method_count = t.methods.len();
                    let vis = if t.is_pub { "pub " } else { "" };
                    return Some(format!("```ruva\n{}trait {}\n```\nTrait with {} methods", vis, t.name, method_count));
                }
                Item::Module(m) if m.name == word => {
                    return Some(format!("```ruva\nmod {}\n```\nModule definition", m.name));
                }
                Item::TypeAlias(ta) if ta.name == word => {
                    let ty = self.type_to_string(&ta.ty);
                    return Some(format!("```ruva\ntype {} = {}\n```\nType alias", ta.name, ty));
                }
                _ => {}
            }
        }

        None
    }

    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Name(n) => n.clone(),
            Type::Path(p) => p.join("::"),
            Type::Reference { inner, is_mut } => {
                let mut_str = if *is_mut { "mut " } else { "" };
                format!("&{}{}", mut_str, self.type_to_string(inner))
            }
            Type::Slice(inner) => format!("[{}]", self.type_to_string(inner)),
            Type::Array { inner, .. } => format!("[{}]", self.type_to_string(inner)),
            Type::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(|t| self.type_to_string(t)).collect();
                format!("({})", inner.join(", "))
            }
            Type::Generic { name, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| self.type_to_string(a)).collect();
                format!("{}<{}>", name, arg_strs.join(", "))
            }
            Type::Function { params, return_type } => {
                let param_strs: Vec<String> = params.iter().map(|p| self.type_to_string(p)).collect();
                format!("fn({}) -> {}", param_strs.join(", "), self.type_to_string(return_type))
            }
            Type::Unit => "()".to_string(),
            Type::Never => "!".to_string(),
            Type::SelfType => "Self".to_string(),
            Type::RawPointer { inner, is_mut } => {
                if *is_mut {
                    format!("*mut {}", self.type_to_string(inner))
                } else {
                    format!("*const {}", self.type_to_string(inner))
                }
            }
        }
    }

    // ─── Go to Definition ───────────────────────────────────────────

    fn handle_definition(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let pos = params.get("position")?;
        let line = pos.get("line")?.as_i64()? as usize;
        let character = pos.get("character")?.as_i64()? as usize;

        let text = self.store.get_text(uri)?;
        let program = self.store.get_parsed(uri)?;

        let word = self.get_word_at_position(text, line, character);
        if word.is_empty() {
            return None;
        }

        // Find the definition location
        let location = self.find_definition(&word, program, uri);

        location.map(|loc| {
            JsonValue::Object(vec![
                ("uri".to_string(), JsonValue::Str(loc.uri)),
                ("range".to_string(), JsonValue::Object(vec![
                    ("start".to_string(), position_to_json(&loc.range.start)),
                    ("end".to_string(), position_to_json(&loc.range.end)),
                ])),
            ])
        })
    }

    fn find_definition(&self, word: &str, program: &Program, uri: &str) -> Option<Location> {
        for item in &program.items {
            let loc = match item {
                Item::Function(f) if f.name == word => {
                    Some(Location {
                        uri: uri.to_string(),
                        range: TextRange {
                            start: Position { line: f.span.line.saturating_sub(1), character: f.span.col.saturating_sub(1) },
                            end: Position { line: f.span.line.saturating_sub(1), character: f.span.col.saturating_sub(1) + f.name.len() },
                        },
                    })
                }
                Item::Struct(s) if s.name == word => {
                    Some(Location {
                        uri: uri.to_string(),
                        range: TextRange {
                            start: Position { line: s.span.line.saturating_sub(1), character: s.span.col.saturating_sub(1) },
                            end: Position { line: s.span.line.saturating_sub(1), character: s.span.col.saturating_sub(1) + s.name.len() },
                        },
                    })
                }
                Item::Class(c) if c.name == word => {
                    Some(Location {
                        uri: uri.to_string(),
                        range: TextRange {
                            start: Position { line: c.span.line.saturating_sub(1), character: c.span.col.saturating_sub(1) },
                            end: Position { line: c.span.line.saturating_sub(1), character: c.span.col.saturating_sub(1) + c.name.len() },
                        },
                    })
                }
                Item::Enum(e) if e.name == word => {
                    Some(Location {
                        uri: uri.to_string(),
                        range: TextRange {
                            start: Position { line: e.span.line.saturating_sub(1), character: e.span.col.saturating_sub(1) },
                            end: Position { line: e.span.line.saturating_sub(1), character: e.span.col.saturating_sub(1) + e.name.len() },
                        },
                    })
                }
                Item::Trait(t) if t.name == word => {
                    Some(Location {
                        uri: uri.to_string(),
                        range: TextRange {
                            start: Position { line: t.span.line.saturating_sub(1), character: t.span.col.saturating_sub(1) },
                            end: Position { line: t.span.line.saturating_sub(1), character: t.span.col.saturating_sub(1) + t.name.len() },
                        },
                    })
                }
                Item::Module(m) if m.name == word => {
                    Some(Location {
                        uri: uri.to_string(),
                        range: TextRange {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: m.name.len() },
                        },
                    })
                }
                Item::TypeAlias(ta) if ta.name == word => {
                    Some(Location {
                        uri: uri.to_string(),
                        range: TextRange {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: ta.name.len() },
                        },
                    })
                }
                // Check methods in impl blocks
                Item::Impl(imp) => {
                    let mut found = None;
                    for method in &imp.methods {
                        if method.name == word {
                            found = Some(Location {
                                uri: uri.to_string(),
                                range: TextRange {
                                    start: Position { line: method.span.line.saturating_sub(1), character: method.span.col.saturating_sub(1) },
                                    end: Position { line: method.span.line.saturating_sub(1), character: method.span.col.saturating_sub(1) + method.name.len() },
                                },
                            });
                            break;
                        }
                    }
                    found
                }
                // Check methods in classes
                Item::Class(c) => {
                    let mut found = None;
                    for method in &c.methods {
                        if method.name == word {
                            found = Some(Location {
                                uri: uri.to_string(),
                                range: TextRange {
                                    start: Position { line: method.span.line.saturating_sub(1), character: method.span.col.saturating_sub(1) },
                                    end: Position { line: method.span.line.saturating_sub(1), character: method.span.col.saturating_sub(1) + method.name.len() },
                                },
                            });
                            break;
                        }
                    }
                    found
                }
                _ => None,
            };

            if loc.is_some() {
                return loc;
            }
        }

        None
    }

    // ─── Completion ─────────────────────────────────────────────────

    fn handle_completion(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let pos = params.get("position")?;
        let line = pos.get("line")?.as_i64()? as usize;
        let character = pos.get("character")?.as_i64()? as usize;

        let text = self.store.get_text(uri).unwrap_or("");
        let program = self.store.get_parsed(uri);

        // Get the prefix (partial word) being typed
        let prefix = self.get_prefix_at_position(text, line, character);

        let mut items = Vec::new();

        // Add keyword completions
        let keywords = vec![
            ("fn", 14), ("let", 13), ("mut", 13), ("pub", 14),
            ("struct", 23), ("class", 7), ("enum", 13), ("impl", 14),
            ("trait", 24), ("type", 14), ("if", 14), ("else", 14),
            ("for", 14), ("while", 14), ("loop", 14), ("return", 14),
            ("break", 14), ("continue", 14), ("match", 14),
            ("self", 14), ("Self", 14), ("true", 12), ("false", 12),
            ("null", 12), ("mod", 14), ("use", 14), ("import", 14),
            ("test", 14), ("where", 14), ("as", 14), ("move", 14),
        ];

        for (kw, kind) in keywords {
            if prefix.is_empty() || kw.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: kw.to_string(),
                    kind,
                    detail: Some("keyword".to_string()),
                    documentation: None,
                    insert_text: Some(kw.to_string()),
                });
            }
        }

        // Add symbol completions from the parsed program
        if let Some(program) = program {
            for item in &program.items {
                match item {
                    Item::Function(f) => {
                        if prefix.is_empty() || f.name.starts_with(&prefix) {
                            let params: Vec<String> = f.params.iter().map(|p| {
                                format!("{}: {}", p.name, self.type_to_string(&p.ty))
                            }).collect();
                            let ret = f.return_type.as_ref().map(|t| format!(" -> {}", self.type_to_string(t))).unwrap_or_default();
                            items.push(CompletionItem {
                                label: f.name.clone(),
                                kind: 12, // Function
                                detail: Some(format!("fn({}){}", params.join(", "), ret)),
                                documentation: Some(format!("Function `{}`", f.name)),
                                insert_text: Some(format!("{}(", f.name)),
                            });
                        }
                    }
                    Item::Struct(s) => {
                        if prefix.is_empty() || s.name.starts_with(&prefix) {
                            items.push(CompletionItem {
                                label: s.name.clone(),
                                kind: 23, // Struct
                                detail: Some(format!("struct {} ({} fields)", s.name, s.fields.len())),
                                documentation: Some(format!("Struct `{}`", s.name)),
                                insert_text: Some(format!("{} ", s.name)),
                            });
                        }
                    }
                    Item::Class(c) => {
                        if prefix.is_empty() || c.name.starts_with(&prefix) {
                            items.push(CompletionItem {
                                label: c.name.clone(),
                                kind: 7, // Class
                                detail: Some(format!("class {} ({} fields, {} methods)", c.name, c.fields.len(), c.methods.len())),
                                documentation: Some(format!("Class `{}`", c.name)),
                                insert_text: Some(format!("{} ", c.name)),
                            });
                        }
                    }
                    Item::Enum(e) => {
                        if prefix.is_empty() || e.name.starts_with(&prefix) {
                            items.push(CompletionItem {
                                label: e.name.clone(),
                                kind: 13, // Enum
                                detail: Some(format!("enum {} ({} variants)", e.name, e.variants.len())),
                                documentation: Some(format!("Enum `{}`", e.name)),
                                insert_text: Some(format!("{} ", e.name)),
                            });
                            // Also add variants
                            for variant in &e.variants {
                                let variant_name = format!("{}::{}", e.name, variant.name);
                                if prefix.is_empty() || variant_name.starts_with(&prefix) || variant.name.starts_with(&prefix) {
                                    items.push(CompletionItem {
                                        label: variant.name.clone(),
                                        kind: 13, // EnumMember
                                        detail: Some(format!("{}::{}", e.name, variant.name)),
                                        documentation: Some(format!("Variant of `{}`", e.name)),
                                        insert_text: Some(format!("{}::{}", e.name, variant.name)),
                                    });
                                }
                            }
                        }
                    }
                    Item::Trait(t) => {
                        if prefix.is_empty() || t.name.starts_with(&prefix) {
                            items.push(CompletionItem {
                                label: t.name.clone(),
                                kind: 24, // Interface
                                detail: Some(format!("trait {} ({} methods)", t.name, t.methods.len())),
                                documentation: Some(format!("Trait `{}`", t.name)),
                                insert_text: Some(format!("{} ", t.name)),
                            });
                        }
                    }
                    Item::Module(m) => {
                        if prefix.is_empty() || m.name.starts_with(&prefix) {
                            items.push(CompletionItem {
                                label: m.name.clone(),
                                kind: 19, // Module
                                detail: Some(format!("mod {}", m.name)),
                                documentation: Some(format!("Module `{}`", m.name)),
                                insert_text: Some(format!("{} ", m.name)),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        // Convert to JSON
        let items_json: Vec<JsonValue> = items.iter().map(|item| {
            let mut fields = vec![
                ("label".to_string(), JsonValue::Str(item.label.clone())),
                ("kind".to_string(), JsonValue::Number(item.kind as f64)),
            ];
            if let Some(ref detail) = item.detail {
                fields.push(("detail".to_string(), JsonValue::Str(detail.clone())));
            }
            if let Some(ref doc) = item.documentation {
                fields.push(("documentation".to_string(), JsonValue::Str(doc.clone())));
            }
            if let Some(ref insert) = item.insert_text {
                fields.push(("insertText".to_string(), JsonValue::Str(insert.clone())));
            }
            JsonValue::Object(fields)
        }).collect();

        Some(JsonValue::Object(vec![
            ("isIncomplete".to_string(), JsonValue::Bool(false)),
            ("items".to_string(), JsonValue::Array(items_json)),
        ]))
    }

    fn get_prefix_at_position(&self, text: &str, line: usize, character: usize) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if line >= lines.len() {
            return String::new();
        }
        let line_text = lines[line];
        let chars: Vec<char> = line_text.chars().collect();
        if character > chars.len() {
            return String::new();
        }

        let mut start = character;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        chars[start..character].iter().collect()
    }

    // ─── Document Symbols ───────────────────────────────────────────

    fn handle_document_symbol(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;

        let symbols = self.store.get_document_symbols(uri);

        let items: Vec<JsonValue> = symbols.iter().map(|(name, kind, pos)| {
            let kind_num = match kind.as_str() {
                "function" => 12,
                "struct" => 23,
                "class" => 7,
                "enum" => 13,
                "trait" => 24,
                "method" => 6,
                "module" => 19,
                "typeAlias" => 14,
                "variable" => 13,
                _ => 1,
            };
            JsonValue::Object(vec![
                ("name".to_string(), JsonValue::Str(name.clone())),
                ("kind".to_string(), JsonValue::Number(kind_num as f64)),
                ("range".to_string(), JsonValue::Object(vec![
                    ("start".to_string(), position_to_json(pos)),
                    ("end".to_string(), JsonValue::Object(vec![
                        ("line".to_string(), JsonValue::Number(pos.line as f64)),
                        ("character".to_string(), JsonValue::Number((pos.character + name.len()) as f64)),
                    ])),
                ])),
                ("selectionRange".to_string(), JsonValue::Object(vec![
                    ("start".to_string(), position_to_json(pos)),
                    ("end".to_string(), JsonValue::Object(vec![
                        ("line".to_string(), JsonValue::Number(pos.line as f64)),
                        ("character".to_string(), JsonValue::Number((pos.character + name.len()) as f64)),
                    ])),
                ])),
            ])
        }).collect();

        Some(JsonValue::Array(items))
    }

    // ─── Diagnostics ────────────────────────────────────────────────

    fn handle_diagnostics(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;

        let diagnostics = self.compute_diagnostics(uri);

        let diag_items: Vec<JsonValue> = diagnostics.iter().map(|d| {
            JsonValue::Object(vec![
                ("range".to_string(), JsonValue::Object(vec![
                    ("start".to_string(), position_to_json(&d.range.start)),
                    ("end".to_string(), position_to_json(&d.range.end)),
                ])),
                ("severity".to_string(), JsonValue::Number(d.severity as f64)),
                ("message".to_string(), JsonValue::Str(d.message.clone())),
                ("source".to_string(), JsonValue::Str(d.source.clone())),
            ])
        }).collect();

        Some(JsonValue::Object(vec![
            ("uri".to_string(), JsonValue::Str(uri.to_string())),
            ("diagnostics".to_string(), JsonValue::Array(diag_items)),
        ]))
    }

    fn compute_diagnostics(&self, uri: &str) -> Vec<DiagnosticInfo> {
        let mut diagnostics = Vec::new();

        // Include any stored parse errors
        if let Some(parse_err) = self.store.parse_errors.get(uri) {
            diagnostics.push(parse_err.clone());
        }

        let text = match self.store.get_text(uri) {
            Some(t) => t.to_string(),
            None => return diagnostics,
        };

        // Try parsing
        match Parser::new(&text) {
            Ok(mut parser) => match parser.parse_program() {
                Ok(program) => {
                    // Run type checker
                    let mut checker = TypeChecker::new();
                    let checker_diags = checker.check(&program);

                    for diag in &checker_diags {
                        let line = diag.line.saturating_sub(1);
                        let col = diag.col.saturating_sub(1);
                        let severity = match diag.kind {
                            crate::typecheck::DiagnosticKind::Error => 1,
                            crate::typecheck::DiagnosticKind::Warning => 2,
                        };
                        diagnostics.push(DiagnosticInfo {
                            range: TextRange {
                                start: Position { line, character: col },
                                end: Position { line, character: col + 1 },
                            },
                            severity,
                            message: diag.message.clone(),
                            source: "ruva".to_string(),
                        });
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    // Try to extract line/col from error message
                    let (line, col) = parse_error_location(&msg);
                    diagnostics.push(DiagnosticInfo {
                        range: TextRange {
                            start: Position { line: line.saturating_sub(1), character: col.saturating_sub(1) },
                            end: Position { line: line.saturating_sub(1), character: col },
                        },
                        severity: 1,
                        message: msg,
                        source: "ruva".to_string(),
                    });
                }
            },
            Err(e) => {
                let msg = e.to_string();
                diagnostics.push(DiagnosticInfo {
                    range: TextRange {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: 1,
                    message: msg,
                    source: "ruva".to_string(),
                });
            }
        }

        diagnostics
    }

    // ─── Find References ─────────────────────────────────────────────

    fn handle_references(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let pos = params.get("position")?;
        let line = pos.get("line")?.as_i64()? as usize;
        let character = pos.get("character")?.as_i64()? as usize;

        let text = self.store.get_text(uri)?;
        let word = self.get_word_at_position(text, line, character);
        if word.is_empty() {
            return Some(JsonValue::Array(Vec::new()));
        }

        // Check include_declaration option (default true)
        let _include_decl = params.get("context")
            .and_then(|ctx| ctx.get("includeDeclaration"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let refs = self.store.find_references(uri, &word);

        let locations: Vec<JsonValue> = refs.iter().map(|r| {
            JsonValue::Object(vec![
                ("uri".to_string(), JsonValue::Str(r.uri.clone())),
                ("range".to_string(), JsonValue::Object(vec![
                    ("start".to_string(), position_to_json(&Position { line: r.line, character: r.character })),
                    ("end".to_string(), position_to_json(&Position { line: r.line, character: r.character + r.length })),
                ])),
            ])
        }).collect();

        Some(JsonValue::Array(locations))
    }

    // ─── Rename ──────────────────────────────────────────────────────

    fn handle_rename(&mut self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let pos = params.get("position")?;
        let line = pos.get("line")?.as_i64()? as usize;
        let character = pos.get("character")?.as_i64()? as usize;
        let new_name = params.get("newName")?.as_str()?;

        let text = self.store.get_text(uri)?;
        let word = self.get_word_at_position(text, line, character);
        if word.is_empty() || word == new_name {
            return None;
        }

        // Find all locations to rename
        let locations = self.store.find_rename_locations(uri, &word);
        if locations.is_empty() {
            return None;
        }

        // Group edits by URI
        let mut edits_by_uri: HashMap<String, Vec<JsonValue>> = HashMap::new();
        for loc in &locations {
            let edit = JsonValue::Object(vec![
                ("range".to_string(), JsonValue::Object(vec![
                    ("start".to_string(), position_to_json(&Position { line: loc.line, character: loc.character })),
                    ("end".to_string(), position_to_json(&Position { line: loc.line, character: loc.character + loc.length })),
                ])),
                ("newText".to_string(), JsonValue::Str(new_name.to_string())),
            ]);
            edits_by_uri.entry(loc.uri.clone()).or_default().push(edit);
        }

        // Build WorkspaceEdit
        let changes: Vec<JsonValue> = edits_by_uri.iter().map(|(uri, edits)| {
            JsonValue::Object(vec![
                ("uri".to_string(), JsonValue::Str(uri.clone())),
                ("edits".to_string(), JsonValue::Array(edits.clone())),
            ])
        }).collect();

        Some(JsonValue::Object(vec![
            ("changes".to_string(), JsonValue::Array(changes)),
        ]))
    }

    // ─── Prepare Rename (check if rename is valid at position) ───────

    fn handle_prepare_rename(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let pos = params.get("position")?;
        let line = pos.get("line")?.as_i64()? as usize;
        let character = pos.get("character")?.as_i64()? as usize;

        let text = self.store.get_text(uri)?;
        let word = self.get_word_at_position(text, line, character);
        if word.is_empty() {
            return None;
        }

        // Check if the word is a keyword — those can't be renamed
        let keywords = ["fn", "let", "mut", "pub", "struct", "class", "enum", "impl",
            "trait", "type", "if", "else", "for", "while", "loop", "return",
            "break", "continue", "match", "self", "Self", "true", "false",
            "null", "mod", "use", "import", "test", "where", "as", "move"];
        if keywords.contains(&word.as_str()) {
            return None;
        }

        // Valid rename target — return the range of the word
        let start = Position { line, character: character.saturating_sub(word.len() / 2) };
        let end = Position { line, character: character + word.len() / 2 };
        Some(JsonValue::Object(vec![
            ("range".to_string(), JsonValue::Object(vec![
                ("start".to_string(), position_to_json(&start)),
                ("end".to_string(), position_to_json(&end)),
            ])),
            ("placeholder".to_string(), JsonValue::Str(word)),
        ]))
    }

    // ─── Signature Help ─────────────────────────────────────────────

    fn handle_signature_help(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let pos = params.get("position")?;
        let line = pos.get("line")?.as_i64()? as usize;
        let character = pos.get("character")?.as_i64()? as usize;

        let text = self.store.get_text(uri)?;
        let program = self.store.get_parsed(uri)?;
        let lines: Vec<&str> = text.lines().collect();
        if line >= lines.len() {
            return None;
        }
        let line_text = lines[line];

        // Find the function being called: scan backwards from cursor to find the function name
        // and count which argument we're on by counting commas
        let chars: Vec<char> = line_text.chars().collect();
        let mut paren_depth: i32 = 0;
        let mut active_param: usize = 0;
        let mut func_name = String::new();

        // Scan backwards from the cursor position
        let mut i = character.min(chars.len());
        while i > 0 {
            i -= 1;
            match chars[i] {
                '(' => {
                    paren_depth -= 1;
                    if paren_depth < 0 {
                        // We found the opening paren of the call — now get the function name
                        // Scan backwards to find the identifier before the (
                        let mut j = i;
                        while j > 0 && chars[j - 1].is_whitespace() {
                            j -= 1;
                        }
                        let name_end = j;
                        while j > 0 && (chars[j - 1].is_alphanumeric() || chars[j - 1] == '_') {
                            j -= 1;
                        }
                        func_name = chars[j..name_end].iter().collect();
                        break;
                    }
                }
                ')' => {
                    paren_depth += 1;
                }
                ',' if paren_depth == 0 => {
                    active_param += 1;
                }
                _ => {}
            }
        }

        if func_name.is_empty() {
            return None;
        }

        // Find the function definition
        let mut signatures = Vec::new();
        self.find_function_signatures(&func_name, program, uri, &mut signatures);

        if signatures.is_empty() {
            return None;
        }

        let active_signature = 0;
        let active_parameter = active_param;

        let sig_infos: Vec<JsonValue> = signatures.iter().map(|sig| {
            let params: Vec<JsonValue> = sig.params.iter().enumerate().map(|(_i, (name, ty))| {
                JsonValue::Object(vec![
                    ("label".to_string(), JsonValue::Str(format!("{}: {}", name, ty))),
                ])
            }).collect();

            let label = if let Some(ref ret) = sig.return_type {
                format!("fn {}({}) -> {}", func_name, params.iter().map(|p|
                    p.get("label").and_then(|v| v.as_str()).unwrap_or("?").to_string()
                ).collect::<Vec<_>>().join(", "), ret)
            } else {
                format!("fn {}({})", func_name, params.iter().map(|p|
                    p.get("label").and_then(|v| v.as_str()).unwrap_or("?").to_string()
                ).collect::<Vec<_>>().join(", "))
            };

            JsonValue::Object(vec![
                ("label".to_string(), JsonValue::Str(label)),
                ("parameters".to_string(), JsonValue::Array(params)),
            ])
        }).collect();

        Some(JsonValue::Object(vec![
            ("signatures".to_string(), JsonValue::Array(sig_infos)),
            ("activeSignature".to_string(), JsonValue::Number(active_signature as f64)),
            ("activeParameter".to_string(), JsonValue::Number(active_parameter as f64)),
        ]))
    }

    fn find_function_signatures(&self, name: &str, program: &Program, _uri: &str, results: &mut Vec<FunctionSig>) {
        for item in &program.items {
            match item {
                Item::Function(f) if f.name == name => {
                    let params: Vec<(String, String)> = f.params.iter().map(|p| {
                        (p.name.clone(), self.type_to_string(&p.ty))
                    }).collect();
                    let ret = f.return_type.as_ref().map(|t| self.type_to_string(t));
                    results.push(FunctionSig { params, return_type: ret });
                }
                Item::Class(c) => {
                    for method in &c.methods {
                        if method.name == name {
                            let params: Vec<(String, String)> = method.params.iter().map(|p| {
                                (p.name.clone(), self.type_to_string(&p.ty))
                            }).collect();
                            let ret = method.return_type.as_ref().map(|t| self.type_to_string(t));
                            results.push(FunctionSig { params, return_type: ret });
                        }
                    }
                }
                Item::Impl(imp) => {
                    for method in &imp.methods {
                        if method.name == name {
                            let params: Vec<(String, String)> = method.params.iter().map(|p| {
                                (p.name.clone(), self.type_to_string(&p.ty))
                            }).collect();
                            let ret = method.return_type.as_ref().map(|t| self.type_to_string(t));
                            results.push(FunctionSig { params, return_type: ret });
                        }
                    }
                }
                Item::Module(m) => {
                    if let Some(ref body) = m.body {
                        self.find_function_signatures(name, &Program { items: body.clone() }, _uri, results);
                    }
                }
                _ => {}
            }
        }
    }

    // ─── Code Actions ───────────────────────────────────────────────

    fn handle_code_action(&self, params: JsonValue) -> Option<JsonValue> {
        let text_doc = params.get("textDocument")?;
        let uri = text_doc.get("uri")?.as_str()?;
        let range = params.get("range")?;

        let diagnostics = self.compute_diagnostics(uri);

        let range_start_line = range.get("start")?.get("line")?.as_i64()? as usize;
        let range_start_char = range.get("start")?.get("character")?.as_i64()? as usize;
        let range_end_line = range.get("end")?.get("line")?.as_i64()? as usize;
        let range_end_char = range.get("end")?.get("character")?.as_i64()? as usize;

        let mut actions = Vec::new();

        for diag in &diagnostics {
            let diag_start = (diag.range.start.line, diag.range.start.character);
            let diag_end = (diag.range.end.line, diag.range.end.character);
            let range_start = (range_start_line, range_start_char);
            let range_end = (range_end_line, range_end_char);
            if !(diag_start <= range_end && diag_end >= range_start) {
                continue;
            }

            let message = &diag.message;
            let d = vec![make_diag_obj(diag)];

            // Fix 1: "Variable 'X' is not defined" -> create a let binding
            if message.starts_with("Variable '") && message.contains("is not defined") {
                let var_name = message.trim_start_matches("Variable '").split("'").next().unwrap_or("");
                if !var_name.is_empty() {
                    let edit1 = make_text_edit(&diag.range, &format!("let {} = ", var_name));
                    actions.push(make_code_action(
                        &format!("Create variable '{}'", var_name), d.clone(),
                        make_workspace_edit(uri, vec![edit1]),
                    ));

                    let edit2 = make_text_edit(&diag.range, &format!("let {} = null", var_name));
                    actions.push(make_code_action(
                        &format!("Initialize '{}' with null", var_name), d.clone(),
                        make_workspace_edit(uri, vec![edit2]),
                    ));
                }
            }

            // Fix 2: "Return type mismatch" -> remove the return type
            if message.starts_with("Return type mismatch") {
                let edit = make_text_edit(&diag.range, "");
                actions.push(make_code_action(
                    "Remove return type annotation", d.clone(),
                    make_workspace_edit(uri, vec![edit]),
                ));
            }

            // Fix 3: "Expected N arguments, got M" -> add or remove arguments
            if message.starts_with("Expected ") && message.contains("arguments, got ") {
                let nums: Vec<&str> = message.split_whitespace().collect();
                if nums.len() >= 5 {
                    let expected: usize = nums[1].parse().unwrap_or(0);
                    let got: usize = nums[4].parse().unwrap_or(0);
                    if got < expected {
                        let diff = expected - got;
                        let placeholders: Vec<String> = (0..diff).map(|_| "null".to_string()).collect();
                        let end_pos = diag.range.end.clone();
                        let insert_range = TextRange { start: end_pos.clone(), end: end_pos };
                        let edit = make_text_edit(&insert_range, &format!(", {}", placeholders.join(", ")));
                        actions.push(make_code_action(
                            &format!("Add {} missing argument(s)", diff), d.clone(),
                            make_workspace_edit(uri, vec![edit]),
                        ));
                    } else if got > expected {
                        actions.push(make_code_action_no_edit(
                            &format!("Remove {} extra argument(s)", got - expected), d.clone(),
                        ));
                    }
                }
            }

            // Fix 4: "Duplicate binding" -> informational action
            if message.starts_with("Duplicate binding") {
                actions.push(make_code_action_no_edit("Rename duplicate binding", d.clone()));
            }
        }

        Some(JsonValue::Array(actions))
    }

    // ─── Workspace Symbols ──────────────────────────────────────────

    fn handle_workspace_symbol(&self, params: JsonValue) -> Option<JsonValue> {
        let query = params.get("query")?.as_str().unwrap_or("");

        let mut results = Vec::new();

        // Search all open documents
        for (uri, program) in &self.store.parsed {
            self.collect_workspace_symbols(query, program, uri, &mut results);
        }

        Some(JsonValue::Array(results))
    }

    fn collect_workspace_symbols(&self, query: &str, program: &Program, uri: &str, results: &mut Vec<JsonValue>) {
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    if fuzzy_match(&f.name, query) {
                        let params_str: Vec<String> = f.params.iter().map(|p| {
                            format!("{}: {}", p.name, self.type_to_string(&p.ty))
                        }).collect();
                        let container = format!("fn({})", params_str.join(", "));
                        results.push(make_workspace_symbol(&f.name, 12, uri, f.span.line.saturating_sub(1), f.span.col.saturating_sub(1), Some(&container)));
                    }
                }
                Item::Struct(s) => {
                    if fuzzy_match(&s.name, query) {
                        results.push(make_workspace_symbol(&s.name, 23, uri, s.span.line.saturating_sub(1), s.span.col.saturating_sub(1), Some("struct")));
                    }
                }
                Item::Class(c) => {
                    if fuzzy_match(&c.name, query) {
                        let container = format!("{} fields, {} methods", c.fields.len(), c.methods.len());
                        results.push(make_workspace_symbol(&c.name, 7, uri, c.span.line.saturating_sub(1), c.span.col.saturating_sub(1), Some(&container)));
                    }
                    // Also search methods
                    for method in &c.methods {
                        if fuzzy_match(&method.name, query) {
                            let params_str: Vec<String> = method.params.iter().map(|p| {
                                format!("{}: {}", p.name, self.type_to_string(&p.ty))
                            }).collect();
                            let container = format!("{}::method({})", c.name, params_str.join(", "));
                            results.push(make_workspace_symbol(&method.name, 6, uri, method.span.line.saturating_sub(1), method.span.col.saturating_sub(1), Some(&container)));
                        }
                    }
                }
                Item::Enum(e) => {
                    if fuzzy_match(&e.name, query) {
                        let container = format!("{} variants", e.variants.len());
                        results.push(make_workspace_symbol(&e.name, 13, uri, e.span.line.saturating_sub(1), e.span.col.saturating_sub(1), Some(&container)));
                    }
                    // Also search variants
                    for variant in &e.variants {
                        if fuzzy_match(&variant.name, query) {
                            let container = format!("{}::variant", e.name);
                            results.push(make_workspace_symbol(&variant.name, 22, uri, e.span.line.saturating_sub(1), e.span.col.saturating_sub(1), Some(&container)));
                        }
                    }
                }
                Item::Trait(t) => {
                    if fuzzy_match(&t.name, query) {
                        let container = format!("{} methods", t.methods.len());
                        results.push(make_workspace_symbol(&t.name, 24, uri, t.span.line.saturating_sub(1), t.span.col.saturating_sub(1), Some(&container)));
                    }
                    for method in &t.methods {
                        if fuzzy_match(&method.name, query) {
                            let container = format!("{}::method", t.name);
                            results.push(make_workspace_symbol(&method.name, 6, uri, 0, 0, Some(&container)));
                        }
                    }
                }
                Item::TypeAlias(ta) => {
                    if fuzzy_match(&ta.name, query) {
                        let ty = self.type_to_string(&ta.ty);
                        results.push(make_workspace_symbol(&ta.name, 14, uri, 0, 0, Some(&ty)));
                    }
                }
                Item::Module(m) => {
                    if fuzzy_match(&m.name, query) {
                        results.push(make_workspace_symbol(&m.name, 19, uri, 0, 0, Some("module")));
                    }
                    if let Some(ref body) = m.body {
                        let inner_program = Program { items: body.clone() };
                        self.collect_workspace_symbols(query, &inner_program, uri, results);
                    }
                }
                Item::Impl(imp) => {
                    for method in &imp.methods {
                        if fuzzy_match(&method.name, query) {
                            let params_str: Vec<String> = method.params.iter().map(|p| {
                                format!("{}: {}", p.name, self.type_to_string(&p.ty))
                            }).collect();
                            let container = format!("impl::method({})", params_str.join(", "));
                            results.push(make_workspace_symbol(&method.name, 6, uri, method.span.line.saturating_sub(1), method.span.col.saturating_sub(1), Some(&container)));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Fuzzy match: returns true if all characters in `query` appear in `name` in order.
/// Case-insensitive. Empty query matches everything.
fn fuzzy_match(name: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let name_lower: Vec<char> = name.to_lowercase().chars().collect();
    let mut qi = 0;
    for &nc in &name_lower {
        if qi < query_lower.len() && nc == query_lower[qi] {
            qi += 1;
        }
    }
    qi == query_lower.len()
}

/// Build a workspace symbol JSON object
fn make_workspace_symbol(name: &str, kind: i64, uri: &str, line: usize, col: usize, container: Option<&str>) -> JsonValue {
    let mut fields = vec![
        ("name".to_string(), JsonValue::Str(name.to_string())),
        ("kind".to_string(), JsonValue::Number(kind as f64)),
        ("location".to_string(), JsonValue::Object(vec![
            ("uri".to_string(), JsonValue::Str(uri.to_string())),
            ("range".to_string(), range_to_json(&TextRange {
                start: Position { line, character: col },
                end: Position { line, character: col + name.len() },
            })),
        ])),
    ];
    if let Some(container_name) = container {
        fields.push(("containerName".to_string(), JsonValue::Str(container_name.to_string())));
    }
    JsonValue::Object(fields)
}

/// Build a diagnostic object for code actions
fn make_diag_obj(d: &DiagnosticInfo) -> JsonValue {
    JsonValue::Object(vec![
        ("range".to_string(), range_to_json(&d.range)),
        ("severity".to_string(), JsonValue::Number(d.severity as f64)),
        ("message".to_string(), JsonValue::Str(d.message.clone())),
        ("source".to_string(), JsonValue::Str("ruva".to_string())),
    ])
}

/// Build a range JSON from a TextRange
fn range_to_json(r: &TextRange) -> JsonValue {
    JsonValue::Object(vec![
        ("start".to_string(), position_to_json(&r.start)),
        ("end".to_string(), position_to_json(&r.end)),
    ])
}

/// Build a workspace edit object for a single-file edit
fn make_workspace_edit(uri: &str, edits: Vec<JsonValue>) -> JsonValue {
    JsonValue::Object(vec![
        ("changes".to_string(), JsonValue::Array(vec![
            JsonValue::Object(vec![
                ("uri".to_string(), JsonValue::Str(uri.to_string())),
                ("edits".to_string(), JsonValue::Array(edits)),
            ])
        ]))
    ])
}

/// Build a text edit at a range
fn make_text_edit(range: &TextRange, new_text: &str) -> JsonValue {
    JsonValue::Object(vec![
        ("range".to_string(), range_to_json(range)),
        ("newText".to_string(), JsonValue::Str(new_text.to_string())),
    ])
}

/// Build a code action with diagnostics and a workspace edit
fn make_code_action(title: &str, diags: Vec<JsonValue>, edit: JsonValue) -> JsonValue {
    JsonValue::Object(vec![
        ("title".to_string(), JsonValue::Str(title.to_string())),
        ("kind".to_string(), JsonValue::Str("quickfix".to_string())),
        ("diagnostics".to_string(), JsonValue::Array(diags)),
        ("edit".to_string(), edit),
    ])
}

/// Build a code action with diagnostics but no edit (informational)
fn make_code_action_no_edit(title: &str, diags: Vec<JsonValue>) -> JsonValue {
    JsonValue::Object(vec![
        ("title".to_string(), JsonValue::Str(title.to_string())),
        ("kind".to_string(), JsonValue::Str("quickfix".to_string())),
        ("diagnostics".to_string(), JsonValue::Array(diags)),
    ])
}

fn position_to_json(pos: &Position) -> JsonValue {
    JsonValue::Object(vec![
        ("line".to_string(), JsonValue::Number(pos.line as f64)),
        ("character".to_string(), JsonValue::Number(pos.character as f64)),
    ])
}

fn parse_error_location(msg: &str) -> (usize, usize) {
    // Try to parse "at line:col" or "at line,col" from error messages
    if let Some(idx) = msg.find(" at ") {
        let rest = &msg[idx + 4..];
        let parts: Vec<&str> = rest.split(|c| c == ':' || c == ',').collect();
        if parts.len() >= 2 {
            let line = parts[0].trim().parse().unwrap_or(1);
            let col = parts[1].trim().parse().unwrap_or(1);
            return (line, col);
        }
    }
    (1, 1)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Document Store Tests ───────────────────────────────────────

    #[test]
    fn test_document_store_open() {
        let mut store = DocumentStore::new();
        let source = "fn main() { let x = 1 }";
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });
        assert!(store.get_text("file:///test.ruva").is_some());
        assert!(store.get_parsed("file:///test.ruva").is_some());
    }

    #[test]
    fn test_document_store_change() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        store.change("file:///test.ruva", vec![
            TextDocumentContentChangeEvent {
                range: None,
                text: "fn main() { let x = 1 }".to_string(),
            }
        ]);

        assert!(store.get_text("file:///test.ruva").unwrap().contains("let x = 1"));
    }

    #[test]
    fn test_document_store_close() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });
        store.close("file:///test.ruva");
        assert!(store.get_text("file:///test.ruva").is_none());
    }

    #[test]
    fn test_document_store_symbols() {
        let mut store = DocumentStore::new();
        let source = r#"
pub fn add(a: i32, b: i32) -> i32 {
    return a + b
}

struct Point {
    pub x: f64,
    pub y: f64,
}

enum Color {
    Red,
    Green,
    Blue,
}
"#;
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let symbols = store.get_all_symbols("file:///test.ruva");
        assert!(symbols.iter().any(|(name, kind, _, _)| name == "add" && kind == "function"));
        assert!(symbols.iter().any(|(name, kind, _, _)| name == "Point" && kind == "struct"));
        assert!(symbols.iter().any(|(name, kind, _, _)| name == "Color" && kind == "enum"));
    }

    // ─── LSP Server Tests ───────────────────────────────────────────

    #[test]
    fn test_lsp_server_initialize() {
        let mut server = LspServer::new();
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":"file:///test","capabilities":{}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        assert!(response.get("result").is_some());
        let caps = response.get("result").unwrap().get("capabilities").unwrap();
        assert!(caps.get("hoverProvider").unwrap().as_bool().unwrap());
        assert!(caps.get("definitionProvider").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_lsp_hover_function() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":3}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let contents = result.get("contents").unwrap();
        let value = contents.get("value").unwrap().as_str().unwrap();
        assert!(value.contains("add"));
        assert!(value.contains("Function"));
    }

    #[test]
    fn test_lsp_hover_keyword() {
        let mut server = LspServer::new();
        let source = "fn main() { return }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":12}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let contents = result.get("contents").unwrap();
        let value = contents.get("value").unwrap().as_str().unwrap();
        assert!(value.contains("Return"));
    }

    #[test]
    fn test_lsp_definition() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}\nfn main() {\n    let x = add(1, 2)\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Cursor on "add" in main function (line 4, char 14)
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":4,"character":14}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        assert!(result.get("uri").is_some());
        let range = result.get("range").unwrap();
        assert!(range.get("start").is_some());
    }

    #[test]
    fn test_lsp_completion_keywords() {
        let mut server = LspServer::new();
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "".to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":0}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        assert!(items.len() > 10); // Should have many keywords
        assert!(items.iter().any(|item| item.get("label").unwrap().as_str().unwrap() == "fn"));
        assert!(items.iter().any(|item| item.get("label").unwrap().as_str().unwrap() == "struct"));
    }

    #[test]
    fn test_lsp_completion_symbols() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b }\nstruct Point { x: f64, y: f64 }\nenum Color { Red, Green }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":0}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        assert!(items.iter().any(|item| item.get("label").unwrap().as_str().unwrap() == "add"));
        assert!(items.iter().any(|item| item.get("label").unwrap().as_str().unwrap() == "Point"));
        assert!(items.iter().any(|item| item.get("label").unwrap().as_str().unwrap() == "Color"));
    }

    #[test]
    fn test_lsp_completion_prefix_filter() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b }\nfn subtract(a: i32, b: i32) -> i32 { return a - b }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":2}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        // Should filter by prefix "fn" (source starts with "fn add...", char 0-1 = "fn")
        assert!(items.iter().any(|item| item.get("label").unwrap().as_str().unwrap() == "fn"));
        // Ensure filtered results are relevant
        for item in items {
            let label = item.get("label").unwrap().as_str().unwrap();
            assert!(label.starts_with("fn"), "Label '{}' should start with prefix 'fn'", label);
        }
    }

    #[test]
    fn test_lsp_document_symbols() {
        let mut server = LspServer::new();
        let source = r#"pub fn add(a: i32, b: i32) -> i32 { return a + b }
struct Point { x: f64, y: f64 }
enum Color { Red, Green }"#;
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///test.ruva"}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert_eq!(symbols.len(), 3);
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "add"));
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "Point"));
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "Color"));
    }

    #[test]
    fn test_lsp_diagnostics() {
        let mut server = LspServer::new();
        let source = "fn main() { let x = y }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/diagnostic","params":{"textDocument":{"uri":"file:///test.ruva"}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let diagnostics = result.get("diagnostics").unwrap().as_array().unwrap();
        assert!(!diagnostics.is_empty()); // Should have at least one diagnostic for undefined variable
        let diag = &diagnostics[0];
        assert!(diag.get("message").unwrap().as_str().unwrap().contains("not defined"));
    }

    #[test]
    fn test_lsp_diagnostics_clean() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}\nfn main() {\n    let x = add(1, 2)\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/diagnostic","params":{"textDocument":{"uri":"file:///test.ruva"}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let diagnostics = result.get("diagnostics").unwrap().as_array().unwrap();
        let errors: Vec<&JsonValue> = diagnostics.iter().filter(|d| d.get("severity").unwrap().as_f64().unwrap() == 1.0).collect();
        assert!(errors.is_empty()); // No errors for valid code
    }

    #[test]
    fn test_lsp_hover_struct() {
        let mut server = LspServer::new();
        let source = "struct Point {\n    pub x: f64,\n    pub y: f64,\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":7}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let contents = result.get("contents").unwrap();
        let value = contents.get("value").unwrap().as_str().unwrap();
        assert!(value.contains("Point"));
        assert!(value.contains("Struct"));
    }

    #[test]
    fn test_lsp_hover_enum() {
        let mut server = LspServer::new();
        let source = "enum Color {\n    Red,\n    Green,\n    Blue,\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":5}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let contents = result.get("contents").unwrap();
        let value = contents.get("value").unwrap().as_str().unwrap();
        assert!(value.contains("Color"));
        assert!(value.contains("Enum"));
    }

    #[test]
    fn test_lsp_hover_class() {
        let mut server = LspServer::new();
        let source = "class Dog {\n    pub let name: string\n    pub fn bark(&self) {\n        println!(\"woof\")\n    }\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":6}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let contents = result.get("contents").unwrap();
        let value = contents.get("value").unwrap().as_str().unwrap();
        assert!(value.contains("Dog"));
        assert!(value.contains("Class"));
    }

    #[test]
    fn test_lsp_hover_no_result() {
        let mut server = LspServer::new();
        let source = "fn main() { }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":0}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        // Should return null result for "fn" keyword... actually fn is a keyword
        let result = response.get("result").unwrap();
        // Keywords should have hover info
        assert!(result.get("contents").is_some());
    }

    #[test]
    fn test_lsp_hover_empty_position() {
        let mut server = LspServer::new();
        let source = "fn main() { }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Position outside the text
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":10,"character":0}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_lsp_definition_struct() {
        let mut server = LspServer::new();
        let source = "struct Point {\n    x: f64,\n    y: f64,\n}\nfn main() {\n    let p = Point\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Cursor on "Point" in main function (line 5, char 12)
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":5,"character":12}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let range = result.get("range").unwrap();
        let start = range.get("start").unwrap();
        // Should point to the struct definition location (within 0-indexed range [0, 5])
        let line = start.get("line").unwrap().as_f64().unwrap();
        assert!(line >= 0.0 && line <= 5.0, "Expected line 0-5, got {}", line);
    }

    #[test]
    fn test_lsp_definition_no_result() {
        let mut server = LspServer::new();
        let source = "fn main() { }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Cursor on "main" which is defined in the same line
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":3}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        // Should find the definition of "main"
        assert!(result.get("uri").is_some());
    }

    #[test]
    fn test_lsp_completion_insert_text() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":0}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        let add_item = items.iter().find(|item| item.get("label").unwrap().as_str().unwrap() == "add").unwrap();
        let insert_text = add_item.get("insertText").unwrap().as_str().unwrap();
        assert!(insert_text.starts_with("add("));
    }

    #[test]
    fn test_document_store_get_document_symbols() {
        let mut store = DocumentStore::new();
        let source = r#"pub fn add(a: i32, b: i32) -> i32 {
    return a + b
}

class Calculator {
    pub fn calculate(&self, x: i32) -> i32 {
        return x
    }
}"#;
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let symbols = store.get_document_symbols("file:///test.ruva");
        assert!(symbols.iter().any(|(name, kind, _)| name == "add" && kind == "function"));
        assert!(symbols.iter().any(|(name, kind, _)| name == "Calculator" && kind == "class"));
        assert!(symbols.iter().any(|(name, kind, _)| name == "calculate" && kind == "method"));
    }

    #[test]
    fn test_parse_error_location() {
        let (line, col) = parse_error_location("Expected LBrace, got Lt at 5:12");
        assert_eq!(line, 5);
        assert_eq!(col, 12);
    }

    #[test]
    fn test_parse_error_location_no_info() {
        let (line, col) = parse_error_location("Some random error message");
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_get_word_at_position() {
        let server = LspServer::new();
        let text = "fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}";
        assert_eq!(server.get_word_at_position(text, 0, 3), "add");
        assert_eq!(server.get_word_at_position(text, 0, 0), "fn");
        assert_eq!(server.get_word_at_position(text, 1, 11), "a");
        assert_eq!(server.get_word_at_position(text, 10, 0), ""); // Out of bounds
    }

    #[test]
    fn test_get_prefix_at_position() {
        let server = LspServer::new();
        let text = "fn main() {\n    let x = add(1)\n}";
        // Position is AFTER the last char of the prefix
        assert_eq!(server.get_prefix_at_position(text, 1, 15), "add");
        assert_eq!(server.get_prefix_at_position(text, 0, 2), "fn");
        assert_eq!(server.get_prefix_at_position(text, 1, 9), "x");
    }

    // ─── References Tests ──────────────────────────────────────────

    #[test]
    fn test_references_basic() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}\nfn main() {\n    let x = add(1, 2)\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/references","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":3},"context":{"includeDeclaration":true}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let locations = result.as_array().unwrap();
        // Should find the definition and usage of "add"
        assert!(locations.len() >= 2, "Expected at least 2 references for 'add', got {}", locations.len());
        // All references should be in the same file
        for loc in locations {
            assert_eq!(loc.get("uri").unwrap().as_str().unwrap(), "file:///test.ruva");
        }
    }

    #[test]
    fn test_references_empty_word() {
        let mut server = LspServer::new();
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // Position on a space (char 2) — should yield empty word
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/references","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":2},"context":{"includeDeclaration":true}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let locations = result.as_array().unwrap();
        assert!(locations.is_empty());
    }

    #[test]
    fn test_references_struct() {
        let mut server = LspServer::new();
        let source = "struct Point {\n    x: f64,\n    y: f64,\n}\nfn main() {\n    let p = Point { x: 1.0, y: 2.0 }\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Cursor on "Point" definition (line 0, char 7)
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/references","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":7},"context":{"includeDeclaration":true}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let locations = result.as_array().unwrap();
        // Should find definition + usage in main
        assert!(locations.len() >= 2, "Expected at least 2 references for 'Point', got {}", locations.len());
    }

    #[test]
    fn test_references_multiple_usages() {
        let mut server = LspServer::new();
        let source = "fn helper(x: i32) -> i32 { return x }\nfn a() -> i32 { return helper(1) }\nfn b() -> i32 { return helper(2) }\nfn c() -> i32 { return helper(3) }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Cursor on "helper" definition (line 0, char 3)
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/references","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":3},"context":{"includeDeclaration":true}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let locations = result.as_array().unwrap();
        // Should find definition + 3 usages = 4
        assert!(locations.len() >= 4, "Expected at least 4 references for 'helper', got {}", locations.len());
    }

    // ─── Rename Tests ───────────────────────────────────────────────

    #[test]
    fn test_rename_basic() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}\nfn main() {\n    let x = add(1, 2)\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Rename "add" to "sum" at definition site
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":3},"newName":"sum"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let changes = result.get("changes").unwrap().as_array().unwrap();
        assert_eq!(changes.len(), 1); // One file changed
        let file_changes = changes[0].get("edits").unwrap().as_array().unwrap();
        assert!(file_changes.len() >= 2, "Expected at least 2 rename edits for 'add', got {}", file_changes.len());
        // All edits should have newText "sum"
        for edit in file_changes {
            assert_eq!(edit.get("newText").unwrap().as_str().unwrap(), "sum");
        }
    }

    #[test]
    fn test_rename_same_name() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32) -> i32 { return a }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Rename "add" to "add" (same name) — should return null
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":3},"newName":"add"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_rename_struct() {
        let mut server = LspServer::new();
        let source = "struct Point {\n    x: f64,\n}\nfn main() {\n    let p = Point { x: 1.0 }\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Rename "Point" to "Vec2"
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":7},"newName":"Vec2"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let changes = result.get("changes").unwrap().as_array().unwrap();
        let file_changes = changes[0].get("edits").unwrap().as_array().unwrap();
        assert!(file_changes.len() >= 2);
        for edit in file_changes {
            assert_eq!(edit.get("newText").unwrap().as_str().unwrap(), "Vec2");
        }
    }

    #[test]
    fn test_rename_empty_word() {
        let mut server = LspServer::new();
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // Position on a space (char 2) — should yield empty word, null result
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":2},"newName":"foo"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        assert!(result.is_null());
    }

    // ─── Symbol Index Tests ─────────────────────────────────────────

    #[test]
    fn test_symbol_index_definitions() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn add(a: i32, b: i32) -> i32 { return a + b }\nstruct Point { x: f64 }\nenum Color { Red }".to_string(),
        });

        let index = store.get_symbol_index("file:///test.ruva").unwrap();
        assert!(index.definitions.contains_key("add"));
        assert!(index.definitions.contains_key("Point"));
        assert!(index.definitions.contains_key("Color"));
    }

    #[test]
    fn test_symbol_index_usages() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn helper(x: i32) -> i32 { return x }\nfn main() { let y = helper(1) }".to_string(),
        });

        let index = store.get_symbol_index("file:///test.ruva").unwrap();
        // "helper" should have a definition and at least one usage
        let defs = index.definitions.get("helper").unwrap();
        assert!(!defs.is_empty());
        let usages = index.usages.get("helper").unwrap();
        assert!(!usages.is_empty());
    }

    #[test]
    fn test_find_references_cross_file() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///a.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "pub fn helper(x: i32) -> i32 { return x }".to_string(),
        });
        store.open(TextDocumentItem {
            uri: "file:///b.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() { let y = helper(1) }".to_string(),
        });

        let refs = store.find_references("file:///a.ruva", "helper");
        // Should find the definition in a.ruva and usage in b.ruva
        let a_refs: Vec<&SymbolLocation> = refs.iter().filter(|r| r.uri == "file:///a.ruva").collect();
        let b_refs: Vec<&SymbolLocation> = refs.iter().filter(|r| r.uri == "file:///b.ruva").collect();
        assert!(!a_refs.is_empty(), "Expected references in a.ruva");
        assert!(!b_refs.is_empty(), "Expected references in b.ruva");
    }

    #[test]
    fn test_apply_rename() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn add(a: i32) -> i32 {\n    return add(a)\n}".to_string(),
        });

        let results = store.apply_rename("file:///test.ruva", "add", "sum");
        let new_text = results.get("file:///test.ruva").unwrap();
        assert!(new_text.contains("fn sum("), "Expected 'fn sum(' in renamed text, got: {}", new_text);
        assert!(new_text.contains("return sum(a)"), "Expected 'return sum(a)' in renamed text, got: {}", new_text);
        assert!(!new_text.contains("add"), "Should not contain old name 'add', got: {}", new_text);
    }

    // ─── Signature Help Tests ─────────────────────────────────────

    #[test]
    fn test_signature_help_basic() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b }\nfn main() {\n    let x = add(1, 2)\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // "    let x = add(1," — char 18 is right after the comma
        // Cursor right after the comma — should show add(a: i32, b: i32) with activeParameter=1
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":2,"character":18}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let sigs = result.get("signatures").unwrap().as_array().unwrap();
        assert_eq!(sigs.len(), 1);
        let label = sigs[0].get("label").unwrap().as_str().unwrap();
        assert!(label.contains("add"), "Expected 'add' in signature label, got: {}", label);
        assert!(label.contains("i32"), "Expected 'i32' in signature label, got: {}", label);
        // After comma, active param should be 1
        let active_param = result.get("activeParameter").unwrap().as_f64().unwrap();
        assert_eq!(active_param, 1.0);
    }

    #[test]
    fn test_signature_help_first_arg() {
        let mut server = LspServer::new();
        let source = "fn greet(name: string) {}\nfn main() {\n    greet(\"hello\")\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // "    greet(" — char 11 is right after the opening paren
        // Cursor right after the opening paren — activeParameter should be 0
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":2,"character":11}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let active_param = result.get("activeParameter").unwrap().as_f64().unwrap();
        assert_eq!(active_param, 0.0);
    }

    #[test]
    fn test_signature_help_no_call() {
        let mut server = LspServer::new();
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "let x = 42".to_string(),
        });

        // Cursor on a number — no function call context
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":0,"character":8}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_signature_help_method() {
        let mut server = LspServer::new();
        let source = "class Dog {\n    pub fn bark(&self, times: i32) {}\n}\nfn main() {\n    let d = Dog { }\n    d.bark(3)\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":5,"character":11}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let sigs = result.get("signatures").unwrap().as_array().unwrap();
        assert_eq!(sigs.len(), 1);
        let label = sigs[0].get("label").unwrap().as_str().unwrap();
        assert!(label.contains("bark"), "Expected 'bark' in signature, got: {}", label);
    }

    // ─── Code Action Tests ────────────────────────────────────────

    #[test]
    fn test_code_action_undefined_var() {
        let mut server = LspServer::new();
        let source = "fn main() { let x = y }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Request code actions for the whole file
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/codeAction","params":{"textDocument":{"uri":"file:///test.ruva"},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":24}},"context":{"diagnostics":[]}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let actions = result.as_array().unwrap();
        assert!(!actions.is_empty(), "Expected at least one code action for undefined variable");
        // Should have "Create variable" and "Initialize with null" actions
        assert!(actions.iter().any(|a| a.get("title").unwrap().as_str().unwrap().contains("Create variable")));
        assert!(actions.iter().any(|a| a.get("title").unwrap().as_str().unwrap().contains("Initialize")));
        // Each action should have an edit
        for action in actions {
            assert!(action.get("edit").is_some(), "Expected code action to have an edit");
        }
    }

    #[test]
    fn test_code_action_clean_code() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Request code actions for clean code — should return empty
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/codeAction","params":{"textDocument":{"uri":"file:///test.ruva"},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":46}},"context":{"diagnostics":[]}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let actions = result.as_array().unwrap();
        assert!(actions.is_empty(), "Expected no code actions for clean code");
    }

    #[test]
    fn test_code_action_has_kind() {
        let mut server = LspServer::new();
        let source = "fn main() { let x = y }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/codeAction","params":{"textDocument":{"uri":"file:///test.ruva"},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":24}},"context":{"diagnostics":[]}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let actions = result.as_array().unwrap();
        // All actions should be quickfix kind
        for action in actions {
            assert_eq!(action.get("kind").unwrap().as_str().unwrap(), "quickfix");
        }
    }

    #[test]
    fn test_signature_help_params() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32, b: i32) -> i32 { return a + b }\nfn main() {\n    add(1, 2)\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // "    add(1, 2)" — char 10 is right after the comma
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///test.ruva"},"position":{"line":2,"character":10}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let sigs = result.get("signatures").unwrap().as_array().unwrap();
        let params = sigs[0].get("parameters").unwrap().as_array().unwrap();
        assert_eq!(params.len(), 2);
        assert!(params[0].get("label").unwrap().as_str().unwrap().contains("a"));
        assert!(params[1].get("label").unwrap().as_str().unwrap().contains("b"));
    }

    // ─── Workspace Symbol Tests ────────────────────────────────────

    #[test]
    fn test_workspace_symbol_empty_query() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32) -> i32 { return a }\nstruct Point { x: f64 }\nenum Color { Red }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":""}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert!(symbols.len() >= 3, "Expected at least 3 symbols (add, Point, Color), got {}", symbols.len());
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "add"));
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "Point"));
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "Color"));
    }

    #[test]
    fn test_workspace_symbol_exact_match() {
        let mut server = LspServer::new();
        let source = "fn add(a: i32) -> i32 { return a }\nfn subtract(a: i32) -> i32 { return a }\nstruct Point { x: f64 }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":"add"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].get("name").unwrap().as_str().unwrap(), "add");
    }

    #[test]
    fn test_workspace_symbol_fuzzy_match() {
        let mut server = LspServer::new();
        let source = "fn get_user_name() {}\nfn set_user_email() {}\nfn get_config() {}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Fuzzy query "gun" should match "get_user_name" (g-u-n in order)
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":"gun"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "get_user_name"));
        // Should NOT match "get_config" (no 'n')
        assert!(!symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "get_config"));
    }

    #[test]
    fn test_workspace_symbol_case_insensitive() {
        let mut server = LspServer::new();
        let source = "fn Calculate() {}\nstruct Point { x: f64 }";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Lowercase query should match uppercase symbol
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":"calc"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "Calculate"));
    }

    #[test]
    fn test_workspace_symbol_has_kind() {
        let mut server = LspServer::new();
        let source = "fn helper() {}\nstruct Box {}\nenum Option {}\ntrait Printable {}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":""}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        for sym in symbols {
            assert!(sym.get("kind").is_some(), "Every symbol should have a kind");
            assert!(sym.get("location").is_some(), "Every symbol should have a location");
            assert!(sym.get("name").is_some(), "Every symbol should have a name");
        }
    }

    #[test]
    fn test_workspace_symbol_no_match() {
        let mut server = LspServer::new();
        let source = "fn add() {}\nstruct Point {}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":"zzz"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert!(symbols.is_empty(), "Expected no matches for 'zzz', got {}", symbols.len());
    }

    #[test]
    fn test_workspace_symbol_cross_file() {
        let mut server = LspServer::new();
        server.store.open(TextDocumentItem {
            uri: "file:///a.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn helper() {}".to_string(),
        });
        server.store.open(TextDocumentItem {
            uri: "file:///b.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn process() {}".to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":""}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert!(symbols.len() >= 2, "Expected symbols from both files, got {}", symbols.len());
        let uris: Vec<&str> = symbols.iter().map(|s| {
            s.get("location").unwrap().get("uri").unwrap().as_str().unwrap()
        }).collect();
        assert!(uris.contains(&"file:///a.ruva"));
        assert!(uris.contains(&"file:///b.ruva"));
    }

    #[test]
    fn test_workspace_symbol_class_methods() {
        let mut server = LspServer::new();
        let source = "class Calculator {\n    pub fn add(&self, a: i32) -> i32 { return a }\n    pub fn subtract(&self, a: i32) -> i32 { return a }\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        // Should find both the class and its methods
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":""}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "Calculator"));
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "add"));
        assert!(symbols.iter().any(|s| s.get("name").unwrap().as_str().unwrap() == "subtract"));
    }

    #[test]
    fn test_workspace_symbol_container_name() {
        let mut server = LspServer::new();
        let source = "class Dog {\n    pub fn bark(&self) {}\n}";
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: source.to_string(),
        });

        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"workspace/symbol","params":{"query":"bark"}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let result = response.get("result").unwrap();
        let symbols = result.as_array().unwrap();
        assert_eq!(symbols.len(), 1);
        let container = symbols[0].get("containerName").unwrap().as_str().unwrap();
        assert!(container.contains("Dog"), "Expected container to contain 'Dog', got: {}", container);
    }

    #[test]
    fn test_fuzzy_match_basic() {
        assert!(fuzzy_match("get_user_name", "gun"));
        assert!(fuzzy_match("get_user_name", "get"));
        assert!(fuzzy_match("get_user_name", "g_n"));
        assert!(fuzzy_match("get_user_name", ""));
        assert!(fuzzy_match("Calculate", "calc"));
        assert!(!fuzzy_match("add", "xyz"));
        assert!(!fuzzy_match("ab", "abc"));
        assert!(fuzzy_match("FunctionDef", "fd"));
        assert!(fuzzy_match("FunctionDef", "FD")); // case insensitive
    }

    // ─── Incremental Sync Tests ─────────────────────────────────────

    #[test]
    fn test_incremental_insert() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // Insert " hello" before ')' (line 0, char 8)
        // "fn main() {}" — char 8 is ')', insert before it
        let change = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 0, character: 8 },
                end: Position { line: 0, character: 8 },
            }),
            text: " hello".to_string(),
        };
        store.change("file:///test.ruva", vec![change]);
        assert_eq!(store.get_text("file:///test.ruva").unwrap(), "fn main( hello) {}");
    }

    #[test]
    fn test_incremental_delete() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // Delete "main" (line 0, char 3 -> char 7)
        let change = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 0, character: 3 },
                end: Position { line: 0, character: 7 },
            }),
            text: String::new(),
        };
        store.change("file:///test.ruva", vec![change]);
        assert_eq!(store.get_text("file:///test.ruva").unwrap(), "fn () {}");
    }

    #[test]
    fn test_incremental_replace() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // Replace "main" with "run" (line 0, char 3 -> char 7)
        let change = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 0, character: 3 },
                end: Position { line: 0, character: 7 },
            }),
            text: "run".to_string(),
        };
        store.change("file:///test.ruva", vec![change]);
        assert_eq!(store.get_text("file:///test.ruva").unwrap(), "fn run() {}");
    }

    #[test]
    fn test_incremental_multiline() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {\n    let x = 1\n}".to_string(),
        });

        // Replace "let x = 1" on line 1 with "let y = 2" (line 1, char 4 -> char 13)
        let change = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 1, character: 4 },
                end: Position { line: 1, character: 13 },
            }),
            text: "let y = 2".to_string(),
        };
        store.change("file:///test.ruva", vec![change]);
        assert_eq!(store.get_text("file:///test.ruva").unwrap(), "fn main() {\n    let y = 2\n}");
    }

    #[test]
    fn test_incremental_full_replacement() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // No range = full document replacement
        let change = TextDocumentContentChangeEvent {
            range: None,
            text: "fn run() {}".to_string(),
        };
        store.change("file:///test.ruva", vec![change]);
        assert_eq!(store.get_text("file:///test.ruva").unwrap(), "fn run() {}");
    }

    #[test]
    fn test_incremental_multiple_edits() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn add(a: i32, b: i32) -> i32 { return a + b }".to_string(),
        });

        // Apply two edits: rename "add" to "sum" and "a" to "x"
        // "fn add(a: i32, b: i32)" — 'add' is chars 3..6, 'a' is char 7
        let edit1 = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 0, character: 3 },
                end: Position { line: 0, character: 6 },
            }),
            text: "sum".to_string(),
        };
        let edit2 = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 0, character: 7 },
                end: Position { line: 0, character: 8 },
            }),
            text: "x".to_string(),
        };
        store.change("file:///test.ruva", vec![edit1, edit2]);
        let text = store.get_text("file:///test.ruva").unwrap();
        assert!(text.contains("fn sum("), "Expected 'fn sum(', got: {}", text);
        assert!(text.contains("x: i32"), "Expected 'x: i32', got: {}", text);
    }

    #[test]
    fn test_incremental_preserves_surrounding() {
        let mut store = DocumentStore::new();
        let original = "// Header\nfn main() {\n    return 42\n}";
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: original.to_string(),
        });

        // Replace just the return value
        let change = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 2, character: 11 },
                end: Position { line: 2, character: 13 },
            }),
            text: "0".to_string(),
        };
        store.change("file:///test.ruva", vec![change]);
        let text = store.get_text("file:///test.ruva").unwrap();
        assert!(text.starts_with("// Header"), "Header preserved: {}", text);
        assert!(text.contains("return 0"), "Return value replaced: {}", text);
        assert!(text.ends_with("}"), "Closing brace preserved: {}", text);
    }

    #[test]
    fn test_version_tracking() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });
        assert_eq!(store.get_version("file:///test.ruva"), Some(1));

        store.set_version("file:///test.ruva", 5);
        assert_eq!(store.get_version("file:///test.ruva"), Some(5));

        assert_eq!(store.get_version("file:///other.ruva"), None);
    }

    #[test]
    fn test_incremental_insert_newline() {
        let mut store = DocumentStore::new();
        store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // Insert a newline + new line after the opening brace
        let change = TextDocumentContentChangeEvent {
            range: Some(TextRange {
                start: Position { line: 0, character: 11 },
                end: Position { line: 0, character: 11 },
            }),
            text: "\n    return 0".to_string(),
        };
        store.change("file:///test.ruva", vec![change]);
        let text = store.get_text("file:///test.ruva").unwrap();
        assert!(text.contains("fn main() {\n    return 0"), "Newline inserted: {}", text);
    }

    #[test]
    fn test_lsp_incremental_sync_capability() {
        let mut server = LspServer::new();
        let msg = json_parse(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":"file:///test","capabilities":{}}}"#).unwrap();
        let response = server.handle_message(&msg).unwrap();
        let caps = response.get("result").unwrap().get("capabilities").unwrap();
        let sync = caps.get("textDocumentSync").unwrap();
        // Change should be 2 for incremental sync
        assert_eq!(sync.get("change").unwrap().as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_lsp_did_change_incremental() {
        let mut server = LspServer::new();
        server.store.open(TextDocumentItem {
            uri: "file:///test.ruva".to_string(),
            language_id: "ruva".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        });

        // Send incremental change via LSP: replace "main" with "run"
        let msg = json_parse(r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///test.ruva","version":2},"contentChanges":[{"range":{"start":{"line":0,"character":3},"end":{"line":0,"character":7}},"text":"run"}]}}"#).unwrap();
        server.handle_message(&msg);

        // Verify the document was updated incrementally
        assert_eq!(server.store.get_text("file:///test.ruva").unwrap(), "fn run() {}");
        assert_eq!(server.store.get_version("file:///test.ruva"), Some(2));
    }
}
