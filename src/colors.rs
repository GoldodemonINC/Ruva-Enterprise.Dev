/// ANSI color codes for terminal output

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const DIM: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";

pub fn success(msg: &str) -> String {
    format!("{}✓{} {}", GREEN, RESET, msg)
}

pub fn error(msg: &str) -> String {
    format!("{}✗{} {}", RED, RESET, msg)
}

pub fn warning(msg: &str) -> String {
    format!("{}⚠{} {}", YELLOW, RESET, msg)
}

pub fn info(msg: &str) -> String {
    format!("{}⟳{} {}", CYAN, RESET, msg)
}

pub fn dim(msg: &str) -> String {
    format!("{}{}{}{}", DIM, msg, RESET, "")
}
