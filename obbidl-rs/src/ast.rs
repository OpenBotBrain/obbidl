use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    fmt,
    hash::{Hash, Hasher},
};

use crate::{
    parser::{Parse, ParseResult, Parser, Span},
    token::{Keyword, Symbol, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Protocol {
    pub name: String,
    pub roles: Option<Vec<Role>>,
    pub seq: Sequence,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sequence(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Message(Span<Message>),
    Par(Sequences),
    Choice(Sequences),
    Fin(Sequence),
    Inf(Sequence),
}

#[derive(Debug, Clone)]
pub struct Sequences(pub Vec<Sequence>);

impl PartialEq for Sequences {
    fn eq(&self, other: &Self) -> bool {
        HashSet::<_>::from_iter(self.0.iter()) == HashSet::from_iter(other.0.iter())
    }
}

impl Hash for Sequences {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut total = 0;
        for item in &self.0 {
            let mut hasher = DefaultHasher::new();
            item.hash(&mut hasher);
            total ^= hasher.finish();
        }
        total.hash(state);
    }
}

impl Eq for Sequences {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message {
    pub label: String,
    pub payload: Payload,
    pub from: Role,
    pub to: Role,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Role(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Bool,
    Int(IntType),
    Array(Box<Type>, Option<u64>),
    Struct(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntType {
    pub signed: bool,
    pub size: IntSize,
}

impl IntType {
    const I64: IntType = IntType {
        signed: true,
        size: IntSize::B64,
    };
    const I32: IntType = IntType {
        signed: true,
        size: IntSize::B32,
    };
    const I16: IntType = IntType {
        signed: true,
        size: IntSize::B16,
    };
    const I8: IntType = IntType {
        signed: true,
        size: IntSize::B8,
    };
    const U64: IntType = IntType {
        signed: false,
        size: IntSize::B64,
    };
    const U32: IntType = IntType {
        signed: false,
        size: IntSize::B32,
    };
    const U16: IntType = IntType {
        signed: false,
        size: IntSize::B16,
    };
    const U8: IntType = IntType {
        signed: false,
        size: IntSize::B8,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntSize {
    B64,
    B32,
    B16,
    B8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub protocols: Vec<Span<Protocol>>,
    pub structs: Vec<Span<Struct>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Payload {
    pub items: Vec<PayloadItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PayloadItem {
    pub name: Option<String>,
    pub ty: Type,
}

impl Payload {
    pub fn empty() -> Payload {
        Payload { items: vec![] }
    }
}

impl Parse for Role {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        Ok(Role(parser.expect_token(TokenType::Ident)?.to_string()))
    }
}

impl Parse for Payload {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        let mut items = vec![];
        while parser
            .eat_token(TokenType::Symbol(Symbol::CloseBrace))
            .is_none()
        {
            let name = if let Some(token) = parser.eat_token(TokenType::Ident) {
                parser.expect_token(TokenType::Symbol(Symbol::Colon))?;
                Some(token.to_string())
            } else {
                None
            };
            let ty = parser.parse()?;
            items.push(PayloadItem { name, ty });
            if !parser.eat_token(TokenType::Symbol(Symbol::Comma)).is_some() {
                parser.expect_token(TokenType::Symbol(Symbol::CloseBrace))?;
                break;
            }
        }
        Ok(Payload { items })
    }
}

impl Parse for Message {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        if let Some(label) = parser.eat_token(TokenType::Ident) {
            let payload = if parser
                .eat_token(TokenType::Symbol(Symbol::OpenBrace))
                .is_some()
            {
                parser.parse()?
            } else {
                Payload::empty()
            };
            parser.expect_token(TokenType::Keyword(Keyword::From))?;
            let from = parser.parse()?;
            parser.expect_token(TokenType::Keyword(Keyword::To))?;
            let to = parser.parse()?;
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

impl Parse for Struct {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        parser.expect_token(TokenType::Keyword(Keyword::Struct))?;
        let name = parser.expect_token(TokenType::Ident)?.to_string();
        parser.expect_token(TokenType::Symbol(Symbol::OpenCurlyBrace))?;
        let mut items = vec![];
        while parser
            .eat_token(TokenType::Symbol(Symbol::CloseCurlyBrace))
            .is_none()
        {
            let field_name = parser.expect_token(TokenType::Ident)?.to_string();
            parser.expect_token(TokenType::Symbol(Symbol::Colon))?;
            let ty = parser.parse::<Type>()?;
            items.push((field_name, ty));
            if parser.eat_token(TokenType::Symbol(Symbol::Comma)).is_none() {
                parser.expect_token(TokenType::Symbol(Symbol::CloseCurlyBrace))?;
                break;
            }
        }
        Ok(Struct {
            name,
            fields: items,
        })
    }
}

impl Parse for Type {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        let mut ty = if parser
            .eat_token(TokenType::Keyword(Keyword::Bool))
            .is_some()
        {
            Type::Bool
        } else if parser.eat_token(TokenType::Keyword(Keyword::U64)).is_some() {
            Type::Int(IntType::U64)
        } else if parser.eat_token(TokenType::Keyword(Keyword::U32)).is_some() {
            Type::Int(IntType::U32)
        } else if parser.eat_token(TokenType::Keyword(Keyword::U16)).is_some() {
            Type::Int(IntType::U16)
        } else if parser.eat_token(TokenType::Keyword(Keyword::U8)).is_some() {
            Type::Int(IntType::U8)
        } else if parser.eat_token(TokenType::Keyword(Keyword::I64)).is_some() {
            Type::Int(IntType::I64)
        } else if parser.eat_token(TokenType::Keyword(Keyword::I32)).is_some() {
            Type::Int(IntType::I32)
        } else if parser.eat_token(TokenType::Keyword(Keyword::I16)).is_some() {
            Type::Int(IntType::I16)
        } else if parser.eat_token(TokenType::Keyword(Keyword::I8)).is_some() {
            Type::Int(IntType::I8)
        } else if parser
            .eat_token(TokenType::Keyword(Keyword::Struct))
            .is_some()
        {
            Type::Struct(parser.expect_token(TokenType::Ident)?.to_string())
        } else {
            return Err(parser.invalid_token());
        };
        while parser
            .eat_token(TokenType::Symbol(Symbol::OpenSquareBrace))
            .is_some()
        {
            let size = if let Some(value) = parser.eat_token(TokenType::Integer) {
                Some(value.parse().unwrap())
            } else {
                None
            };
            parser.expect_token(TokenType::Symbol(Symbol::CloseSquareBrace))?;
            ty = Type::Array(Box::new(ty), size)
        }
        Ok(ty)
    }
}

impl Parse for Stmt {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
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
            Ok(Stmt::Choice(Sequences(blocks)))
        } else if parser.eat_token(TokenType::Keyword(Keyword::Par)).is_some() {
            let mut blocks = vec![];
            blocks.push(parser.parse()?);
            while parser.eat_token(TokenType::Keyword(Keyword::And)).is_some() {
                blocks.push(parser.parse()?);
            }
            Ok(Stmt::Par(Sequences(blocks)))
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
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
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

impl Parse for Protocol {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
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
                roles.push(parser.parse()?);
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
        Ok(Protocol { name, roles, seq })
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Parse for File {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        let mut protocols = vec![];
        let mut structs = vec![];
        while parser.eat_token(TokenType::End).is_none() {
            if let Some(protocol) = parser.parse_maybe::<Span<Protocol>>()? {
                protocols.push(protocol);
            } else if let Some(struct_) = parser.parse_maybe::<Span<Struct>>()? {
                structs.push(struct_);
            } else {
                return Err(parser.invalid_token());
            }
        }
        Ok(File { protocols, structs })
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool")?,
            Type::Int(ty) => write!(f, "{}", ty)?,
            Type::Array(ty, size) => match size {
                Some(size) => write!(f, "{}[{}]", ty, size)?,
                None => write!(f, "{}[]", ty)?,
            },
            Type::Struct(name) => write!(f, "struct {}", name)?,
        }
        Ok(())
    }
}

impl fmt::Display for PayloadItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}: ", name)?;
        }
        write!(f, "{}", self.ty)?;
        Ok(())
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)?;
        if self.payload.items.len() > 0 {
            write!(f, "({})", display_utils::join(&self.payload.items, ", "))?;
        }
        write!(f, " from {} to {};", self.from, self.to)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{IntSize, IntType, Message, Payload, PayloadItem, Struct, Type},
        parser::parse,
        report::Report,
    };

    use super::Role;

    fn role(name: impl Into<String>) -> Role {
        Role(name.into())
    }

    #[test]
    fn test_parse_msg() {
        let msg = parse::<Message>("X from Y to Z;").report();
        assert_eq!(
            msg,
            Message {
                label: "X".to_string(),
                payload: Payload::empty(),
                from: role("Y"),
                to: role("Z"),
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
                payload: Payload {
                    items: vec![PayloadItem {
                        name: Some("x".to_string()),
                        ty: Type::Int(IntType {
                            signed: false,
                            size: IntSize::B32
                        })
                    }]
                },
                from: role("Y"),
                to: role("Z"),
            }
        )
    }

    #[test]
    fn test_parse_struct() {
        let struct_ = parse::<Struct>("struct Point { x: u32, y: u32 }").report();
        assert_eq!(
            struct_,
            Struct {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Type::Int(IntType::U32)),
                    ("y".to_string(), Type::Int(IntType::U32))
                ]
            }
        )
    }
}
