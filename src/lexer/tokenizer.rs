use super::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerError {
    pub msg: String,
    pub line: usize,
    pub column: usize,
}

pub struct Tokenizer {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume_whitespace();
                continue;
            }
            if ch == '-' && self.peek_next() == Some('-') {
                self.consume_comment();
                continue;
            }
            if ch == ':' {
                tokens.push(self.simple(TokenKind::Colon, ":"));
                self.advance();
                continue;
            }
            if matches!(ch, '>' | '<' | '=' | '!') {
                tokens.push(self.consume_comparator()?);
                continue;
            }
            if ch == '"' {
                tokens.push(self.consume_string()?);
                continue;
            }
            if ch.is_ascii_digit() {
                tokens.push(self.consume_number_or_time());
                continue;
            }
            if ch.is_ascii_alphabetic() || ch == '_' {
                tokens.push(self.consume_word());
                continue;
            }
            return Err(LexerError {
                msg: format!("Unrecognized character '{ch}'"),
                line: self.line,
                column: self.column,
            });
        }
        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            line: self.line,
        });
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.index).copied();
        if let Some(c) = ch {
            self.index += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if !ch.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    fn consume_comment(&mut self) {
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '\n' {
                break;
            }
        }
    }

    fn simple(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            line: self.line,
        }
    }

    fn consume_string(&mut self) -> Result<Token, LexerError> {
        let line = self.line;
        self.advance();
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::String,
                    lexeme: value,
                    line,
                });
            }
            value.push(ch);
            self.advance();
        }
        Err(LexerError {
            msg: "Unclosed string".to_string(),
            line,
            column: self.column,
        })
    }

    fn consume_number_or_time(&mut self) -> Token {
        let line = self.line;
        let mut lexeme = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == ':' {
                lexeme.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        let kind = if lexeme.contains(':') {
            TokenKind::Time
        } else {
            TokenKind::Number
        };
        Token { kind, lexeme, line }
    }

    fn consume_comparator(&mut self) -> Result<Token, LexerError> {
        let line = self.line;
        let first = self.advance().ok_or(LexerError {
            msg: "Unexpected end of input".to_string(),
            line,
            column: self.column,
        })?;
        let mut lexeme = String::new();
        lexeme.push(first);
        if let Some(next) = self.peek() {
            if matches!(
                (first, next),
                ('>', '=') | ('<', '=') | ('=', '=') | ('!', '=')
            ) {
                lexeme.push(next);
                self.advance();
            }
        }
        Ok(Token {
            kind: TokenKind::Comparator,
            lexeme,
            line,
        })
    }

    fn consume_word(&mut self) -> Token {
        let line = self.line;
        let mut lexeme = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                lexeme.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        let kind = match lexeme.as_str() {
            "RESTRICCION" => TokenKind::KeywordRestriccion,
            "CONTEXTO" => TokenKind::KeywordContexto,
            "CUANDO" => TokenKind::KeywordCuando,
            "ENTONCES" => TokenKind::KeywordEntonces,
            "SIEMPRE" => TokenKind::KeywordSiempre,
            "UNIDAD" => TokenKind::KeywordUnidad,
            "SEVERIDAD" => TokenKind::KeywordSeveridad,
            "NORMA" => TokenKind::KeywordNorma,
            "FUERA_DE" => TokenKind::KeywordFueraDe,
            _ => TokenKind::Identifier,
        };
        Token { kind, lexeme, line }
    }
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, Tokenizer};

    #[test]
    fn tokenizes_reference_rule_file() {
        let input = r#"
-- comentario
RESTRICCION descanso_minimo_piloto:
  CONTEXTO piloto
  CUANDO vuelo.duracion > 6
  ENTONCES descanso_siguiente >= 10
  UNIDAD horas
  SEVERIDAD critica
  NORMA "EASA ORO.FTL.235"
"#;
        let tokens = Tokenizer::new(input).tokenize().expect("tokenize");
        assert_eq!(tokens[0].kind, TokenKind::KeywordRestriccion);
        assert!(tokens.iter().any(|t| t.kind == TokenKind::KeywordFueraDe) == false);
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Comparator));
        assert_eq!(tokens.last().expect("eof").kind, TokenKind::Eof);
    }
}
