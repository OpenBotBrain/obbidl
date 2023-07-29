use std::hash::Hash;

use crate::{
    parser::{Parse, ParseResult},
    token::{Keyword, Symbol, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolDef {
    pub name: String,
    pub roles: Option<Vec<String>>,
    pub seq: Sequence,
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
    pub defs: Vec<ProtocolDef>,
}

impl Parse for Message {
    fn parse<'a>(parser: &mut crate::parser::Parser<'a>) -> ParseResult<'a, Self> {
        if let Some(label) = parser.eat_token(TokenType::Ident) {
            let payload = if parser
                .eat_token(TokenType::Symbol(Symbol::OpenBrace))
                .is_some()
            {
                let mut payload = vec![];
                while parser
                    .eat_token(TokenType::Symbol(Symbol::CloseBrace))
                    .is_none()
                {
                    let name = parser.expect_token(TokenType::Ident)?;
                    parser.expect_token(TokenType::Symbol(Symbol::Colon))?;
                    let ty = parser.parse()?;
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
            Ok(Message {
                label: label.to_string(),
                payload,
                from,
                to,
            })
        } else {
            Err(parser.invalid_token())
        }
    }
}

impl Parse for Type {
    fn parse<'a>(parser: &mut crate::parser::Parser<'a>) -> ParseResult<'a, Self> {
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
    fn parse<'a>(parser: &mut crate::parser::Parser<'a>) -> ParseResult<'a, Self> {
        if let Some(msg) = parser.parse_maybe()? {
            Ok(Stmt::Message(msg))
        } else if parser
            .eat_token(TokenType::Keyword(Keyword::Choice))
            .is_some()
        {
            let mut blocks = vec![];
            blocks.push(Sequence::parse(parser)?);
            while parser.eat_token(TokenType::Keyword(Keyword::Or)).is_some() {
                blocks.push(parser.parse()?);
            }
            Ok(Stmt::Choice(blocks))
        } else if parser.eat_token(TokenType::Keyword(Keyword::Par)).is_some() {
            let mut blocks = vec![];
            blocks.push(parser.parse()?);
            while parser.eat_token(TokenType::Keyword(Keyword::And)).is_some() {
                blocks.push(parser.parse()?);
            }
            Ok(Stmt::Par(blocks))
        } else if parser.eat_token(TokenType::Keyword(Keyword::Fin)).is_some() {
            Ok(Stmt::Fin(parser.parse()?))
        } else if parser.eat_token(TokenType::Keyword(Keyword::Inf)).is_some() {
            Ok(Stmt::Inf(parser.parse()?))
        } else {
            Err(parser.invalid_token())
        }
    }
}

impl Parse for Sequence {
    fn parse<'a>(parser: &mut crate::parser::Parser<'a>) -> ParseResult<'a, Self> {
        parser.expect_token(TokenType::Symbol(Symbol::OpenCurlyBrace))?;
        let mut stmts = vec![];
        while parser
            .eat_token(TokenType::Symbol(Symbol::CloseCurlyBrace))
            .is_none()
        {
            stmts.push(parser.parse()?)
        }
        Ok(Sequence(stmts))
    }
}

impl Parse for ProtocolDef {
    fn parse<'a>(parser: &mut crate::parser::Parser<'a>) -> ParseResult<'a, Self> {
        parser.expect_token(TokenType::Keyword(Keyword::Protocol))?;
        let name = parser.expect_token(TokenType::Ident)?.to_string();
        let roles = if parser
            .eat_token(TokenType::Symbol(Symbol::OpenBrace))
            .is_some()
        {
            let mut roles = vec![];
            while parser
                .eat_token(TokenType::Symbol(Symbol::CloseBrace))
                .is_none()
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

        let seq = parser.parse()?;
        Ok(ProtocolDef { name, roles, seq })
    }
}

impl Parse for Program {
    fn parse<'a>(parser: &mut crate::parser::Parser<'a>) -> ParseResult<'a, Self> {
        let mut defs = vec![];
        while parser.eat_token(TokenType::End).is_none() {
            defs.push(parser.parse()?)
        }
        Ok(Program { defs })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{IntSize, Message, Type},
        parser::parse,
        report::Report,
    };

    #[test]
    fn test_parse_msg() {
        let msg = parse::<Message>("X from Y to Z;").report();
        assert_eq!(
            msg,
            Message {
                label: "X".to_string(),
                payload: None,
                from: "Y".to_string(),
                to: "Z".to_string(),
            }
        )
    }

    #[test]
    fn test_parse_msg_payload() {
        let msg = parse::<Message>("X(x: u32) from Y to Z;").report();
        assert_eq!(
            msg,
            Message {
                label: "X".to_string(),
                payload: Some(vec![(
                    "x".to_string(),
                    Type::Int {
                        signed: false,
                        size: IntSize::B32
                    }
                )]),
                from: "Y".to_string(),
                to: "Z".to_string(),
            }
        )
    }
}
