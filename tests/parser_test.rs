use sage::lexer::Lexer;
use sage::parser::Parser;
use sage::parser::ast::*;

fn parse(source: &str) -> (Vec<Stmt>, Vec<sage::parser::ParseError>) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    assert!(
        lexer.errors().is_empty(),
        "Lexer errors: {:?}",
        lexer.errors()
    );
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    let errors = parser.errors().to_vec();
    (stmts, errors)
}

fn parse_ok(source: &str) -> Vec<Stmt> {
    let (stmts, errors) = parse(source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    stmts
}

fn parse_single_expr(source: &str) -> Expr {
    let stmts = parse_ok(source);
    assert_eq!(stmts.len(), 1, "Expected 1 statement, got {}", stmts.len());
    match stmts.into_iter().next().unwrap() {
        Stmt::Expression(expr) => expr,
        other => panic!("Expected expression statement, got {:?}", other),
    }
}

#[test]
fn parser_constructs_without_panic() {
    let _ = parse_ok("");
}

#[test]
fn parser_handles_empty_source() {
    let stmts = parse_ok("");
    assert!(stmts.is_empty());
}

// -- Task 2: Literals, binary ops, unary ops, grouping --

#[test]
fn parse_integer_literal() {
    let expr = parse_single_expr("42");
    assert_eq!(expr, Expr::IntLiteral(42));
}

#[test]
fn parse_float_literal() {
    let expr = parse_single_expr("3.14");
    assert_eq!(expr, Expr::FloatLiteral(3.14));
}

#[test]
fn parse_string_literal() {
    let expr = parse_single_expr("\"hello\"");
    assert_eq!(expr, Expr::StringLiteral("hello".to_string()));
}

#[test]
fn parse_bool_true() {
    let expr = parse_single_expr("true");
    assert_eq!(expr, Expr::BoolLiteral(true));
}

#[test]
fn parse_bool_false() {
    let expr = parse_single_expr("false");
    assert_eq!(expr, Expr::BoolLiteral(false));
}

#[test]
fn parse_identifier() {
    let expr = parse_single_expr("foo");
    assert_eq!(expr, Expr::Identifier("foo".to_string()));
}

#[test]
fn parse_binary_add() {
    let expr = parse_single_expr("1 + 2");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::IntLiteral(1)),
            op: BinOp::Add,
            right: Box::new(Expr::IntLiteral(2)),
        }
    );
}

#[test]
fn parse_precedence_mul_over_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let expr = parse_single_expr("1 + 2 * 3");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::IntLiteral(1)),
            op: BinOp::Add,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::IntLiteral(2)),
                op: BinOp::Mul,
                right: Box::new(Expr::IntLiteral(3)),
            }),
        }
    );
}

#[test]
fn parse_precedence_left_associative() {
    // 1 - 2 - 3 should parse as (1 - 2) - 3
    let expr = parse_single_expr("1 - 2 - 3");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::IntLiteral(1)),
                op: BinOp::Sub,
                right: Box::new(Expr::IntLiteral(2)),
            }),
            op: BinOp::Sub,
            right: Box::new(Expr::IntLiteral(3)),
        }
    );
}

#[test]
fn parse_grouped_expression() {
    // (1 + 2) * 3
    let expr = parse_single_expr("(1 + 2) * 3");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::IntLiteral(1)),
                op: BinOp::Add,
                right: Box::new(Expr::IntLiteral(2)),
            }),
            op: BinOp::Mul,
            right: Box::new(Expr::IntLiteral(3)),
        }
    );
}

#[test]
fn parse_unary_neg() {
    let expr = parse_single_expr("-x");
    assert_eq!(
        expr,
        Expr::UnaryOp {
            op: UnOp::Neg,
            expr: Box::new(Expr::Identifier("x".to_string())),
        }
    );
}

#[test]
fn parse_unary_not() {
    let expr = parse_single_expr("!flag");
    assert_eq!(
        expr,
        Expr::UnaryOp {
            op: UnOp::Not,
            expr: Box::new(Expr::Identifier("flag".to_string())),
        }
    );
}

#[test]
fn parse_comparison() {
    let expr = parse_single_expr("a > b");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::Identifier("a".to_string())),
            op: BinOp::Gt,
            right: Box::new(Expr::Identifier("b".to_string())),
        }
    );
}

#[test]
fn parse_logical_and_or() {
    // a && b || c should parse as (a && b) || c
    let expr = parse_single_expr("a && b || c");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Identifier("a".to_string())),
                op: BinOp::And,
                right: Box::new(Expr::Identifier("b".to_string())),
            }),
            op: BinOp::Or,
            right: Box::new(Expr::Identifier("c".to_string())),
        }
    );
}

#[test]
fn parse_all_binary_ops() {
    // Just verify each op parses correctly
    for (src, op) in [
        ("a + b", BinOp::Add),
        ("a - b", BinOp::Sub),
        ("a * b", BinOp::Mul),
        ("a / b", BinOp::Div),
        ("a % b", BinOp::Mod),
        ("a == b", BinOp::Eq),
        ("a != b", BinOp::NotEq),
        ("a < b", BinOp::Lt),
        ("a > b", BinOp::Gt),
        ("a <= b", BinOp::LtEq),
        ("a >= b", BinOp::GtEq),
        ("a && b", BinOp::And),
        ("a || b", BinOp::Or),
    ] {
        let expr = parse_single_expr(src);
        match expr {
            Expr::BinaryOp { op: parsed_op, .. } => {
                assert_eq!(parsed_op, op, "Failed for: {}", src)
            }
            other => panic!("Expected BinaryOp for '{}', got {:?}", src, other),
        }
    }
}

#[test]
fn parse_complex_precedence() {
    // 1 + 2 * 3 - 4 / 2 should parse as (1 + (2 * 3)) - (4 / 2)
    let expr = parse_single_expr("1 + 2 * 3 - 4 / 2");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::IntLiteral(1)),
                op: BinOp::Add,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::IntLiteral(2)),
                    op: BinOp::Mul,
                    right: Box::new(Expr::IntLiteral(3)),
                }),
            }),
            op: BinOp::Sub,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::IntLiteral(4)),
                op: BinOp::Div,
                right: Box::new(Expr::IntLiteral(2)),
            }),
        }
    );
}

#[test]
fn parse_unary_in_binary() {
    // -a + b should parse as (-a) + b
    let expr = parse_single_expr("-a + b");
    assert_eq!(
        expr,
        Expr::BinaryOp {
            left: Box::new(Expr::UnaryOp {
                op: UnOp::Neg,
                expr: Box::new(Expr::Identifier("a".to_string())),
            }),
            op: BinOp::Add,
            right: Box::new(Expr::Identifier("b".to_string())),
        }
    );
}
