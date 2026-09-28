use logos::Logos;
use thiserror::Error;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
pub enum Token {
    #[token("investigation")]
    Investigation,
    #[token("target")]
    Target,
    #[token("host")]
    Host,
    #[token("agent")]
    Agent,
    #[token("collect")]
    Collect,
    #[token("analyze")]
    Analyze,
    #[token("where")]
    Where,
    #[token("contains")]
    Contains,
    #[token("equals")]
    Equals,
    #[token("as")]
    As,
    #[token("report")]
    Report,
    #[token("include")]
    Include,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[regex(r#""([^"\\]|\\.)*""#, |lex| unquote(lex.slice()))]
    String(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LexError {
    #[error("invalid token at byte {0}")]
    InvalidToken(usize),
}

pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    while let Some(item) = lexer.next() {
        let span = lexer.span();
        match item {
            Ok(token) => tokens.push(SpannedToken {
                token,
                start: span.start,
                end: span.end,
            }),
            Err(()) => return Err(LexError::InvalidToken(span.start)),
        }
    }
    Ok(tokens)
}

fn unquote(raw: &str) -> String {
    raw.trim_matches('"')
        .replace("\\\"", "\"")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_keywords() {
        let tokens = lex("investigation \"x\" { target host(\"h\") }").unwrap();
        assert!(matches!(tokens[0].token, Token::Investigation));
        assert!(tokens.iter().any(|t| matches!(t.token, Token::Host)));
    }
}

