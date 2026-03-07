# Sage - Concurrency Model

## Philosophy

6-layer hybrid model. Take the best from Go, Rust, Kotlin, Zig, Mojo, and Erlang. No JS-style microtask/macrotask queues. No colored function problem.

**Core principle:** Structured concurrency (`scope`) is the PRIMARY pattern. Raw `spawn` is the escape hatch for fire-and-forget. This ensures tasks never leak.

## User Decision Rule

```
"Do something in background"      → spawn { ... }
"Group tasks, cancel if one fails" → scope |s| { ... }
"Use all CPU cores"               → parallel x |i| { ... }
"Send data between tasks"         → Channel<T>
"Process 8 numbers at once"       → simd[f32, 8]
"Auto-restart on crash"           → @supervised
"Just normal code"                → write it normally
```

## Layer 1: Structured Concurrency - THE DEFAULT (from Kotlin)

The primary way to do concurrency. Parent-child task relationships. Children can't outlive parents. Automatic cancellation. No task leaks.

```
fn fetch_all() -> Result<Data, Error> {
    scope |s| {
        let a = s.spawn(fetch("api1"))
        let b = s.spawn(fetch("api2"))
        let c = s.spawn(fetch("api3"))

        // If ANY fails → all others auto-cancel
        // When scope exits → all children guaranteed done
        return merge(a.await, b.await, c.await)
    }
    // ← Nothing leaks past here. Ever.
}
```

## Layer 2: Raw Spawn - Escape Hatch (from Go)

Unstructured, fire-and-forget. Lightweight tasks (~4KB each). Use when you explicitly don't need lifecycle management.

```
fn main() {
    spawn fetch("api1")                // fire and forget
    let handle = spawn fetch("api2")   // get a handle
    let result = handle.await           // wait for result
}
```

## Layer 3: Colorblind Async (from Zig)

No colored functions. Same function works sync AND async. Runtime decides.

```
fn read_file(path: str) -> Result<str, Error> {
    return fs.read(path)    // runtime decides sync vs async
}

// Works synchronously:
let data = read_file("config.json")

// Also works in concurrent context:
spawn read_file("config.json")
```

## Layer 4: CPU Parallelism (from Mojo + Rust Rayon)

Use all cores for heavy compute.

```
fn process_data(items: [Tensor]) -> [Tensor] {
    return parallel items |item| {
        compute_gradient(item)
    }
}

// SIMD for single-core vectorization
fn dot_product(a: simd[f32, 8], b: simd[f32, 8]) -> f32 {
    return (a * b).reduce_add()
}
```

## Layer 5: Channels (from Go)

Safe communication between tasks.

```
fn pipeline() {
    let ch = Channel<i32>.new(buffer: 100)

    spawn {
        for i in 0..1000 {
            ch.send(i)
        }
        ch.close()
    }

    for val in ch {
        println("Got: {val}")
    }
}
```

## Layer 6: Supervision (from Erlang)

Crashed tasks auto-restart.

```
@supervised(restart: .on_failure, max_retries: 3)
fn mcp_worker(request: Request) -> Response {
    return process(request)
}
```

## Runtime Architecture

```
┌─────────────────────────────────────────────────┐
│              SAGE RUNTIME                        │
│                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ OS Thread│ │ OS Thread│ │ OS Thread│ (= CPU  │
│  │ (Core 1) │ │ (Core 2) │ │ (Core 3) │  cores) │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘        │
│       │             │             │              │
│  ┌────┴────┐   ┌────┴────┐  ┌────┴────┐        │
│  │Task Task│   │Task Task│  │Task Task│         │
│  │Task Task│   │Task Task│  │Task Task│         │
│  │Task Task│   │Task     │  │Task Task│         │
│  └─────────┘   └─────────┘  └─────────┘        │
│                                                  │
│  Work-stealing scheduler: idle cores steal tasks │
│  from busy cores automatically                   │
└─────────────────────────────────────────────────┘
```

- One global task queue + per-core local queues
- Work-stealing: idle cores steal from busy cores
- `await` yields the task, core picks up other work
- No single-thread bottleneck (unlike JS event loop)
