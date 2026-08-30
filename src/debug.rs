pub fn print_tokens(source: &str) {
    let mut lexer = crate::lexer::Lexer::new(source);
    match lexer.tokenize() {
        Ok(tokens) => {
            for (i, (tok, span)) in tokens.iter().enumerate() {
                println!("{:3}: {:?}  ({}:{})", i, tok, span.line, span.col);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

