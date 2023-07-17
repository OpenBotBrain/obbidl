use crate::{
    ast::{Block, IntSize, Message, Program, Protocol, Stmt, Type},
    lexer::Lexer,
    token::{Keyword, Symbol, Token},
};

struct Parser<'a> {
    source: &'a str,
    lexer: Lexer<'a>,
    token: Token,
    contents: &'a str,
    expected_tokens: Vec<Token>,
}

#[derive(Debug)]
pub struct ParseError {
    pub expected_tokens: Vec<Token>,
    pub line: u32,
    pub column: u32,
}

pub fn parse<'a>(source: &'a str) -> Result<Program, ParseError> {
    let mut parser = Parser::new(source);
    let mut protocols = vec![];
    while parser.token != Token::End {
        protocols.push(parser.parse_protocol()?);
    }
    Ok(Program { protocols })
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        let mut lexer = Lexer::new(source);
        let (token, contents) = lexer.next_token();
        Parser {
            lexer,
            token,
            contents,
            source,
            expected_tokens: vec![],
        }
    }
    fn next_token(&mut self) -> &'a str {
        self.expected_tokens.truncate(0);
        let (token, contents) = self.lexer.next_token();
        let old_contents = self.contents;
        self.token = token;
        self.contents = contents;
        old_contents
    }
    fn eat_token(&mut self, token: Token) -> Option<&'a str> {
        if self.token == token {
            Some(self.next_token())
        } else {
            self.expected_tokens.push(token);
            None
        }
    }
    fn expect_token(&mut self, token: Token) -> Result<&'a str, ParseError> {
        self.eat_token(token).ok_or_else(|| self.error())
    }
    fn error(&self) -> ParseError {
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
            expected_tokens: self.expected_tokens.clone(),
            line,
            column,
        }
    }
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if self.eat_token(Token::Keyword(Keyword::Bool)).is_some() {
            Ok(Type::Bool)
        } else if self.eat_token(Token::Keyword(Keyword::String)).is_some() {
            Ok(Type::String)
        } else if self.eat_token(Token::Keyword(Keyword::U32)).is_some() {
            Ok(Type::Int {
                signed: false,
                size: IntSize::B32,
            })
        } else if self.eat_token(Token::Keyword(Keyword::I32)).is_some() {
            Ok(Type::Int {
                signed: true,
                size: IntSize::B32,
            })
        } else {
            Err(self.error())
        }
    }
    fn parse_stmt(&mut self) -> Result<Stmt<'a>, ParseError> {
        if let Some(label) = self.eat_token(Token::Ident) {
            let payload = if self.eat_token(Token::Symbol(Symbol::OpenBrace)).is_some() {
                let mut payload = vec![];
                while !self.eat_token(Token::Symbol(Symbol::CloseBrace)).is_some() {
                    let name = self.expect_token(Token::Ident)?;
                    self.expect_token(Token::Symbol(Symbol::Colon))?;
                    let ty = self.parse_type()?;
                    payload.push((name, ty));
                    if !self.eat_token(Token::Symbol(Symbol::Comma)).is_some() {
                        self.expect_token(Token::Symbol(Symbol::CloseBrace))?;
                        break;
                    }
                }
                Some(payload)
            } else {
                None
            };
            self.expect_token(Token::Keyword(Keyword::From))?;
            let from = self.expect_token(Token::Ident)?;
            self.expect_token(Token::Keyword(Keyword::To))?;
            let to = self.expect_token(Token::Ident)?;
            self.expect_token(Token::Symbol(Symbol::Semicolon))?;
            Ok(Stmt::Message(Message {
                label,
                payload,
                from,
                to,
            }))
        } else if self.eat_token(Token::Keyword(Keyword::Choice)).is_some() {
            let mut blocks = vec![];
            blocks.push(self.parse_block()?);
            while self.eat_token(Token::Keyword(Keyword::Or)).is_some() {
                blocks.push(self.parse_block()?);
            }
            Ok(Stmt::Choice(blocks))
        } else if self.eat_token(Token::Keyword(Keyword::Par)).is_some() {
            let mut blocks = vec![];
            blocks.push(self.parse_block()?);
            while self.eat_token(Token::Keyword(Keyword::And)).is_some() {
                blocks.push(self.parse_block()?);
            }
            Ok(Stmt::Par(blocks))
        } else if self.eat_token(Token::Keyword(Keyword::Fin)).is_some() {
            Ok(Stmt::Fin(self.parse_block()?))
        } else if self.eat_token(Token::Keyword(Keyword::Inf)).is_some() {
            Ok(Stmt::Inf(self.parse_block()?))
        } else {
            Err(self.error())
        }
    }
    fn parse_block(&mut self) -> Result<Block<'a>, ParseError> {
        self.expect_token(Token::Symbol(Symbol::OpenCurlyBrace))?;
        let mut stmts = vec![];
        while !self
            .eat_token(Token::Symbol(Symbol::CloseCurlyBrace))
            .is_some()
        {
            stmts.push(self.parse_stmt()?)
        }
        Ok(Block { stmts })
    }
    fn parse_protocol(&mut self) -> Result<Protocol<'a>, ParseError> {
        self.expect_token(Token::Keyword(Keyword::Protocol))?;
        let name = self.expect_token(Token::Ident)?;
        let roles = if self.eat_token(Token::Symbol(Symbol::OpenBrace)).is_some() {
            let mut roles = vec![];
            while !self.eat_token(Token::Symbol(Symbol::CloseBrace)).is_some() {
                self.expect_token(Token::Keyword(Keyword::Role))?;
                roles.push(self.expect_token(Token::Ident)?);
                if !self.eat_token(Token::Symbol(Symbol::Comma)).is_some() {
                    self.expect_token(Token::Symbol(Symbol::CloseBrace))?;
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
        token::Token,
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
        assert_eq!(parser.token, Token::End);
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
