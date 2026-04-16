#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub expected: String,
    pub found: String,
    pub line: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected {}, found {} at line {}",
            self.expected, self.found, self.line
        )
    }
}

impl std::error::Error for ParseError {}
