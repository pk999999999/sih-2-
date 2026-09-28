use jocky_ast::{Argument, ForensicCall, Investigation, Literal, Predicate, Program, Statement, Target};
use jocky_lexer::{lex, LexError, SpannedToken, Token};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    Lex(#[from] LexError),
    #[error("expected {expected}, found {found} at byte {offset}")]
    Unexpected {
        expected: &'static str,
        found: String,
        offset: usize,
    },
    #[error("unexpected end of input while expecting {0}")]
    Eof(&'static str),
}

pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    Parser::new(lex(source)?).program()
}

struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn program(&mut self) -> Result<Program, ParseError> {
        let mut investigations = Vec::new();
        while !self.is_eof() {
            investigations.push(self.investigation()?);
        }
        Ok(Program { investigations })
    }

    fn investigation(&mut self) -> Result<Investigation, ParseError> {
        self.expect_keyword(TokenKind::Investigation, "investigation")?;
        let title = self.expect_string()?;
        self.expect_keyword(TokenKind::LBrace, "{")?;
        let target = self.target()?;
        let mut statements = Vec::new();
        while !self.check(TokenKind::RBrace) {
            statements.push(self.statement()?);
        }
        self.expect_keyword(TokenKind::RBrace, "}")?;
        Ok(Investigation {
            title,
            target,
            statements,
        })
    }

    fn target(&mut self) -> Result<Target, ParseError> {
        self.expect_keyword(TokenKind::Target, "target")?;
        if self.matches(TokenKind::Host) {
            self.expect_keyword(TokenKind::LParen, "(")?;
            let id = self.expect_string()?;
            self.expect_keyword(TokenKind::RParen, ")")?;
            Ok(Target::Host(id))
        } else if self.matches(TokenKind::Agent) {
            self.expect_keyword(TokenKind::LParen, "(")?;
            let id = self.expect_string()?;
            self.expect_keyword(TokenKind::RParen, ")")?;
            Ok(Target::Agent(id))
        } else {
            self.unexpected("host or agent")
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.matches(TokenKind::Collect) {
            let call = self.call()?;
            self.expect_keyword(TokenKind::As, "as")?;
            let alias = self.expect_identifier()?;
            Ok(Statement::Collect { call, alias })
        } else if self.matches(TokenKind::Analyze) {
            let source = self.expect_identifier()?;
            self.expect_keyword(TokenKind::Where, "where")?;
            let field = self.expect_identifier()?;
            let predicate = if self.matches(TokenKind::Contains) {
                Predicate::Contains {
                    field,
                    value: self.expect_string()?,
                }
            } else if self.matches(TokenKind::Equals) {
                Predicate::Equals {
                    field,
                    value: self.expect_string()?,
                }
            } else {
                return self.unexpected("contains or equals");
            };
            Ok(Statement::Analyze { source, predicate })
        } else if self.matches(TokenKind::Report) {
            let name = self.expect_string()?;
            self.expect_keyword(TokenKind::LBrace, "{")?;
            let mut includes = Vec::new();
            while !self.check(TokenKind::RBrace) {
                self.expect_keyword(TokenKind::Include, "include")?;
                includes.push(self.expect_identifier()?);
            }
            self.expect_keyword(TokenKind::RBrace, "}")?;
            Ok(Statement::Report { name, includes })
        } else {
            self.unexpected("statement")
        }
    }

    fn call(&mut self) -> Result<ForensicCall, ParseError> {
        let namespace = self.expect_identifier()?;
        self.expect_keyword(TokenKind::Dot, ".")?;
        let function = self.expect_identifier()?;
        self.expect_keyword(TokenKind::LParen, "(")?;
        let mut args = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                args.push(self.argument()?);
                if !self.matches(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect_keyword(TokenKind::RParen, ")")?;
        Ok(ForensicCall {
            namespace,
            function,
            args,
        })
    }

    fn argument(&mut self) -> Result<Argument, ParseError> {
        if let Some(SpannedToken { token: Token::Identifier(_), .. }) = self.peek() {
            Ok(Argument::Binding(self.expect_identifier()?))
        } else {
            Ok(Argument::Literal(self.literal()?))
        }
    }

    fn literal(&mut self) -> Result<Literal, ParseError> {
        if let Some(token) = self.advance() {
            match token.token {
                Token::String(s) => Ok(Literal::String(s)),
                Token::Integer(i) => Ok(Literal::Integer(i)),
                Token::True => Ok(Literal::Boolean(true)),
                Token::False => Ok(Literal::Boolean(false)),
                other => Err(ParseError::Unexpected {
                    expected: "literal",
                    found: format!("{other:?}"),
                    offset: token.start,
                }),
            }
        } else {
            Err(ParseError::Eof("literal"))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Some(SpannedToken {
                token: Token::Identifier(value),
                ..
            }) => Ok(value),
            Some(token) => Err(ParseError::Unexpected {
                expected: "identifier",
                found: format!("{:?}", token.token),
                offset: token.start,
            }),
            None => Err(ParseError::Eof("identifier")),
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Some(SpannedToken {
                token: Token::String(value),
                ..
            }) => Ok(value),
            Some(token) => Err(ParseError::Unexpected {
                expected: "string",
                found: format!("{:?}", token.token),
                offset: token.start,
            }),
            None => Err(ParseError::Eof("string")),
        }
    }

    fn expect_keyword(&mut self, kind: TokenKind, expected: &'static str) -> Result<(), ParseError> {
        if self.matches(kind) {
            Ok(())
        } else {
            self.unexpected(expected)
        }
    }

    fn matches(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|token| token_matches(&token.token, kind))
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn unexpected<T>(&self, expected: &'static str) -> Result<T, ParseError> {
        match self.peek() {
            Some(token) => Err(ParseError::Unexpected {
                expected,
                found: format!("{:?}", token.token),
                offset: token.start,
            }),
            None => Err(ParseError::Eof(expected)),
        }
    }
}

#[derive(Copy, Clone)]
enum TokenKind {
    Investigation,
    Target,
    Host,
    Agent,
    Collect,
    Analyze,
    Where,
    Contains,
    Equals,
    As,
    Report,
    Include,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Dot,
    Comma,
}

fn token_matches(token: &Token, kind: TokenKind) -> bool {
    matches!(
        (token, kind),
        (Token::Investigation, TokenKind::Investigation)
            | (Token::Target, TokenKind::Target)
            | (Token::Host, TokenKind::Host)
            | (Token::Agent, TokenKind::Agent)
            | (Token::Collect, TokenKind::Collect)
            | (Token::Analyze, TokenKind::Analyze)
            | (Token::Where, TokenKind::Where)
            | (Token::Contains, TokenKind::Contains)
            | (Token::Equals, TokenKind::Equals)
            | (Token::As, TokenKind::As)
            | (Token::Report, TokenKind::Report)
            | (Token::Include, TokenKind::Include)
            | (Token::LBrace, TokenKind::LBrace)
            | (Token::RBrace, TokenKind::RBrace)
            | (Token::LParen, TokenKind::LParen)
            | (Token::RParen, TokenKind::RParen)
            | (Token::Dot, TokenKind::Dot)
            | (Token::Comma, TokenKind::Comma)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_investigation() {
        let program = parse_program(
            r#"investigation "x" { target host("h") collect system.info() as sys }"#,
        )
        .unwrap();
        assert_eq!(program.investigations.len(), 1);
    }

    #[test]
    fn parses_binding_and_literal_arguments() {
        let program = parse_program(r#"investigation "x" { target host("h") collect hashing.sha256("abc") as digest collect reporting.summary(digest) as summary }"#).unwrap();
        let Statement::Collect { call, .. } = &program.investigations[0].statements[1] else { panic!("expected collect") };
        assert_eq!(call.args, vec![Argument::Binding("digest".into())]);
    }
}
