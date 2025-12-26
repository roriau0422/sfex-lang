use super::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;

const MAX_INDENT: usize = 100;
const TAB_SIZE: usize = 8;
const ALT_TAB_SIZE: usize = 1;

#[derive(Debug, Clone)]
pub enum LexerErrorKind {
    TooDeep,
    DedentError,
    IndentError,
    UnexpectedChar(char),
    UnterminatedString,
    NewlineInString,
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    _source: &'a str,

    // Position tracking
    line: usize,
    column: usize,
    position: usize,

    // Indentation tracking
    indent_stack: Vec<usize>,
    alt_indent_stack: Vec<usize>,
    atbol: bool,
    pendin: i32,

    // Buffered tokens
    token_buffer: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.chars().peekable(),
            _source: source,
            line: 1,
            column: 1,
            position: 0,
            indent_stack: vec![0], // Start with base level 0
            alt_indent_stack: vec![0],
            atbol: true, // Start at beginning of line
            pendin: 0,
            token_buffer: Vec::new(),
        }
    }

    // Main tokenization function
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn error(&self, kind: LexerErrorKind) -> LexerError {
        LexerError {
            kind,
            line: self.line,
            column: self.column,
        }
    }

    // Get next token (handles indentation logic)
    fn next_token(&mut self) -> Result<Token, LexerError> {
        // check for buffered tokens
        if !self.token_buffer.is_empty() {
            return Ok(self.token_buffer.remove(0));
        }

        // Check for pending INDENT/DEDENT tokens from previous line
        if self.pendin != 0 {
            return Ok(self.emit_pending_indent_dedent());
        }

        if self.atbol {
            self.handle_indentation()?;
            if self.pendin != 0 {
                return Ok(self.emit_pending_indent_dedent());
            }
        }

        // skip remaining whitespace, spaces between tokens
        self.skip_whitespace();
        self.read_token()
    }

    /// Indentation handling
    fn handle_indentation(&mut self) -> Result<(), LexerError> {
        self.atbol = false; // Reset flag

        let mut col = 0;
        let mut alt_col = 0;

        // Calculate indentation of current line
        loop {
            match self.peek_char() {
                Some(' ') => {
                    self.advance();
                    col += 1;
                    alt_col += 1;
                }
                Some('\t') => {
                    self.advance();
                    col = (col / TAB_SIZE + 1) * TAB_SIZE;
                    alt_col = (alt_col / ALT_TAB_SIZE + 1) * ALT_TAB_SIZE;
                }
                Some('\x0C') => {
                    // Form feed, Ctrl+L
                    self.advance();
                    col = 0;
                    alt_col = 0;
                }
                _ => break,
            }
        }

        // Skip empty lines and comments
        match self.peek_char() {
            Some('#') | Some('\n') | Some('\r') | None => {
                return Ok(());
            }
            _ => {}
        }

        // Compare calculated 'col' against indentation stack
        let current_indent = *self.indent_stack.last().unwrap();

        // CASE A: No change, same level
        if col == current_indent {
            // Check for mixed tabs/spaces
            let current_alt = *self.alt_indent_stack.last().unwrap();
            if alt_col != current_alt {
                return Err(self.error(LexerErrorKind::IndentError));
            }
            // No INDENT or DEDENT needed
        }
        // CASE B: INDENT (deeper level)
        else if col > current_indent {
            // Check stack overflow
            if self.indent_stack.len() >= MAX_INDENT {
                return Err(self.error(LexerErrorKind::TooDeep));
            }

            // Check for mixed tabs/spaces
            let current_alt = *self.alt_indent_stack.last().unwrap();
            if alt_col <= current_alt {
                return Err(self.error(LexerErrorKind::IndentError));
            }

            // Push new level
            self.pendin += 1;
            self.indent_stack.push(col);
            self.alt_indent_stack.push(alt_col);
        }
        // CASE C: DEDENT shallower level
        else {
            // Pop stack until its find matching level
            while self.indent_stack.len() > 1 && col < *self.indent_stack.last().unwrap() {
                self.pendin -= 1;
                self.indent_stack.pop();
                self.alt_indent_stack.pop();
            }

            // Must land exactly on a known indentation level
            if col != *self.indent_stack.last().unwrap() {
                return Err(self.error(LexerErrorKind::DedentError));
            }

            // Check mixed tabs/spaces
            let current_alt = *self.alt_indent_stack.last().unwrap();
            if alt_col != current_alt {
                return Err(self.error(LexerErrorKind::IndentError));
            }
        }

        Ok(())
    }

    /// Emit pending INDENT or DEDENT token
    fn emit_pending_indent_dedent(&mut self) -> Token {
        let token_type = if self.pendin < 0 {
            self.pendin += 1;
            TokenType::Dedent
        } else {
            self.pendin -= 1;
            TokenType::Indent
        };

        Token::new(token_type, self.line, self.column, 0)
    }

    /// Read the next token
    fn read_token(&mut self) -> Result<Token, LexerError> {
        match self.peek_char() {
            None => {
                // EOF: Emit remaining DEDENTs
                if self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    self.alt_indent_stack.pop();
                    self.pendin -= 1;
                    return Ok(Token::new(TokenType::Dedent, self.line, self.column, 0));
                }
                Ok(Token::new(TokenType::Eof, self.line, self.column, 0))
            }

            Some('\r') => {
                self.advance();
                // Handle CRLF as a single newline (Windows line endings)
                if matches!(self.input.peek(), Some('\n')) {
                    self.advance(); // Skip the '\n' part of CRLF
                }
                self.line += 1;
                self.column = 1;
                self.atbol = true;
                Ok(Token::new(TokenType::Newline, self.line - 1, 1, 1))
            }

            Some('\n') => {
                self.advance();
                self.line += 1;
                self.column = 1;
                self.atbol = true;
                Ok(Token::new(TokenType::Newline, self.line - 1, 1, 1))
            }

            Some('#') => self.read_comment(),

            Some('"') | Some('\'') => {
                let quote = self.peek_char().unwrap();
                // Save the first quote
                self.advance();

                if self.peek_char() == Some(quote) {
                    self.advance(); // consume second quote

                    // Check if third character is also a quote
                    if self.peek_char() == Some(quote) {
                        // Triple quote detected!
                        self.advance(); // consume third quote
                        return self.read_triple_quoted_string(quote);
                    } else {
                        // Only two quotes, this is a empty string "maybe"
                        let length = 2;
                        return Ok(Token::new(
                            TokenType::String_(String::new()),
                            self.line,
                            self.column - length,
                            length,
                        ));
                    }
                } else {
                    // Single quote - read regular string content
                    return self.read_string_content(quote);
                }
            }

            Some(c) if c.is_ascii_digit() => self.read_number(),

            Some(c) if c.is_alphabetic() || c == '_' => self.read_identifier_or_keyword(),

            Some(':') => {
                self.advance();
                Ok(Token::new(TokenType::Colon, self.line, self.column - 1, 1))
            }

            Some(',') => {
                self.advance();
                Ok(Token::new(TokenType::Comma, self.line, self.column - 1, 1))
            }

            Some('(') => {
                self.advance();
                Ok(Token::new(
                    TokenType::LeftParen,
                    self.line,
                    self.column - 1,
                    1,
                ))
            }

            Some(')') => {
                self.advance();
                Ok(Token::new(
                    TokenType::RightParen,
                    self.line,
                    self.column - 1,
                    1,
                ))
            }

            Some('[') => {
                self.advance();
                Ok(Token::new(
                    TokenType::LeftBracket,
                    self.line,
                    self.column - 1,
                    1,
                ))
            }

            Some(']') => {
                self.advance();
                Ok(Token::new(
                    TokenType::RightBracket,
                    self.line,
                    self.column - 1,
                    1,
                ))
            }

            Some('{') => {
                self.advance();
                Ok(Token::new(
                    TokenType::LeftBrace,
                    self.line,
                    self.column - 1,
                    1,
                ))
            }

            Some('}') => {
                self.advance();
                Ok(Token::new(
                    TokenType::RightBrace,
                    self.line,
                    self.column - 1,
                    1,
                ))
            }

            Some('.') => {
                self.advance();
                Ok(Token::new(TokenType::Dot, self.line, self.column - 1, 1))
            }

            Some('+') => {
                self.advance();
                Ok(Token::new(TokenType::Plus, self.line, self.column - 1, 1))
            }

            Some('-') => {
                self.advance();
                Ok(Token::new(TokenType::Minus, self.line, self.column - 1, 1))
            }

            Some('*') => {
                self.advance();
                Ok(Token::new(TokenType::Star, self.line, self.column - 1, 1))
            }

            Some('/') => {
                self.advance();
                Ok(Token::new(TokenType::Slash, self.line, self.column - 1, 1))
            }

            Some('%') => {
                self.advance();
                Ok(Token::new(
                    TokenType::Percent,
                    self.line,
                    self.column - 1,
                    1,
                ))
            }

            Some('=') => {
                self.advance();
                Ok(Token::new(TokenType::Equals, self.line, self.column - 1, 1))
            }

            Some('>') => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Ok(Token::new(
                        TokenType::GreaterEq,
                        self.line,
                        self.column - 2,
                        2,
                    ))
                } else {
                    Ok(Token::new(
                        TokenType::Greater,
                        self.line,
                        self.column - 1,
                        1,
                    ))
                }
            }

            Some('<') => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Ok(Token::new(TokenType::LessEq, self.line, self.column - 2, 2))
                } else {
                    Ok(Token::new(TokenType::Less, self.line, self.column - 1, 1))
                }
            }

            Some('!') => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Ok(Token::new(
                        TokenType::NotEquals,
                        self.line,
                        self.column - 2,
                        2,
                    ))
                } else {
                    Err(self.error(LexerErrorKind::UnexpectedChar('!')))
                }
            }

            Some(c) => Err(self.error(LexerErrorKind::UnexpectedChar(c))),
        }
    }

    // Helper methods
    fn peek_char(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if let Some(ch) = c {
            self.position += ch.len_utf8();
            self.column += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_comment(&mut self) -> Result<Token, LexerError> {
        let start_col = self.column;
        self.advance(); // Skip '#'

        let mut comment = String::new();
        while let Some(c) = self.peek_char() {
            if c == '\n' || c == '\r' {
                break;
            }
            comment.push(c);
            self.advance();
        }

        Ok(Token::new(
            TokenType::Comment(comment.trim().to_string()),
            self.line,
            start_col,
            comment.len() + 1,
        ))
    }

    fn read_string_content(&mut self, quote: char) -> Result<Token, LexerError> {
        let start_col = self.column - 1;
        let mut value = String::new();

        loop {
            match self.peek_char() {
                None => return Err(self.error(LexerErrorKind::UnterminatedString)),

                Some(c) if c == quote => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    if let Some(escaped) = self.advance() {
                        match escaped {
                            'n' => value.push('\n'),
                            't' => value.push('\t'),
                            'r' => value.push('\r'),
                            '\\' => value.push('\\'),
                            c if c == quote => value.push(c),
                            c => {
                                value.push('\\');
                                value.push(c);
                            }
                        }
                    }
                }
                Some('\r') | Some('\n') => {
                    return Err(self.error(LexerErrorKind::NewlineInString));
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
            }
        }

        let length = value.len() + 2;

        Ok(Token::new(
            TokenType::String_(value),
            self.line,
            start_col,
            length,
        ))
    }

    fn read_triple_quoted_string(&mut self, quote: char) -> Result<Token, LexerError> {
        let start_col = self.column;
        let start_line = self.line;
        let mut value = String::new();

        loop {
            match self.peek_char() {
                None => return Err(self.error(LexerErrorKind::UnterminatedString)),

                Some(c) if c == quote => {
                    // Check if this is the start of closing triple quotes
                    self.advance();

                    if self.peek_char() == Some(quote) {
                        self.advance(); // Second quote

                        if self.peek_char() == Some(quote) {
                            self.advance(); // Third quote - done
                            break;
                        } else {
                            // Only two quotes - add them both to the value
                            value.push(quote);
                            value.push(quote);
                        }
                    } else {
                        // Only one quote - add it to the value
                        value.push(quote);
                    }
                }
                Some('\\') => {
                    self.advance();
                    if let Some(escaped) = self.advance() {
                        match escaped {
                            'n' => value.push('\n'),
                            't' => value.push('\t'),
                            'r' => value.push('\r'),
                            '\\' => value.push('\\'),
                            c if c == quote => value.push(c),
                            c => {
                                value.push('\\');
                                value.push(c);
                            }
                        }
                    }
                }
                // Allow literal newlines in triple-quoted strings
                Some('\r') => {
                    self.advance();
                    // Handle CRLF as a single newline
                    if matches!(self.peek_char(), Some('\n')) {
                        self.advance();
                    }
                    value.push('\n'); // Normalize to LF
                    self.line += 1;
                    self.column = 1;
                }
                Some('\n') => {
                    self.advance();
                    value.push('\n');
                    self.line += 1;
                    self.column = 1;
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
            }
        }

        let length = value.len() + 6;

        Ok(Token::new(
            TokenType::String_(value),
            start_line,
            start_col,
            length,
        ))
    }

    fn read_number(&mut self) -> Result<Token, LexerError> {
        let start_col = self.column;
        let mut number = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() || c == '.' {
                number.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let length = number.len();
        Ok(Token::new(
            TokenType::Number(number),
            self.line,
            start_col,
            length,
        ))
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token, LexerError> {
        let start_col = self.column;
        let mut ident = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let length = ident.len();
        let token_type = match ident.as_str() {
            // Length 2
            "Do" => TokenType::Do,
            "If" => TokenType::If,
            "To" => TokenType::To,
            "in" => TokenType::In,
            "is" => TokenType::Is,
            "on" => TokenType::Identifier("on".to_string()),
            "or" => TokenType::Or,
            "to" => TokenType::To_,

            // Length 3
            "Use" => TokenType::Use,
            "For" => TokenType::For,
            "Try" => TokenType::Try,
            "Set" => TokenType::Identifier("Set".to_string()),
            "and" => TokenType::And,
            "not" => TokenType::Not,
            "off" => TokenType::Identifier("off".to_string()),

            // Length 4
            "Else" => TokenType::Else,
            "True" => TokenType::True_,
            "When" => TokenType::When,
            "each" => TokenType::Each,
            "with" => TokenType::With,

            // Length 5
            "Break" => TokenType::Break,
            "Catch" => TokenType::Catch,
            "False" => TokenType::False_,
            "Story" => TokenType::Story,
            "times" => TokenType::Times,
            "while" => TokenType::While,

            // Length 6
            "Adjust" => TokenType::Adjust,
            "Always" => TokenType::Always,
            "Create" => TokenType::Create,
            "Called" => TokenType::Called,
            "Repeat" => TokenType::Repeat,
            "Return" => TokenType::Return,
            "Switch" => TokenType::Identifier("Switch".to_string()),

            // Length 7
            "Concept" => TokenType::Concept,
            "Proceed" => TokenType::Proceed,

            // Length 8
            "Continue" => TokenType::Continue,

            // Length 9
            "Situation" => TokenType::Situation,
            "Otherwise" => TokenType::Otherwise,

            // Length 10
            "background" => TokenType::Background,

            // Default
            _ => TokenType::Identifier(ident),
        };

        Ok(Token::new(token_type, self.line, start_col, length))
    }
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.kind {
            LexerErrorKind::TooDeep => "Indentation too deep".to_string(),
            LexerErrorKind::DedentError => "Invalid dedent level".to_string(),
            LexerErrorKind::IndentError => {
                "Inconsistent indentation (mixed tabs/spaces)".to_string()
            }
            LexerErrorKind::UnexpectedChar(ch) => format!("Unexpected character '{}'", ch),
            LexerErrorKind::UnterminatedString => "Unterminated string literal".to_string(),
            LexerErrorKind::NewlineInString => "Newline in string literal".to_string(),
        };
        write!(
            f,
            "Lexer error at line {}, column {}: {}",
            self.line, self.column, message
        )
    }
}

impl std::error::Error for LexerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_indentation() {
        let source = r#"Story:
    Print "Hello"
    If True:
        Print "Indented"
    Print "Back"
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        // Should have INDENT and DEDENT tokens
        let has_indent = tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::Indent));
        let has_dedent = tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::Dedent));

        assert!(has_indent, "Should have INDENT token");
        assert!(has_dedent, "Should have DEDENT token");
    }

    #[test]
    fn test_mixed_tabs_spaces_error() {
        let source = "Story:\n    Print \"Tab\"\n\tPrint \"Space\"";

        let mut lexer = Lexer::new(source);
        let result = lexer.tokenize();

        assert!(result.is_err(), "Should fail on mixed tabs/spaces");
    }
}
