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

// -- Task 3: Postfix expressions and function calls --

#[test]
fn parse_function_call_no_args() {
    let expr = parse_single_expr("foo()");
    assert_eq!(
        expr,
        Expr::FnCall {
            callee: Box::new(Expr::Identifier("foo".to_string())),
            args: vec![],
        }
    );
}

#[test]
fn parse_function_call_one_arg() {
    let expr = parse_single_expr("println(\"hello\")");
    assert_eq!(
        expr,
        Expr::FnCall {
            callee: Box::new(Expr::Identifier("println".to_string())),
            args: vec![Expr::StringLiteral("hello".to_string())],
        }
    );
}

#[test]
fn parse_function_call_multiple_args() {
    let expr = parse_single_expr("add(1, 2)");
    assert_eq!(
        expr,
        Expr::FnCall {
            callee: Box::new(Expr::Identifier("add".to_string())),
            args: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
        }
    );
}

#[test]
fn parse_field_access() {
    let expr = parse_single_expr("user.name");
    assert_eq!(
        expr,
        Expr::FieldAccess {
            object: Box::new(Expr::Identifier("user".to_string())),
            field: "name".to_string(),
        }
    );
}

#[test]
fn parse_chained_field_access() {
    // a.b.c -> FieldAccess { object: FieldAccess { object: a, field: "b" }, field: "c" }
    let expr = parse_single_expr("a.b.c");
    assert_eq!(
        expr,
        Expr::FieldAccess {
            object: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Identifier("a".to_string())),
                field: "b".to_string(),
            }),
            field: "c".to_string(),
        }
    );
}

#[test]
fn parse_safe_access() {
    let expr = parse_single_expr("user?.name");
    assert_eq!(
        expr,
        Expr::SafeAccess {
            object: Box::new(Expr::Identifier("user".to_string())),
            field: "name".to_string(),
        }
    );
}

#[test]
fn parse_method_call() {
    let expr = parse_single_expr("list.push(42)");
    assert_eq!(
        expr,
        Expr::MethodCall {
            object: Box::new(Expr::Identifier("list".to_string())),
            method: "push".to_string(),
            args: vec![Expr::IntLiteral(42)],
        }
    );
}

#[test]
fn parse_index_access() {
    let expr = parse_single_expr("items[0]");
    assert_eq!(
        expr,
        Expr::Index {
            object: Box::new(Expr::Identifier("items".to_string())),
            index: Box::new(Expr::IntLiteral(0)),
        }
    );
}

#[test]
fn parse_null_coalesce() {
    let expr = parse_single_expr("user?.name ?? \"Unknown\"");
    assert_eq!(
        expr,
        Expr::NullCoalesce {
            left: Box::new(Expr::SafeAccess {
                object: Box::new(Expr::Identifier("user".to_string())),
                field: "name".to_string(),
            }),
            right: Box::new(Expr::StringLiteral("Unknown".to_string())),
        }
    );
}

#[test]
fn parse_chained_method_calls() {
    // a.foo().bar() -> MethodCall { object: FnCall on FieldAccess... }
    // Actually: a.foo() is MethodCall, then .bar() is another MethodCall on the result
    let expr = parse_single_expr("a.foo().bar()");
    assert_eq!(
        expr,
        Expr::MethodCall {
            object: Box::new(Expr::MethodCall {
                object: Box::new(Expr::Identifier("a".to_string())),
                method: "foo".to_string(),
                args: vec![],
            }),
            method: "bar".to_string(),
            args: vec![],
        }
    );
}

#[test]
fn parse_call_on_field() {
    // obj.field(1, 2) is a method call
    let expr = parse_single_expr("obj.field(1, 2)");
    assert_eq!(
        expr,
        Expr::MethodCall {
            object: Box::new(Expr::Identifier("obj".to_string())),
            method: "field".to_string(),
            args: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
        }
    );
}

// -- Task 4: Let, return, type annotations --

#[test]
fn parse_let_with_type() {
    let stmts = parse_ok("let x: i32 = 42");
    assert_eq!(
        stmts[0],
        Stmt::Let {
            name: "x".to_string(),
            mutable: false,
            type_ann: Some(Type::Simple("i32".to_string())),
            value: Expr::IntLiteral(42),
        }
    );
}

#[test]
fn parse_let_mutable() {
    let stmts = parse_ok("let mut count = 0");
    assert_eq!(
        stmts[0],
        Stmt::Let {
            name: "count".to_string(),
            mutable: true,
            type_ann: None,
            value: Expr::IntLiteral(0),
        }
    );
}

#[test]
fn parse_let_inferred() {
    let stmts = parse_ok("let name = \"Alice\"");
    assert_eq!(
        stmts[0],
        Stmt::Let {
            name: "name".to_string(),
            mutable: false,
            type_ann: None,
            value: Expr::StringLiteral("Alice".to_string()),
        }
    );
}

#[test]
fn parse_let_with_expression() {
    let stmts = parse_ok("let sum = a + b");
    assert_eq!(
        stmts[0],
        Stmt::Let {
            name: "sum".to_string(),
            mutable: false,
            type_ann: None,
            value: Expr::BinaryOp {
                left: Box::new(Expr::Identifier("a".to_string())),
                op: BinOp::Add,
                right: Box::new(Expr::Identifier("b".to_string())),
            },
        }
    );
}

#[test]
fn parse_return_value() {
    let stmts = parse_ok("return x + 1");
    assert_eq!(
        stmts[0],
        Stmt::Return(Some(Expr::BinaryOp {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinOp::Add,
            right: Box::new(Expr::IntLiteral(1)),
        }))
    );
}

#[test]
fn parse_return_void() {
    let stmts = parse_ok("return");
    assert_eq!(stmts[0], Stmt::Return(None));
}

#[test]
fn parse_expression_statement() {
    let stmts = parse_ok("println(\"hello\")");
    assert_eq!(
        stmts[0],
        Stmt::Expression(Expr::FnCall {
            callee: Box::new(Expr::Identifier("println".to_string())),
            args: vec![Expr::StringLiteral("hello".to_string())],
        })
    );
}

#[test]
fn parse_nullable_type() {
    let stmts = parse_ok("let user: User? = find()");
    match &stmts[0] {
        Stmt::Let { type_ann, .. } => {
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
    match &stmts[0] {
        Stmt::Let { type_ann, .. } => {
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
    match &stmts[0] {
        Stmt::Let { type_ann, .. } => {
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
    match &stmts[0] {
        Stmt::Let { type_ann, .. } => {
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
    match &stmts[0] {
        Stmt::Let { type_ann, .. } => {
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
        Stmt::FnDef {
            name: "main".to_string(),
            params: vec![],
            return_type: None,
            body: vec![Stmt::Expression(Expr::FnCall {
                callee: Box::new(Expr::Identifier("println".to_string())),
                args: vec![Expr::StringLiteral("Hello, Sage!".to_string())],
            })],
            decorators: vec![],
        }
    );
}

#[test]
fn parse_fn_with_params_and_return() {
    let stmts = parse_ok("fn add(a: i32, b: i32) -> i32 {\n    return a + b\n}");
    assert_eq!(
        stmts[0],
        Stmt::FnDef {
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
            body: vec![Stmt::Return(Some(Expr::BinaryOp {
                left: Box::new(Expr::Identifier("a".to_string())),
                op: BinOp::Add,
                right: Box::new(Expr::Identifier("b".to_string())),
            }))],
            decorators: vec![],
        }
    );
}

#[test]
fn parse_fn_no_return_type() {
    let stmts = parse_ok("fn greet(name: str) {\n    println(name)\n}");
    match &stmts[0] {
        Stmt::FnDef {
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
    match &stmts[0] {
        Stmt::FnDef { body, .. } => {
            assert_eq!(body.len(), 3);
            assert!(matches!(&body[0], Stmt::Let { name, .. } if name == "x"));
            assert!(matches!(&body[1], Stmt::Let { name, .. } if name == "y"));
            assert!(matches!(&body[2], Stmt::Return(Some(_))));
        }
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_fn_empty_body() {
    let stmts = parse_ok("fn noop() {}");
    match &stmts[0] {
        Stmt::FnDef { body, .. } => assert!(body.is_empty()),
        other => panic!("Expected FnDef, got {:?}", other),
    }
}

#[test]
fn parse_multiple_top_level_fns() {
    let src = "fn foo() {\n    1\n}\nfn bar() {\n    2\n}";
    let stmts = parse_ok(src);
    assert_eq!(stmts.len(), 2);
    assert!(matches!(&stmts[0], Stmt::FnDef { name, .. } if name == "foo"));
    assert!(matches!(&stmts[1], Stmt::FnDef { name, .. } if name == "bar"));
}
