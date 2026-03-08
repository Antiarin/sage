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

#[test]
fn parser_constructs_without_panic() {
    let _ = parse_ok("");
}

#[test]
fn parser_handles_empty_source() {
    let stmts = parse_ok("");
    assert!(stmts.is_empty());
}
