<div align="center">

<img src="assets/sage-mode-naruto.jpg" width="600" alt="Naruto Sage Mode">

# SAGE

### Fast like C. Productive like Python. AI-native.

*Inspired by Sage Mode. Balance of power, speed, and wisdom.*

<br>

```
            ____    _    ____ _____
           / ___|  / \  / ___| ____|
           \___ \ / _ \| |  _|  _|
            ___) / ___ \ |_| | |___
           |____/_/   \_\____|_____|
```

![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)
![Status](https://img.shields.io/badge/status-in%20development-orange)
![Written In](https://img.shields.io/badge/compiler-Rust-red)
![Transpiles To](https://img.shields.io/badge/transpiles%20to-C-green)

</div>

---

## What is Sage?

Sage is a programming language that takes the best ideas from every major language and puts them together. It compiles down to C, so you get native speed without the pain.

| What | How | Stolen From |
|------|-----|-------------|
| C-level speed | Transpiles to C, zero-cost abstractions, no GC pauses | C, Rust, Mojo |
| Python productivity | Clean syntax, type inference, list comprehensions | Python, Kotlin |
| Java scalability | Traits, modules, structured concurrency, lightweight tasks | Java, Go |
| Type safety | Compile-time checks, null safety, pattern matching | Rust, TypeScript |
| Memory safety | ARC by default, opt-in borrow checker for hot paths | Rust, Swift |
| AI-native | Built-in MCP servers, AI agents, LLM calls, tensor math | Nobody did this before |

---

## The Way of the Sage

> *"In the ninja world, those who break the rules are scum. But those who abandon their friends are worse than scum."* - Kakashi

| Principle | What it means |
|-----------|---------------|
| **Sage Mode** | Speed AND productivity. Not one or the other. |
| **Shadow Clones** | Spawn millions of concurrent tasks without breaking a sweat |
| **Sharingan** | The compiler *sees* your bugs before you even run the code |
| **Will of Fire** | Open source, community-driven, built for everyone |

---

## Quick Start

```bash
# Install Sage
curl -sSL https://sage-lang.dev/install | sh

# Create a new project
sage init my-project
cd my-project

# Run your first program
sage run src/main.sg

# Enter Sage Mode (REPL)
sage repl
```

---

## Hello, Sage!

```sage
fn main() {
    println("Hello, Sage!")
}
```

```bash
$ sage run hello.sg
Hello, Sage!
```

---

## Language Tour

### Variables & Types

```sage
let name = "Alice"                        // immutable, type inferred
let mut count: i32 = 100                  // mutable, explicit type
let user: User? = find_user("bob")        // nullable (safe!)
let label = user?.name ?? "Unknown"       // null-safe access
```

### Functions

```sage
fn greet(name: str) -> str {
    return "Hello, {name}!"
}

// short form
fn double(x: i32) -> i32 = x * 2
```

### Structs & Traits

```sage
struct User {
    name: str
    email: str
    age: i32
}

trait Printable {
    fn display(self) -> str
}

impl Printable for User {
    fn display(self) -> str {
        return "{self.name} ({self.email})"
    }
}
```

### Pattern Matching

```sage
match status {
    "active"    => println("User is active")
    "suspended" => println("Account suspended")
    "pending"   => println("Awaiting verification")
    "banned"    => println("Access denied")
    _           => println("Unknown status")
}
```

### Error Handling

```sage
// Result type + ? operator (zero overhead)
fn load_config(path: str) -> Result<Config, Error> {
    let content = fs.read(path)?
    let config = parse(content)?
    return Ok(config)
}

// or use try/catch when you just want stuff to work
try {
    let data = fetch("https://api.example.com/users")?
} catch err {
    println("Request failed: {err}")
}
```

### List Comprehensions

```sage
let active_users = [u.name for u in users if u.is_active]
```

---

## Concurrency

<div align="center">
<img src="assets/shadow-clones.jpg" width="500" alt="Shadow Clone Jutsu">
</div>

<br>

Think of concurrency in Sage like Shadow Clone Jutsu. You can spin up millions of lightweight tasks across all your CPU cores with barely any effort.

### Structured Concurrency (the default way)

```sage
fn gather_results() -> Result<Report, Error> {
    scope |s| {
        let users  = s.spawn(fetch_users())
        let orders = s.spawn(fetch_orders())
        let stats  = s.spawn(fetch_stats())

        // if any task fails, all others auto-cancel
        // when scope exits, all tasks are guaranteed done
        return merge(users.await, orders.await, stats.await)
    }
}
```

### Spawn (fire and forget)

```sage
spawn send_email("Job complete")
```

### Parallel (use all CPU cores)

```sage
let results = parallel data_chunks |chunk| {
    compute_gradient(chunk)    // each chunk runs on a different core
}
```

### Channels (pass data between tasks)

```sage
let ch = Channel<str>.new(buffer: 10)

spawn {
    ch.send("Result from worker")
}

let message = ch.recv()
```

---

## Memory Model

> *"Chakra control is the key to mastering any jutsu."*

Two modes. You pick what fits.

| Mode | What happens | When to use |
|------|-------------|-------------|
| **ARC (default)** | Automatic reference counting, you don't think about memory at all | 90% of your code |
| **Borrow checker** | Opt-in Rust-style ownership, zero overhead | Hot paths, ML training, tight loops |

```sage
// normal code, ARC handles everything
fn build_team() -> Team {
    let members = ["Alice", "Bob", "Charlie"]
    return Team.new(members)    // freed automatically when done
}

// performance-critical stuff, opt into zero-overhead mode
@owned
fn train_model(data: &[Tensor]) -> Model {
    let gradients = parallel data |batch| {
        compute_gradient(batch)
    }
    return optimize(gradients)
}
```

---

## AI-Native

<div align="center">
<img src="assets/sage-of-six-paths.jpg" width="500" alt="Sage of Six Paths">
</div>

<br>

Sage is the first language with built-in support for MCP servers, AI agents, and LLM calls. No external SDKs needed.

### Build MCP Servers

```sage
@mcp_server(name: "product-catalog", version: "1.0")
module CatalogServer {

    @tool(description: "Look up a product by name")
    fn lookup(name: str) -> ProductInfo {
        return db.query("SELECT * FROM products WHERE name = ?", name)
    }

    @resource("product://{name}/details")
    fn product_resource(name: str) -> str {
        return lookup(name).to_json()
    }
}
```

### Create AI Agents

```sage
agent ResearchAgent {
    model: "claude-sonnet-4-20250514"
    tools: [WebSearch, FileRead, Summarize]
    max_steps: 20
    system: "You are a research assistant."
}

fn main() {
    let agent = ResearchAgent.new()
    let result = agent.run("Summarize recent advances in battery technology")
    println(result.output)
}
```

### Call LLMs

```sage
import std.ai

let response = ai.complete(
    model: "claude-sonnet-4-20250514",
    prompt: "Explain how transformers work",
    max_tokens: 500
)
println(response.text)
```

---

## CLI

```bash
sage build main.sg       # compile
sage run main.sg         # compile and run
sage repl                # interactive mode
sage init                # create new project
sage add <pkg>           # install a dependency
sage test                # run tests
sage fmt                 # format source files
sage check               # type-check without compiling
sage docs                # AI-powered docs
```

---

## Compiler

```
Source (.sg) -> Lexer -> Parser -> Type Checker -> C Codegen -> GCC/Clang -> Native Binary
```

The compiler is written in Rust and transpiles Sage to C. The C compiler (GCC or Clang) handles the final optimization and binary generation. An LLVM backend is planned for Phase 2, which will unlock JIT compilation, WebAssembly, and GPU support.

---

## Roadmap

| Phase | Status | What |
|-------|--------|------|
| 0 | Planned | Project setup, CLI scaffold |
| 1 | Planned | Lexer |
| 2 | Planned | Parser |
| 3 | Planned | Type checker |
| 4 | Planned | C code generation + runtime |
| 5 | Planned | **Hello World end-to-end** |
| 6 | Planned | REPL |
| 7 | Planned | Standard library |
| 8 | Planned | Concurrency runtime |
| 9 | Planned | Package manager (`sage add`) |
| 10 | Planned | AI / MCP integration |
| 11 | Planned | Test framework (`sage test`) |
| 12 | Future | LLVM backend |
| 13 | Future | Self-hosting (Sage compiles Sage) |

Full step-by-step plan in [IMPLEMENTATION.md](./IMPLEMENTATION.md).

---

## Design Docs

- [Language Identity](./docs/plans/01-language-identity.md)
- [Syntax Design](./docs/plans/02-syntax-design.md)
- [Concurrency Model](./docs/plans/03-concurrency-model.md)
- [Memory Model](./docs/plans/04-memory-model.md)
- [AI/MCP Integration](./docs/plans/05-ai-mcp-integration.md)
- [Compiler Architecture](./docs/plans/06-compiler-architecture.md)
- [Design Summary](./docs/plans/07-design-summary.md)

---

## Contributing

Sage is in early development and we're looking for contributors.

1. Fork the repo
2. Create your branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -m "Add cool new feature"`)
4. Push (`git push origin feature/my-feature`)
5. Open a Pull Request

---

## License

Dual-licensed under [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE).

---

<div align="center">

<img src="assets/naruto-believe-it.jpg" width="500" alt="Believe it!">

<br><br>

*"I'm not gonna run away. I never go back on my word. That's my nindo, my ninja way!"*

**Sage** - the language that never gives up.

</div>
