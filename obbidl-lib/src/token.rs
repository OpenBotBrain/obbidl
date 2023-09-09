use strum::EnumIter;

use crate::lexer::Position;

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub ty: TokenType,
    pub contents: &'a str,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Ident,
    Integer,
    Keyword(Keyword),
    Symbol(Symbol),
    Invalid,
    End,
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
pub enum Keyword {
    Protocol,
    From,
    To,
    Par,
    Fin,
    Inf,
    Choice,
    And,
    Or,
    String,
    Bool,
    Role,
    U64,
    U32,
    U16,
    U8,
    I64,
    I32,
    I16,
    I8,
    Struct,
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
pub enum Symbol {
    OpenBrace,
    CloseBrace,
    OpenCurlyBrace,
    CloseCurlyBrace,
    Semicolon,
    Colon,
    Comma,
    OpenSquareBrace,
    CloseSquareBrace,
}

impl Keyword {
    pub fn as_str(&self) -> &str {
        match self {
            Keyword::Protocol => "protocol",
            Keyword::From => "from",
            Keyword::To => "to",
            Keyword::Par => "par",
            Keyword::Fin => "fin",
            Keyword::Inf => "inf",
            Keyword::Choice => "choice",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::String => "string",
            Keyword::U32 => "u32",
            Keyword::I32 => "i32",
            Keyword::Bool => "bool",
            Keyword::Role => "role",
            Keyword::U64 => "u64",
            Keyword::U16 => "u16",
            Keyword::U8 => "u8",
            Keyword::I64 => "i64",
            Keyword::I16 => "i16",
            Keyword::I8 => "i8",
            Keyword::Struct => "struct",
        }
    }
}

impl Symbol {
    pub fn as_char(&self) -> char {
        match self {
            Symbol::OpenBrace => '(',
            Symbol::CloseBrace => ')',
            Symbol::OpenCurlyBrace => '{',
            Symbol::CloseCurlyBrace => '}',
            Symbol::Semicolon => ';',
            Symbol::Colon => ':',
            Symbol::Comma => ',',
            Symbol::OpenSquareBrace => '[',
            Symbol::CloseSquareBrace => ']',
        }
    }
}
