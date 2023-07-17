use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Ident,
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
    U32,
    I32,
    Bool,
    Role,
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
        }
    }
}
