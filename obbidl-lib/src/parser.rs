use std::{fmt, hash, mem::replace};

use colored::Colorize;

use crate::{
    lexer::{Lexer, Position},
    token::{Token, TokenType},
};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token<'a>,
    pos: Position,
    expected_tokens: Vec<TokenType>,
}

#[derive(Debug, Clone)]
pub struct Span<T> {
    pub span: RawSpan,
    pub inner: T,
}

#[derive(Debug, Clone, Copy)]
pub struct RawSpan {
    pub start: Position,
    pub end: Position,
}

pub struct PrettyPrintSpan<'a> {
    span: RawSpan,
    source: &'a str,
}

impl<T> Span<T> {
    pub fn pretty_print<'a>(&self, source: &'a str) -> PrettyPrintSpan<'a> {
        PrettyPrintSpan {
            span: self.span,
            source,
        }
    }
    pub fn map<T1>(self, f: impl FnOnce(T) -> T1) -> Span<T1> {
        Span {
            span: self.span,
            inner: f(self.inner),
        }
    }
    pub fn as_ref(&self) -> Span<&T> {
        Span {
            span: self.span,
            inner: &self.inner,
        }
    }
}

impl<'a> fmt::Display for PrettyPrintSpan<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source = &self.source[self.span.start.offset..self.span.end.offset];
        let mut lines = source.lines();
        let Some(first) = lines.next() else {
            return Ok(());
        };
        write!(f, "{:>3} | ", self.span.start.line)?;
        for _ in 1..self.span.start.column {
            write!(f, " ")?;
        }
        writeln!(f, "{}", first)?;
        for (i, line) in lines.enumerate() {
            writeln!(f, "{:>3} | {}", self.span.start.line + i as u32 + 1, line)?;
        }
        Ok(())
    }
}

impl<T: PartialEq> PartialEq for Span<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Eq> Eq for Span<T> {}

impl<T: hash::Hash> hash::Hash for Span<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T: Parse> Parse for Span<T> {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        let start = parser.token.start;
        let inner = parser.parse::<T>()?;
        let end = parser.pos;
        Ok(Span {
            span: RawSpan { start, end },
            inner,
        })
    }
}

#[derive(Debug)]
pub struct ParseError<'a> {
    pub token: Token<'a>,
    pub expected_tokens: Vec<TokenType>,
    pub pos: Position,
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
            TokenType::Integer => write!(f, "an integer"),
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
            TokenType::Integer => write!(f, "the integer '{}'", self.0.contents),
        }
    }
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: line {}, column {}",
            "SYNTAX ERROR".red(),
            self.pos.line,
            self.pos.column
        )?;
        writeln!(f, "  Found {}", TokenName(self.token))?;
        write!(f, "  Expected one of the following:",)?;
        for token in &self.expected_tokens {
            writeln!(f)?;
            write!(f, "    - {}", TokenTypeName(*token))?;
        }
        Ok(())
    }
}

pub trait Parse
where
    Self: Sized,
{
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self>;
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token();
        Parser {
            lexer,
            token,
            expected_tokens: vec![],
            pos: Position::START,
        }
    }
    pub fn next_token(&mut self) -> &'a str {
        self.expected_tokens.truncate(0);
        let old_token = self.token;
        self.pos = old_token.end;
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
        ParseError {
            expected_tokens: replace(&mut self.expected_tokens, vec![]),
            pos: self.token.start,
            token: self.token,
        }
    }
    pub fn parse<T: Parse>(&mut self) -> ParseResult<'a, T> {
        T::parse(self)
    }
    pub fn parse_maybe<T: Parse>(&mut self) -> ParseResult<'a, Option<T>> {
        let start = self.token.start;
        let res = T::parse(self);
        match res {
            Ok(res) => Ok(Some(res)),
            Err(_) if self.token.start == start => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub fn parse<'a, T: Parse>(source: &'a str) -> ParseResult<'a, T> {
    let mut parser = Parser::new(source);
    let res = parser.parse()?;
    parser.expect_token(TokenType::End)?;
    Ok(res)
}
