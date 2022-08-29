//! Contains the Lox parser and bytecode compiler.
use crate::chunk::WrittenOpcode;
use crate::gc::ActiveGC;
use crate::prelude::*;

/////////////////////////////////////////// Public API ////////////////////////////////////////////

/// Compiles the given Lox source code and, if successful returns one bytecode [Chunk].
/// An [ActiveGC] is required because string literals will be allocated and owned by the GC.
pub fn compile(source: &str, gc: &'_ ActiveGC) -> crate::Result<Chunk> {
    let parser = Parser::new(source, gc);
    let compiler = Compiler::new(parser);
    compiler.compile()
}

///////////////////////////////////// Implementation details //////////////////////////////////////

/// Precedence rules for [Token]s in Lox.
///
/// Precedence rules have a well-defined partial ordering ([PartialOrd]), which is required for use
/// in the Pratt parsing algorithm.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq)]
enum Precedence {
    // Todo: Change to "Undefined?
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

/// A rule in the Pratt parser table. See [Compiler::parse_precedence()] for usage.
#[derive(Copy, Clone)]
struct ParserRule {
    prefix: Option<ParserFn>,
    infix: Option<ParserFn>,
    precedence: Precedence,
}

/// Any possible action taken from the parsing table. Actions take the entire compiler state, and
/// convert it, usually emitting bytecode.
type ParserFn = fn(&mut Compiler, bool) -> ();

/// Contains the parser state. For some strange reason, this also includes error status.
///
/// The reference to [ActiveGC] is required, but never accessed, because having a reference to it
/// guarentees that the static (global) garbage collector is installed, We need all this so that
/// string literals can be owned by the GC for the running program.
#[derive(Debug)]
struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Lexeme<'a>,
    previous: Lexeme<'a>,
    had_error: bool,
    panic_mode: bool,
    _active_gc: &'a ActiveGC,
}

/// Contains the compiler state, which includes the [Parser] and the current chunk being produced.
struct Compiler<'a> {
    parser: Parser<'a>,
    compiling_chunk: Chunk,
}

impl Precedence {
    /// Returns the next higher level of precedence.
    ///
    /// # Panics
    ///
    /// Panics if trying to obtain a higher-level of precedence than the maximum,
    /// [Precedence::Primary], which is the precedence of literals and l-values.
    #[inline]
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
    /// Returns one level of precedence higher than the rule's precedence.
    /// See [Precedence::higher_precedence()].
    #[inline(always)]
    fn higher_precedence(&self) -> Precedence {
        self.precedence.higher_precedence()
    }
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given source code.
    /// Note that parsing string literals requires an active GC.
    fn new(source: &'a str, active_gc: &'a ActiveGC) -> Parser<'a> {
        let mut scanner = Scanner::new(source);
        let first_token = scanner.scan_token();
        let error_token = scanner.make_sentinel("<before first token>");

        Parser {
            scanner,
            previous: error_token,
            current: first_token,
            had_error: false,
            panic_mode: false,
            _active_gc: active_gc,
        }
    }

    /// Update self.previous and self.current such that they move one token further in the token
    /// stream.
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

    /// Return true if the current token is equal to the given token.
    fn check(&self, token: Token) -> bool {
        self.current.token() == token
    }

    /// Scan the next token. Advances if the token matches `desired_token`. Returns whether
    /// `desired_token` was matched.
    fn match_and_advance(&mut self, desired_token: Token) -> bool {
        if self.check(desired_token) {
            self.advance();
            return true;
        }
        false
    }

    /// Emit a compiler error, located at the previous [Lexeme]. In Pratt parsing, this is the
    /// handler you usually want to call, because the previous lexeme decided which [ParserRule]
    /// was accepted.
    fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message)
    }

    /// Emit a compiler error, located at the current [Lexeme].
    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message)
    }

    /// Emit a compiler error, located at the given [Lexeme].
    fn error_at(&mut self, lexeme: Lexeme<'a>, message: &str) {
        // *Attempt* to prevent a deluge of spurious syntax errors:
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        self.had_error = true;

        // Print the actual message:
        eprint!("[line {}] Error:", lexeme.line());
        if lexeme.token() == Token::Eof {
            eprint!(" at end");
        } else if lexeme.token() == Token::Error {
            // Nothing
        } else {
            eprint!(" at '{}'", lexeme.text());
        }
        eprintln!(": {message}");
    }

    /// Synchronize after being in panic mode.
    ///
    /// The heuristic is that we're going to gobble up and discard tokens until we **think** we're
    /// a point that makes sense in the grammar. Points that make sense in a grammar are the start
    /// of statements (statement boundaries). We could be wrong!
    ///
    /// Note: this is not a fool-proof heuristic, but we're implementing it anyway!
    fn synchronize(&mut self) {
        // TODO: Eddie, your research is all about avoiding cascading errors. What should we do
        // instead of creating cascading errors. Perhaps read Diekmann & Tratt 2020.

        self.panic_mode = false;
        while self.current.token() != Token::Eof {
            if self.previous.token() == Token::Semicolon {
                return;
            }

            match self.current.token() {
                Token::Class
                | Token::Fun
                | Token::Var
                | Token::For
                | Token::If
                | Token::While
                | Token::Print
                | Token::Return => return,
                _ => (), // continue panicing
            }
        }
    }
}

impl<'a> Compiler<'a> {
    /// Creates a new compiler with the given [Parser].
    fn new(parser: Parser) -> Compiler {
        Compiler {
            parser,
            compiling_chunk: Chunk::default(),
        }
    }

    /// Takes ownership of the compiler, and returns the chunk
    fn compile(mut self) -> crate::Result<Chunk> {
        while !self.match_and_advance(Token::Eof) {
            self.declaration();
        }
        self.end_compiler();

        if self.parser.had_error {
            return Err(InterpretationError::CompileError);
        }

        Ok(self.compiling_chunk)
    }

    /// Signal the end of compilation.
    // Note: Could consider "finalizing" compilation here by taking ownership of the compiler and
    // returning some sort of "CompilationResult", making it impossible to write any more bytes to
    // the now finished chunk.
    fn end_compiler(&mut self) {
        self.emit_return();

        // Print a listing of the bytecode to manually inspect compiled output.
        if cfg!(feature = "print_code") && !self.parser.had_error {
            crate::debug::disassemble_chunk(self.current_chunk(), "code");
        }
    }

    /// The core of the Pratt parsing algorithm.
    ///
    /// See: <https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing>
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let can_assign = precedence <= Precedence::Assignment;

        // First, figure out how to parse the prefix.
        // TODO: rename to prefix_rule
        if let Some(parse_prefix) = self.rule_from_previous().prefix {
            parse_prefix(self, can_assign);
        } else {
            // TODO: better error message. This is difficult because we lack
            // state that lets us know how "far off" the token stream is to something that
            // would parse properly.
            self.parser
                .error("Could not figure out how to understand symbol in this context");
            return;
        }

        while precedence <= self.rule_from_current().precedence {
            // current is now previous:
            self.advance();
            let infix_rule = self
                .rule_from_previous()
                .infix
                .expect("a rule with a defined precedence must always have an infix rule");

            infix_rule(self, can_assign);
        }
    }

    /// Add the identifier text to the current chunk's constants table.
    fn identifier_constant(&mut self, lexeme: Lexeme) -> u8 {
        self.make_constant(lexeme.text().into())
    }

    /// Consume the next identifer and interpret it as a variable.
    /// Returns the constant for the indentifier name.
    fn parse_variable(&mut self, error_message: &'static str) -> u8 {
        self.parser.consume(Token::Identifier, error_message);
        let lexeme = self.parser.previous.clone();
        self.identifier_constant(lexeme)
    }

    /// Define a new global variable.
    fn define_variable(&mut self, global: u8) {
        self.emit_instruction(OpCode::DefineGlobal)
            .with_operand(global);
    }

    /// Parse a variable. This could either be a variable access or an assignment, depending on
    /// `can_assign` and the syntactic context.
    fn named_variable(&mut self, lexeme: Lexeme, can_assign: bool) {
        let arg = self.identifier_constant(lexeme);

        // Peek ahead and look if we're assigning.
        // This only works if we're parsing at a lower or equal precedence to assignment.
        if can_assign && self.match_and_advance(Token::Equal) {
            // We're in an assignment expression!
            // Parse the right-hand side:
            self.expression();
            self.emit_instruction(OpCode::SetGlobal).with_operand(arg);
        } else {
            // A reference to a variable.
            self.emit_instruction(OpCode::GetGlobal).with_operand(arg);
        }
    }

    /// Parse a declaration.
    fn declaration(&mut self) {
        if self.match_and_advance(Token::Var) {
            self.var_statement();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.parser.synchronize();
        }
    }

    /// Parse a statement.
    fn statement(&mut self) {
        if self.match_and_advance(Token::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    /// Parse an expression.
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    /// Parse a variable declaration. Assumes `var` has already been consumed
    fn var_statement(&mut self) {
        let global = self.parse_variable("need a variable name after var");

        if self.match_and_advance(Token::Equal) {
            self.expression();
        } else {
            self.emit_instruction(OpCode::Nil);
        }

        self.parser
            .consume(Token::Semicolon, "expect ; after this variable declaration");

        self.define_variable(global);
    }

    /// Parse an expression statement (e.g., assignments, function calls).
    fn expression_statement(&mut self) {
        self.expression();
        self.parser.consume(
            Token::Semicolon,
            // A better error message would highlight the statement,
            // then show where the semicolon is PROBABLY missing.
            "expected semicolon to end this statement",
        );
        // Expressions have 0 stack effect, meaning they can't leave anything on the stack.
        // Expressions produce a thing on the stack, and we need to get rid of it!
        self.emit_instruction(OpCode::Pop);
    }

    /// Parse a print statement. Assumes `print` has already been consumed.
    fn print_statement(&mut self) {
        self.expression();
        self.parser.consume(
            Token::Semicolon,
            "expected semicolon to end print statement",
        );
        self.emit_instruction(OpCode::Print);
    }

    /// Appends [OpCode::Return] to current [Chunk].
    fn emit_return(&mut self) {
        self.emit_instruction(OpCode::Return);
    }

    /// Appends [OpCode::Constant] to current [Chunk], using the current value.
    fn emit_constant(&mut self, value: Value) {
        let index = self.make_constant(value);
        self.emit_instruction(OpCode::Constant).with_operand(index);
    }

    /// Appends a new constant to the current [Chunk].
    ///
    /// # Error
    ///
    /// When the constant index is greater than 255 (and thus can no longer be represented as a
    /// u8), this signals a compiler error and returns `0u8`. The current [Chunk] can still be
    /// appended to, however, it is invalid, and should not be emitted as a valid program.
    fn make_constant(&mut self, value: Value) -> u8 {
        if let Some(index) = self.current_chunk().add_constant(value) {
            index
        } else {
            self.parser.error("Too many constants in one chunk");
            0
        }
    }

    /// Writes an [OpCode] to the current [Chunk].
    /// Returns a [WrittenOpcode], with which you can write an operand.
    fn emit_instruction(&mut self, opcode: OpCode) -> WrittenOpcode {
        let line = self.line_number_of_prefix();
        self.current_chunk().write_opcode(opcode, line)
    }

    /// Writes two [OpCode] to the current [Chunk].
    fn emit_instructions(&mut self, op1: OpCode, op2: OpCode) -> WrittenOpcode {
        let line = self.line_number_of_prefix();
        self.current_chunk().write_opcode(op1, line);
        self.current_chunk().write_opcode(op2, line)
    }

    ///////////////////////////////////////// Aliases /////////////////////////////////////////////

    /// Returns the current [Chunk].
    #[inline(always)]
    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    /// Advance one token in scanner, such that:
    /// ```text
    /// (previous, current) = (current, scanner.next_token())
    /// ```
    #[inline(always)]
    fn advance(&mut self) {
        self.parser.advance()
    }

    /// Returns the line number of the prefix token, a.k.a., `self.parser.previous`.
    #[inline(always)]
    fn line_number_of_prefix(&self) -> usize {
        self.parser.previous.line()
    }

    /// Delegates to [Parser::match_and_advance]. Returns true if the token was matched.
    #[inline(always)]
    fn match_and_advance(&mut self, desired_token: Token) -> bool {
        self.parser.match_and_advance(desired_token)
    }

    /// Returns the token of the prefix in the process of being parsed.
    #[inline(always)]
    fn rule_from_previous(&self) -> ParserRule {
        get_rule(self.previous_token())
    }

    /// Returns the token of the prefix in the process of being parsed.
    #[inline(always)]
    fn rule_from_current(&self) -> ParserRule {
        get_rule(self.parser.current.token())
    }

    /// Return the token (type) of the previous value. This is useful in prefix parser functions.
    #[inline(always)]
    fn previous_token(&self) -> Token {
        self.parser.previous.token()
    }
}

////////////////////////////////////////// Parser rules ///////////////////////////////////////////

/// Makes defining [ParserRule]s a bit cleaner looking.
macro_rules! rule {
    ($prefix:expr, $infix:expr, $precedence:expr) => {
        ParserRule {
            prefix: $prefix,
            infix: $infix,
            precedence: $precedence,
        }
    };
}

#[rustfmt::skip]
fn get_rule(token: Token) -> ParserRule {
    use Token::*;
    match token {
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
        Bang         => rule!{ Some(unary),    None,         Precedence::None },
        BangEqual    => rule!{ None,           Some(binary), Precedence::Equality },
        Equal        => rule!{ None,           None,         Precedence::None },
        EqualEqual   => rule!{ None,           Some(binary), Precedence::Equality },
        Greater      => rule!{ None,           Some(binary), Precedence::Comparison },
        GreaterEqual => rule!{ None,           Some(binary), Precedence::Comparison },
        Less         => rule!{ None,           Some(binary), Precedence::Comparison },
        LessEqual    => rule!{ None,           Some(binary), Precedence::Comparison },
        Identifier   => rule!{ Some(variable), None,         Precedence::None },
        StrLiteral   => rule!{ Some(string),   None,         Precedence::None },
        Number       => rule!{ Some(number),   None,         Precedence::None },
        And          => rule!{ None,           None,         Precedence::None },
        Class        => rule!{ None,           None,         Precedence::None },
        Else         => rule!{ None,           None,         Precedence::None },
        False        => rule!{ Some(literal),  None,         Precedence::None },
        For          => rule!{ None,           None,         Precedence::None },
        Fun          => rule!{ None,           None,         Precedence::None },
        If           => rule!{ None,           None,         Precedence::None },
        Nil          => rule!{ Some(literal),  None,         Precedence::None },
        Or           => rule!{ None,           None,         Precedence::None },
        Print        => rule!{ None,           None,         Precedence::None },
        Return       => rule!{ None,           None,         Precedence::None },
        Super        => rule!{ None,           None,         Precedence::None },
        This         => rule!{ None,           None,         Precedence::None },
        True         => rule!{ Some(literal),  None,         Precedence::None },
        Var          => rule!{ None,           None,         Precedence::None },
        While        => rule!{ None,           None,         Precedence::None },
        Error        => rule!{ None,           None,         Precedence::None },
        Eof          => rule!{ None,           None,         Precedence::None },
    }
}

/// Parse '(' as a prefix. Assumes '(' has been consumed.
fn grouping(compiler: &mut Compiler, _can_assign: bool) {
    debug_assert_eq!(Token::LeftParen, compiler.previous_token());
    compiler.expression();
    compiler
        .parser
        .consume(Token::RightParen, "Expect ')' after grouping.");
}

/// Parse a number literal as a prefix. Assumes number has been consumed.
fn number(compiler: &mut Compiler, _can_assign: bool) {
    debug_assert_eq!(Token::Number, compiler.previous_token());
    let value = compiler
        .parser
        .previous
        .text()
        .parse::<f64>()
        .expect("Internal error: Token::Number MUST parse as a float, but didn't?");
    compiler.emit_constant(value.into());
}

/// Parse an unary operator as a prefix. Assumes the operator has been consumed.
fn unary(compiler: &mut Compiler, _can_assign: bool) {
    let operator = compiler.previous_token();

    // Compile the operand, so that it's placed on the stack.
    compiler.parse_precedence(Precedence::Unary);

    match operator {
        Token::Bang => compiler.emit_instruction(OpCode::Not),
        Token::Minus => compiler.emit_instruction(OpCode::Negate),
        _ => unreachable!(),
    };
}

/// Parse a binary operator as an infix. Assumes the operator has been consumed.
fn binary(compiler: &mut Compiler, _can_assign: bool) {
    let operator = compiler.previous_token();
    let rule = get_rule(operator);

    compiler.parse_precedence(rule.higher_precedence());
    match operator {
        Token::BangEqual => compiler.emit_instructions(OpCode::Equal, OpCode::Not),
        Token::EqualEqual => compiler.emit_instruction(OpCode::Equal),
        Token::Greater => compiler.emit_instruction(OpCode::Greater),
        Token::GreaterEqual => compiler.emit_instructions(OpCode::Less, OpCode::Not),
        Token::Less => compiler.emit_instruction(OpCode::Less),
        Token::LessEqual => compiler.emit_instructions(OpCode::Greater, OpCode::Not),
        Token::Plus => compiler.emit_instruction(OpCode::Add),
        Token::Minus => compiler.emit_instruction(OpCode::Subtract),
        Token::Star => compiler.emit_instruction(OpCode::Multiply),
        Token::Slash => compiler.emit_instruction(OpCode::Divide),
        _ => unreachable!(),
    };
}

/// Parse a keyword literal as a prefix. Assumes the keyword has been consumed.
fn literal(compiler: &mut Compiler, _can_assign: bool) {
    match compiler.previous_token() {
        Token::False => compiler.emit_instruction(OpCode::False),
        Token::Nil => compiler.emit_instruction(OpCode::Nil),
        Token::True => compiler.emit_instruction(OpCode::True),
        _ => unreachable!(),
    };
}

/// Parse a string literal. Add it to the constant pool.
fn string(compiler: &mut Compiler, _can_assign: bool) {
    debug_assert_eq!(Token::StrLiteral, compiler.previous_token());

    // Access the string contents (without the quotes)
    let literal = compiler.parser.previous.text();
    debug_assert!(literal.len() >= 2);
    debug_assert!(literal.starts_with('"'));
    debug_assert!(literal.ends_with('"'));

    let last_index = literal.len() - 1;
    let contents = &literal[1..last_index];
    compiler.emit_constant(contents.into());
}

/// Parse a variable. It can be either a variable access or assignment, which is why `can_assign`
/// is required by all callbacks!
fn variable(compiler: &mut Compiler, can_assign: bool) {
    compiler.named_variable(compiler.parser.previous.clone(), can_assign);
}

////////////////////////////////////////////// Tests //////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn precedence_confidence_check() {
        // High-level precedence (C-like)
        assert!(Precedence::Assignment < Precedence::Or);
        assert!(Precedence::Or < Precedence::And);
        assert!(Precedence::And < Precedence::Equality);
        assert!(Precedence::Equality < Precedence::Comparison);

        // PEDMAS
        // () has greater precedence than */
        assert!(Precedence::Call > Precedence::Factor);
        // */ has greater precedence than +-
        assert!(Precedence::Factor > Precedence::Term);

        // ``and should be one level of precedence higher than `or`
        assert_eq!(Precedence::And, Precedence::Or.higher_precedence());
        assert_eq!(Precedence::Factor, Precedence::Term.higher_precedence());
    }
}
