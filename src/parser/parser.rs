use crate::ast::{
    Comparator, Condition, Constraint, ContextKind, Metadata, Operand, Rule, RuleSet, Severity,
    Statement,
};
use crate::lexer::{Token, TokenKind};

use super::error::ParseError;

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn parse_rule_set(&mut self) -> Result<RuleSet, ParseError> {
        let mut rules = Vec::new();
        while !self.check(TokenKind::Eof) {
            rules.push(self.parse_rule()?);
        }
        Ok(RuleSet { rules })
    }

    fn parse_rule(&mut self) -> Result<Rule, ParseError> {
        self.expect(TokenKind::KeywordRestriccion, "RESTRICCION")?;
        let name = self.expect_identifier("rule name")?.lexeme;
        self.expect(TokenKind::Colon, ":")?;
        self.expect(TokenKind::KeywordContexto, "CONTEXTO")?;
        let context_token = self.expect_identifier("context")?;
        let context = self.parse_context_kind(context_token)?;
        let statement = self.parse_statement()?;
        let metadata = self.parse_metadata_list()?;
        Ok(Rule {
            name,
            context,
            statement,
            metadata,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_kind(TokenKind::KeywordCuando) {
            let condition = self.parse_condition()?;
            self.expect(TokenKind::KeywordEntonces, "ENTONCES")?;
            let constraint = self.parse_constraint()?;
            Ok(Statement::Conditional {
                condition,
                constraint,
            })
        } else if self.match_kind(TokenKind::KeywordSiempre) {
            let constraint = self.parse_constraint()?;
            Ok(Statement::Always { constraint })
        } else {
            Err(self.error_expected("CUANDO or SIEMPRE"))
        }
    }

    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        let field = self.expect_identifier("condition field")?.lexeme;
        let comparator = self.parse_comparator()?;
        let value = self.parse_operand()?;
        Ok(Condition {
            field,
            comparator,
            value,
        })
    }

    fn parse_constraint(&mut self) -> Result<Constraint, ParseError> {
        let field = self.expect_identifier("constraint field")?.lexeme;
        if self.match_kind(TokenKind::KeywordFueraDe) {
            let from = self.expect_time("TIME")?.lexeme;
            let to = self.expect_time("TIME")?.lexeme;
            Ok(Constraint::OutsideRange { field, from, to })
        } else {
            let comparator = self.parse_comparator()?;
            let value = self.parse_operand()?;
            Ok(Constraint::Comparison {
                field,
                comparator,
                value,
            })
        }
    }

    fn parse_operand(&mut self) -> Result<Operand, ParseError> {
        if self.check(TokenKind::Number) {
            let token = self.advance().clone();
            let value = token.lexeme.parse::<f64>().map_err(|_| ParseError {
                expected: "number".to_string(),
                found: token.lexeme,
                line: token.line,
            })?;
            Ok(Operand::Number(value))
        } else if self.check(TokenKind::Identifier) {
            Ok(Operand::Field(self.advance().lexeme.clone()))
        } else {
            Err(self.error_expected("number or identifier"))
        }
    }

    fn parse_metadata_list(&mut self) -> Result<Metadata, ParseError> {
        let mut unit = None;
        let mut severity = None;
        let mut norm = None;
        while !self.check(TokenKind::Eof) && !self.check(TokenKind::KeywordRestriccion) {
            if self.match_kind(TokenKind::KeywordUnidad) {
                unit = Some(self.expect_identifier("unit")?.lexeme);
            } else if self.match_kind(TokenKind::KeywordSeveridad) {
                let ident = self.expect_identifier("severity")?.lexeme;
                severity = Some(self.parse_severity(&ident)?);
            } else if self.match_kind(TokenKind::KeywordNorma) {
                norm = Some(self.expect_string("string")?.lexeme);
            } else {
                let token = self.advance().clone();
                return Err(ParseError {
                    expected: "metadata or next rule".to_string(),
                    found: token.lexeme,
                    line: token.line,
                });
            }
        }
        Ok(Metadata {
            unit,
            severity: severity.unwrap_or(Severity::Operacional),
            norm,
        })
    }

    fn parse_context_kind(&self, token: Token) -> Result<ContextKind, ParseError> {
        match token.lexeme.as_str() {
            "piloto" => Ok(ContextKind::Piloto),
            "aeronave" => Ok(ContextKind::Aeronave),
            "vuelo" => Ok(ContextKind::Vuelo),
            "aeropuerto" => Ok(ContextKind::Aeropuerto),
            "tripulacion" => Ok(ContextKind::Tripulacion),
            _ => Err(ParseError {
                expected: "valid context".to_string(),
                found: token.lexeme,
                line: token.line,
            }),
        }
    }

    fn parse_severity(&self, value: &str) -> Result<Severity, ParseError> {
        match value {
            "critica" => Ok(Severity::Critica),
            "regulatoria" => Ok(Severity::Regulatoria),
            "operacional" => Ok(Severity::Operacional),
            _ => Err(ParseError {
                expected: "valid severity".to_string(),
                found: value.to_string(),
                line: self.peek().line,
            }),
        }
    }

    fn parse_comparator(&mut self) -> Result<Comparator, ParseError> {
        let token = self.expect(TokenKind::Comparator, "comparator")?;
        match token.lexeme.as_str() {
            ">=" => Ok(Comparator::Gte),
            "<=" => Ok(Comparator::Lte),
            ">" => Ok(Comparator::Gt),
            "<" => Ok(Comparator::Lt),
            "==" => Ok(Comparator::Eq),
            "!=" => Ok(Comparator::Neq),
            _ => Err(ParseError {
                expected: "comparator".to_string(),
                found: token.lexeme,
                line: token.line,
            }),
        }
    }

    fn expect(&mut self, kind: TokenKind, expected: &str) -> Result<Token, ParseError> {
        if self.check(kind.clone()) {
            Ok(self.advance().clone())
        } else {
            Err(self.error_expected(expected))
        }
    }

    fn expect_identifier(&mut self, expected: &str) -> Result<Token, ParseError> {
        self.expect(TokenKind::Identifier, expected)
    }

    fn expect_time(&mut self, expected: &str) -> Result<Token, ParseError> {
        self.expect(TokenKind::Time, expected)
    }

    fn expect_string(&mut self, expected: &str) -> Result<Token, ParseError> {
        self.expect(TokenKind::String, expected)
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn advance(&mut self) -> &Token {
        let current = self.index;
        if !self.check(TokenKind::Eof) {
            self.index += 1;
        }
        &self.tokens[current]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn error_expected(&self, expected: &str) -> ParseError {
        let token = self.peek();
        ParseError {
            expected: expected.to_string(),
            found: token.lexeme.clone(),
            line: token.line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::lexer::Tokenizer;

    #[test]
    fn parses_reference_rules() {
        let rules = include_str!("../../restricciones.aero");
        let tokens = Tokenizer::new(rules).tokenize().expect("tokenize");
        let mut parser = Parser::new(tokens);
        let rule_set = parser.parse_rule_set().expect("parse");
        assert_eq!(rule_set.rules.len(), 4);
        assert_eq!(rule_set.rules[0].name, "descanso_minimo_piloto");
    }
}
