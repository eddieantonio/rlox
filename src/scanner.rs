pub struct Token<'a> {
    pub ttype: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
}

#[derive(Debug, PartialEq, Eq)]
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

pub struct Scanner<'a> {
    start: &'a str,
    current: &'a str,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            start: source,
            current: source,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

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

    fn error_token(&self, message: &'a str) -> Token<'a> {
        assert_ne!(self.start, self.current);
        Token {
            ttype: TokenType::Error,
            lexeme: message,
            line: self.line,
        }
    }

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

    fn next<'b>(&mut self) -> Option<Token<'a>> {
        Some(self.scan_token())
    }
}

impl<'a> Token<'a> {
    pub fn len(&self) -> usize {
        self.lexeme.len()
    }

    pub fn is_empty(&self) -> bool {
        assert_ne!(0, self.len());
        false
    }
}
