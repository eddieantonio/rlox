//! Handle Lox's lexical analysis.
//!
//! Contains the [Scanner] which implements an [Iterator] that yields [Lexeme]s, each of which
//! represents a [Token].
//!
//! # Example
//!
//! ```
//! use rlox::scanner::{Scanner, Lexeme, Token};
//! let scanner = Scanner::new("print 1 + 2;");
//! let tokens: Vec<_> = scanner
//!     .map(|lexeme| lexeme.token())
//!     .take_while(|&token| token != Token::Eof) // scanner will yield Eof forever...
//!     .collect();
//!
//! use Token::*;
//! assert_eq!(
//!     vec![Print, Number, Plus, Number, Semicolon],
//!     tokens
//! );
//! ```
//!
//! # Note on terminology
//!
//! I did NOT use the terminology in Crafting Interpreters.  Frankly, the terminology surrounding
//! the nouns in field of lexical analysis confuses me, so I'm just using some terms that make
//! sense and avoid using "type" as an identifier.  Thus, when Crafting Interpreters says:
//!
//! - Token, in this code it's a [Lexeme].
//! - TokenType, in this code it's a [Token].
//! - lexme, in this code it's [Lexeme::text()].

use enum_map::Enum;

/// A lexme from one contiguous string from some Lox source code.
#[derive(Clone, Debug)]
pub struct Lexeme<'a> {
    /// The [Token] of this lexeme.
    token: Token,
    /// The actual text from the source code.
    text: &'a str,
    /// The line where this lexeme came from.
    line: usize,
}

/// What _type_ of [Lexeme] you have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
#[rustfmt::skip]
pub enum Token {
    // Single-character tokens.
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Semicolon, Star, Slash,
    Question, Colon,
    // Or or two characte tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    // Literals
    Identifier, StrLiteral, Number,
    // Keywords
    And, Class, Else, False,
    For, Fun, If, Nil, Or,
    Print, Return, Super, This,
    True, Var, While,

    // Others
    Error, Eof
}

/// Scans Lox source code and iteratively yields [Lexeme]s.
///
/// The scanner is stateful, and therefore, can only be used to do one pass over the source code
/// string. Once the whole source code has been scanned, the scanner will forever yield
/// [Token::Eof].
#[derive(Debug)]
pub struct Scanner<'a> {
    start: &'a str,
    current: &'a str,
    line: usize,
}

impl<'a> Scanner<'a> {
    /// Start scanning the given string of source code.
    pub fn new(source: &'a str) -> Self {
        Scanner {
            start: source,
            current: source,
            line: 1,
        }
    }

    /// Yield the next [Lexeme] from the string. Once the scanner has reached the end-of-file, this
    /// function will always return an end-of-file lexeme.
    pub fn scan_token(&mut self) -> Lexeme<'a> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_lexeme(Token::Eof);
        }

        match self.advance() {
            c if is_id_start(c) => self.identifier(),
            c if c.is_ascii_digit() => self.number(),
            '(' => self.make_lexeme(Token::LeftParen),
            ')' => self.make_lexeme(Token::RightParen),
            '{' => self.make_lexeme(Token::LeftBrace),
            '}' => self.make_lexeme(Token::RightBrace),
            ';' => self.make_lexeme(Token::Semicolon),
            ',' => self.make_lexeme(Token::Comma),
            '.' => self.make_lexeme(Token::Dot),
            '-' => self.make_lexeme(Token::Minus),
            '+' => self.make_lexeme(Token::Plus),
            '/' => self.make_lexeme(Token::Slash),
            '*' => self.make_lexeme(Token::Star),
            '?' => self.make_lexeme(Token::Question),
            ':' => self.make_lexeme(Token::Colon),
            '!' => {
                let followed_by_equal = self.match_and_advance('=');
                self.make_lexeme(if followed_by_equal {
                    Token::BangEqual
                } else {
                    Token::Bang
                })
            }
            '=' => {
                let followed_by_equal = self.match_and_advance('=');
                self.make_lexeme(if followed_by_equal {
                    Token::EqualEqual
                } else {
                    Token::Equal
                })
            }
            '<' => {
                let followed_by_equal = self.match_and_advance('=');
                self.make_lexeme(if followed_by_equal {
                    Token::LessEqual
                } else {
                    Token::Less
                })
            }
            '>' => {
                let followed_by_equal = self.match_and_advance('=');
                self.make_lexeme(if followed_by_equal {
                    Token::GreaterEqual
                } else {
                    Token::Greater
                })
            }
            '"' => self.string(),
            _ => self.error_token("Unexpected character"),
        }
    }

    /// Returns `true` if we've reached the end of the source code.
    pub fn is_at_end(&self) -> bool {
        self.current.is_empty()
    }

    pub fn make_sentinel(&self, message: &'static str) -> Lexeme<'a> {
        Lexeme {
            token: Token::Error,
            text: message,
            line: 0,
        }
    }

    /// Advances self.current, s.t., self.start < self.current are a reference to the same str.
    /// Returns the next valid char.
    ///
    /// # Panics
    ///
    /// If this is called at the end of string.
    fn advance(&mut self) -> char {
        let c = match self.current.chars().next() {
            Some(c) => c,
            None => panic!("called advance() at end of file"),
        };

        let len = c.len_utf8();
        self.current = &self.current[len..];
        assert!(self.current.len() < self.start.len());

        c
    }

    /// Peek at the first char in self.current.
    fn peek(&self) -> char {
        self.current.chars().next().unwrap_or('\0')
    }

    /// Peek at the second char in self.current.
    fn peek_next(&self) -> char {
        let mut chars = self.current.chars();
        chars.next();
        chars.next().unwrap_or('\0')
    }

    /// Matches the expected character. If the next character matches, returns true and advances
    /// self.current. Otherwise, return false and does not update anything.
    fn match_and_advance(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let next_char = self.peek();
        if next_char != expected {
            return false;
        }

        self.current = &self.current[next_char.len_utf8()..];
        true
    }

    /// Skips whitespace and comments.
    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    // Count the newline
                    self.line += 1;
                    self.advance();
                }
                // Comments are "whitespace"
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            };
        }
    }

    /// Scan an identifier or keyword.
    fn identifier(&mut self) -> Lexeme<'a> {
        while is_id_continue(self.peek()) {
            self.advance();
        }

        self.make_lexeme(self.identifier_type())
    }

    /// Scan a string literal. Expects the starting quote to have been consumed.
    fn string(&mut self) -> Lexeme<'a> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string");
        }

        assert_eq!('"', self.advance());
        self.make_lexeme(Token::StrLiteral)
    }

    /// Scan a number literal. Expects the first digit to have already been consumed.
    fn number(&mut self) -> Lexeme<'a> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the decimal point
            self.advance();

            // Consume the digts after the decimal point
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_lexeme(Token::Number)
    }

    /// Check if the identifier is a keyword, or a normal identifier.
    fn identifier_type(&self) -> Token {
        let mut chars = self.start.chars();

        // Note: I changed this code a bit from Crafting Interpreters to do less
        // index shenanigans that are pointless in Rust.
        match chars.next().unwrap_or('\0') {
            'a' => self.check_keyword("and", Token::And),
            'c' => self.check_keyword("class", Token::Class),
            'e' => self.check_keyword("else", Token::Else),
            'f' => match chars.next().unwrap_or('\0') {
                'a' => self.check_keyword("false", Token::False),
                'o' => self.check_keyword("for", Token::For),
                'u' => self.check_keyword("fun", Token::Fun),
                _ => Token::Identifier,
            },
            'i' => self.check_keyword("if", Token::If),
            'n' => self.check_keyword("nil", Token::Nil),
            'o' => self.check_keyword("or", Token::Or),
            'p' => self.check_keyword("print", Token::Print),
            'r' => self.check_keyword("return", Token::Return),
            's' => self.check_keyword("super", Token::Super),
            't' => match chars.next().unwrap_or('\0') {
                'h' => self.check_keyword("this", Token::This),
                'r' => self.check_keyword("true", Token::True),
                _ => Token::Identifier,
            },
            'v' => self.check_keyword("var", Token::Var),
            'w' => self.check_keyword("while", Token::While),
            _ => Token::Identifier,
        }
    }

    /// Confirms that the current lexeme is a keyword or lexeme.
    fn check_keyword(&self, keyword_text: &'static str, keyword: Token) -> Token {
        let token_length = self.start.len() - self.current.len();
        let lexeme = &self.start[..token_length];

        if lexeme == keyword_text {
            keyword
        } else {
            Token::Identifier
        }
    }

    /// Returns an lexeme with [Token::Error] as its token.
    fn error_token(&self, message: &'a str) -> Lexeme<'a> {
        assert_ne!(self.start, self.current);
        Lexeme {
            token: Token::Error,
            text: message,
            line: self.line,
        }
    }

    /// Returns a [Lexeme] from the span between self.start and self.current with the given
    /// [Token].
    fn make_lexeme(&self, token: Token) -> Lexeme<'a> {
        assert!(self.current.len() <= self.start.len());
        let extent = self.start.len() - self.current.len();
        let text = &self.start[..extent];

        Lexeme {
            token,
            text,
            line: self.line,
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Lexeme<'a>;

    fn next(&mut self) -> Option<Lexeme<'a>> {
        Some(self.scan_token())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // This iterator is infinite.
        (usize::MAX, None)
    }
}

impl<'a> Lexeme<'a> {
    /// Return the line number this token was found on.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Return the literal text of this token. For string literals, this always includes the
    /// quotes.
    pub fn text(&self) -> &'a str {
        self.text
    }

    /// Return the [Token] of this lexeme.
    pub fn token(&self) -> Token {
        self.token
    }
}

///////////////////////////////////////////// Helpers /////////////////////////////////////////////

/// Returns true if this char can start an identifier or keyword.
///
/// Note: this differs from Crafting Interpreters, as it uses isAlpha().
fn is_id_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Returns true if this char can be used after the first character of an identifier or keyword.
fn is_id_continue(c: char) -> bool {
    is_id_start(c) || c.is_ascii_digit()
}

////////////////////////////////////////////// Tests //////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scanning_every_keyword() {
        use Token::*;

        let source_code = "class classic {
            fun fund() {
                if (ifree and anders or orvile) {
                    print printer;
                } else {
                    for (former = 0; former < 10; former = former + 1) {
                    nill = nil;
                    }
                    super.falseFlag = truede;
                    this.thistle = true;
                    superMario = false or true;
                    return returned;
                }
                var varied;
                while (whileLoop) {
                    0;
                }
            }
        }";

        // I copied the indentation of the code above.
        #[rustfmt::skip]
        let expected_tokens = vec![
            Class, Identifier, LeftBrace,
                Fun, Identifier, LeftParen, RightParen, LeftBrace,
                    If, LeftParen, Identifier, And, Identifier, Or, Identifier, RightParen, LeftBrace,
                        Print, Identifier, Semicolon,
                    RightBrace, Else, LeftBrace,
                        For, LeftParen, Identifier, Equal, Number, Semicolon, Identifier, Less, Number, Semicolon, Identifier, Equal, Identifier, Plus, Number, RightParen, LeftBrace,
                            Identifier, Equal, Nil, Semicolon,
                        RightBrace,
                        Super, Dot, Identifier, Equal, Identifier, Semicolon,
                        This, Dot, Identifier, Equal,
                        True, Semicolon, Identifier, Equal, False, Or, True, Semicolon,
                        Return, Identifier, Semicolon,
                    RightBrace,
                    Var, Identifier, Semicolon,
                    While, LeftParen, Identifier, RightParen, LeftBrace,
                        Number, Semicolon,
                    RightBrace,
                RightBrace,
            RightBrace,
        ];

        let actual_tokens: Vec<_> = Scanner::new(source_code)
            .map(|lexeme| lexeme.token())
            .take_while(|&token| token != Eof)
            .collect();
        assert_eq!(expected_tokens, actual_tokens);
    }
}
