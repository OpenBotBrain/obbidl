use strum::IntoEnumIterator;

use crate::token::{Keyword, Symbol, Token};

pub struct Lexer<'a> {
    source: &'a str,
    pub offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer { source, offset: 0 }
    }
    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }
    fn next_char(&mut self) {
        if let Some(char) = self.peek_char() {
            self.offset += char.len_utf8();
        }
    }
    fn consume_str(&mut self, s: &str) -> bool {
        let starts_with = self.source[self.offset..].starts_with(s);
        if starts_with {
            self.offset += s.len();
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
    fn lex_token(&mut self, ch: char) -> Token {
        if ch.is_alphabetic() || ch == '_' {
            let start = self.offset;
            self.next_char();
            while self
                .peek_char()
                .map_or(false, |ch| ch.is_alphanumeric() || ch == '_')
            {
                self.next_char();
            }
            let end = self.offset;
            let ident = &self.source[start..end];

            for keyword in Keyword::iter() {
                if keyword.as_str() == ident {
                    return Token::Keyword(keyword);
                }
            }

            return Token::Ident;
        }

        for symbol in Symbol::iter() {
            if symbol.as_char() == ch {
                self.next_char();
                return Token::Symbol(symbol);
            }
        }

        self.next_char();
        return Token::Invalid;
    }
    pub fn next_token(&mut self) -> (Token, &'a str) {
        loop {
            let Some(ch) = self.peek_char() else {
                return (Token::End, "");
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

            let start = self.offset;
            let token = self.lex_token(ch);
            let end = self.offset;
            return (token, &self.source[start..end]);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::{Keyword, Symbol, Token};

    use super::Lexer;

    #[test]
    fn test_lex_ident() {
        let mut lexer = Lexer::new("something");
        assert_eq!(lexer.next_token(), (Token::Ident, "something"));
        assert_eq!(lexer.next_token().0, Token::End);
    }

    #[test]
    fn test_lex_keyword() {
        let mut lexer = Lexer::new("protocol");
        assert_eq!(lexer.next_token().0, Token::Keyword(Keyword::Protocol));
        assert_eq!(lexer.next_token().0, Token::End);
    }

    #[test]
    fn test_lex_whitespace() {
        let mut lexer = Lexer::new("         ");
        assert_eq!(lexer.next_token().0, Token::End);
    }

    #[test]
    fn test_lex_symbol() {
        let mut lexer = Lexer::new("{ }");
        assert_eq!(lexer.next_token().0, Token::Symbol(Symbol::OpenCurlyBrace));
        assert_eq!(lexer.next_token().0, Token::Symbol(Symbol::CloseCurlyBrace));
        assert_eq!(lexer.next_token().0, Token::End);
    }

    #[test]
    fn text_lex_line_comment() {
        let mut lexer = Lexer::new("(*) line comment");
        assert_eq!(lexer.next_token().0, Token::End);
    }

    #[test]
    fn text_lex_multi_line_comment() {
        let mut lexer = Lexer::new("(* multi \n line \n comment *)");
        assert_eq!(lexer.next_token().0, Token::End);
    }

    #[test]
    fn text_lex_nested_comments() {
        let mut lexer = Lexer::new("(* (* something *) *)");
        assert_eq!(lexer.next_token().0, Token::End);
    }

    #[test]
    fn test_lex_invalid() {
        let mut lexer = Lexer::new(".");
        assert_eq!(lexer.next_token(), (Token::Invalid, "."));
        assert_eq!(lexer.next_token(), (Token::End, ""));
    }
}
