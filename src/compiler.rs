use enum_map::enum_map;

use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq)]
enum Precedence {
    // Todo: Change to "undefined?
    None,
    /// `=`
    Assignment,
    /// `or`
    Or,
    /// `and`
    And,
    /// `==` `!=`
    Equality,
    /// `<` `>` `<=` `>=`
    Comparison,
    /// + -
    Term,
    /// `*` `/`
    Factor,
    /// `!` `-`
    Unary,
    /// `.` `()`
    Call,
    /// Literals, and groupings
    Primary,
}

type ParserFn = fn(&mut Compiler) -> ();

#[derive(Copy, Clone)]
struct ParserRule {
    prefix: Option<ParserFn>,
    infix: Option<ParserFn>,
    precedence: Precedence,
}

#[derive(Debug)]
struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Lexeme<'a>,
    previous: Lexeme<'a>,
    had_error: bool,
    panic_mode: bool,
}

struct Compiler<'a> {
    parser: Parser<'a>,
    compiling_chunk: Chunk,
}

impl Precedence {
    fn higher_precedence(self) -> Precedence {
        use Precedence::*;
        match self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => panic!("Tried to get higher precedence than primary"),
        }
    }
}

impl ParserRule {
    #[inline(always)]
    fn higher_precedence(&self) -> Precedence {
        self.precedence.higher_precedence()
    }
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Parser {
        let scanner = Scanner::new(source);
        let error_token = scanner.make_sentinel("<before start>");

        Parser {
            scanner,
            current: error_token.clone(),
            previous: error_token,
            had_error: false,
            panic_mode: false,
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        // Get tokens until we get a non-error token.
        loop {
            self.current = self.scanner.scan_token();
            if self.current.token() != Token::Error {
                break;
            }

            self.error_at_current(self.current.text())
        }
    }

    /// Scan the next token. If the token is not of the desired type, an error message is printed.
    fn consume(&mut self, desired_token: Token, message: &'static str) {
        if self.current.token() == desired_token {
            return self.advance();
        }

        self.error_at_current(message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message)
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message)
    }

    fn error_at(&mut self, lexeme: Lexeme<'a>, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;

        eprint!("[line {}] Error:", lexeme.line());

        if lexeme.token() == Token::Eof {
            eprint!(" at end");
        } else if lexeme.token() == Token::Error {
            // Nothing
        } else {
            eprint!(" at '{}'", lexeme.text());
        }

        eprintln!(": {message}");
        self.had_error = true;
    }
}

impl<'a> Compiler<'a> {
    fn new(parser: Parser) -> Compiler {
        Compiler {
            parser,
            compiling_chunk: Chunk::default(),
        }
    }

    /// Takes ownership of the compiler, and returns the chunk
    fn compile(mut self) -> crate::Result<Chunk> {
        self.parser.advance();
        self.expression();
        self.parser
            .consume(Token::Eof, "Expected end of expression");
        self.end_compiler();

        if self.parser.had_error {
            return Err(InterpretationError::CompileError);
        }

        Ok(self.compiling_chunk)
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        // Print a listing of the bytecode to manually inspect compiled output.
        if cfg!(feature = "print_code") && !self.parser.had_error {
            crate::debug::disassemble_chunk(self.current_chunk(), "code");
        }
    }

    /// The core of the Pratt parsing algorithm. See [Pratt1973].
    ///
    /// [Pratt1973]: http://hall.org.ua/halls/wizzard/pdf/Vaughan.Pratt.TDOP.pdf
    fn parse_precedence(&mut self, precedence: Precedence) {
        // Get the next token.
        self.parser.advance();

        // What rule should we use right now?
        let prefix_rule = get_rule(self.parser.previous.token()).prefix;
        match prefix_rule {
            Some(parse_prefix) => parse_prefix(self),
            None => {
                self.parser.error("Expected expression");
                return;
            }
        }

        while precedence <= get_rule(self.parser.current.token()).precedence {
            self.parser.advance();
            let infix_rule = get_rule(self.parser.previous.token())
                .infix
                .expect("a rule with a defined precedence must always have an infix rule");

            infix_rule(self);
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn emit_return(&mut self) {
        unsafe { self.emit_byte(OpCode::Return as u8) }
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.make_constant(value);
        unsafe { self.emit_bytes(OpCode::Constant as u8, index) }
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant_usize(value);
        match u8::try_from(constant) {
            Ok(index) => index,
            Err(_) => {
                self.parser.error("Too many constants in one chunk");
                0
            }
        }
    }

    unsafe fn emit_byte(&mut self, byte: u8) {
        let line = self.parser.previous.line();
        self.current_chunk().write_unchecked(byte, line);
    }

    unsafe fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }
}

pub fn compile(source: &str) -> crate::Result<Chunk> {
    let parser = Parser::new(source);
    let compiler = Compiler::new(parser);

    compiler.compile()
}

////////////////////////////////////////// Parser rules ///////////////////////////////////////////

macro_rules! rule {
    ($prefix:expr, $infix:expr, $precedence:expr) => {
        ParserRule {
            prefix: $prefix,
            infix: $infix,
            precedence: $precedence,
        }
    };
}

fn get_rule(token: Token) -> ParserRule {
    use Token::*;

    #[rustfmt::skip]
    let rules = enum_map! {
        //                     Prefix          Infix         Precedence
        LeftParen    => rule!{ Some(grouping), None,         Precedence::None },
        RightParen   => rule!{ None,           None,         Precedence::None },
        LeftBrace    => rule!{ None,           None,         Precedence::None },
        RightBrace   => rule!{ None,           None,         Precedence::None },
        Comma        => rule!{ None,           None,         Precedence::None },
        Dot          => rule!{ None,           None,         Precedence::None },
        Minus        => rule!{ Some(unary),    Some(binary), Precedence::Term },
        Plus         => rule!{ None,           Some(binary), Precedence::Term },
        Semicolon    => rule!{ None,           None,         Precedence::None },
        Slash        => rule!{ None,           Some(binary), Precedence::Factor },
        Star         => rule!{ None,           Some(binary), Precedence::Factor },
        Bang         => rule!{ None,           None,         Precedence::None },
        BangEqual    => rule!{ None,           None,         Precedence::None },
        Equal        => rule!{ None,           None,         Precedence::None },
        EqualEqual   => rule!{ None,           None,         Precedence::None },
        Greater      => rule!{ None,           None,         Precedence::None },
        GreaterEqual => rule!{ None,           None,         Precedence::None },
        Less         => rule!{ None,           None,         Precedence::None },
        LessEqual    => rule!{ None,           None,         Precedence::None },
        Identifier   => rule!{ None,           None,         Precedence::None },
        StrLiteral   => rule!{ None,           None,         Precedence::None },
        Number       => rule!{ Some(number),   None,         Precedence::None },
        And          => rule!{ None,           None,         Precedence::None },
        Class        => rule!{ None,           None,         Precedence::None },
        Else         => rule!{ None,           None,         Precedence::None },
        False        => rule!{ None,           None,         Precedence::None },
        For          => rule!{ None,           None,         Precedence::None },
        Fun          => rule!{ None,           None,         Precedence::None },
        If           => rule!{ None,           None,         Precedence::None },
        Nil          => rule!{ None,           None,         Precedence::None },
        Or           => rule!{ None,           None,         Precedence::None },
        Print        => rule!{ None,           None,         Precedence::None },
        Return       => rule!{ None,           None,         Precedence::None },
        Super        => rule!{ None,           None,         Precedence::None },
        This         => rule!{ None,           None,         Precedence::None },
        True         => rule!{ None,           None,         Precedence::None },
        Var          => rule!{ None,           None,         Precedence::None },
        While        => rule!{ None,           None,         Precedence::None },
        Error        => rule!{ None,           None,         Precedence::None },
        Eof          => rule!{ None,           None,         Precedence::None },
    };

    rules[token]
}

fn grouping(compiler: &mut Compiler) {
    compiler.expression();
    compiler
        .parser
        .consume(Token::RightParen, "Expect ')' after grouping.");
}

fn number(compiler: &mut Compiler) {
    assert_eq!(Token::Number, compiler.parser.previous.token());
    let value = compiler
        .parser
        .previous
        .text()
        .parse::<f64>()
        .expect("Token MUST parse as a float");
    compiler.emit_constant(value.into());
}

fn unary(compiler: &mut Compiler) {
    let operator = compiler.parser.previous.token();

    // Compile the operand, so that it's placed on the stack.
    compiler.parse_precedence(Precedence::Unary);

    match operator {
        Token::Minus => unsafe { compiler.emit_byte(OpCode::Negate as u8) },
        _ => unreachable!(),
    }
}

fn binary(compiler: &mut Compiler) {
    let operator = compiler.parser.previous.token();
    let rule = get_rule(operator);

    compiler.parse_precedence(rule.higher_precedence());
    match operator {
        Token::Plus => unsafe { compiler.emit_byte(OpCode::Add as u8) },
        Token::Minus => unsafe { compiler.emit_byte(OpCode::Subtract as u8) },
        Token::Star => unsafe { compiler.emit_byte(OpCode::Multiply as u8) },
        Token::Slash => unsafe { compiler.emit_byte(OpCode::Divide as u8) },
        _ => unreachable!(),
    }
}
