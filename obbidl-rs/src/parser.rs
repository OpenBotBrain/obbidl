use core::fmt;
use std::mem::replace;

use colored::Colorize;

use crate::{
    ast::{Block, IntSize, Message, Program, Protocol, Stmt, Type},
    lexer::Lexer,
    token::{Keyword, Symbol, Token, TokenType},
};

struct Parser<'a> {
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

type ParseResult<'a, T> = Result<T, ParseError<'a>>;

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

pub fn parse<'a>(source: &'a str) -> ParseResult<'a, Program> {
    let mut parser = Parser::new(source);
    let mut protocols = vec![];
    while !parser.eat_token(TokenType::End).is_some() {
        protocols.push(parser.parse_protocol()?);
    }
    Ok(Program { protocols })
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
    fn next_token(&mut self) -> &'a str {
        self.expected_tokens.truncate(0);
        let old_token = self.token;
        self.token = self.lexer.next_token();
        old_token.contents
    }
    fn eat_token(&mut self, token: TokenType) -> Option<&'a str> {
        if self.token.ty == token {
            Some(self.next_token())
        } else {
            self.expected_tokens.push(token);
            None
        }
    }
    fn expect_token(&mut self, token: TokenType) -> ParseResult<'a, &'a str> {
        if let Some(source) = self.eat_token(token) {
            return Ok(source);
        }
        Err(self.invalid_token())
    }
    fn invalid_token(&mut self) -> ParseError<'a> {
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
    fn parse_type(&mut self) -> ParseResult<'a, Type> {
        if self.eat_token(TokenType::Keyword(Keyword::Bool)).is_some() {
            Ok(Type::Bool)
        } else if self
            .eat_token(TokenType::Keyword(Keyword::String))
            .is_some()
        {
            Ok(Type::String)
        } else if self.eat_token(TokenType::Keyword(Keyword::U32)).is_some() {
            Ok(Type::Int {
                signed: false,
                size: IntSize::B32,
            })
        } else if self.eat_token(TokenType::Keyword(Keyword::I32)).is_some() {
            Ok(Type::Int {
                signed: true,
                size: IntSize::B32,
            })
        } else {
            Err(self.invalid_token())
        }
    }
    fn parse_stmt(&mut self) -> ParseResult<'a, Stmt<'a>> {
        if let Some(label) = self.eat_token(TokenType::Ident) {
            let payload = if self
                .eat_token(TokenType::Symbol(Symbol::OpenBrace))
                .is_some()
            {
                let mut payload = vec![];
                while !self
                    .eat_token(TokenType::Symbol(Symbol::CloseBrace))
                    .is_some()
                {
                    let name = self.expect_token(TokenType::Ident)?;
                    self.expect_token(TokenType::Symbol(Symbol::Colon))?;
                    let ty = self.parse_type()?;
                    payload.push((name, ty));
                    if !self.eat_token(TokenType::Symbol(Symbol::Comma)).is_some() {
                        self.expect_token(TokenType::Symbol(Symbol::CloseBrace))?;
                        break;
                    }
                }
                Some(payload)
            } else {
                None
            };
            self.expect_token(TokenType::Keyword(Keyword::From))?;
            let from = self.expect_token(TokenType::Ident)?;
            self.expect_token(TokenType::Keyword(Keyword::To))?;
            let to = self.expect_token(TokenType::Ident)?;
            self.expect_token(TokenType::Symbol(Symbol::Semicolon))?;
            Ok(Stmt::Message(Message {
                label,
                payload,
                from,
                to,
            }))
        } else if self
            .eat_token(TokenType::Keyword(Keyword::Choice))
            .is_some()
        {
            let mut blocks = vec![];
            blocks.push(self.parse_block()?);
            while self.eat_token(TokenType::Keyword(Keyword::Or)).is_some() {
                blocks.push(self.parse_block()?);
            }
            Ok(Stmt::Choice(blocks))
        } else if self.eat_token(TokenType::Keyword(Keyword::Par)).is_some() {
            let mut blocks = vec![];
            blocks.push(self.parse_block()?);
            while self.eat_token(TokenType::Keyword(Keyword::And)).is_some() {
                blocks.push(self.parse_block()?);
            }
            Ok(Stmt::Par(blocks))
        } else if self.eat_token(TokenType::Keyword(Keyword::Fin)).is_some() {
            Ok(Stmt::Fin(self.parse_block()?))
        } else if self.eat_token(TokenType::Keyword(Keyword::Inf)).is_some() {
            Ok(Stmt::Inf(self.parse_block()?))
        } else {
            Err(self.invalid_token())
        }
    }
    fn parse_block(&mut self) -> ParseResult<'a, Block<'a>> {
        self.expect_token(TokenType::Symbol(Symbol::OpenCurlyBrace))?;
        let mut stmts = vec![];
        while !self
            .eat_token(TokenType::Symbol(Symbol::CloseCurlyBrace))
            .is_some()
        {
            stmts.push(self.parse_stmt()?)
        }
        Ok(Block { stmts })
    }
    fn parse_protocol(&mut self) -> ParseResult<'a, Protocol<'a>> {
        self.expect_token(TokenType::Keyword(Keyword::Protocol))?;
        let name = self.expect_token(TokenType::Ident)?;
        let roles = if self
            .eat_token(TokenType::Symbol(Symbol::OpenBrace))
            .is_some()
        {
            let mut roles = vec![];
            while !self
                .eat_token(TokenType::Symbol(Symbol::CloseBrace))
                .is_some()
            {
                self.expect_token(TokenType::Keyword(Keyword::Role))?;
                roles.push(self.expect_token(TokenType::Ident)?);
                if !self.eat_token(TokenType::Symbol(Symbol::Comma)).is_some() {
                    self.expect_token(TokenType::Symbol(Symbol::CloseBrace))?;
                    break;
                }
            }
            Some(roles)
        } else {
            None
        };

        let block = self.parse_block()?;
        Ok(Protocol { name, roles, block })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Block, Message, Program, Protocol, Stmt},
        token::TokenType,
    };

    use super::{parse, Parser};

    #[test]
    fn test_parse_message() {
        let mut parser = Parser::new("init from A to B;");
        let stmt = parser.parse_stmt().unwrap();
        assert_eq!(
            stmt,
            Stmt::Message(Message {
                label: "init",
                payload: None,
                from: "A",
                to: "B"
            })
        );
        assert_eq!(parser.token.ty, TokenType::End);
    }

    #[test]
    fn test_parse_protocol() {
        let ast = parse("protocol Test(role A, role B) {}").unwrap();
        assert_eq!(
            ast,
            Program {
                protocols: vec![Protocol {
                    name: "Test",
                    roles: Some(vec!["A", "B"]),
                    block: Block { stmts: vec![] }
                }]
            }
        )
    }
}
