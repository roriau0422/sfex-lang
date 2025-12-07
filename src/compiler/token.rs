#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Eof,
    Newline,
    ErrorToken,
    Indent,
    Dedent,
    Story,
    Concept,
    Situation,
    To,
    With,
    Adjust,
    Use,
    Create,
    Called,
    Proceed,
    If,
    Else,
    ElseIf,
    Repeat,
    While,
    For,
    Each,
    In,
    Times,
    Break,
    Continue,
    Return,
    When,
    Otherwise,
    Try,
    Catch,
    Always,
    Do,
    Background,

    Is,
    To_,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Equals,
    NotEquals,
    Greater,
    Less,
    GreaterEq,
    LessEq,

    And,
    Or,
    Not,

    Number(String),
    String_(String),
    True_,
    False_,
    Identifier(String),
    Colon,
    Comma,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Dot,

    Comment(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize, length: usize) -> Self {
        Self {
            token_type,
            line,
            column,
            length,
        }
    }
}
