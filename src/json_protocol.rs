


#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_f64().map(|n| n as i64)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
}



pub struct JsonParser {
    chars: Vec<char>,
    pos: usize,

    depth: u32,
    max_depth: u32,
}

impl JsonParser {

    const DEFAULT_MAX_DEPTH: u32 = 64;

    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            depth: 0,
            max_depth: Self::DEFAULT_MAX_DEPTH,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    #[allow(dead_code)]
    fn expect_char(&mut self, expected: char) -> bool {
        self.skip_whitespace();
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn parse_value(&mut self) -> Option<JsonValue> {
        self.skip_whitespace();
        match self.peek()? {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' => self.parse_string().map(JsonValue::Str),
            't' | 'f' => self.parse_bool(),
            'n' => {
                self.advance();
                self.advance();
                self.advance();
                self.advance();
                Some(JsonValue::Null)
            }
            _ => self.parse_number(),
        }
    }

    fn parse_object(&mut self) -> Option<JsonValue> {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            return None;

        }
        let result = self.parse_object_inner();
        self.depth -= 1;
        result
    }

    fn parse_object_inner(&mut self) -> Option<JsonValue> {
        self.advance();

        let mut pairs = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.advance();
            return Some(JsonValue::Object(pairs));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.advance();

            let value = self.parse_value()?;
            pairs.push((key, value));
            self.skip_whitespace();
            match self.peek()? {
                ',' => { self.advance(); }
                '}' => { self.advance(); break; }
                _ => return None,
            }
        }
        Some(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Option<JsonValue> {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            return None;

        }
        let result = self.parse_array_inner();
        self.depth -= 1;
        result
    }

    fn parse_array_inner(&mut self) -> Option<JsonValue> {
        self.advance();

        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.advance();
            return Some(JsonValue::Array(items));
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_whitespace();
            match self.peek()? {
                ',' => { self.advance(); }
                ']' => { self.advance(); break; }
                _ => return None,
            }
        }
        Some(JsonValue::Array(items))
    }

    fn parse_string(&mut self) -> Option<String> {
        self.skip_whitespace();
        if self.peek() != Some('"') {
            return None;
        }
        self.advance();

        let mut s = String::new();
        loop {
            match self.advance()? {
                '"' => return Some(s),
                '\\' => match self.advance()? {
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    '/' => s.push('/'),
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'b' => s.push('\u{0008}'),
                    'f' => s.push('\u{000C}'),
                    'u' => {
                        let hex: String = (0..4).filter_map(|_| self.advance()).collect();
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(c) = char::from_u32(code) {
                                s.push(c);
                            }
                        }
                    }
                    c => s.push(c),
                },
                c => s.push(c),
            }
        }
    }

    fn parse_number(&mut self) -> Option<JsonValue> {
        self.skip_whitespace();
        let start = self.pos;
        if self.peek() == Some('-') {
            self.advance();
        }
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        if start == self.pos {
            return None;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>().ok().map(JsonValue::Number)
    }

    fn parse_bool(&mut self) -> Option<JsonValue> {
        self.skip_whitespace();
        let remaining: String = self.chars[self.pos..].iter().take(5).collect();
        if remaining.starts_with("true") {
            self.pos += 4;
            Some(JsonValue::Bool(true))
        } else if remaining.starts_with("false") {
            self.pos += 5;
            Some(JsonValue::Bool(false))
        } else {
            None
        }
    }
}

pub fn json_parse(input: &str) -> Option<JsonValue> {
    JsonParser::new(input).parse_value()
}



pub fn json_stringify(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => {
            if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        JsonValue::Str(s) => {
            let mut out = String::from('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        JsonValue::Array(items) => {
            let inner: Vec<String> = items.iter().map(json_stringify).collect();
            format!("[{}]", inner.join(","))
        }
        JsonValue::Object(pairs) => {
            let inner: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}:{}", json_stringify(&JsonValue::Str(k.clone())), json_stringify(v)))
                .collect();
            format!("{{{}}}", inner.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse_null() {
        let val = json_parse("null").unwrap();
        assert_eq!(val, JsonValue::Null);
    }

    #[test]
    fn test_json_parse_bool() {
        let val = json_parse("true").unwrap();
        assert_eq!(val, JsonValue::Bool(true));
        let val = json_parse("false").unwrap();
        assert_eq!(val, JsonValue::Bool(false));
    }

    #[test]
    fn test_json_parse_number() {
        let val = json_parse("42").unwrap();
        assert_eq!(val, JsonValue::Number(42.0));
        let val = json_parse("-3.14").unwrap();
        assert_eq!(val, JsonValue::Number(-3.14));
    }

    #[test]
    fn test_json_parse_string() {
        let val = json_parse("\"hello\"").unwrap();
        assert_eq!(val, JsonValue::Str("hello".to_string()));
        let val = json_parse("\"escape\\nnewline\"").unwrap();
        assert_eq!(val, JsonValue::Str("escape\nnewline".to_string()));
    }

    #[test]
    fn test_json_parse_array() {
        let val = json_parse("[1, 2, 3]").unwrap();
        let arr = val.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], JsonValue::Number(1.0));
    }

    #[test]
    fn test_json_parse_object() {
        let val = json_parse("{\"key\": \"value\", \"num\": 42}").unwrap();
        assert_eq!(val.get("key").unwrap().as_str().unwrap(), "value");
        assert_eq!(val.get("num").unwrap().as_f64().unwrap(), 42.0);
    }

    #[test]
    fn test_json_parse_nested() {
        let json = r#"{"arr": [1, {"nested": true}], "str": "hello"}"#;
        let val = json_parse(json).unwrap();
        assert_eq!(val.get("str").unwrap().as_str().unwrap(), "hello");
        let arr = val.get("arr").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_json_stringify_null() {
        assert_eq!(json_stringify(&JsonValue::Null), "null");
    }

    #[test]
    fn test_json_stringify_bool() {
        assert_eq!(json_stringify(&JsonValue::Bool(true)), "true");
        assert_eq!(json_stringify(&JsonValue::Bool(false)), "false");
    }

    #[test]
    fn test_json_stringify_string() {
        assert_eq!(json_stringify(&JsonValue::Str("hello".to_string())), "\"hello\"");
        assert_eq!(json_stringify(&JsonValue::Str("a\"b".to_string())), "\"a\\\"b\"");
    }

    #[test]
    fn test_json_stringify_array() {
        let val = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Str("two".to_string()),
        ]);
        assert_eq!(json_stringify(&val), "[1,\"two\"]");
    }

    #[test]
    fn test_json_stringify_object() {
        let val = JsonValue::Object(vec![
            ("a".to_string(), JsonValue::Number(1.0)),
        ]);
        assert_eq!(json_stringify(&val), "{\"a\":1}");
    }

    #[test]
    fn test_json_roundtrip() {
        let original = r#"{"name":"Ruva","version":1,"tags":["fun","fast"],"config":{"debug":true}}"#;
        let parsed = json_parse(original).unwrap();
        let serialized = json_stringify(&parsed);
        let reparsed = json_parse(&serialized).unwrap();
        assert_eq!(json_stringify(&parsed), json_stringify(&reparsed));
    }

    #[test]
    fn test_json_escape_special_chars() {
        let val = JsonValue::Str("line1\nline2\ttab\"quote".to_string());
        let s = json_stringify(&val);
        assert!(s.contains("\\n"));
        assert!(s.contains("\\t"));
        assert!(s.contains("\\\""));
    }

    #[test]
    fn test_json_number_int_vs_float() {
        let val = JsonValue::Number(42.0);
        let s = json_stringify(&val);
        assert_eq!(s, "42");

        let val = JsonValue::Number(3.14);
        let s = json_stringify(&val);
        assert!(s.contains('.'));
    }
}

