use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'a> {
    Ident(&'a str),
    Keyword(Keyword),
    Symbol(Symbol),
    Invalid(&'a str),
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
pub enum Keyword {
    Protocol,
    From,
    To,
    Par,
    Fin,
    Inf,
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
