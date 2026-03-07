# Sage - Compiler Architecture

## Implementation Language

**Rust** - chosen for safety, pattern matching (perfect for AST), and Cargo tooling.

## Compilation Pipeline

```
Source (.sg)
    │
    ▼
┌──────────┐
│  LEXER   │  Tokenizes source code into tokens
│          │  (keywords, identifiers, literals, operators)
└────┬─────┘
     │ Token stream
     ▼
┌──────────┐
│  PARSER  │  Builds Abstract Syntax Tree (AST)
│          │  Recursive descent parser
└────┬─────┘
     │ AST
     ▼
┌──────────────┐
│  TYPE CHECK  │  Type inference, type safety verification
│  + ANALYZE   │  Null safety checks, borrow checking (@owned)
└────┬─────────┘
     │ Typed AST
     ▼
┌──────────────┐
│  C CODEGEN   │  Phase 1: Transpile to C code
│  (Phase 1)   │  Maps Sage constructs to C equivalents
└────┬─────────┘
     │ Generated .c files
     ▼
┌──────────────┐
│  GCC/Clang   │  Compiles C to native binary
│              │  Optimizations handled by C compiler
└────┬─────────┘
     │
     ▼
  Native Binary
```

## Phase 2: LLVM Backend (Future)

```
     Typed AST
         │
         ├──────────────────┐
         ▼                  ▼
┌──────────────┐   ┌──────────────┐
│  C CODEGEN   │   │  LLVM CODEGEN│
│  (Phase 1)   │   │  (Phase 2)   │
└──────────────┘   └────┬─────────┘
                        │ LLVM IR
                        ▼
                 ┌──────────────┐
                 │  LLVM        │
                 │  Optimizer   │
                 │  + Backend   │
                 └────┬─────────┘
                      │
                      ▼
               Native Binary / WASM / GPU
```

## Phase 3: Self-Hosting (Future)

Rewrite the Sage compiler in Sage itself.

```
Phase 1: Rust compiler → generates C → binary
Phase 2: Rust compiler → generates LLVM IR → binary
Phase 3: Sage compiler (written in Sage) → compiles itself
```

## AST Representation (in Rust)

```rust
enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Variable(String),
    BinaryOp { left: Box<Expr>, op: Op, right: Box<Expr> },
    UnaryOp { op: Op, expr: Box<Expr> },
    FnCall { name: String, args: Vec<Expr> },
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    Spawn(Box<Expr>),
    Parallel { collection: Box<Expr>, param: String, body: Box<Expr> },
    Match { expr: Box<Expr>, arms: Vec<MatchArm> },
    Scope(Vec<Stmt>),
    IfElse { condition: Box<Expr>, then: Box<Expr>, else_: Option<Box<Expr>> },
    ListComprehension { expr: Box<Expr>, var: String, iter: Box<Expr>, filter: Option<Box<Expr>> },
}

enum Stmt {
    Let { name: String, mutable: bool, type_: Option<Type>, value: Expr },
    FnDef { name: String, params: Vec<Param>, return_type: Type, body: Vec<Stmt> },
    StructDef { name: String, fields: Vec<Field> },
    TraitDef { name: String, methods: Vec<FnSignature> },
    ImplBlock { trait_: String, for_: String, methods: Vec<Stmt> },
    AgentDef { name: String, config: AgentConfig, handlers: Vec<Stmt> },
    McpServer { name: String, version: String, tools: Vec<Stmt> },
    Return(Expr),
    Expression(Expr),
}
```

## REPL Architecture

Interpreter-based REPL that maintains state across inputs.

```
$ sage repl
Sage v0.1.0 - Interactive Mode (Sage Mode activated)

>>> let x = 42
>>> x * 2
84

>>> :type x        // inspect type
i32

>>> :time fib(30)  // benchmark
832040 (0.003ms)
```

## Build System

```toml
# sage.toml
[project]
name = "my-mcp-server"
version = "0.1.0"
sage = "0.1.0"

[dependencies]
http = "1.0"
json = "2.1"

[test]
parallel = true
timeout = 30
```
