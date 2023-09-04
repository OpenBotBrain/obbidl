use strum::IntoEnumIterator;

use crate::token::{Keyword, Symbol, Token, TokenType};

pub struct Lexer<'a> {
    source: &'a str,
    pos: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

impl Position {
    pub const START: Position = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer {
            source,
            pos: Position::START,
        }
    }
    fn peek_char(&self) -> Option<char> {
        self.source[self.pos.offset..].chars().next()
    }
    fn next_char(&mut self) {
        if let Some(char) = self.peek_char() {
            self.pos.offset += char.len_utf8();
            if char == '\n' {
                self.pos.line += 1;
                self.pos.column = 1;
            } else {
                self.pos.column += 1;
            }
        } else {
            panic!()
        }
    }
    fn consume_str(&mut self, s: &str) -> bool {
        let starts_with = self.source[self.pos.offset..].starts_with(s);
        if starts_with {
            self.pos.offset += s.len();
        }
        starts_with
    }
    fn lex_comment(&mut self) {
        while self.peek_char().is_some() {
            if self.consume_str("(*") {
                self.lex_comment();
                continue;
            }
            if self.consume_str("*)") {
                return;
            }
            self.next_char();
        }
    }
    fn lex_token(&mut self, ch: char) -> TokenType {
        if ch.is_alphabetic() || ch == '_' {
            let start = self.pos.offset;
            self.next_char();
            while self
                .peek_char()
                .map_or(false, |ch| ch.is_alphanumeric() || ch == '_')
            {
                self.next_char();
            }
            let end = self.pos.offset;
            let ident = &self.source[start..end];

            for keyword in Keyword::iter() {
                if keyword.as_str() == ident {
                    return TokenType::Keyword(keyword);
                }
            }

            return TokenType::Ident;
        }

        if ch.is_numeric() {
            self.next_char();
            while self.peek_char().map_or(false, |ch| ch.is_numeric()) {
                self.next_char();
            }
            return TokenType::Integer;
        }

        for symbol in Symbol::iter() {
            if symbol.as_char() == ch {
                self.next_char();
                return TokenType::Symbol(symbol);
            }
        }

        self.next_char();
        return TokenType::Invalid;
    }
    pub fn next_token(&mut self) -> Token<'a> {
        loop {
            let Some(ch) = self.peek_char() else {
                return Token { ty: TokenType::End, contents: "", start: self.pos, end: self.pos };
            };

            if ch.is_whitespace() {
                self.next_char();
                continue;
            }

            if self.consume_str("(*)") {
                while self.peek_char().map_or(false, |ch| ch != '\n') {
                    self.next_char();
                }
                continue;
            }

            if self.consume_str("(*") {
                self.lex_comment();
                continue;
            }

            let start = self.pos;
            let ty = self.lex_token(ch);
            let end = self.pos;
            return Token {
                ty,
                contents: &self.source[start.offset..end.offset],
                start,
                end,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::{Keyword, Symbol, TokenType};

    use super::Lexer;

    #[test]
    fn test_lex_ident() {
        let mut lexer = Lexer::new("something");
        assert_eq!(lexer.next_token().ty, TokenType::Ident);
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }

    #[test]
    fn test_lex_keyword() {
        let mut lexer = Lexer::new("protocol");
        assert_eq!(lexer.next_token().ty, TokenType::Keyword(Keyword::Protocol));
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }

    #[test]
    fn test_lex_whitespace() {
        let mut lexer = Lexer::new("         ");
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }

    #[test]
    fn test_lex_symbol() {
        let mut lexer = Lexer::new("{ }");
        assert_eq!(
            lexer.next_token().ty,
            TokenType::Symbol(Symbol::OpenCurlyBrace)
        );
        assert_eq!(
            lexer.next_token().ty,
            TokenType::Symbol(Symbol::CloseCurlyBrace)
        );
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }

    #[test]
    fn text_lex_line_comment() {
        let mut lexer = Lexer::new("(*) line comment");
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }

    #[test]
    fn text_lex_multi_line_comment() {
        let mut lexer = Lexer::new("(* multi \n line \n comment *)");
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }

    #[test]
    fn text_lex_nested_comments() {
        let mut lexer = Lexer::new("(* (* something *) *)");
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }

    #[test]
    fn test_lex_invalid() {
        let mut lexer = Lexer::new(".");
        assert_eq!(lexer.next_token().ty, TokenType::Invalid);
        assert_eq!(lexer.next_token().ty, TokenType::End);
    }
}
