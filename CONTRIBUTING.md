# Contributing to Sage

Thanks for wanting to help build Sage. Whether you're fixing a typo, adding a feature to the compiler, or building something on top of the language, every contribution matters.

## Before You Start

### Claim an issue first

To avoid stepping on someone else's work:

1. Browse the [open issues](https://github.com/Antiarin/sage/issues) or create a new one
2. Drop a comment saying you want to work on it
3. Wait for a maintainer to assign it to you
4. Start building

PRs for unassigned issues might get delayed or closed if someone else was already working on it.

### Exceptions

You don't need to claim an issue for:

- **Docs fixes** - typos, clarifications, broken links. Just submit the PR.
- **Micro-fixes** - less than 20 lines, no logic changes, no new features. Label it `micro-fix`.

## Getting Started

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/sage.git
cd sage

# Add upstream
git remote add upstream git@github.com:Antiarin/sage.git

# Sync with latest
git fetch upstream
git checkout main
git merge upstream/main

# Create your branch
git checkout -b feature/your-thing

# Build the compiler
cargo build

# Run tests
cargo test
```

## Project Structure

```
sage/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lexer/           # Tokenizer
│   ├── parser/          # AST builder
│   ├── typechecker/     # Type analysis
│   ├── codegen/         # C code generation
│   └── runtime/         # Runtime (ARC, scheduler)
├── tests/               # Integration tests
├── examples/            # Example .sg files
├── docs/plans/          # Design documents
└── sage.toml            # Project config
```

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/). Keep it simple.

```
type(scope): what you did
```

**Types:**

| Type | When |
|------|------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation |
| `refactor` | Restructuring code without changing behavior |
| `test` | Adding or fixing tests |
| `chore` | Build, CI, tooling changes |

**Examples:**

```
feat(lexer): add string interpolation tokenization
fix(parser): handle trailing comma in function args
docs(readme): fix broken image link
test(codegen): add tests for struct generation
```

## Pull Request Process

1. Make sure you're assigned to the issue
2. Write tests for new stuff
3. Run `cargo test` and make sure everything passes
4. Run `cargo clippy` for lint checks
5. Keep the PR focused. One thing per PR.
6. Write a clear description of what you changed and why

### PR Title

Same format as commits:

```
feat(parser): support pattern matching expressions
```

## Code Style

- Rust 2021 edition
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Name things clearly. If a variable name needs a comment to explain it, rename the variable.
- Keep functions small and focused
- Write tests for anything that could break

## What We Need Help With

Here's where contributions would make the biggest impact:

| Area | Difficulty | What's needed |
|------|-----------|---------------|
| **Lexer** | Beginner | Token types, edge cases, better error messages |
| **Parser** | Intermediate | New syntax constructs, error recovery |
| **Type checker** | Advanced | Type inference, null safety analysis |
| **C codegen** | Advanced | Translating AST nodes to C code |
| **Runtime** | Advanced | ARC implementation, task scheduler |
| **Std library** | All levels | String ops, file I/O, JSON, HTTP |
| **Examples** | Beginner | Write cool `.sg` programs |
| **Docs** | Beginner | Tutorials, fix errors, improve clarity |
| **Tests** | All levels | More test coverage everywhere |

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test lexer
cargo test parser

# With output
cargo test -- --nocapture
```

## Design Docs

Before making big changes, read the relevant design doc in `docs/plans/`. If your change goes against the design, open an issue to discuss it first. We're open to changes, but want to talk about them before code gets written.

## License

By submitting a PR, you agree that your code will be licensed under the same dual MIT/Apache 2.0 license as the rest of the project.

## Questions?

Open an issue. No question is too basic.
