# Sage - Memory Model

## Philosophy

Hybrid: ARC by default (productive like Python) + opt-in borrow checker (fast like C). 90% of code just works. 10% hot paths get zero overhead.

## Default: Automatic Reference Counting (ARC)

Like Swift. No thinking about memory. Great for apps, MCP servers, agents.

```
fn build_agent() -> Agent {
    let config = Config.load("agent.json")   // reference counted
    let agent = Agent.new(config)            // automatic cleanup
    return agent                             // refcount handles it
}
// Config freed automatically when no one references it.
```

### How ARC Works

- Every object has a hidden counter tracking how many references point to it
- When a reference is created → counter increments
- When a reference goes out of scope → counter decrements
- When counter reaches 0 → object is immediately freed
- No garbage collector pauses, deterministic cleanup

### Cycle Detection

ARC can leak memory with reference cycles. Sage handles this with:

```
struct Node {
    value: i32
    next: Node?         // strong reference (default)
    parent: weak Node?  // weak reference (breaks cycles)
}
```

- `weak` keyword for back-references that don't prevent deallocation
- Compiler warns when it detects potential cycles

## Opt-in: Borrow Checker (for hot paths)

For performance-critical code. Rust-like ownership rules. Zero overhead.

```
@owned
fn matrix_multiply(a: &Tensor, b: &Tensor) -> Tensor {
    let result = Tensor.zeros(a.rows, b.cols)
    parallel for i in 0..a.rows {
        for j in 0..b.cols {
            result[i][j] = dot(a.row(i), b.col(j))
        }
    }
    return result
}
```

### Rules inside `@owned` blocks

- Each value has exactly one owner
- `&` for shared (immutable) references
- `&mut` for exclusive (mutable) references
- Can't have `&mut` and `&` at the same time
- No ARC overhead - compiler tracks lifetimes

## Stack vs Heap

```
let x: i32 = 42              // stack (primitive, always stack)
let point = Point { x: 1, y: 2 }  // stack (small struct, compiler decides)
let data = [i32; 1000000]     // heap (large, compiler decides)
let name: str = "hello"       // stack pointer + heap data
```

- Compiler decides stack vs heap based on size and escape analysis
- Developer doesn't need to think about it (unlike C/C++)
- `@owned` blocks give explicit control when needed

## Summary

| Context | Memory Model | Overhead | Effort |
|---------|-------------|----------|--------|
| Normal code | ARC | Tiny (refcount) | Zero thought |
| Hot paths | Borrow checker | Zero | Rust-like annotations |
| Primitives | Stack | Zero | Automatic |
