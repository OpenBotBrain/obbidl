use strum::IntoEnumIterator;

use crate::token::{Keyword, Symbol, Token};

pub struct Lexer<'a> {
    source: &'a str,
    offset: usize,
}

fn is_valid_ch(ch: char) -> bool {
    if ch.is_whitespace() {
        return true;
    }
    if ch.is_alphanumeric() {
        return true;
    }
    for symbol in Symbol::iter() {
        if ch == symbol.as_char() {
            return true;
        }
    }
    false
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
    pub fn next_token(&mut self) -> Option<Token> {
        loop {
            let Some(ch) = self.peek_char() else {
                return None;
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
                        return Some(Token::Keyword(keyword));
                    }
                }

                return Some(Token::Ident(ident));
            }

            for symbol in Symbol::iter() {
                if symbol.as_char() == ch {
                    self.next_char();
                    return Some(Token::Symbol(symbol));
                }
            }

            let start = self.offset;
            while self.peek_char().map_or(false, |ch| !is_valid_ch(ch)) {
                self.next_char();
            }
            let end = self.offset;
            return Some(Token::Invalid(&self.source[start..end]));
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
        assert_eq!(lexer.next_token(), Some(Token::Ident("something")));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_keyword() {
        let mut lexer = Lexer::new("protocol");
        assert_eq!(lexer.next_token(), Some(Token::Keyword(Keyword::Protocol)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_whitespace() {
        let mut lexer = Lexer::new("         ");
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_lex_symbol() {
        let mut lexer = Lexer::new("{ }");
        assert_eq!(
            lexer.next_token(),
            Some(Token::Symbol(Symbol::OpenCurlyBrace))
        );
        assert_eq!(
            lexer.next_token(),
            Some(Token::Symbol(Symbol::CloseCurlyBrace))
        );
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn text_lex_line_comment() {
        let mut lexer = Lexer::new("(*) line comment");
        assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn text_lex_multi_line_comment() {
        let mut lexer = Lexer::new("(* multi \n line \n comment *)");
        assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn text_lex_nested_comments() {
        let mut lexer = Lexer::new("(* (* something *) *)");
        assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn test_lex_invalid() {
        let mut lexer = Lexer::new("...");
        assert_eq!(lexer.next_token(), Some(Token::Invalid("...")));
        assert!(lexer.next_token().is_none());
    }
}
