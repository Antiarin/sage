#![allow(dead_code)]

pub mod ast;

use crate::lexer::token::{Span, Token, TokenKind};
use ast::*;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error: {} at line {}, column {}",
            self.message, self.span.line, self.span.column
        )
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        self.skip_newlines();
        while !self.at(&TokenKind::EOF) {
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    break;
                }
            }
            self.skip_newlines();
        }

        stmts
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    // -- Statement parsing --

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr(0)?;
        Ok(Stmt::Expression(expr))
    }

    // -- Pratt expression parsing --

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        // Parse prefix (nud)
        let mut lhs = self.parse_prefix()?;

        // Parse infix/postfix (led)
        loop {
            let op = self.peek().clone();

            // Postfix: function call
            if op == TokenKind::LParen {
                if 15 < min_bp {
                    break;
                }
                let args = self.parse_args()?;
                lhs = Expr::FnCall {
                    callee: Box::new(lhs),
                    args,
                };
                continue;
            }

            // Postfix: index access
            if op == TokenKind::LBracket {
                if 15 < min_bp {
                    break;
                }
                self.advance();
                let index = self.parse_expr(0)?;
                self.expect(&TokenKind::RBracket)?;
                lhs = Expr::Index {
                    object: Box::new(lhs),
                    index: Box::new(index),
                };
                continue;
            }

            // Postfix: field access / method call
            if op == TokenKind::Dot {
                if 15 < min_bp {
                    break;
                }
                self.advance();
                let field = match self.peek().clone() {
                    TokenKind::Identifier(name) => {
                        self.advance();
                        name
                    }
                    _ => return Err(self.error("Expected field name after '.'".to_string())),
                };
                // If followed by '(', it's a method call
                if *self.peek() == TokenKind::LParen {
                    let args = self.parse_args()?;
                    lhs = Expr::MethodCall {
                        object: Box::new(lhs),
                        method: field,
                        args,
                    };
                } else {
                    lhs = Expr::FieldAccess {
                        object: Box::new(lhs),
                        field,
                    };
                }
                continue;
            }

            // Postfix: safe access
            if op == TokenKind::QuestionDot {
                if 15 < min_bp {
                    break;
                }
                self.advance();
                let field = match self.peek().clone() {
                    TokenKind::Identifier(name) => {
                        self.advance();
                        name
                    }
                    _ => return Err(self.error("Expected field name after '?.'".to_string())),
                };
                lhs = Expr::SafeAccess {
                    object: Box::new(lhs),
                    field,
                };
                continue;
            }

            // Infix: binary operators
            if let Some((l_bp, r_bp)) = Self::infix_binding_power(&op) {
                if l_bp < min_bp {
                    break;
                }
                self.advance();

                let rhs = self.parse_expr(r_bp)?;
                lhs = if op == TokenKind::DoubleQuestion {
                    Expr::NullCoalesce {
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }
                } else {
                    self.make_binary(lhs, &op, rhs)
                };
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut args = Vec::new();
        if !self.at(&TokenKind::RParen) {
            args.push(self.parse_expr(0)?);
            while self.at(&TokenKind::Comma) {
                self.advance();
                args.push(self.parse_expr(0)?);
            }
        }
        self.expect(&TokenKind::RParen)?;
        Ok(args)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            TokenKind::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            TokenKind::FloatLiteral(n) => {
                self.advance();
                Ok(Expr::FloatLiteral(n))
            }
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::Identifier(name))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr(0)?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::Minus => {
                self.advance();
                let bp = Self::prefix_binding_power(&TokenKind::Minus);
                let expr = self.parse_expr(bp)?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Not => {
                self.advance();
                let bp = Self::prefix_binding_power(&TokenKind::Not);
                let expr = self.parse_expr(bp)?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => Err(self.error(format!("Expected expression, found {:?}", self.peek()))),
        }
    }

    fn make_binary(&self, left: Expr, op: &TokenKind, right: Expr) -> Expr {
        let bin_op = match op {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            TokenKind::Percent => BinOp::Mod,
            TokenKind::Eq => BinOp::Eq,
            TokenKind::NotEq => BinOp::NotEq,
            TokenKind::Lt => BinOp::Lt,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::LtEq => BinOp::LtEq,
            TokenKind::GtEq => BinOp::GtEq,
            TokenKind::And => BinOp::And,
            TokenKind::Or => BinOp::Or,
            _ => unreachable!("Not a binary operator: {:?}", op),
        };
        Expr::BinaryOp {
            left: Box::new(left),
            op: bin_op,
            right: Box::new(right),
        }
    }

    // Binding power for prefix (unary) operators
    fn prefix_binding_power(op: &TokenKind) -> u8 {
        match op {
            TokenKind::Minus | TokenKind::Not => 13,
            _ => 0,
        }
    }

    // Binding power for infix (binary) operators: (left_bp, right_bp)
    // Left-associative: right_bp = left_bp + 1
    // Right-associative: right_bp = left_bp - 1
    fn infix_binding_power(op: &TokenKind) -> Option<(u8, u8)> {
        match op {
            TokenKind::DoubleQuestion => Some((2, 1)), // right-assoc
            TokenKind::Or => Some((3, 4)),
            TokenKind::And => Some((5, 6)),
            TokenKind::Eq | TokenKind::NotEq => Some((7, 8)),
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Some((7, 8)),
            TokenKind::Plus | TokenKind::Minus => Some((9, 10)),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((11, 12)),
            _ => None,
        }
    }

    // -- Token cursor methods --

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn at(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, ParseError> {
        if self.at(expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(format!("Expected {:?}, found {:?}", expected, self.peek())))
        }
    }

    fn skip_newlines(&mut self) {
        while self.at(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn error(&self, message: String) -> ParseError {
        ParseError {
            message,
            span: self.peek_token().span.clone(),
        }
    }
}
