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
    match stmts.into_iter().next().unwrap().kind {
        StmtKind::Expression(expr) => expr,
        other => panic!("Expected expression statement, got {:?}", other),
    }
}

// Helpers: wrap ExprKind/StmtKind into Expr/Stmt with dummy spans.
// PartialEq on Expr/Stmt ignores span, so these compare correctly.
fn e(kind: ExprKind) -> Expr {
    Expr::dummy(kind)
}

fn be(kind: ExprKind) -> Box<Expr> {
    Box::new(e(kind))
}

fn s(kind: StmtKind) -> Stmt {
    Stmt::dummy(kind)
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
    assert_eq!(expr, e(ExprKind::IntLiteral(42)));
}

#[test]
fn parse_float_literal() {
    let expr = parse_single_expr("3.14");
    assert_eq!(expr, e(ExprKind::FloatLiteral(3.14)));
}

#[test]
fn parse_string_literal() {
    let expr = parse_single_expr("\"hello\"");
    assert_eq!(expr, e(ExprKind::StringLiteral("hello".to_string())));
}

#[test]
fn parse_bool_true() {
    let expr = parse_single_expr("true");
    assert_eq!(expr, e(ExprKind::BoolLiteral(true)));
}

#[test]
fn parse_bool_false() {
    let expr = parse_single_expr("false");
    assert_eq!(expr, e(ExprKind::BoolLiteral(false)));
}

#[test]
fn parse_identifier() {
    let expr = parse_single_expr("foo");
    assert_eq!(expr, e(ExprKind::Identifier("foo".to_string())));
}

#[test]
fn parse_binary_add() {
    let expr = parse_single_expr("1 + 2");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: be(ExprKind::IntLiteral(1)),
            op: BinOp::Add,
            right: be(ExprKind::IntLiteral(2)),
        })
    );
}

#[test]
fn parse_precedence_mul_over_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let expr = parse_single_expr("1 + 2 * 3");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: be(ExprKind::IntLiteral(1)),
            op: BinOp::Add,
            right: Box::new(e(ExprKind::BinaryOp {
                left: be(ExprKind::IntLiteral(2)),
                op: BinOp::Mul,
                right: be(ExprKind::IntLiteral(3)),
            })),
        })
    );
}

#[test]
fn parse_precedence_left_associative() {
    // 1 - 2 - 3 should parse as (1 - 2) - 3
    let expr = parse_single_expr("1 - 2 - 3");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: Box::new(e(ExprKind::BinaryOp {
                left: be(ExprKind::IntLiteral(1)),
                op: BinOp::Sub,
                right: be(ExprKind::IntLiteral(2)),
            })),
            op: BinOp::Sub,
            right: be(ExprKind::IntLiteral(3)),
        })
    );
}

#[test]
fn parse_grouped_expression() {
    // (1 + 2) * 3
    let expr = parse_single_expr("(1 + 2) * 3");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: Box::new(e(ExprKind::BinaryOp {
                left: be(ExprKind::IntLiteral(1)),
                op: BinOp::Add,
                right: be(ExprKind::IntLiteral(2)),
            })),
            op: BinOp::Mul,
            right: be(ExprKind::IntLiteral(3)),
        })
    );
}

#[test]
fn parse_unary_neg() {
    let expr = parse_single_expr("-x");
    assert_eq!(
        expr,
        e(ExprKind::UnaryOp {
            op: UnOp::Neg,
            expr: be(ExprKind::Identifier("x".to_string())),
        })
    );
}

#[test]
fn parse_unary_not() {
    let expr = parse_single_expr("!flag");
    assert_eq!(
        expr,
        e(ExprKind::UnaryOp {
            op: UnOp::Not,
            expr: be(ExprKind::Identifier("flag".to_string())),
        })
    );
}

#[test]
fn parse_comparison() {
    let expr = parse_single_expr("a > b");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: be(ExprKind::Identifier("a".to_string())),
            op: BinOp::Gt,
            right: be(ExprKind::Identifier("b".to_string())),
        })
    );
}

#[test]
fn parse_logical_and_or() {
    // a && b || c should parse as (a && b) || c
    let expr = parse_single_expr("a && b || c");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: Box::new(e(ExprKind::BinaryOp {
                left: be(ExprKind::Identifier("a".to_string())),
                op: BinOp::And,
                right: be(ExprKind::Identifier("b".to_string())),
            })),
            op: BinOp::Or,
            right: be(ExprKind::Identifier("c".to_string())),
        })
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
        match expr.kind {
            ExprKind::BinaryOp { op: parsed_op, .. } => {
                assert_eq!(parsed_op, op, "Failed for: {}", src)
            }
            ref other => panic!("Expected BinaryOp for '{}', got {:?}", src, other),
        }
    }
}

#[test]
fn parse_complex_precedence() {
    // 1 + 2 * 3 - 4 / 2 should parse as (1 + (2 * 3)) - (4 / 2)
    let expr = parse_single_expr("1 + 2 * 3 - 4 / 2");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: Box::new(e(ExprKind::BinaryOp {
                left: be(ExprKind::IntLiteral(1)),
                op: BinOp::Add,
                right: Box::new(e(ExprKind::BinaryOp {
                    left: be(ExprKind::IntLiteral(2)),
                    op: BinOp::Mul,
                    right: be(ExprKind::IntLiteral(3)),
                })),
            })),
            op: BinOp::Sub,
            right: Box::new(e(ExprKind::BinaryOp {
                left: be(ExprKind::IntLiteral(4)),
                op: BinOp::Div,
                right: be(ExprKind::IntLiteral(2)),
            })),
        })
    );
}

#[test]
fn parse_unary_in_binary() {
    // -a + b should parse as (-a) + b
    let expr = parse_single_expr("-a + b");
    assert_eq!(
        expr,
        e(ExprKind::BinaryOp {
            left: Box::new(e(ExprKind::UnaryOp {
                op: UnOp::Neg,
                expr: be(ExprKind::Identifier("a".to_string())),
            })),
            op: BinOp::Add,
            right: be(ExprKind::Identifier("b".to_string())),
        })
    );
}

// -- Task 3: Postfix expressions and function calls --

#[test]
fn parse_function_call_no_args() {
    let expr = parse_single_expr("foo()");
    assert_eq!(
        expr,
        e(ExprKind::FnCall {
            callee: be(ExprKind::Identifier("foo".to_string())),
            args: vec![],
        })
    );
}

#[test]
fn parse_function_call_one_arg() {
    let expr = parse_single_expr("println(\"hello\")");
    assert_eq!(
        expr,
        e(ExprKind::FnCall {
            callee: be(ExprKind::Identifier("println".to_string())),
            args: vec![e(ExprKind::StringLiteral("hello".to_string()))],
        })
    );
}

#[test]
fn parse_function_call_multiple_args() {
    let expr = parse_single_expr("add(1, 2)");
    assert_eq!(
        expr,
        e(ExprKind::FnCall {
            callee: be(ExprKind::Identifier("add".to_string())),
            args: vec![e(ExprKind::IntLiteral(1)), e(ExprKind::IntLiteral(2))],
        })
    );
}

#[test]
fn parse_field_access() {
    let expr = parse_single_expr("user.name");
    assert_eq!(
        expr,
        e(ExprKind::FieldAccess {
            object: be(ExprKind::Identifier("user".to_string())),
            field: "name".to_string(),
        })
    );
}

#[test]
fn parse_chained_field_access() {
    let expr = parse_single_expr("a.b.c");
    assert_eq!(
        expr,
        e(ExprKind::FieldAccess {
            object: Box::new(e(ExprKind::FieldAccess {
                object: be(ExprKind::Identifier("a".to_string())),
                field: "b".to_string(),
            })),
            field: "c".to_string(),
        })
    );
}

#[test]
fn parse_safe_access() {
    let expr = parse_single_expr("user?.name");
    assert_eq!(
        expr,
        e(ExprKind::SafeAccess {
            object: be(ExprKind::Identifier("user".to_string())),
            field: "name".to_string(),
        })
    );
}

#[test]
fn parse_method_call() {
    let expr = parse_single_expr("list.push(42)");
    assert_eq!(
        expr,
        e(ExprKind::MethodCall {
            object: be(ExprKind::Identifier("list".to_string())),
            method: "push".to_string(),
            args: vec![e(ExprKind::IntLiteral(42))],
        })
    );
}

#[test]
fn parse_index_access() {
    let expr = parse_single_expr("items[0]");
    assert_eq!(
        expr,
        e(ExprKind::Index {
            object: be(ExprKind::Identifier("items".to_string())),
            index: be(ExprKind::IntLiteral(0)),
        })
    );
}

#[test]
fn parse_null_coalesce() {
    let expr = parse_single_expr("user?.name ?? \"Unknown\"");
    assert_eq!(
        expr,
        e(ExprKind::NullCoalesce {
            left: Box::new(e(ExprKind::SafeAccess {
                object: be(ExprKind::Identifier("user".to_string())),
                field: "name".to_string(),
            })),
            right: be(ExprKind::StringLiteral("Unknown".to_string())),
        })
    );
}

#[test]
fn parse_chained_method_calls() {
    let expr = parse_single_expr("a.foo().bar()");
    assert_eq!(
        expr,
        e(ExprKind::MethodCall {
            object: Box::new(e(ExprKind::MethodCall {
                object: be(ExprKind::Identifier("a".to_string())),
                method: "foo".to_string(),
                args: vec![],
            })),
            method: "bar".to_string(),
            args: vec![],
        })
    );
}

#[test]
fn parse_call_on_field() {
    let expr = parse_single_expr("obj.field(1, 2)");
    assert_eq!(
        expr,
        e(ExprKind::MethodCall {
            object: be(ExprKind::Identifier("obj".to_string())),
            method: "field".to_string(),
            args: vec![e(ExprKind::IntLiteral(1)), e(ExprKind::IntLiteral(2))],
        })
    );
}

// -- Task 4: Let, return, type annotations --

#[test]
fn parse_let_with_type() {
    let stmts = parse_ok("let x: i32 = 42");
    assert_eq!(
        stmts[0],
        s(StmtKind::Let {
            name: "x".to_string(),
            mutable: false,
            type_ann: Some(Type::Simple("i32".to_string())),
            value: e(ExprKind::IntLiteral(42)),
        })
    );
}

#[test]
fn parse_let_mutable() {
    let stmts = parse_ok("let mut count = 0");
    assert_eq!(
        stmts[0],
        s(StmtKind::Let {
            name: "count".to_string(),
            mutable: true,
            type_ann: None,
            value: e(ExprKind::IntLiteral(0)),
        })
    );
}

#[test]
fn parse_let_inferred() {
    let stmts = parse_ok("let name = \"Alice\"");
    assert_eq!(
        stmts[0],
        s(StmtKind::Let {
            name: "name".to_string(),
            mutable: false,
            type_ann: None,
            value: e(ExprKind::StringLiteral("Alice".to_string())),
        })
    );
}

#[test]
fn parse_let_with_expression() {
    let stmts = parse_ok("let sum = a + b");
    assert_eq!(
        stmts[0],
        s(StmtKind::Let {
            name: "sum".to_string(),
            mutable: false,
            type_ann: None,
            value: e(ExprKind::BinaryOp {
                left: be(ExprKind::Identifier("a".to_string())),
                op: BinOp::Add,
                right: be(ExprKind::Identifier("b".to_string())),
            }),
        })
    );
}

#[test]
fn parse_return_value() {
    let stmts = parse_ok("return x + 1");
    assert_eq!(
        stmts[0],
        s(StmtKind::Return(Some(e(ExprKind::BinaryOp {
            left: be(ExprKind::Identifier("x".to_string())),
            op: BinOp::Add,
            right: be(ExprKind::IntLiteral(1)),
        }))))
    );
}

#[test]
fn parse_return_void() {
    let stmts = parse_ok("return");
    assert_eq!(stmts[0], s(StmtKind::Return(None)));
}

#[test]
fn parse_expression_statement() {
    let stmts = parse_ok("println(\"hello\")");
    assert_eq!(
        stmts[0],
        s(StmtKind::Expression(e(ExprKind::FnCall {
            callee: be(ExprKind::Identifier("println".to_string())),
            args: vec![e(ExprKind::StringLiteral("hello".to_string()))],
        })))
    );
}

#[test]
fn parse_nullable_type() {
    let stmts = parse_ok("let user: User? = find()");
    match &stmts[0].kind {
        StmtKind::Let { type_ann, .. } => {
            assert_eq!(
                *type_ann,
                Some(Type::Nullable(Box::new(Type::Simple("User".to_string()))))
            );
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn parse_generic_type() {
    let stmts = parse_ok("let items: Vec<i32> = create()");
    match &stmts[0].kind {
        StmtKind::Let { type_ann, .. } => {
            assert_eq!(
                *type_ann,
                Some(Type::Generic {
                    name: "Vec".to_string(),
                    params: vec![Type::Simple("i32".to_string())],
                })
            );
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn parse_reference_type() {
    let stmts = parse_ok("let name: &str = get()");
    match &stmts[0].kind {
        StmtKind::Let { type_ann, .. } => {
            assert_eq!(
                *type_ann,
                Some(Type::Reference {
                    mutable: false,
                    inner: Box::new(Type::Simple("str".to_string())),
                })
            );
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn parse_mutable_reference_type() {
    let stmts = parse_ok("let data: &mut i32 = get()");
    match &stmts[0].kind {
        StmtKind::Let { type_ann, .. } => {
            assert_eq!(
                *type_ann,
                Some(Type::Reference {
                    mutable: true,
                    inner: Box::new(Type::Simple("i32".to_string())),
                })
            );
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn parse_multi_generic_type() {
    let stmts = parse_ok("let result: Result<i32, str> = try_it()");
    match &stmts[0].kind {
        StmtKind::Let { type_ann, .. } => {
            assert_eq!(
                *type_ann,
                Some(Type::Generic {
                    name: "Result".to_string(),
                    params: vec![
                        Type::Simple("i32".to_string()),
                        Type::Simple("str".to_string()),
                    ],
                })
            );
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// -- Task 5: Function definitions and blocks --

#[test]
fn parse_hello_world() {
    let stmts = parse_ok("fn main() {\n    println(\"Hello, Sage!\")\n}");
    assert_eq!(
        stmts[0],
        s(StmtKind::FnDef {
            name: "main".to_string(),
            params: vec![],
            return_type: None,
            body: vec![s(StmtKind::Expression(e(ExprKind::FnCall {
                callee: be(ExprKind::Identifier("println".to_string())),
                args: vec![e(ExprKind::StringLiteral("Hello, Sage!".to_string()))],
            })))],
            decorators: vec![],
        })
    );
}

#[test]
fn parse_fn_with_params_and_return() {
    let stmts = parse_ok("fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}");
    assert_eq!(
        stmts[0],
        s(StmtKind::FnDef {
            name: "add".to_string(),
            params: vec![
                Param {
                    name: "a".to_string(),
                    type_ann: Some(Type::Simple("i32".to_string())),
                },
                Param {
                    name: "b".to_string(),
                    type_ann: Some(Type::Simple("i32".to_string())),
                },
            ],
            return_type: Some(Type::Simple("i32".to_string())),
            body: vec![s(StmtKind::Return(Some(e(ExprKind::BinaryOp {
                left: be(ExprKind::Identifier("a".to_string())),
                op: BinOp::Add,
                right: be(ExprKind::Identifier("b".to_string())),
            }))))],
            decorators: vec![],
        })
    );
}

#[test]
fn parse_fn_no_return_type() {
    let stmts = parse_ok("fn greet(name: str) {\n    println(name)\n}");
    match &stmts[0].kind {
        StmtKind::FnDef {
            name,
            params,
            return_type,
            ..
        } => {
            assert_eq!(name, "greet");
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "name");
            assert!(return_type.is_none());
        }
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_fn_multiple_statements() {
    let stmts = parse_ok("fn calc() -> i32 {\n    let x = 1\n    let y = 2\n    return x + y\n}");
    match &stmts[0].kind {
        StmtKind::FnDef { body, .. } => {
            assert_eq!(body.len(), 3);
            assert!(matches!(&body[0].kind, StmtKind::Let { name, .. } if name == "x"));
            assert!(matches!(&body[1].kind, StmtKind::Let { name, .. } if name == "y"));
            assert!(matches!(&body[2].kind, StmtKind::Return(Some(_))));
        }
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_fn_empty_body() {
    let stmts = parse_ok("fn noop() {}");
    match &stmts[0].kind {
        StmtKind::FnDef { body, .. } => assert!(body.is_empty()),
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_multiple_top_level_fns() {
    let src = "fn foo() {\n    1\n}\nfn bar() {\n    2\n}";
    let stmts = parse_ok(src);
    assert_eq!(stmts.len(), 2);
    assert!(matches!(&stmts[0].kind, StmtKind::FnDef { name, .. } if name == "foo"));
    assert!(matches!(&stmts[1].kind, StmtKind::FnDef { name, .. } if name == "bar"));
}

// -- Task 6: Control flow --

#[test]
fn parse_if_else() {
    let expr = parse_single_expr("if x > 0 {\n    1\n} else {\n    2\n}");
    assert_eq!(
        expr,
        e(ExprKind::IfElse {
            condition: Box::new(e(ExprKind::BinaryOp {
                left: be(ExprKind::Identifier("x".to_string())),
                op: BinOp::Gt,
                right: be(ExprKind::IntLiteral(0)),
            })),
            then_block: vec![s(StmtKind::Expression(e(ExprKind::IntLiteral(1))))],
            else_block: Some(vec![s(StmtKind::Expression(e(ExprKind::IntLiteral(2))))]),
        })
    );
}

#[test]
fn parse_if_no_else() {
    let expr = parse_single_expr("if flag {\n    do_stuff()\n}");
    match expr.kind {
        ExprKind::IfElse {
            condition,
            then_block,
            else_block,
        } => {
            assert_eq!(*condition, e(ExprKind::Identifier("flag".to_string())));
            assert_eq!(then_block.len(), 1);
            assert!(else_block.is_none());
        }
        other => panic!("Expected IfElse, got {:?}", other),
    }
}

#[test]
fn parse_if_else_if() {
    let expr = parse_single_expr("if a {\n    1\n} else if b {\n    2\n} else {\n    3\n}");
    match expr.kind {
        ExprKind::IfElse {
            else_block: Some(else_stmts),
            ..
        } => {
            // else branch contains an expression statement wrapping another IfElse
            assert_eq!(else_stmts.len(), 1);
            match &else_stmts[0].kind {
                StmtKind::Expression(inner) => match &inner.kind {
                    ExprKind::IfElse {
                        else_block: Some(inner_else),
                        ..
                    } => {
                        assert_eq!(inner_else.len(), 1);
                    }
                    other => panic!("Expected nested IfElse, got {:?}", other),
                },
                other => panic!("Expected Expression, got {:?}", other),
            }
        }
        other => panic!("Expected IfElse with else, got {:?}", other),
    }
}

#[test]
fn parse_for_loop() {
    let stmts = parse_ok("for i in items {\n    println(i)\n}");
    match &stmts[0].kind {
        StmtKind::ForLoop { var, iter, body } => {
            assert_eq!(var, "i");
            assert_eq!(*iter, e(ExprKind::Identifier("items".to_string())));
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected ForLoop, got {:?}", other),
    }
}

#[test]
fn parse_for_loop_range() {
    let stmts = parse_ok("for i in 0..10 {\n    println(i)\n}");
    match &stmts[0].kind {
        StmtKind::ForLoop { var, iter, .. } => {
            assert_eq!(var, "i");
            assert!(matches!(iter.kind, ExprKind::BinaryOp { .. }));
        }
        other => panic!("Expected ForLoop, got {:?}", other),
    }
}

#[test]
fn parse_while_loop() {
    let stmts = parse_ok("while count < 10 {\n    process(count)\n}");
    match &stmts[0].kind {
        StmtKind::WhileLoop {
            condition, body, ..
        } => {
            assert!(matches!(
                condition.kind,
                ExprKind::BinaryOp { op: BinOp::Lt, .. }
            ));
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected WhileLoop, got {:?}", other),
    }
}

#[test]
fn parse_match_expression() {
    let expr =
        parse_single_expr("match x {\n    1 => \"one\"\n    2 => \"two\"\n    _ => \"other\"\n}");
    match expr.kind {
        ExprKind::Match {
            expr: matched,
            arms,
        } => {
            assert_eq!(*matched, e(ExprKind::Identifier("x".to_string())));
            assert_eq!(arms.len(), 3);
            assert_eq!(arms[0].pattern, e(ExprKind::IntLiteral(1)));
            assert_eq!(arms[0].body, e(ExprKind::StringLiteral("one".to_string())));
            assert_eq!(arms[2].pattern, e(ExprKind::Identifier("_".to_string())));
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn parse_match_with_commas() {
    let expr = parse_single_expr("match x { 1 => \"a\", 2 => \"b\" }");
    match expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// -- Task 7: Struct, trait, impl --

#[test]
fn parse_struct_def() {
    let stmts = parse_ok("struct User {\n    name: str\n    age: i32\n}");
    assert_eq!(
        stmts[0],
        s(StmtKind::StructDef {
            name: "User".to_string(),
            fields: vec![
                Field {
                    name: "name".to_string(),
                    type_ann: Type::Simple("str".to_string()),
                },
                Field {
                    name: "age".to_string(),
                    type_ann: Type::Simple("i32".to_string()),
                },
            ],
        })
    );
}

#[test]
fn parse_struct_comma_separated() {
    let stmts = parse_ok("struct Point { x: f64, y: f64 }");
    match &stmts[0].kind {
        StmtKind::StructDef { name, fields } => {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected StructDef, got {:?}", other),
    }
}

#[test]
fn parse_struct_empty() {
    let stmts = parse_ok("struct Empty {}");
    assert_eq!(
        stmts[0],
        s(StmtKind::StructDef {
            name: "Empty".to_string(),
            fields: vec![],
        })
    );
}

#[test]
fn parse_trait_def() {
    let stmts = parse_ok("trait Greetable {\n    fn greet(self) -> str\n}");
    assert_eq!(
        stmts[0],
        s(StmtKind::TraitDef {
            name: "Greetable".to_string(),
            methods: vec![FnSignature {
                name: "greet".to_string(),
                params: vec![Param {
                    name: "self".to_string(),
                    type_ann: None,
                }],
                return_type: Some(Type::Simple("str".to_string())),
            }],
        })
    );
}

#[test]
fn parse_trait_multiple_methods() {
    let stmts = parse_ok("trait Animal {\n    fn name(self) -> str\n    fn speak(self) -> str\n}");
    match &stmts[0].kind {
        StmtKind::TraitDef { methods, .. } => {
            assert_eq!(methods.len(), 2);
            assert_eq!(methods[0].name, "name");
            assert_eq!(methods[1].name, "speak");
        }
        other => panic!("Expected TraitDef, got {:?}", other),
    }
}

#[test]
fn parse_impl_block() {
    let stmts = parse_ok("impl User {\n    fn new(name: str) -> User {\n        name\n    }\n}");
    match &stmts[0].kind {
        StmtKind::ImplBlock {
            trait_name,
            target,
            methods,
        } => {
            assert!(trait_name.is_none());
            assert_eq!(target, "User");
            assert_eq!(methods.len(), 1);
            assert!(matches!(&methods[0].kind, StmtKind::FnDef { name, .. } if name == "new"));
        }
        other => panic!("Expected ImplBlock, got {:?}", other),
    }
}

#[test]
fn parse_impl_trait_for_type() {
    let stmts = parse_ok(
        "impl Greetable for User {\n    fn greet(self) -> str {\n        self.name\n    }\n}",
    );
    match &stmts[0].kind {
        StmtKind::ImplBlock {
            trait_name,
            target,
            methods,
        } => {
            assert_eq!(*trait_name, Some("Greetable".to_string()));
            assert_eq!(target, "User");
            assert_eq!(methods.len(), 1);
        }
        other => panic!("Expected ImplBlock, got {:?}", other),
    }
}

// -- Task 8: Decorators, imports, closures, concurrency, etc --

#[test]
fn parse_decorator_simple() {
    let stmts = parse_ok("@tool\nfn lookup() {}");
    match &stmts[0].kind {
        StmtKind::FnDef {
            name, decorators, ..
        } => {
            assert_eq!(name, "lookup");
            assert_eq!(decorators.len(), 1);
            assert_eq!(decorators[0].name, "tool");
            assert!(decorators[0].args.is_empty());
        }
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_decorator_with_args() {
    let stmts = parse_ok("@tool(description: \"look up a user\")\nfn lookup() {}");
    match &stmts[0].kind {
        StmtKind::FnDef { decorators, .. } => {
            assert_eq!(decorators.len(), 1);
            assert_eq!(decorators[0].name, "tool");
            assert_eq!(decorators[0].args.len(), 1);
            assert_eq!(decorators[0].args[0].0, "description");
            assert_eq!(
                decorators[0].args[0].1,
                e(ExprKind::StringLiteral("look up a user".to_string()))
            );
        }
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_multiple_decorators() {
    let stmts = parse_ok("@mcp_server\n@tool\nfn handler() {}");
    match &stmts[0].kind {
        StmtKind::FnDef { decorators, .. } => {
            assert_eq!(decorators.len(), 2);
            assert_eq!(decorators[0].name, "mcp_server");
            assert_eq!(decorators[1].name, "tool");
        }
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_import() {
    let stmts = parse_ok("import std.io");
    assert_eq!(
        stmts[0],
        s(StmtKind::Import {
            path: vec!["std".to_string(), "io".to_string()],
        })
    );
}

#[test]
fn parse_import_deep() {
    let stmts = parse_ok("import std.collections.HashMap");
    assert_eq!(
        stmts[0],
        s(StmtKind::Import {
            path: vec![
                "std".to_string(),
                "collections".to_string(),
                "HashMap".to_string(),
            ],
        })
    );
}

#[test]
fn parse_try_catch() {
    let stmts = parse_ok("try {\n    risky()\n} catch e {\n    handle(e)\n}");
    match &stmts[0].kind {
        StmtKind::TryCatch {
            try_block,
            catch_var,
            catch_block,
        } => {
            assert_eq!(try_block.len(), 1);
            assert_eq!(catch_var, "e");
            assert_eq!(catch_block.len(), 1);
        }
        other => panic!("Expected TryCatch, got {:?}", other),
    }
}

#[test]
fn parse_test_fn() {
    let stmts = parse_ok("test \"addition works\" {\n    assert_eq(1 + 1, 2)\n}");
    match &stmts[0].kind {
        StmtKind::TestFn { name, body } => {
            assert_eq!(name, "addition works");
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected TestFn, got {:?}", other),
    }
}

#[test]
fn parse_closure_simple() {
    let expr = parse_single_expr("|x| x + 1");
    match expr.kind {
        ExprKind::Closure { params, body } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "x");
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected Closure, got {:?}", other),
    }
}

#[test]
fn parse_closure_multiple_params() {
    let expr = parse_single_expr("|a, b| a + b");
    match expr.kind {
        ExprKind::Closure { params, .. } => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[1].name, "b");
        }
        other => panic!("Expected Closure, got {:?}", other),
    }
}

#[test]
fn parse_closure_with_block() {
    let stmts = parse_ok("|a, b| {\n    return a + b\n}");
    match &stmts[0].kind {
        StmtKind::Expression(closure) => match &closure.kind {
            ExprKind::Closure { params, body } => {
                assert_eq!(params.len(), 2);
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0].kind, StmtKind::Return(Some(_))));
            }
            other => panic!("Expected Closure, got {:?}", other),
        },
        other => panic!("Expected Expression, got {:?}", other),
    }
}

#[test]
fn parse_list_literal() {
    let expr = parse_single_expr("[1, 2, 3]");
    assert_eq!(
        expr,
        e(ExprKind::ListLiteral(vec![
            e(ExprKind::IntLiteral(1)),
            e(ExprKind::IntLiteral(2)),
            e(ExprKind::IntLiteral(3)),
        ]))
    );
}

#[test]
fn parse_list_empty() {
    let expr = parse_single_expr("[]");
    assert_eq!(expr, e(ExprKind::ListLiteral(vec![])));
}

#[test]
fn parse_string_interpolation() {
    let expr = parse_single_expr("\"Hello {name}!\"");
    match expr.kind {
        ExprKind::StringInterpolation { parts } => {
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0], StringPart::Literal("Hello ".to_string()));
            assert_eq!(
                parts[1],
                StringPart::Expr(e(ExprKind::Identifier("name".to_string())))
            );
            assert_eq!(parts[2], StringPart::Literal("!".to_string()));
        }
        other => panic!("Expected StringInterpolation, got {:?}", other),
    }
}

#[test]
fn parse_spawn() {
    let expr = parse_single_expr("spawn fetch_data()");
    match expr.kind {
        ExprKind::Spawn(inner) => {
            assert!(matches!(inner.kind, ExprKind::FnCall { .. }));
        }
        other => panic!("Expected Spawn, got {:?}", other),
    }
}

#[test]
fn parse_parallel() {
    let stmts = parse_ok("parallel items |item| {\n    process(item)\n}");
    match &stmts[0].kind {
        StmtKind::Expression(expr) => match &expr.kind {
            ExprKind::Parallel {
                collection,
                param,
                body,
            } => {
                assert_eq!(**collection, e(ExprKind::Identifier("items".to_string())));
                assert_eq!(param, "item");
                assert!(matches!(body.kind, ExprKind::Closure { .. }));
            }
            other => panic!("Expected Parallel, got {:?}", other),
        },
        other => panic!("Expected Expression, got {:?}", other),
    }
}

// -- Task 9: Error recovery --

#[test]
fn parse_error_unexpected_token() {
    let (_, errors) = parse("let = 42");
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("Expected variable name"));
}

#[test]
fn parse_error_missing_rbrace() {
    let (_, errors) = parse("fn main() { let x = 1");
    assert!(!errors.is_empty());
}

#[test]
fn parse_error_recovery_continues() {
    // First statement has an error, but second should still parse
    let (stmts, errors) = parse("let = 42\nlet y = 10");
    assert!(!errors.is_empty());
    // After recovery, the second let should parse
    assert!(
        stmts
            .iter()
            .any(|s| matches!(&s.kind, StmtKind::Let { name, .. } if name == "y")),
        "Should recover and parse second let statement"
    );
}

#[test]
fn parse_multiple_errors() {
    let (_, errors) = parse("let = 1\nlet = 2");
    assert!(
        errors.len() >= 2,
        "Should report multiple errors, got {}",
        errors.len()
    );
}

// -- Fix verification tests --

#[test]
fn fix2_depth_limit_prevents_stack_overflow() {
    // Build deeply nested parens: (((((...42...)))))
    let depth = 200;
    let mut src = String::new();
    for _ in 0..depth {
        src.push('(');
    }
    src.push_str("42");
    for _ in 0..depth {
        src.push(')');
    }
    let (_, errors) = parse(&src);
    assert!(
        errors.iter().any(|e| e.message.contains("depth exceeded")),
        "Should error on deeply nested expressions"
    );
}

#[test]
fn fix3_synchronize_no_infinite_loop() {
    // This used to infinite loop: synchronize lands on 'let', parse fails on same 'let' again
    let (_, errors) = parse("let = 1\nlet = 2\nlet = 3");
    // Should terminate and report multiple errors
    assert!(
        errors.len() >= 3,
        "Should report errors, got {}",
        errors.len()
    );
}

#[test]
fn fix4_decorator_on_non_fn_is_error() {
    let (_, errors) = parse("@cached\nlet x = 5");
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Decorators are only valid")),
        "Should error on decorator before let"
    );
}

#[test]
fn fix5_parallel_empty_closure_is_error() {
    let (_, errors) = parse("parallel items || { process() }");
    assert!(
        !errors.is_empty(),
        "Should error on parallel with empty closure params"
    );
}

#[test]
fn fix7_nullish_coalescing_mixed_with_logical_is_error() {
    let (_, errors) = parse("a ?? b || c");
    assert!(
        errors.iter().any(|e| e.message.contains("Cannot mix")),
        "Should error on ?? mixed with ||"
    );

    let (_, errors) = parse("a || b ?? c");
    assert!(
        errors.iter().any(|e| e.message.contains("Cannot mix")),
        "Should error on || mixed with ??"
    );
}

#[test]
fn fix7_nullish_coalescing_with_parens_is_ok() {
    // Parenthesized sub-expressions should be fine
    let _ = parse_ok("(a ?? b) || c");
    let _ = parse_ok("a ?? (b || c)");
}

#[test]
fn fix1_exprs_have_spans() {
    let stmts = parse_ok("42");
    // The expression statement should carry a non-default span
    assert!(
        stmts[0].span.line > 0
            || stmts[0].span.column > 0
            || stmts[0].span.start > 0
            || stmts[0].span.end > 0,
        "Stmt should have a non-trivial span"
    );
    match &stmts[0].kind {
        StmtKind::Expression(expr) => {
            assert!(
                expr.span.end > expr.span.start,
                "Expr span should cover the literal"
            );
        }
        other => panic!("Expected Expression, got {:?}", other),
    }
}
