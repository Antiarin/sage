pub mod token;

use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::CharIndices;
use token::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {} at line {}, column {}",
            self.message, self.line, self.column
        )
    }
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    column: usize,
    errors: Vec<LexerError>,
    pending_tokens: VecDeque<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            chars: source.char_indices().peekable(),
            line: 1,
            column: 1,
            errors: Vec::new(),
            pending_tokens: VecDeque::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token() {
            tokens.push(token);
        }

        tokens.push(Token {
            kind: TokenKind::EOF,
            span: Span {
                start: self.source.len(),
                end: self.source.len(),
                line: self.line,
                column: self.column,
            },
        });

        tokens
    }

    pub fn errors(&self) -> &[LexerError] {
        &self.errors
    }

    fn next_token(&mut self) -> Option<Token> {
        // Return any pending tokens from string interpolation first
        if let Some(token) = self.pending_tokens.pop_front() {
            return Some(token);
        }

        self.skip_whitespace();

        let (start, ch) = *self.chars.peek()?;
        let line = self.line;
        let column = self.column;

        match ch {
            '\n' => {
                self.advance();
                Some(self.make_token(TokenKind::Newline, start, start + 1, line, column))
            }

            '(' => self.single_char_token(TokenKind::LParen, start, line, column),
            ')' => self.single_char_token(TokenKind::RParen, start, line, column),
            '{' => self.single_char_token(TokenKind::LBrace, start, line, column),
            '}' => self.single_char_token(TokenKind::RBrace, start, line, column),
            '[' => self.single_char_token(TokenKind::LBracket, start, line, column),
            ']' => self.single_char_token(TokenKind::RBracket, start, line, column),
            ',' => self.single_char_token(TokenKind::Comma, start, line, column),
            ':' => self.single_char_token(TokenKind::Colon, start, line, column),
            ';' => self.single_char_token(TokenKind::Semicolon, start, line, column),
            '@' => self.single_char_token(TokenKind::At, start, line, column),
            '#' => self.single_char_token(TokenKind::Hash, start, line, column),

            '+' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Some(self.make_token(TokenKind::PlusAssign, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Plus, start, start + 1, line, column))
                }
            }

            '-' => {
                self.advance();
                match self.peek_char() {
                    Some('=') => {
                        self.advance();
                        Some(self.make_token(
                            TokenKind::MinusAssign,
                            start,
                            start + 2,
                            line,
                            column,
                        ))
                    }
                    Some('>') => {
                        self.advance();
                        Some(self.make_token(TokenKind::Arrow, start, start + 2, line, column))
                    }
                    _ => Some(self.make_token(TokenKind::Minus, start, start + 1, line, column)),
                }
            }

            '*' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Some(self.make_token(TokenKind::StarAssign, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Star, start, start + 1, line, column))
                }
            }

            '/' => {
                self.advance();
                match self.peek_char() {
                    Some('/') => {
                        self.skip_line_comment();
                        self.next_token()
                    }
                    Some('*') => {
                        self.skip_block_comment(line, column);
                        self.next_token()
                    }
                    Some('=') => {
                        self.advance();
                        Some(self.make_token(
                            TokenKind::SlashAssign,
                            start,
                            start + 2,
                            line,
                            column,
                        ))
                    }
                    _ => Some(self.make_token(TokenKind::Slash, start, start + 1, line, column)),
                }
            }

            '%' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Some(self.make_token(TokenKind::PercentAssign, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Percent, start, start + 1, line, column))
                }
            }

            '=' => {
                self.advance();
                match self.peek_char() {
                    Some('=') => {
                        self.advance();
                        Some(self.make_token(TokenKind::Eq, start, start + 2, line, column))
                    }
                    Some('>') => {
                        self.advance();
                        Some(self.make_token(TokenKind::FatArrow, start, start + 2, line, column))
                    }
                    _ => Some(self.make_token(TokenKind::Assign, start, start + 1, line, column)),
                }
            }

            '!' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Some(self.make_token(TokenKind::NotEq, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Not, start, start + 1, line, column))
                }
            }

            '<' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Some(self.make_token(TokenKind::LtEq, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Lt, start, start + 1, line, column))
                }
            }

            '>' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Some(self.make_token(TokenKind::GtEq, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Gt, start, start + 1, line, column))
                }
            }

            '&' => {
                self.advance();
                if self.peek_char() == Some('&') {
                    self.advance();
                    Some(self.make_token(TokenKind::And, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Ampersand, start, start + 1, line, column))
                }
            }

            '|' => {
                self.advance();
                if self.peek_char() == Some('|') {
                    self.advance();
                    Some(self.make_token(TokenKind::Or, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Pipe, start, start + 1, line, column))
                }
            }

            '?' => {
                self.advance();
                match self.peek_char() {
                    Some('.') => {
                        self.advance();
                        Some(self.make_token(
                            TokenKind::QuestionDot,
                            start,
                            start + 2,
                            line,
                            column,
                        ))
                    }
                    Some('?') => {
                        self.advance();
                        Some(self.make_token(
                            TokenKind::DoubleQuestion,
                            start,
                            start + 2,
                            line,
                            column,
                        ))
                    }
                    _ => Some(self.make_token(
                        TokenKind::QuestionMark,
                        start,
                        start + 1,
                        line,
                        column,
                    )),
                }
            }

            '.' => {
                self.advance();
                if self.peek_char() == Some('.') {
                    self.advance();
                    Some(self.make_token(TokenKind::DotDot, start, start + 2, line, column))
                } else {
                    Some(self.make_token(TokenKind::Dot, start, start + 1, line, column))
                }
            }

            '"' => self.lex_string(start, line, column),

            c if c.is_ascii_digit() => self.lex_number(start, line, column),

            c if c.is_ascii_alphabetic() || c == '_' => self.lex_identifier(start, line, column),

            _ => {
                self.advance();
                self.errors.push(LexerError {
                    message: format!("Unexpected character '{}'", ch),
                    line,
                    column,
                });
                self.next_token()
            }
        }
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((_, ch)) = result {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        result
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, ch)| *ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self, start_line: usize, start_column: usize) {
        self.advance(); // consume the '*' after '/'
        let mut depth = 1;

        while depth > 0 {
            match self.advance() {
                Some((_, '/')) => {
                    if self.peek_char() == Some('*') {
                        self.advance();
                        depth += 1;
                    }
                }
                Some((_, '*')) => {
                    if self.peek_char() == Some('/') {
                        self.advance();
                        depth -= 1;
                    }
                }
                None => {
                    self.errors.push(LexerError {
                        message: "Unterminated block comment".to_string(),
                        line: start_line,
                        column: start_column,
                    });
                    break;
                }
                _ => {}
            }
        }
    }

    fn single_char_token(
        &mut self,
        kind: TokenKind,
        start: usize,
        line: usize,
        column: usize,
    ) -> Option<Token> {
        let (_, ch) = self.advance().unwrap();
        Some(self.make_token(kind, start, start + ch.len_utf8(), line, column))
    }

    fn make_token(
        &self,
        kind: TokenKind,
        start: usize,
        end: usize,
        line: usize,
        column: usize,
    ) -> Token {
        Token {
            kind,
            span: Span {
                start,
                end,
                line,
                column,
            },
        }
    }

    fn lex_number(&mut self, start: usize, line: usize, column: usize) -> Option<Token> {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        // Check for float
        if self.peek_char() == Some('.') {
            // Look ahead past the dot - if the next char is also a dot, it's a range (..)
            let source_bytes = self.source.as_bytes();
            let current_pos = self
                .chars
                .peek()
                .map(|(i, _)| *i)
                .unwrap_or(self.source.len());
            let after_dot = current_pos + 1;

            if after_dot < source_bytes.len() && source_bytes[after_dot] == b'.' {
                // It's a range like 3..10, don't consume the dots
                let text = &self.source[start..current_pos];
                let value: i64 = match text.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.errors.push(LexerError {
                            message: format!("Integer literal '{}' is too large", text),
                            line,
                            column,
                        });
                        0
                    }
                };
                return Some(self.make_token(
                    TokenKind::IntLiteral(value),
                    start,
                    current_pos,
                    line,
                    column,
                ));
            }

            if after_dot < source_bytes.len() && source_bytes[after_dot].is_ascii_digit() {
                self.advance(); // consume the '.'
                while let Some(&(_, ch)) = self.chars.peek() {
                    if ch.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }

                let end = self
                    .chars
                    .peek()
                    .map(|(i, _)| *i)
                    .unwrap_or(self.source.len());
                let text = &self.source[start..end];
                let value: f64 = match text.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.errors.push(LexerError {
                            message: format!("Invalid float literal '{}'", text),
                            line,
                            column,
                        });
                        0.0
                    }
                };
                return Some(self.make_token(
                    TokenKind::FloatLiteral(value),
                    start,
                    end,
                    line,
                    column,
                ));
            }
        }

        let end = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        let text = &self.source[start..end];
        let value: i64 = match text.parse() {
            Ok(v) => v,
            Err(_) => {
                self.errors.push(LexerError {
                    message: format!("Integer literal '{}' is too large", text),
                    line,
                    column,
                });
                0
            }
        };
        Some(self.make_token(TokenKind::IntLiteral(value), start, end, line, column))
    }

    fn lex_string(&mut self, start: usize, line: usize, column: usize) -> Option<Token> {
        self.advance(); // consume opening '"'

        let mut result: Vec<Token> = Vec::new();
        let mut current_string = String::new();
        let mut string_segment_start = start + 1;
        let mut has_interpolation = false;

        loop {
            match self.chars.peek() {
                None => {
                    self.errors.push(LexerError {
                        message: "Unterminated string literal".to_string(),
                        line,
                        column,
                    });
                    break;
                }
                Some(&(_, '"')) => {
                    self.advance();
                    break;
                }
                Some(&(pos, '{')) => {
                    has_interpolation = true;

                    // Push the string part before the interpolation
                    if !current_string.is_empty() {
                        result.push(self.make_token(
                            TokenKind::StringLiteral(current_string.clone()),
                            string_segment_start,
                            pos,
                            line,
                            column,
                        ));
                        current_string.clear();
                    }

                    self.advance(); // consume '{'
                    result.push(self.make_token(
                        TokenKind::InterpolationStart,
                        pos,
                        pos + 1,
                        self.line,
                        self.column,
                    ));

                    // Lex tokens inside the interpolation until we hit '}'
                    let mut brace_depth = 1;
                    while brace_depth > 0 {
                        self.skip_whitespace();
                        match self.chars.peek() {
                            None => {
                                self.errors.push(LexerError {
                                    message: "Unterminated string interpolation".to_string(),
                                    line,
                                    column,
                                });
                                return result.into_iter().next().or(Some(self.make_token(
                                    TokenKind::EOF,
                                    start,
                                    start,
                                    line,
                                    column,
                                )));
                            }
                            Some(&(end_pos, '}')) => {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    self.advance();
                                    result.push(self.make_token(
                                        TokenKind::InterpolationEnd,
                                        end_pos,
                                        end_pos + 1,
                                        self.line,
                                        self.column,
                                    ));
                                    // Track where the next string segment starts
                                    string_segment_start = end_pos + 1;
                                } else if let Some(token) = self.next_token() {
                                    result.push(token);
                                }
                            }
                            Some(&(_, '{')) => {
                                brace_depth += 1;
                                if let Some(token) = self.next_token() {
                                    result.push(token);
                                }
                            }
                            _ => {
                                if let Some(token) = self.next_token() {
                                    result.push(token);
                                }
                            }
                        }
                    }
                }
                Some(&(_, '\\')) => {
                    self.advance();
                    match self.chars.peek() {
                        Some(&(_, 'n')) => {
                            self.advance();
                            current_string.push('\n');
                        }
                        Some(&(_, 't')) => {
                            self.advance();
                            current_string.push('\t');
                        }
                        Some(&(_, '\\')) => {
                            self.advance();
                            current_string.push('\\');
                        }
                        Some(&(_, '"')) => {
                            self.advance();
                            current_string.push('"');
                        }
                        Some(&(_, '{')) => {
                            self.advance();
                            current_string.push('{');
                        }
                        Some(&(_, c)) => {
                            self.advance();
                            self.errors.push(LexerError {
                                message: format!("Unknown escape sequence '\\{}'", c),
                                line: self.line,
                                column: self.column - 1,
                            });
                            current_string.push(c);
                        }
                        None => {
                            self.errors.push(LexerError {
                                message: "Unterminated escape sequence".to_string(),
                                line: self.line,
                                column: self.column,
                            });
                        }
                    }
                }
                Some(&(_, ch)) => {
                    self.advance();
                    current_string.push(ch);
                }
            }
        }

        if has_interpolation {
            if !current_string.is_empty() {
                let end = self
                    .chars
                    .peek()
                    .map(|(i, _)| *i)
                    .unwrap_or(self.source.len());
                result.push(self.make_token(
                    TokenKind::StringLiteral(current_string),
                    string_segment_start,
                    end,
                    self.line,
                    self.column,
                ));
            }

            if result.len() == 1 {
                return result.into_iter().next();
            }

            let mut iter = result.into_iter();
            let first = iter.next();
            self.pending_tokens = iter.collect::<VecDeque<_>>();
            return first;
        }

        let end = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        Some(self.make_token(
            TokenKind::StringLiteral(current_string),
            start,
            end,
            line,
            column,
        ))
    }

    fn lex_identifier(&mut self, start: usize, line: usize, column: usize) -> Option<Token> {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let end = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        let text = &self.source[start..end];

        let kind = TokenKind::lookup_keyword(text)
            .unwrap_or_else(|| TokenKind::Identifier(text.to_string()));

        Some(self.make_token(kind, start, end, line, column))
    }
}
