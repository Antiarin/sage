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
        match self.peek() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Fn => self.parse_fn(vec![]),
            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Impl => self.parse_impl(),
            _ => {
                let expr = self.parse_expr(0)?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'let'

        let mutable = if self.at(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected variable name after 'let'".to_string())),
        };

        let type_ann = if self.at(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Assign)?;
        let value = self.parse_expr(0)?;

        Ok(Stmt::Let {
            name,
            mutable,
            type_ann,
            value,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'return'

        // return with no value: next token is newline, }, or EOF
        if matches!(
            self.peek(),
            TokenKind::Newline | TokenKind::RBrace | TokenKind::EOF
        ) {
            return Ok(Stmt::Return(None));
        }

        let expr = self.parse_expr(0)?;
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_fn(&mut self, decorators: Vec<Decorator>) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'fn'

        let name = match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected function name after 'fn'".to_string())),
        };

        let params = self.parse_params()?;

        let return_type = if self.at(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Stmt::FnDef {
            name,
            params,
            return_type,
            body,
            decorators,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();

        if !self.at(&TokenKind::RParen) {
            params.push(self.parse_param()?);
            while self.at(&TokenKind::Comma) {
                self.advance();
                params.push(self.parse_param()?);
            }
        }

        self.expect(&TokenKind::RParen)?;
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected parameter name".to_string())),
        };

        let type_ann = if self.at(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(Param { name, type_ann })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();

        self.skip_newlines();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(stmts)
    }

    // -- Type parsing --

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let ty = match self.peek().clone() {
            TokenKind::Ampersand => {
                self.advance();
                let mutable = if self.at(&TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let inner = self.parse_type()?;
                Type::Reference {
                    mutable,
                    inner: Box::new(inner),
                }
            }
            TokenKind::Identifier(name) => {
                self.advance();
                self.parse_type_suffix(name)?
            }
            // Type keywords: i32, i64, f32, f64, str, bool
            TokenKind::I32 => {
                self.advance();
                self.parse_type_suffix("i32".to_string())?
            }
            TokenKind::I64 => {
                self.advance();
                self.parse_type_suffix("i64".to_string())?
            }
            TokenKind::F32 => {
                self.advance();
                self.parse_type_suffix("f32".to_string())?
            }
            TokenKind::F64 => {
                self.advance();
                self.parse_type_suffix("f64".to_string())?
            }
            TokenKind::Str => {
                self.advance();
                self.parse_type_suffix("str".to_string())?
            }
            TokenKind::Bool => {
                self.advance();
                self.parse_type_suffix("bool".to_string())?
            }
            _ => return Err(self.error(format!("Expected type, found {:?}", self.peek()))),
        };

        Ok(ty)
    }

    fn parse_type_suffix(&mut self, name: String) -> Result<Type, ParseError> {
        // Check for generic params: Name<T, U>
        if self.at(&TokenKind::Lt) {
            self.advance();
            let mut params = vec![self.parse_type()?];
            while self.at(&TokenKind::Comma) {
                self.advance();
                params.push(self.parse_type()?);
            }
            self.expect(&TokenKind::Gt)?;
            let ty = Type::Generic { name, params };
            // Check for ? after generic
            if self.at(&TokenKind::QuestionMark) {
                self.advance();
                return Ok(Type::Nullable(Box::new(ty)));
            }
            return Ok(ty);
        }

        // Check for nullable: Name?
        if self.at(&TokenKind::QuestionMark) {
            self.advance();
            return Ok(Type::Nullable(Box::new(Type::Simple(name))));
        }

        Ok(Type::Simple(name))
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
                if 17 < min_bp {
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
                if 17 < min_bp {
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
                if 17 < min_bp {
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
                if 17 < min_bp {
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
            TokenKind::If => self.parse_if(),
            TokenKind::Match => self.parse_match(),
            _ => Err(self.error(format!("Expected expression, found {:?}", self.peek()))),
        }
    }

    // -- Control flow --

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // consume 'if'

        let condition = self.parse_expr(0)?;
        let then_block = self.parse_block()?;

        let else_block = if self.at(&TokenKind::Else) {
            self.advance();
            if self.at(&TokenKind::If) {
                // else if -> the else block is a single expression statement containing another if
                let nested_if = self.parse_if()?;
                Some(vec![Stmt::Expression(nested_if)])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Expr::IfElse {
            condition: Box::new(condition),
            then_block,
            else_block,
        })
    }

    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // consume 'match'

        let matched = self.parse_expr(0)?;
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            let pattern = self.parse_expr(0)?;
            self.expect(&TokenKind::FatArrow)?;
            let body = self.parse_expr(0)?;
            arms.push(MatchArm { pattern, body });

            // Arms separated by commas or newlines
            if self.at(&TokenKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Match {
            expr: Box::new(matched),
            arms,
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'for'

        let var = match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected variable name after 'for'".to_string())),
        };

        self.expect(&TokenKind::In)?;
        let iter = self.parse_expr(0)?;
        let body = self.parse_block()?;

        Ok(Stmt::ForLoop { var, iter, body })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'while'

        let condition = self.parse_expr(0)?;
        let body = self.parse_block()?;

        Ok(Stmt::WhileLoop { condition, body })
    }

    // -- Type definitions --

    fn parse_struct(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'struct'

        let name = match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected struct name".to_string())),
        };

        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            let field_name = match self.peek().clone() {
                TokenKind::Identifier(name) => {
                    self.advance();
                    name
                }
                _ => return Err(self.error("Expected field name".to_string())),
            };
            self.expect(&TokenKind::Colon)?;
            let type_ann = self.parse_type()?;
            fields.push(Field {
                name: field_name,
                type_ann,
            });

            if self.at(&TokenKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(Stmt::StructDef { name, fields })
    }

    fn parse_trait(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'trait'

        let name = match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected trait name".to_string())),
        };

        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            self.expect(&TokenKind::Fn)?;
            let method_name = match self.peek().clone() {
                TokenKind::Identifier(name) => {
                    self.advance();
                    name
                }
                _ => return Err(self.error("Expected method name".to_string())),
            };

            let params = self.parse_params()?;

            let return_type = if self.at(&TokenKind::Arrow) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };

            methods.push(FnSignature {
                name: method_name,
                params,
                return_type,
            });
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(Stmt::TraitDef { name, methods })
    }

    fn parse_impl(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'impl'

        let first_name = match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected type name after 'impl'".to_string())),
        };

        // Check for "impl Trait for Type"
        let (trait_name, target) = if self.at(&TokenKind::For) {
            self.advance();
            let target = match self.peek().clone() {
                TokenKind::Identifier(name) => {
                    self.advance();
                    name
                }
                _ => return Err(self.error("Expected type name after 'for'".to_string())),
            };
            (Some(first_name), target)
        } else {
            (None, first_name)
        };

        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            match self.peek() {
                TokenKind::Fn => methods.push(self.parse_fn(vec![])?),
                _ => return Err(self.error("Expected 'fn' in impl block".to_string())),
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(Stmt::ImplBlock {
            trait_name,
            target,
            methods,
        })
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
            TokenKind::DotDot => BinOp::Range,
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
            TokenKind::Minus | TokenKind::Not => 15,
            _ => 0,
        }
    }

    // Binding power for infix (binary) operators: (left_bp, right_bp)
    // Left-associative: right_bp = left_bp + 1
    // Right-associative: right_bp = left_bp - 1
    fn infix_binding_power(op: &TokenKind) -> Option<(u8, u8)> {
        match op {
            TokenKind::DotDot => Some((1, 2)),
            TokenKind::DoubleQuestion => Some((4, 3)), // right-assoc
            TokenKind::Or => Some((5, 6)),
            TokenKind::And => Some((7, 8)),
            TokenKind::Eq | TokenKind::NotEq => Some((9, 10)),
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Some((9, 10)),
            TokenKind::Plus | TokenKind::Minus => Some((11, 12)),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((13, 14)),
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
