use sage::lexer::Lexer;
use sage::lexer::token::TokenKind;

fn token_kinds(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    tokens.into_iter().map(|t| t.kind).collect()
}

#[test]
fn lex_simple_expression() {
    let kinds = token_kinds("1 + 2 * 3");
    assert_eq!(
        kinds,
        vec![
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Star,
            TokenKind::IntLiteral(3),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_variable_declaration() {
    let kinds = token_kinds("let x: i32 = 42");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Colon,
            TokenKind::I32,
            TokenKind::Assign,
            TokenKind::IntLiteral(42),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_mutable_variable() {
    let kinds = token_kinds("let mut count = 0");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Mut,
            TokenKind::Identifier("count".to_string()),
            TokenKind::Assign,
            TokenKind::IntLiteral(0),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_function_definition() {
    let kinds = token_kinds("fn add(a: i32, b: i32) -> i32 { return a + b }");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Identifier("add".to_string()),
            TokenKind::LParen,
            TokenKind::Identifier("a".to_string()),
            TokenKind::Colon,
            TokenKind::I32,
            TokenKind::Comma,
            TokenKind::Identifier("b".to_string()),
            TokenKind::Colon,
            TokenKind::I32,
            TokenKind::RParen,
            TokenKind::Arrow,
            TokenKind::I32,
            TokenKind::LBrace,
            TokenKind::Return,
            TokenKind::Identifier("a".to_string()),
            TokenKind::Plus,
            TokenKind::Identifier("b".to_string()),
            TokenKind::RBrace,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_float_literal() {
    let kinds = token_kinds("3.14");
    assert_eq!(kinds, vec![TokenKind::FloatLiteral(3.14), TokenKind::EOF,]);
}

#[test]
fn lex_range_not_float() {
    let kinds = token_kinds("3..10");
    assert_eq!(
        kinds,
        vec![
            TokenKind::IntLiteral(3),
            TokenKind::DotDot,
            TokenKind::IntLiteral(10),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_string_literal() {
    let kinds = token_kinds("\"hello world\"");
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello world".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_string_escape_sequences() {
    let kinds = token_kinds("\"hello\\nworld\\t!\"");
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello\nworld\t!".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_string_interpolation() {
    let kinds = token_kinds("\"Hello {name}!\"");
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("Hello ".to_string()),
            TokenKind::InterpolationStart,
            TokenKind::Identifier("name".to_string()),
            TokenKind::InterpolationEnd,
            TokenKind::StringLiteral("!".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_multi_char_operators() {
    let kinds = token_kinds("-> => .. ?. ?? += -= *= /= == != <= >=");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Arrow,
            TokenKind::FatArrow,
            TokenKind::DotDot,
            TokenKind::QuestionDot,
            TokenKind::DoubleQuestion,
            TokenKind::PlusAssign,
            TokenKind::MinusAssign,
            TokenKind::StarAssign,
            TokenKind::SlashAssign,
            TokenKind::Eq,
            TokenKind::NotEq,
            TokenKind::LtEq,
            TokenKind::GtEq,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_logical_operators() {
    let kinds = token_kinds("a && b || !c");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::And,
            TokenKind::Identifier("b".to_string()),
            TokenKind::Or,
            TokenKind::Not,
            TokenKind::Identifier("c".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_line_comment() {
    let kinds = token_kinds("x + y // this is a comment\nz");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("x".to_string()),
            TokenKind::Plus,
            TokenKind::Identifier("y".to_string()),
            TokenKind::Newline,
            TokenKind::Identifier("z".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_block_comment() {
    let kinds = token_kinds("x + /* comment */ y");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("x".to_string()),
            TokenKind::Plus,
            TokenKind::Identifier("y".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_nested_block_comment() {
    let kinds = token_kinds("x /* outer /* inner */ still comment */ y");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("x".to_string()),
            TokenKind::Identifier("y".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_keywords() {
    let kinds = token_kinds("fn let mut return if else match for while in struct trait impl");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Let,
            TokenKind::Mut,
            TokenKind::Return,
            TokenKind::If,
            TokenKind::Else,
            TokenKind::Match,
            TokenKind::For,
            TokenKind::While,
            TokenKind::In,
            TokenKind::Struct,
            TokenKind::Trait,
            TokenKind::Impl,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_sage_keywords() {
    let kinds = token_kinds("spawn parallel scope agent module try catch import test");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Spawn,
            TokenKind::Parallel,
            TokenKind::Scope,
            TokenKind::Agent,
            TokenKind::Module,
            TokenKind::Try,
            TokenKind::Catch,
            TokenKind::Import,
            TokenKind::Test,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_booleans() {
    let kinds = token_kinds("true false");
    assert_eq!(
        kinds,
        vec![TokenKind::True, TokenKind::False, TokenKind::EOF,]
    );
}

#[test]
fn lex_type_keywords() {
    let kinds = token_kinds("i32 i64 f32 f64 str bool");
    assert_eq!(
        kinds,
        vec![
            TokenKind::I32,
            TokenKind::I64,
            TokenKind::F32,
            TokenKind::F64,
            TokenKind::Str,
            TokenKind::Bool,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_decorator() {
    let kinds = token_kinds("@mcp_server");
    assert_eq!(
        kinds,
        vec![
            TokenKind::At,
            TokenKind::Identifier("mcp_server".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_struct_definition() {
    let kinds = token_kinds("struct User { name: str, age: i32 }");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Struct,
            TokenKind::Identifier("User".to_string()),
            TokenKind::LBrace,
            TokenKind::Identifier("name".to_string()),
            TokenKind::Colon,
            TokenKind::Str,
            TokenKind::Comma,
            TokenKind::Identifier("age".to_string()),
            TokenKind::Colon,
            TokenKind::I32,
            TokenKind::RBrace,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_match_expression() {
    let kinds = token_kinds("match x { 1 => 2 }");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Match,
            TokenKind::Identifier("x".to_string()),
            TokenKind::LBrace,
            TokenKind::IntLiteral(1),
            TokenKind::FatArrow,
            TokenKind::IntLiteral(2),
            TokenKind::RBrace,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_null_safety() {
    let kinds = token_kinds("user?.name ?? \"Unknown\"");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("user".to_string()),
            TokenKind::QuestionDot,
            TokenKind::Identifier("name".to_string()),
            TokenKind::DoubleQuestion,
            TokenKind::StringLiteral("Unknown".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_empty_source() {
    let kinds = token_kinds("");
    assert_eq!(kinds, vec![TokenKind::EOF]);
}

#[test]
fn lex_hello_world() {
    let source = "fn main() {\n    println(\"Hello, Sage!\")\n}";
    let kinds = token_kinds(source);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Identifier("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Newline,
            TokenKind::Identifier("println".to_string()),
            TokenKind::LParen,
            TokenKind::StringLiteral("Hello, Sage!".to_string()),
            TokenKind::RParen,
            TokenKind::Newline,
            TokenKind::RBrace,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_error_unexpected_character() {
    let mut lexer = Lexer::new("x $ y");
    lexer.tokenize();
    assert_eq!(lexer.errors().len(), 1);
    assert!(
        lexer.errors()[0]
            .message
            .contains("Unexpected character '$'")
    );
}

#[test]
fn lex_error_unterminated_string() {
    let mut lexer = Lexer::new("\"hello");
    lexer.tokenize();
    assert_eq!(lexer.errors().len(), 1);
    assert!(
        lexer.errors()[0]
            .message
            .contains("Unterminated string literal")
    );
}

#[test]
fn lex_span_tracking() {
    let mut lexer = Lexer::new("let x = 42");
    let tokens = lexer.tokenize();

    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);

    // 'x' is at column 5
    assert_eq!(tokens[1].span.line, 1);
    assert_eq!(tokens[1].span.column, 5);
}

#[test]
fn lex_multiline_span_tracking() {
    let mut lexer = Lexer::new("let x = 1\nlet y = 2");
    let tokens = lexer.tokenize();

    // 'let' on line 2
    let second_let = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Let)
        .nth(1)
        .unwrap();
    assert_eq!(second_let.span.line, 2);
    assert_eq!(second_let.span.column, 1);
}

#[test]
fn lex_ampersand_and_reference() {
    let kinds = token_kinds("&x &mut y");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ampersand,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Ampersand,
            TokenKind::Mut,
            TokenKind::Identifier("y".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_question_mark_operator() {
    let kinds = token_kinds("result?");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("result".to_string()),
            TokenKind::QuestionMark,
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_pipe_for_closure() {
    let kinds = token_kinds("|x| x + 1");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Pipe,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Pipe,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Plus,
            TokenKind::IntLiteral(1),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_escaped_brace_in_string() {
    let kinds = token_kinds("\"hello \\{world}\"");
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello {world}".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_multiple_interpolations() {
    let kinds = token_kinds("\"Hello {first} {last}!\"");
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("Hello ".to_string()),
            TokenKind::InterpolationStart,
            TokenKind::Identifier("first".to_string()),
            TokenKind::InterpolationEnd,
            TokenKind::StringLiteral(" ".to_string()),
            TokenKind::InterpolationStart,
            TokenKind::Identifier("last".to_string()),
            TokenKind::InterpolationEnd,
            TokenKind::StringLiteral("!".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_integer_overflow_reports_error() {
    let mut lexer = Lexer::new("99999999999999999999999999999");
    lexer.tokenize();
    assert_eq!(lexer.errors().len(), 1);
    assert!(lexer.errors()[0].message.contains("too large"));
}

#[test]
fn lex_underscore_identifiers() {
    let kinds = token_kinds("_private __internal _");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("_private".to_string()),
            TokenKind::Identifier("__internal".to_string()),
            TokenKind::Identifier("_".to_string()),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_percent_assign() {
    let kinds = token_kinds("x %= 2");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("x".to_string()),
            TokenKind::PercentAssign,
            TokenKind::IntLiteral(2),
            TokenKind::EOF,
        ]
    );
}

#[test]
fn lex_negative_number_as_unary() {
    let kinds = token_kinds("-42");
    assert_eq!(
        kinds,
        vec![TokenKind::Minus, TokenKind::IntLiteral(42), TokenKind::EOF,]
    );
}
