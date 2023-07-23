use std::hash::Hash;

use crate::{
    parse::{MaybeParse, Parse, ParseResult},
    token::{Keyword, Symbol, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolDef {
    pub name: String,
    pub roles: Option<Vec<String>>,
    pub block: Sequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sequence(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Message(Message),
    Par(Vec<Sequence>),
    Choice(Vec<Sequence>),
    Fin(Sequence),
    Inf(Sequence),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub label: String,
    pub payload: Option<Vec<(String, Type)>>,
    pub from: String,
    pub to: String,
}

impl Eq for Message {}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.from == other.from && self.to == other.to
    }
}

impl Hash for Message {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
        self.from.hash(state);
        self.to.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Bool,
    Int { signed: bool, size: IntSize },
    String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntSize {
    B32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub protocols: Vec<ProtocolDef>,
}

impl MaybeParse for Message {
    fn parse<'a>(parser: &mut crate::parse::Parser<'a>) -> ParseResult<'a, Option<Self>> {
        if let Some(label) = parser.eat_token(TokenType::Ident) {
            let payload = if parser
                .eat_token(TokenType::Symbol(Symbol::OpenBrace))
                .is_some()
            {
                let mut payload = vec![];
                while !parser
                    .eat_token(TokenType::Symbol(Symbol::CloseBrace))
                    .is_some()
                {
                    let name = parser.expect_token(TokenType::Ident)?;
                    parser.expect_token(TokenType::Symbol(Symbol::Colon))?;
                    let ty = Type::parse(parser)?;
                    payload.push((name.to_string(), ty));
                    if !parser.eat_token(TokenType::Symbol(Symbol::Comma)).is_some() {
                        parser.expect_token(TokenType::Symbol(Symbol::CloseBrace))?;
                        break;
                    }
                }
                Some(payload)
            } else {
                None
            };
            parser.expect_token(TokenType::Keyword(Keyword::From))?;
            let from = parser.expect_token(TokenType::Ident)?.to_string();
            parser.expect_token(TokenType::Keyword(Keyword::To))?;
            let to = parser.expect_token(TokenType::Ident)?.to_string();
            parser.expect_token(TokenType::Symbol(Symbol::Semicolon))?;
            Ok(Some(Message {
                label: label.to_string(),
                payload,
                from,
                to,
            }))
        } else {
            Ok(None)
        }
    }
}

impl Parse for Type {
    fn parse<'a>(parser: &mut crate::parse::Parser<'a>) -> ParseResult<'a, Self> {
        if parser
            .eat_token(TokenType::Keyword(Keyword::Bool))
            .is_some()
        {
            Ok(Type::Bool)
        } else if parser
            .eat_token(TokenType::Keyword(Keyword::String))
            .is_some()
        {
            Ok(Type::String)
        } else if parser.eat_token(TokenType::Keyword(Keyword::U32)).is_some() {
            Ok(Type::Int {
                signed: false,
                size: IntSize::B32,
            })
        } else if parser.eat_token(TokenType::Keyword(Keyword::I32)).is_some() {
            Ok(Type::Int {
                signed: true,
                size: IntSize::B32,
            })
        } else {
            Err(parser.invalid_token())
        }
    }
}

impl Parse for Stmt {
    fn parse<'a>(parser: &mut crate::parse::Parser<'a>) -> ParseResult<'a, Self> {
        if let Some(msg) = Message::parse(parser)? {
            Ok(Stmt::Message(msg))
        } else if parser
            .eat_token(TokenType::Keyword(Keyword::Choice))
            .is_some()
        {
            let mut blocks = vec![];
            blocks.push(Sequence::parse(parser)?);
            while parser.eat_token(TokenType::Keyword(Keyword::Or)).is_some() {
                blocks.push(Sequence::parse(parser)?);
            }
            Ok(Stmt::Choice(blocks))
        } else if parser.eat_token(TokenType::Keyword(Keyword::Par)).is_some() {
            let mut blocks = vec![];
            blocks.push(Sequence::parse(parser)?);
            while parser.eat_token(TokenType::Keyword(Keyword::And)).is_some() {
                blocks.push(Sequence::parse(parser)?);
            }
            Ok(Stmt::Par(blocks))
        } else if parser.eat_token(TokenType::Keyword(Keyword::Fin)).is_some() {
            Ok(Stmt::Fin(Sequence::parse(parser)?))
        } else if parser.eat_token(TokenType::Keyword(Keyword::Inf)).is_some() {
            Ok(Stmt::Inf(Sequence::parse(parser)?))
        } else {
            Err(parser.invalid_token())
        }
    }
}

impl Parse for Sequence {
    fn parse<'a>(parser: &mut crate::parse::Parser<'a>) -> ParseResult<'a, Self> {
        parser.expect_token(TokenType::Symbol(Symbol::OpenCurlyBrace))?;
        let mut stmts = vec![];
        while !parser
            .eat_token(TokenType::Symbol(Symbol::CloseCurlyBrace))
            .is_some()
        {
            stmts.push(Stmt::parse(parser)?)
        }
        Ok(Sequence(stmts))
    }
}

impl Parse for ProtocolDef {
    fn parse<'a>(parser: &mut crate::parse::Parser<'a>) -> ParseResult<'a, Self> {
        parser.expect_token(TokenType::Keyword(Keyword::Protocol))?;
        let name = parser.expect_token(TokenType::Ident)?.to_string();
        let roles = if parser
            .eat_token(TokenType::Symbol(Symbol::OpenBrace))
            .is_some()
        {
            let mut roles = vec![];
            while !parser
                .eat_token(TokenType::Symbol(Symbol::CloseBrace))
                .is_some()
            {
                parser.expect_token(TokenType::Keyword(Keyword::Role))?;
                roles.push(parser.expect_token(TokenType::Ident)?.to_string());
                if !parser.eat_token(TokenType::Symbol(Symbol::Comma)).is_some() {
                    parser.expect_token(TokenType::Symbol(Symbol::CloseBrace))?;
                    break;
                }
            }
            Some(roles)
        } else {
            None
        };

        let block = Sequence::parse(parser)?;
        Ok(ProtocolDef { name, roles, block })
    }
}
