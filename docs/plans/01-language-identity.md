# Sage - Language Identity

## Overview

| Property | Value |
|----------|-------|
| **Name** | Sage |
| **Extension** | `.sg` |
| **Tagline** | Fast like C. Productive like Python. AI-native. |
| **Inspiration** | Balance of power, speed, and wisdom |
| **Compiler** | Written in Rust, transpiles to C (LLVM backend later) |
| **Paradigm** | Multi-paradigm (imperative, functional, concurrent, AI-native) |

## Ecosystem & CLI

| Command | Purpose |
|---------|---------|
| `sage build` | Compile project |
| `sage run` | Execute program |
| `sage init` | Create new project |
| `sage pkg` | Package manager |
| `sage add` | Install dependencies |
| `sage test` | Run tests |
| `sage fmt` | Format source code |
| `sage check` | Type check only (no codegen) |
| `sage docs` | AI docs/assistant |
| `sage repl` | Interactive mode |

## File Structure

```
project/
├── sage.toml          # project config (like Cargo.toml)
├── src/
│   └── main.sg        # entry point
├── packages/          # dependencies
├── tests/             # tests
└── docs/
```
