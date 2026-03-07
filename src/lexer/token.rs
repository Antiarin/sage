#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // String interpolation
    InterpolationStart,
    InterpolationEnd,

    // Identifiers & Keywords
    Identifier(String),
    Fn,
    Let,
    Mut,
    Return,
    If,
    Else,
    Match,
    For,
    While,
    In,
    Spawn,
    Parallel,
    Scope,
    Struct,
    Trait,
    Impl,
    Test,
    Import,
    Try,
    Catch,
    Agent,
    Module,
    True,
    False,

    // Type keywords
    I32,
    I64,
    F32,
    F64,
    Str,
    Bool,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Not,
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    Arrow,
    FatArrow,
    QuestionMark,
    QuestionDot,
    DoubleQuestion,
    Dot,
    DotDot,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,

    // Special
    At,
    Pipe,
    Ampersand,
    Hash,

    // Meta
    Newline,
    EOF,
}

impl TokenKind {
    pub fn lookup_keyword(ident: &str) -> Option<TokenKind> {
        match ident {
            "fn" => Some(TokenKind::Fn),
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "return" => Some(TokenKind::Return),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "match" => Some(TokenKind::Match),
            "for" => Some(TokenKind::For),
            "while" => Some(TokenKind::While),
            "in" => Some(TokenKind::In),
            "spawn" => Some(TokenKind::Spawn),
            "parallel" => Some(TokenKind::Parallel),
            "scope" => Some(TokenKind::Scope),
            "struct" => Some(TokenKind::Struct),
            "trait" => Some(TokenKind::Trait),
            "impl" => Some(TokenKind::Impl),
            "test" => Some(TokenKind::Test),
            "import" => Some(TokenKind::Import),
            "try" => Some(TokenKind::Try),
            "catch" => Some(TokenKind::Catch),
            "agent" => Some(TokenKind::Agent),
            "module" => Some(TokenKind::Module),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "i32" => Some(TokenKind::I32),
            "i64" => Some(TokenKind::I64),
            "f32" => Some(TokenKind::F32),
            "f64" => Some(TokenKind::F64),
            "str" => Some(TokenKind::Str),
            "bool" => Some(TokenKind::Bool),
            _ => None,
        }
    }
}
