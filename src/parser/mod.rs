pub mod ast;

use crate::lexer::token::{Span, Token, TokenKind};
use ast::*;

const MAX_DEPTH: usize = 128;

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

/// Tracks whether we're inside a `??` or `||`/`&&` chain,
/// so we can reject mixing them without explicit parentheses.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ExprContext {
    Normal,
    Coalescing,
    Logical,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    depth: usize,
    expr_context: ExprContext,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
            depth: 0,
            expr_context: ExprContext::Normal,
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
                    let before = self.pos;
                    self.synchronize();
                    // Fix #3: prevent infinite loop if synchronize didn't advance
                    if self.pos == before {
                        self.advance();
                    }
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
        let start = self.pos;

        // Collect decorators before the statement
        let decorators = if self.at(&TokenKind::At) {
            self.parse_decorators()?
        } else {
            vec![]
        };

        // Fix #4: error if decorators precede a non-function statement
        if !decorators.is_empty() && !matches!(self.peek(), TokenKind::Fn) {
            return Err(self.error("Decorators are only valid on function definitions".to_string()));
        }

        match self.peek() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Fn => self.parse_fn(decorators, start),
            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Impl => self.parse_impl(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Try => self.parse_try_catch(),
            TokenKind::Test => self.parse_test(),
            _ => {
                let expr = self.parse_expr(0)?;
                let span = expr.span.clone();
                Ok(Stmt::new(StmtKind::Expression(expr), span))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'let'

        let mutable = if self.at(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = self.expect_identifier("Expected variable name after 'let'")?;

        let type_ann = if self.at(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Assign)?;
        let value = self.parse_expr(0)?;

        Ok(Stmt::new(
            StmtKind::Let {
                name,
                mutable,
                type_ann,
                value,
            },
            self.span_from(start),
        ))
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'return'

        // return with no value: next token is newline, }, or EOF
        if matches!(
            self.peek(),
            TokenKind::Newline | TokenKind::RBrace | TokenKind::EOF
        ) {
            return Ok(Stmt::new(StmtKind::Return(None), self.span_from(start)));
        }

        let expr = self.parse_expr(0)?;
        Ok(Stmt::new(
            StmtKind::Return(Some(expr)),
            self.span_from(start),
        ))
    }

    fn parse_fn(&mut self, decorators: Vec<Decorator>, start: usize) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'fn'

        let name = self.expect_identifier("Expected function name after 'fn'")?;
        let params = self.parse_params()?;

        let return_type = if self.at(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Stmt::new(
            StmtKind::FnDef {
                name,
                params,
                return_type,
                body,
                decorators,
            },
            self.span_from(start),
        ))
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
        let name = self.expect_identifier("Expected parameter name")?;

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
    // Note (issue #8): parse_type_suffix greedily consumes `<` as generic
    // opening. This works because types and expressions are parsed in separate
    // contexts today. If we ever add type ascription in expressions (e.g.
    // `x as List<i32>`), we'll need lookahead to disambiguate `<` as generic
    // vs comparison.

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
        // Fix #2: depth limit to prevent stack overflow on deeply nested input
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(self.error("Expression nesting depth exceeded".to_string()));
        }

        // Fix #7: save/restore coalescing context across fresh expression boundaries
        let saved_context = self.expr_context;
        if min_bp == 0 {
            self.expr_context = ExprContext::Normal;
        }

        let result = self.parse_expr_inner(min_bp);

        self.expr_context = saved_context;
        self.depth -= 1;
        result
    }

    fn parse_expr_inner(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let start = self.pos;

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
                lhs = Expr::new(
                    ExprKind::FnCall {
                        callee: Box::new(lhs),
                        args,
                    },
                    self.span_from(start),
                );
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
                lhs = Expr::new(
                    ExprKind::Index {
                        object: Box::new(lhs),
                        index: Box::new(index),
                    },
                    self.span_from(start),
                );
                continue;
            }

            // Postfix: field access / method call
            if op == TokenKind::Dot {
                if 17 < min_bp {
                    break;
                }
                self.advance();
                let field = self.expect_identifier("Expected field name after '.'")?;
                // If followed by '(', it's a method call
                if *self.peek() == TokenKind::LParen {
                    let args = self.parse_args()?;
                    lhs = Expr::new(
                        ExprKind::MethodCall {
                            object: Box::new(lhs),
                            method: field,
                            args,
                        },
                        self.span_from(start),
                    );
                } else {
                    lhs = Expr::new(
                        ExprKind::FieldAccess {
                            object: Box::new(lhs),
                            field,
                        },
                        self.span_from(start),
                    );
                }
                continue;
            }

            // Postfix: safe access
            if op == TokenKind::QuestionDot {
                if 17 < min_bp {
                    break;
                }
                self.advance();
                let field = self.expect_identifier("Expected field name after '?.'")?;
                lhs = Expr::new(
                    ExprKind::SafeAccess {
                        object: Box::new(lhs),
                        field,
                    },
                    self.span_from(start),
                );
                continue;
            }

            // Infix: binary operators
            if let Some((l_bp, r_bp)) = Self::infix_binding_power(&op) {
                if l_bp < min_bp {
                    break;
                }

                // Fix #7: reject mixing ?? with || / &&
                if op == TokenKind::DoubleQuestion {
                    if self.expr_context == ExprContext::Logical {
                        return Err(self
                            .error("Cannot mix ?? with || or && without parentheses".to_string()));
                    }
                    self.expr_context = ExprContext::Coalescing;
                }
                if matches!(op, TokenKind::And | TokenKind::Or) {
                    if self.expr_context == ExprContext::Coalescing {
                        return Err(self
                            .error("Cannot mix ?? with || or && without parentheses".to_string()));
                    }
                    self.expr_context = ExprContext::Logical;
                }

                self.advance();

                let rhs = self.parse_expr(r_bp)?;
                lhs = if op == TokenKind::DoubleQuestion {
                    Expr::new(
                        ExprKind::NullCoalesce {
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        },
                        self.span_from(start),
                    )
                } else {
                    Expr::new(self.make_binary(lhs, &op, rhs), self.span_from(start))
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
        let start = self.pos;
        match self.peek().clone() {
            TokenKind::IntLiteral(n) => {
                self.advance();
                Ok(Expr::new(ExprKind::IntLiteral(n), self.span_from(start)))
            }
            TokenKind::FloatLiteral(n) => {
                self.advance();
                Ok(Expr::new(ExprKind::FloatLiteral(n), self.span_from(start)))
            }
            TokenKind::StringLiteral(s) => {
                self.advance();
                // Check if this is the start of string interpolation
                if self.at(&TokenKind::InterpolationStart) {
                    self.parse_string_interpolation(s, start)
                } else {
                    Ok(Expr::new(ExprKind::StringLiteral(s), self.span_from(start)))
                }
            }
            TokenKind::InterpolationStart => {
                // Interpolation at start of string (empty prefix)
                self.parse_string_interpolation(String::new(), start)
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::new(
                    ExprKind::BoolLiteral(true),
                    self.span_from(start),
                ))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::new(
                    ExprKind::BoolLiteral(false),
                    self.span_from(start),
                ))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::new(ExprKind::Identifier(name), self.span_from(start)))
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
                Ok(Expr::new(
                    ExprKind::UnaryOp {
                        op: UnOp::Neg,
                        expr: Box::new(expr),
                    },
                    self.span_from(start),
                ))
            }
            TokenKind::Not => {
                self.advance();
                let bp = Self::prefix_binding_power(&TokenKind::Not);
                let expr = self.parse_expr(bp)?;
                Ok(Expr::new(
                    ExprKind::UnaryOp {
                        op: UnOp::Not,
                        expr: Box::new(expr),
                    },
                    self.span_from(start),
                ))
            }
            TokenKind::If => self.parse_if(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Pipe => self.parse_closure(),
            TokenKind::LBracket => self.parse_list_literal(),
            TokenKind::Spawn => self.parse_spawn(),
            TokenKind::Parallel => self.parse_parallel(),
            _ => Err(self.error(format!("Expected expression, found {:?}", self.peek()))),
        }
    }

    // -- Control flow --

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'if'

        let condition = self.parse_expr(0)?;
        let then_block = self.parse_block()?;

        let else_block = if self.at(&TokenKind::Else) {
            self.advance();
            if self.at(&TokenKind::If) {
                // else if -> the else block is a single expression statement containing another if
                let nested_if = self.parse_if()?;
                let span = nested_if.span.clone();
                Some(vec![Stmt::new(StmtKind::Expression(nested_if), span)])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Expr::new(
            ExprKind::IfElse {
                condition: Box::new(condition),
                then_block,
                else_block,
            },
            self.span_from(start),
        ))
    }

    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
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
        Ok(Expr::new(
            ExprKind::Match {
                expr: Box::new(matched),
                arms,
            },
            self.span_from(start),
        ))
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'for'

        let var = self.expect_identifier("Expected variable name after 'for'")?;

        self.expect(&TokenKind::In)?;
        let iter = self.parse_expr(0)?;
        let body = self.parse_block()?;

        Ok(Stmt::new(
            StmtKind::ForLoop { var, iter, body },
            self.span_from(start),
        ))
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'while'

        let condition = self.parse_expr(0)?;
        let body = self.parse_block()?;

        Ok(Stmt::new(
            StmtKind::WhileLoop { condition, body },
            self.span_from(start),
        ))
    }

    // -- Decorators, imports, try/catch, test --

    fn parse_decorators(&mut self) -> Result<Vec<Decorator>, ParseError> {
        let mut decorators = Vec::new();

        while self.at(&TokenKind::At) {
            self.advance(); // consume '@'
            let name = self.expect_identifier("Expected decorator name after '@'")?;

            let args = if self.at(&TokenKind::LParen) {
                self.advance();
                let mut args = Vec::new();
                if !self.at(&TokenKind::RParen) {
                    loop {
                        let key = self.expect_identifier("Expected argument name in decorator")?;
                        self.expect(&TokenKind::Colon)?;
                        let value = self.parse_expr(0)?;
                        args.push((key, value));
                        if !self.at(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RParen)?;
                args
            } else {
                vec![]
            };

            decorators.push(Decorator { name, args });
            self.skip_newlines();
        }

        Ok(decorators)
    }

    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'import'

        let mut path = Vec::new();
        let first = self.expect_identifier("Expected module path after 'import'")?;
        path.push(first);

        while self.at(&TokenKind::Dot) {
            self.advance();
            let segment = self.expect_identifier("Expected identifier in import path")?;
            path.push(segment);
        }

        Ok(Stmt::new(StmtKind::Import { path }, self.span_from(start)))
    }

    fn parse_try_catch(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'try'
        let try_block = self.parse_block()?;

        self.expect(&TokenKind::Catch)?;
        let catch_var = self.expect_identifier("Expected variable name after 'catch'")?;
        let catch_block = self.parse_block()?;

        Ok(Stmt::new(
            StmtKind::TryCatch {
                try_block,
                catch_var,
                catch_block,
            },
            self.span_from(start),
        ))
    }

    fn parse_test(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'test'

        let name = match self.peek().clone() {
            TokenKind::StringLiteral(s) => {
                self.advance();
                s
            }
            _ => return Err(self.error("Expected test name string after 'test'".to_string())),
        };

        let body = self.parse_block()?;
        Ok(Stmt::new(
            StmtKind::TestFn { name, body },
            self.span_from(start),
        ))
    }

    // -- Closures, lists, string interpolation, concurrency --

    fn parse_closure(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        self.advance(); // consume '|'

        let mut params = Vec::new();
        if !self.at(&TokenKind::Pipe) {
            loop {
                let name = self.expect_identifier("Expected parameter name in closure")?;
                let type_ann = if self.at(&TokenKind::Colon) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                params.push(Param { name, type_ann });
                if !self.at(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect(&TokenKind::Pipe)?;

        let body = if self.at(&TokenKind::LBrace) {
            self.parse_block()?
        } else {
            let expr = self.parse_expr(0)?;
            let span = expr.span.clone();
            vec![Stmt::new(StmtKind::Expression(expr), span)]
        };

        Ok(Expr::new(
            ExprKind::Closure { params, body },
            self.span_from(start),
        ))
    }

    fn parse_list_literal(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        self.advance(); // consume '['

        let mut elements = Vec::new();
        if !self.at(&TokenKind::RBracket) {
            elements.push(self.parse_expr(0)?);
            while self.at(&TokenKind::Comma) {
                self.advance();
                if self.at(&TokenKind::RBracket) {
                    break; // trailing comma
                }
                elements.push(self.parse_expr(0)?);
            }
        }

        self.expect(&TokenKind::RBracket)?;
        Ok(Expr::new(
            ExprKind::ListLiteral(elements),
            self.span_from(start),
        ))
    }

    fn parse_string_interpolation(
        &mut self,
        first_literal: String,
        start: usize,
    ) -> Result<Expr, ParseError> {
        let mut parts = Vec::new();

        if !first_literal.is_empty() {
            parts.push(StringPart::Literal(first_literal));
        }

        while self.at(&TokenKind::InterpolationStart) {
            self.advance(); // consume InterpolationStart
            let expr = self.parse_expr(0)?;
            self.expect(&TokenKind::InterpolationEnd)?;
            parts.push(StringPart::Expr(expr));

            // Check for trailing string literal
            if let TokenKind::StringLiteral(s) = self.peek().clone() {
                self.advance();
                parts.push(StringPart::Literal(s));
            }
        }

        Ok(Expr::new(
            ExprKind::StringInterpolation { parts },
            self.span_from(start),
        ))
    }

    fn parse_spawn(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'spawn'
        let expr = self.parse_expr(0)?;
        Ok(Expr::new(
            ExprKind::Spawn(Box::new(expr)),
            self.span_from(start),
        ))
    }

    fn parse_parallel(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'parallel'
        let collection = self.parse_expr(0)?;
        let closure = self.parse_closure()?;

        // Fix #5: validate closure has at least one parameter
        let param = match &closure.kind {
            ExprKind::Closure { params, .. } if !params.is_empty() => params[0].name.clone(),
            ExprKind::Closure { .. } => {
                return Err(self
                    .error("Parallel requires a closure with at least one parameter".to_string()));
            }
            _ => {
                return Err(self.error("Expected closure after parallel collection".to_string()));
            }
        };

        Ok(Expr::new(
            ExprKind::Parallel {
                collection: Box::new(collection),
                param,
                body: Box::new(closure),
            },
            self.span_from(start),
        ))
    }

    // -- Type definitions --

    fn parse_struct(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'struct'

        let name = self.expect_identifier("Expected struct name")?;

        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            let field_name = self.expect_identifier("Expected field name")?;
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
        Ok(Stmt::new(
            StmtKind::StructDef { name, fields },
            self.span_from(start),
        ))
    }

    fn parse_trait(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'trait'

        let name = self.expect_identifier("Expected trait name")?;

        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            self.expect(&TokenKind::Fn)?;
            let method_name = self.expect_identifier("Expected method name")?;

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
        Ok(Stmt::new(
            StmtKind::TraitDef { name, methods },
            self.span_from(start),
        ))
    }

    fn parse_impl(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.advance(); // consume 'impl'

        let first_name = self.expect_identifier("Expected type name after 'impl'")?;

        // Check for "impl Trait for Type"
        let (trait_name, target) = if self.at(&TokenKind::For) {
            self.advance();
            let target = self.expect_identifier("Expected type name after 'for'")?;
            (Some(first_name), target)
        } else {
            (None, first_name)
        };

        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::EOF) {
            match self.peek() {
                TokenKind::Fn => {
                    let fn_start = self.pos;
                    methods.push(self.parse_fn(vec![], fn_start)?);
                }
                _ => return Err(self.error("Expected 'fn' in impl block".to_string())),
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(Stmt::new(
            StmtKind::ImplBlock {
                trait_name,
                target,
                methods,
            },
            self.span_from(start),
        ))
    }

    fn make_binary(&self, left: Expr, op: &TokenKind, right: Expr) -> ExprKind {
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
        ExprKind::BinaryOp {
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

    // -- Error recovery --

    fn synchronize(&mut self) {
        // Skip tokens until we find a statement boundary
        loop {
            match self.peek() {
                TokenKind::EOF => break,
                // Statement-starting keywords
                TokenKind::Let
                | TokenKind::Fn
                | TokenKind::Struct
                | TokenKind::Trait
                | TokenKind::Impl
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Return
                | TokenKind::Import
                | TokenKind::Try
                | TokenKind::Test
                | TokenKind::At => break,
                TokenKind::Newline => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
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

    /// Fix #6: centralized identifier extraction to reduce clone boilerplate.
    /// Still clones the string from the token -- a string interner would
    /// eliminate these allocations but isn't warranted yet.
    fn expect_identifier(&mut self, msg: &str) -> Result<String, ParseError> {
        match self.peek().clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            _ => Err(self.error(msg.to_string())),
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

    fn span_from(&self, start: usize) -> Span {
        let start_span = &self.tokens[start].span;
        let end_idx = if self.pos > 0 { self.pos - 1 } else { 0 };
        let end_span = &self.tokens[end_idx].span;
        Span {
            start: start_span.start,
            end: end_span.end,
            line: start_span.line,
            column: start_span.column,
        }
    }
}
