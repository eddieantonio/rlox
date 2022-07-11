//! Handle Lox's lexical analysis.
//!
//! Contains the [Scanner] which implements an [Iterator] that yields [Token]s, each of which
//! belongs to a [TokenType].
//!
//! # Example
//!
//! ```
//! use rlox::scanner::{Scanner, Token, TokenType};
//! use TokenType::*;
//! let scanner = Scanner::new("print 1 + 2;");
//! let tokens: Vec<_> = scanner
//!     .map(|lexeme| lexeme.ttype)
//!     .take_while(|&kind| kind != Eof) // scanner will yield Eof forever...
//!     .collect();
//! assert_eq!(
//!     vec![Print, Number, Plus, Number, Semicolon],
//!     tokens
//! );
//! ```

/// A token from Lox's lexical grammar.
///
/// Note: I used Crafting Interpreters's terminology here, although I would personally rename them
/// as thus:
///
/// - Token       → Lexeme     -- an individual lexeme from the source code
/// - TokenType   → Token      -- the type of a lexeme
/// - Token.lexme → Token.text -- the actual text of the lexeme
pub struct Token<'a> {
    /// The [TokenType] of this token.
    pub ttype: TokenType,
    /// The actual text in the source code file.
    pub lexeme: &'a str,
    /// The line this token is found.
    pub line: usize,
}

/// What kind of [Token] you have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Semicolon, Star, Slash,
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

/// Scans Lox source code and iteratively yields [Token]s.
/// The scanner is stateful, and therefore, can only be used to do one pass over the source string.
/// The s
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

    /// Yield the next [Token] from the string. If the scanner has reached the end-of-file, this
    /// function will always return an end-of-file token.
    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        if is_id_start(c) {
            return self.identifier();
        }
        if c.is_ascii_digit() {
            return self.number();
        }

        match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                let ttype = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(ttype)
            }
            '=' => {
                let ttype = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(ttype)
            }
            '<' => {
                let ttype = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(ttype)
            }
            '>' => {
                let ttype = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(ttype)
            }
            '"' => self.string(),
            _ => self.error_token("Unexpected character"),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current.is_empty()
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
    fn match_char(&mut self, expected: char) -> bool {
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

    /// Returns an Error token.
    fn error_token(&self, message: &'a str) -> Token<'a> {
        assert_ne!(self.start, self.current);
        Token {
            ttype: TokenType::Error,
            lexeme: message,
            line: self.line,
        }
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

    /// Confirms that the current lexeme is a keyword or lexeme.
    fn check_keyword(&self, keyword_text: &'static str, keyword: TokenType) -> TokenType {
        let token_length = self.start.len() - self.current.len();
        let lexeme = &self.start[..token_length];

        if lexeme == keyword_text {
            keyword
        } else {
            TokenType::Identifier
        }
    }

    /// Check if the identifier is a keyword, or a normal identifier.
    fn identifier_type(&self) -> TokenType {
        let mut chars = self.start.chars();

        // Note: I changed this code a bit from Crafting Interpreters to do less
        // index shenanigans that are pointless in Rust.
        match chars.next().unwrap_or('\0') {
            'a' => self.check_keyword("and", TokenType::And),
            'c' => self.check_keyword("class", TokenType::Class),
            'e' => self.check_keyword("else", TokenType::Else),
            'f' => match chars.next().unwrap_or('\0') {
                'a' => self.check_keyword("false", TokenType::False),
                'o' => self.check_keyword("for", TokenType::For),
                'u' => self.check_keyword("fun", TokenType::Fun),
                _ => TokenType::Identifier,
            },
            'i' => self.check_keyword("if", TokenType::If),
            'n' => self.check_keyword("nil", TokenType::Nil),
            'o' => self.check_keyword("or", TokenType::Or),
            'p' => self.check_keyword("print", TokenType::Print),
            'r' => self.check_keyword("return", TokenType::Return),
            's' => self.check_keyword("super", TokenType::Super),
            't' => match chars.next().unwrap_or('\0') {
                'h' => self.check_keyword("this", TokenType::This),
                'r' => self.check_keyword("true", TokenType::True),
                _ => TokenType::Identifier,
            },
            'v' => self.check_keyword("var", TokenType::Var),
            'w' => self.check_keyword("while", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    /// Scan an identifier or keyword.
    fn identifier(&mut self) -> Token<'a> {
        while is_id_continue(self.peek()) {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    /// Scan a string literal. Expects the starting quote to have been consumed.
    fn string(&mut self) -> Token<'a> {
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
        self.make_token(TokenType::StrLiteral)
    }

    /// Scan a number literal. Expects the first digit to have already been consumed.
    fn number(&mut self) -> Token<'a> {
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

        self.make_token(TokenType::Number)
    }

    /// Returns a [Token] from the span between self.start and self.current with the given
    /// [TokenType].
    fn make_token(&self, ttype: TokenType) -> Token<'a> {
        assert!(self.current.len() <= self.start.len());
        let extent = self.start.len() - self.current.len();
        let lexeme = &self.start[..extent];

        Token {
            ttype,
            lexeme,
            line: self.line,
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        Some(self.scan_token())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // This iterator is infinite.
        (usize::MAX, None)
    }
}

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
