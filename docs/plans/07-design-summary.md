# Sage - Complete Design Summary

**Date:** 2026-03-07

## What is Sage?

Sage is a new programming language that combines:
- **Speed of C** - transpiles to C, zero-cost abstractions
- **Productivity of Python** - clean syntax, type inference, list comprehensions
- **Scalability of Java** - traits, modules, structured concurrency
- **Type safety** - compile-time type checking, null safety, pattern matching
- **Memory safety** - ARC by default, opt-in borrow checker for hot paths
- **AI-native** - first-class MCP servers, agents, LLM calls, tensor math

## Decisions Summary

| Decision | Choice |
|----------|--------|
| Name | Sage (`.sg`) |
| Compilation | Transpile to C (Phase 1) → LLVM (Phase 2) → Self-host (Phase 3) |
| Syntax | Best-of-all-languages hybrid (Rust + TS + Python + Go + Kotlin) |
| Concurrency | 6-layer hybrid: spawn, scope, colorblind, parallel, channels, supervision |
| Memory | ARC default + opt-in borrow checker (`@owned`) |
| Error handling | `Result<T,E>` + `?` operator + `try/catch` |
| AI/MCP | First-class primitives (`@mcp_server`, `agent`, `std.ai`, `std.tensor`) |
| Compiler written in | Rust |
| REPL | Yes, interpreter-based |
| Package manager | `sage pkg` (built-in) |
| Testing | `sage test` (built-in, `test fn`) |

## Design Documents

1. [Language Identity](./01-language-identity.md)
2. [Syntax Design](./02-syntax-design.md)
3. [Concurrency Model](./03-concurrency-model.md)
4. [Memory Model](./04-memory-model.md)
5. [AI/MCP Integration](./05-ai-mcp-integration.md)
6. [Compiler Architecture](./06-compiler-architecture.md)

## Hello World

```
fn main() {
    println("Hello, Sage!")
}
```

## What Sage Looks Like in Practice

```
import std.ai
import std.mcp

@mcp_server(name: "research-assistant", version: "1.0")
module ResearchServer {

    agent Researcher {
        model: "claude-sonnet-4-20250514"
        tools: [WebSearch, Summarize]
        max_steps: 10
        system: "You are a research assistant."
    }

    @tool(description: "Research a topic and return a summary")
    fn research(topic: str) -> str {
        let agent = Researcher.new()
        let result = agent.run("Research: {topic}")
        return result.output
    }

    @tool(description: "Summarize text")
    fn summarize(text: str) -> str {
        return ai.complete(
            model: "claude-sonnet-4-20250514",
            prompt: "Summarize concisely: {text}",
            max_tokens: 200
        ).text
    }
}

fn main() {
    ResearchServer.start(port: 3000)
    println("MCP Server running on port 3000")
}
```
