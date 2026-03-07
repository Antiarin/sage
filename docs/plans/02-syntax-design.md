# Sage - Syntax Design

## Philosophy

Cherry-pick the best ideas from every language. Familiar to anyone who knows TS, Rust, Python, Go, or Kotlin.

## Variables (from Rust)

```
let name = "Sage"                     // type inferred, immutable
let mut count: i32 = 0                // explicit mutable
let user: User? = find_user(42)       // nullable (from Kotlin)
let name = user?.name ?? "Anonymous"  // safe access + default
```

## Functions (from Rust + Python brevity)

```
fn greet(name: str) -> str {
    return "Hello {name}"             // string interpolation (Kotlin)
}

// Short form for single expression
fn double(x: i32) -> i32 = x * 2
```

## Structs + Traits (from Rust + Java scalability)

```
struct Server {
    host: str
    port: i32
}

trait McpHandler {
    fn handle(self, request: Request) -> Response
}

impl McpHandler for Server {
    fn handle(self, request: Request) -> Response {
        return Response.ok("handled")
    }
}
```

## Pattern Matching (from Rust/Scala)

```
match response.status {
    200..299 => handle_success(response)
    404      => println("Not found")
    500      => retry(request)
    _        => println("Status: {response.status}")
}
```

## Null Safety (from Kotlin/Swift)

```
let user: User? = find_user(42)       // ? means nullable
let name = user?.name ?? "Anonymous"  // safe chain + default
let sure = user!                      // force unwrap (crashes if null)
```

## Error Handling (from Rust + Python)

```
// Result type + ? operator (Rust)
fn read_config(path: str) -> Result<Config, Error> {
    let content = fs.read(path)?          // propagate error with ?
    let config = parse_json(content)?
    return Ok(config)
}

// try/catch for quick scripting (Python/JS)
try {
    let data = fetch("https://api.example.com")?
} catch err {
    println("Failed: {err}")
}
```

## List Comprehensions (from Python)

```
let evens = [x for x in range(10) if x % 2 == 0]
let names = [user.name for user in users if user.active]
```

## Decorators (from Python/TS)

```
@route("/api/users")
fn get_users() -> [User] {
    return db.query("SELECT * FROM users")
}
```

## Extension Methods (from Kotlin/C#)

```
extend str {
    fn is_email(self) -> bool {
        return self.contains("@") && self.contains(".")
    }
}

let valid = "test@sage.dev".is_email()   // true
```

## Built-in Tests (from Go/Rust)

```
test fn test_greet() {
    assert_eq(greet("World"), "Hello World")
}
```

## Misc

- Semicolons: **optional**
- Indentation: **not significant** (curly braces define blocks)
- Comments: `//` single line, `/* */` multi-line
- String interpolation: `"Hello {name}"` with `{}` syntax
