# Sage - Implementation Plan

**Date:** 2026-03-07
**Language:** Rust (compiler) → transpiles to C → native binary
**Goal:** Working Sage compiler that can compile `.sg` files

---

## Phase 0: Project Setup
**Goal:** Rust project scaffold, CI, basic tooling

- [ ] **Step 0.1** - Initialize Rust project with Cargo
  ```
  sage/
  ├── Cargo.toml
  ├── src/
  │   ├── main.rs          # CLI entry point (sage build, sage run, etc.)
  │   ├── lib.rs           # Library root
  │   ├── lexer/           # Tokenizer
  │   ├── parser/          # AST builder
  │   ├── typechecker/     # Type analysis
  │   ├── codegen/         # C code generation
  │   └── runtime/         # Runtime support (ARC, scheduler stubs)
  ├── tests/               # Integration tests
  ├── examples/            # Example .sg files
  └── docs/plans/          # Design docs (already done)
  ```
- [ ] **Step 0.2** - Set up basic CLI with `clap` crate
  - `sage build <file.sg>` - compile a file
  - `sage run <file.sg>` - compile and execute
  - `sage repl` - interactive mode (stub)
  - `sage version` - print version
- [ ] **Step 0.3** - Create first example: `examples/hello.sg`
  ```
  fn main() {
      println("Hello, Sage!")
  }
  ```
- [ ] **Step 0.4** - Set up test infrastructure with `cargo test`
- [ ] **Step 0.5** - Initialize git repo with `.gitignore`

---

## Phase 1: Lexer (Tokenizer)
**Goal:** Turn source code into a stream of tokens

- [ ] **Step 1.1** - Define token types enum
  ```rust
  enum Token {
      // Literals
      IntLiteral(i64),
      FloatLiteral(f64),
      StringLiteral(String),
      BoolLiteral(bool),

      // Identifiers & Keywords
      Identifier(String),
      Fn, Let, Mut, Return, If, Else, Match,
      For, While, In, Spawn, Parallel, Scope,
      Struct, Trait, Impl, Test, Import,
      Try, Catch, Async, Await,

      // Types
      I32, I64, F32, F64, Str, Bool,

      // Operators
      Plus, Minus, Star, Slash, Percent,
      Eq, NotEq, Lt, Gt, LtEq, GtEq,
      And, Or, Not, Assign,
      Arrow,        // ->
      FatArrow,     // =>
      QuestionMark, // ?
      Dot, DotDot,  // . and ..

      // Delimiters
      LParen, RParen, LBrace, RBrace,
      LBracket, RBracket,
      Comma, Colon, Semicolon,

      // Special
      At,           // @ for decorators
      Pipe,         // | for closures
      Newline,
      EOF,
  }
  ```
- [ ] **Step 1.2** - Implement lexer struct with source tracking (line, column, span)
- [ ] **Step 1.3** - Implement tokenization logic
  - Number literals (int, float)
  - String literals with interpolation `"Hello {name}"`
  - Identifiers and keyword detection
  - Operators (single and multi-char: `->`, `=>`, `..`, `?`)
  - Comments (`//` and `/* */`)
  - Skip whitespace, track newlines
- [ ] **Step 1.4** - Error reporting with line/column info
  ```
  Error: Unexpected character '$' at line 3, column 12
  ```
- [ ] **Step 1.5** - Write tests for lexer
  - Tokenize simple expressions
  - Tokenize function definitions
  - Tokenize string interpolation
  - Edge cases: nested comments, unicode identifiers
  - Error cases: unterminated strings, invalid characters

---

## Phase 2: Parser (AST Builder)
**Goal:** Turn token stream into an Abstract Syntax Tree

- [ ] **Step 2.1** - Define AST node types
  ```rust
  enum Expr {
      IntLiteral(i64),
      FloatLiteral(f64),
      StringLiteral(String),
      BoolLiteral(bool),
      Identifier(String),
      BinaryOp { left: Box<Expr>, op: BinOp, right: Box<Expr> },
      UnaryOp { op: UnOp, expr: Box<Expr> },
      FnCall { name: String, args: Vec<Expr> },
      MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
      FieldAccess { object: Box<Expr>, field: String },
      Index { object: Box<Expr>, index: Box<Expr> },
      Match { expr: Box<Expr>, arms: Vec<MatchArm> },
      IfElse { condition: Box<Expr>, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>> },
      Closure { params: Vec<Param>, body: Vec<Stmt> },
      ListLiteral(Vec<Expr>),
      ListComprehension { expr: Box<Expr>, var: String, iter: Box<Expr>, filter: Option<Box<Expr>> },
      Spawn(Box<Expr>),
      Parallel { collection: Box<Expr>, param: String, body: Box<Expr> },
      Scope { body: Vec<Stmt> },
      Await(Box<Expr>),
      NullCoalesce { left: Box<Expr>, right: Box<Expr> },
      SafeAccess { object: Box<Expr>, field: String },
  }

  enum Stmt {
      Let { name: String, mutable: bool, type_ann: Option<Type>, value: Expr },
      FnDef { name: String, params: Vec<Param>, return_type: Option<Type>, body: Vec<Stmt>, decorators: Vec<Decorator> },
      StructDef { name: String, fields: Vec<Field> },
      TraitDef { name: String, methods: Vec<FnSignature> },
      ImplBlock { trait_name: Option<String>, target: String, methods: Vec<Stmt> },
      Return(Option<Expr>),
      Expression(Expr),
      ForLoop { var: String, iter: Expr, body: Vec<Stmt> },
      WhileLoop { condition: Expr, body: Vec<Stmt> },
      Import { path: Vec<String> },
      TryCatch { try_block: Vec<Stmt>, catch_var: String, catch_block: Vec<Stmt> },
      TestFn { name: String, body: Vec<Stmt> },
      AgentDef { name: String, config: Vec<(String, Expr)>, handlers: Vec<Stmt> },
  }
  ```
- [ ] **Step 2.2** - Implement recursive descent parser
  - Expression parsing with operator precedence (Pratt parsing)
  - Statement parsing
  - Function definitions
  - Struct and trait definitions
  - Match expressions
  - Import statements
- [ ] **Step 2.3** - Parse decorators (`@tool`, `@mcp_server`, etc.)
- [ ] **Step 2.4** - Parse concurrency constructs (`spawn`, `scope`, `parallel`)
- [ ] **Step 2.5** - Error recovery - continue parsing after errors to report multiple issues
- [ ] **Step 2.6** - Write tests for parser
  - Parse hello world
  - Parse functions with types
  - Parse structs and traits
  - Parse match expressions
  - Parse spawn/scope/parallel
  - Error cases: missing braces, unexpected tokens

---

## Phase 3: Type Checker & Semantic Analysis
**Goal:** Validate types, resolve names, check safety

- [ ] **Step 3.1** - Build symbol table / scope management
  - Variable scoping (block scope)
  - Function signatures
  - Struct and trait definitions
  - Import resolution
- [ ] **Step 3.2** - Implement type inference
  - `let x = 42` → infer `i32`
  - `let name = "hello"` → infer `str`
  - Function return type inference
- [ ] **Step 3.3** - Type checking
  - Binary operations: both sides must match
  - Function call argument types vs parameter types
  - Return type matches function signature
  - Struct field types
- [ ] **Step 3.4** - Null safety checking
  - `User?` vs `User` - track nullable types
  - Require `?.` or null check before accessing nullable
  - `??` operator type validation
- [ ] **Step 3.5** - Pattern match exhaustiveness checking
  - Warn on non-exhaustive match (missing `_` or variants)
- [ ] **Step 3.6** - Basic borrow/ownership analysis for `@owned` blocks
  - Track ownership transfer
  - Validate `&` and `&mut` references
  - Error on use-after-move
- [ ] **Step 3.7** - Write tests for type checker

---

## Phase 4: C Code Generation
**Goal:** Transpile typed AST to C code

- [ ] **Step 4.1** - Set up C code emitter
  - String builder for generated C
  - Indent management
  - Include standard headers
- [ ] **Step 4.2** - Generate C for basic constructs
  ```
  Sage                          →  C
  ─────────────────────────────────────────────
  let x: i32 = 42              →  int32_t x = 42;
  let name: str = "hello"      →  sage_str name = sage_str_new("hello");
  fn add(a: i32, b: i32)->i32  →  int32_t add(int32_t a, int32_t b)
  println("Hello {x}")         →  printf("Hello %d\n", x);
  if x > 0 { ... }             →  if (x > 0) { ... }
  for i in 0..10 { ... }       →  for (int32_t i=0; i<10; i++) { ... }
  ```
- [ ] **Step 4.3** - Generate C for structs
  ```
  struct User { name: str, age: i32 }
  →
  typedef struct { sage_str name; int32_t age; } User;
  ```
- [ ] **Step 4.4** - Generate C for function calls and methods
- [ ] **Step 4.5** - Generate C for match expressions → switch/if-else chains
- [ ] **Step 4.6** - Generate C for string interpolation → sprintf/snprintf calls
- [ ] **Step 4.7** - Write Sage runtime library in C
  - `sage_str` - string type with length tracking
  - `sage_vec` - dynamic array
  - `sage_rc` - reference counting (ARC implementation)
  - `sage_print` - println implementation
  - `sage_result` - Result<T,E> type
- [ ] **Step 4.8** - Invoke GCC/Clang from Rust to compile generated C
  - Use `cc` crate or `std::process::Command`
  - Link runtime library
  - Output native binary
- [ ] **Step 4.9** - Write end-to-end tests
  - Compile `.sg` → C → binary → run → check output

---

## Phase 5: Hello World - End to End
**Goal:** `sage run examples/hello.sg` prints "Hello, Sage!"

- [ ] **Step 5.1** - Wire together: CLI → Lexer → Parser → TypeCheck → Codegen → Compile → Run
- [ ] **Step 5.2** - Get `hello.sg` working end to end
  ```
  fn main() {
      println("Hello, Sage!")
  }
  ```
- [ ] **Step 5.3** - Get variables and arithmetic working
  ```
  fn main() {
      let x = 10
      let y = 20
      println("Sum: {x + y}")
  }
  ```
- [ ] **Step 5.4** - Get functions working
  ```
  fn add(a: i32, b: i32) -> i32 {
      return a + b
  }
  fn main() {
      println("Result: {add(3, 4)}")
  }
  ```
- [ ] **Step 5.5** - Get if/else and loops working
- [ ] **Step 5.6** - Get structs working
- [ ] **Step 5.7** - Celebrate the first working Sage program

---

## Phase 6: REPL
**Goal:** `sage repl` - interactive Sage Mode

- [ ] **Step 6.1** - Build interpreter-based REPL loop (read → eval → print)
- [ ] **Step 6.2** - Maintain state across inputs (variables persist)
- [ ] **Step 6.3** - Multi-line input support (detect unclosed `{`)
- [ ] **Step 6.4** - REPL commands: `:type`, `:time`, `:quit`
- [ ] **Step 6.5** - Line editing with `rustyline` crate (history, arrow keys)

---

## Phase 7: Standard Library (Core)
**Goal:** Basic standard library modules

- [ ] **Step 7.1** - `std.io` - file read/write, stdin/stdout
- [ ] **Step 7.2** - `std.str` - string manipulation (split, trim, contains, replace)
- [ ] **Step 7.3** - `std.collections` - Vec, HashMap, HashSet
- [ ] **Step 7.4** - `std.math` - basic math functions
- [ ] **Step 7.5** - `std.json` - JSON parse/serialize
- [ ] **Step 7.6** - `std.http` - basic HTTP client
- [ ] **Step 7.7** - Error types and Result<T,E> implementation

---

## Phase 8: Concurrency Runtime
**Goal:** spawn, scope, parallel, channels working

- [ ] **Step 8.1** - Implement lightweight task struct (~4KB stack)
- [ ] **Step 8.2** - Implement work-stealing scheduler (thread pool = CPU cores)
- [ ] **Step 8.3** - `spawn` - create lightweight task, return handle
- [ ] **Step 8.4** - `scope` - structured concurrency with auto-cancellation
- [ ] **Step 8.5** - `parallel` - split work across cores
- [ ] **Step 8.6** - `Channel<T>` - buffered/unbuffered channels
- [ ] **Step 8.7** - `Mutex<T>`, `Atomic<T>` - safe shared state
- [ ] **Step 8.8** - `@supervised` - supervision trees with restart policies
- [ ] **Step 8.9** - Generate C code that uses pthreads / platform threading

---

## Phase 9: Package Manager (`sage pkg`)
**Goal:** Dependency management and package distribution

- [ ] **Step 9.1** - `sage.toml` config parsing
- [ ] **Step 9.2** - `sage init` - create new project
- [ ] **Step 9.3** - `sage add <package>` - download and install packages
- [ ] **Step 9.4** - Dependency resolution (version constraints)
- [ ] **Step 9.5** - Package registry (start with GitHub-based)
- [ ] **Step 9.6** - `sage pkg publish` - publish a package

---

## Phase 10: AI / MCP Integration
**Goal:** First-class AI primitives working

- [ ] **Step 10.1** - `std.ai` - LLM API calls (HTTP-based)
- [ ] **Step 10.2** - `std.mcp` - MCP protocol implementation
- [ ] **Step 10.3** - `@mcp_server` decorator → generates MCP-compliant server
- [ ] **Step 10.4** - `@tool`, `@resource`, `@prompt` decorators
- [ ] **Step 10.5** - `agent` keyword → agent type with lifecycle hooks
- [ ] **Step 10.6** - `std.tensor` - basic tensor operations
- [ ] **Step 10.7** - Streaming support for LLM responses

---

## Phase 11: Testing Framework (`sage test`)
**Goal:** Built-in test runner

- [ ] **Step 11.1** - Discover `test fn` blocks in source files
- [ ] **Step 11.2** - `assert_eq`, `assert_ne`, `assert` built-ins
- [ ] **Step 11.3** - `sage test` CLI - run all tests, show results
- [ ] **Step 11.4** - `sage test --parallel` - run tests on all cores
- [ ] **Step 11.5** - Test coverage reporting

---

## Phase 12: LLVM Backend (Future)
**Goal:** Direct LLVM IR generation for better optimization

- [ ] **Step 12.1** - Add LLVM as optional backend (`sage build --backend=llvm`)
- [ ] **Step 12.2** - Emit LLVM IR from typed AST (same frontend, new backend)
- [ ] **Step 12.3** - JIT compilation for REPL
- [ ] **Step 12.4** - WebAssembly output target
- [ ] **Step 12.5** - GPU code generation (CUDA/ROCm)

---

## Phase 13: Self-Hosting (Future)
**Goal:** Rewrite the Sage compiler in Sage

- [ ] **Step 13.1** - Sage is mature enough to write a compiler
- [ ] **Step 13.2** - Port lexer from Rust to Sage
- [ ] **Step 13.3** - Port parser from Rust to Sage
- [ ] **Step 13.4** - Port type checker from Rust to Sage
- [ ] **Step 13.5** - Port codegen from Rust to Sage
- [ ] **Step 13.6** - Sage compiler compiles itself

---

## Build Order (What to do first)

```
Phase 0 ─→ Phase 1 ─→ Phase 2 ─→ Phase 3 ─→ Phase 4 ─→ Phase 5 (HELLO WORLD!)
                                                              │
                                                              ▼
                                                         Phase 6 (REPL)
                                                              │
                                              ┌───────────────┼───────────────┐
                                              ▼               ▼               ▼
                                         Phase 7         Phase 8         Phase 9
                                        (std lib)     (concurrency)   (pkg manager)
                                              │               │               │
                                              └───────┬───────┘               │
                                                      ▼                       │
                                                 Phase 10                     │
                                                (AI / MCP)                    │
                                                      │                       │
                                                      ├───────────────────────┘
                                                      ▼
                                                 Phase 11
                                               (test framework)
                                                      │
                                                      ▼
                                                 Phase 12
                                               (LLVM backend)
                                                      │
                                                      ▼
                                                 Phase 13
                                               (self-hosting)
```

## Current Status

- [x] Design documents complete
- [x] Design verified against web research
- [ ] Phase 0: Project Setup ← **START HERE**
