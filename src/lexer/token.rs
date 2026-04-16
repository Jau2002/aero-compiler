#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    KeywordRestriccion,
    KeywordContexto,
    KeywordCuando,
    KeywordEntonces,
    KeywordSiempre,
    KeywordUnidad,
    KeywordSeveridad,
    KeywordNorma,
    KeywordFueraDe,
    Identifier,
    Number,
    Time,
    String,
    Comparator,
    Colon,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}
