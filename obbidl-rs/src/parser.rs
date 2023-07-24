use std::{fmt, mem::replace};

use colored::Colorize;

use crate::{
    lexer::Lexer,
    token::{Token, TokenType},
};

pub struct Parser<'a> {
    source: &'a str,
    lexer: Lexer<'a>,
    token: Token<'a>,
    expected_tokens: Vec<TokenType>,
}

#[derive(Debug)]
pub struct ParseError<'a> {
    pub token: Token<'a>,
    pub expected_tokens: Vec<TokenType>,
    pub line: u32,
    pub column: u32,
}

pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

struct TokenTypeName(TokenType);

impl fmt::Display for TokenTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TokenType::Ident => write!(f, "an identifier"),
            TokenType::Keyword(keyword) => write!(f, "the keyword '{}'", keyword.as_str()),
            TokenType::Symbol(symbol) => write!(f, "the symbol '{}'", symbol.as_char()),
            TokenType::End => write!(f, "the end of the input"),
            TokenType::Invalid => panic!(),
        }
    }
}

struct TokenName<'a>(Token<'a>);

impl<'a> fmt::Display for TokenName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.ty {
            TokenType::Ident => write!(f, "the identifier '{}'", self.0.contents),
            TokenType::Keyword(keyword) => write!(f, "the keyword '{}'", keyword.as_str()),
            TokenType::Symbol(symbol) => write!(f, "the symbol '{}'", symbol.as_char()),
            TokenType::Invalid => write!(f, "the invalid character '{}'", self.0.contents),
            TokenType::End => write!(f, "the end of the input"),
        }
    }
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: line {}, column {}",
            "SYNTAX ERROR".red(),
            self.line,
            self.column
        )?;
        writeln!(f, "  Expected one of the following:",)?;
        for token in &self.expected_tokens {
            writeln!(f, "    - {}", TokenTypeName(*token))?;
        }
        writeln!(f, "  Instead found {}", TokenName(self.token))?;
        Ok(())
    }
}

pub trait Parse
where
    Self: Sized,
{
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self>;
}

pub trait MaybeParse
where
    Self: Sized,
{
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Option<Self>>;
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token();
        Parser {
            lexer,
            token,
            source,
            expected_tokens: vec![],
        }
    }
    pub fn next_token(&mut self) -> &'a str {
        self.expected_tokens.truncate(0);
        let old_token = self.token;
        self.token = self.lexer.next_token();
        old_token.contents
    }
    pub fn eat_token(&mut self, token: TokenType) -> Option<&'a str> {
        if self.token.ty == token {
            Some(self.next_token())
        } else {
            self.expected_tokens.push(token);
            None
        }
    }
    pub fn expect_token(&mut self, token: TokenType) -> ParseResult<'a, &'a str> {
        if let Some(source) = self.eat_token(token) {
            return Ok(source);
        }
        Err(self.invalid_token())
    }
    pub fn invalid_token(&mut self) -> ParseError<'a> {
        let mut line = 1;
        let mut column = 1;
        for (offset, ch) in self.source.char_indices() {
            if offset == self.lexer.offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        ParseError {
            expected_tokens: replace(&mut self.expected_tokens, vec![]),
            line,
            column,
            token: self.token,
        }
    }
}

pub fn parse<'a, T: Parse>(source: &'a str) -> ParseResult<'a, T> {
    let mut parser = Parser::new(source);
    let res = T::parse(&mut parser)?;
    parser.expect_token(TokenType::End)?;
    Ok(res)
}

pub fn parse_maybe<'a, T: MaybeParse>(source: &'a str) -> ParseResult<'a, Option<T>> {
    let mut parser = Parser::new(source);
    let res = T::parse(&mut parser)?;
    parser.expect_token(TokenType::End)?;
    Ok(res)
}
