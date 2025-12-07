# Contributing to SFX

Thank you for your interest in contributing to SFX (Situation Framework eXchange)! We welcome contributions from the community.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Coding Guidelines](#coding-guidelines)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [License](#license)

---

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow:

- **Be respectful** - Treat everyone with respect and consideration
- **Be collaborative** - Work together and help each other
- **Be inclusive** - Welcome newcomers and diverse perspectives
- **Be constructive** - Provide helpful feedback and criticism
- **Be professional** - Keep discussions focused and on-topic

Unacceptable behavior will not be tolerated. Please report any issues to roriau@gmail.com.

---

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check the [issue tracker](https://github.com/roriau0422/sfex-lang/issues) to avoid duplicates.

**When reporting a bug, include:**
- SFX version (`sfex version`)
- Operating system and version
- Rust version (`rustc --version`)
- Minimal code example that reproduces the issue
- Expected vs actual behavior
- Any error messages or stack traces

### Suggesting Features

We welcome feature suggestions! Please:
- Check existing issues first
- Describe the feature and its use case
- Explain why it would benefit SFX users
- Consider implementation complexity

### Contributing Code

We especially welcome contributions in these areas:

**High Priority:**
- Standard library modules (database, graphics, etc.)
- Performance optimizations (JIT, stdlib)
- Bug fixes and stability improvements
- Documentation and examples

**Medium Priority:**
- Language features (new syntax, operators)
- Tooling (LSP, debugger, REPL)
- Test coverage improvements
- Error messages and diagnostics

**Nice to Have:**
- Package manager functionality
- IDE integrations
- More example programs
- Tutorials and guides

---

## Development Setup

### Prerequisites

- Rust 1.75 or higher
- Git
- A code editor (VS Code with Rust Analyzer recommended)

### Setting Up

```bash
# Clone the repository
git clone https://github.com/roriau0422/sfex-lang.git
cd sfex-lang

# Build the project
cargo build

# Run tests
cargo test

# Run the SFX test suite
cargo run -- run run_tests.sfex

# Install locally (optional)
cargo install --path .
```

### Project Structure

```
sfex-lang/
├── src/
│   ├── compiler/       # Lexer, parser, AST
│   ├── runtime/        # Interpreter and value system
│   ├── jit/            # JIT compiler (Cranelift)
│   ├── stdlib/         # Standard library modules
│   ├── lib.rs          # Public API
│   └── main.rs         # CLI entry point
├── tests/              # Test files
│   ├── core/           # Core language tests
│   ├── oop/            # OOP and context tests
│   ├── io/             # I/O tests
│   ├── concurrency/    # Concurrency tests
│   └── benchmarks/     # Performance benchmarks
├── Cargo.toml          # Dependencies
└── README.md           # Main documentation
```

---

## Pull Request Process

### Before Submitting

1. **Create an issue first** for significant changes
2. **Fork the repository** and create a feature branch
3. **Write tests** for new functionality
5. **Run the test suite** and ensure everything passes
6. **Format your code** with `cargo fmt`
7. **Run the linter** with `cargo clippy`

### Branch Naming

Use descriptive branch names:
- `feature/add-database-module`
- `fix/parser-string-interpolation`
- `docs/update-stdlib-guide`
- `perf/optimize-jit-compilation`

### Commit Messages

Follow this format:

```
Brief summary (50 chars or less)

Detailed explanation of what changed and why.
Include any relevant issue numbers.

- Bullet points for specific changes
- Keep lines under 72 characters
```

**Examples:**

```
Add PostgreSQL database module

Implements a new stdlib module for PostgreSQL database access:
- Connection management with connection pooling
- Query execution with parameterized queries
- Transaction support
- Result mapping to SFX types

Closes #123
```

```
Fix parser crash on nested string interpolation

The parser would crash when encountering nested {var} in strings.
Fixed by properly tracking brace depth during interpolation parsing.

Fixes #456
```

### Pull Request Description

Include:
- **What** - What does this PR do?
- **Why** - Why is this change needed?
- **How** - How does it work?
- **Testing** - What tests were added/updated?
- **Breaking Changes** - Any breaking changes?
- **Checklist** - Mark completed items:
  - [ ] Tests added/updated
  - [ ] Documentation updated
  - [ ] `cargo test` passes
  - [ ] `cargo fmt` applied
  - [ ] `cargo clippy` clean

---

## Coding Guidelines

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` with default settings
- Address all `cargo clippy` warnings
- Write idiomatic Rust code
- Avoid unsafe code unless absolutely necessary

### SFX Language Design

When adding language features:
- **Keep it simple** - Beginners should understand it
- **Be consistent** - Follow existing patterns
- **Be explicit** - Avoid magic or hidden behavior
- **No surprises** - Behavior should be predictable
- **Document well** - Explain with examples

### Performance

- JIT-compiled code should be fast
- Avoid allocations in hot paths
- Use `FastNumber` for performance-critical math
- Profile before optimizing
- Document performance characteristics

### Error Messages

- Be helpful and actionable
- Suggest fixes when possible
- Include context (line numbers, code snippets)
- Use plain language
- Avoid jargon

**Good:**
```
Error at line 42: Cannot divide by zero
  | Result is X / 0
  |              ^ Division by zero
Help: Check that the divisor is not zero before dividing
```

**Bad:**
```
Runtime error: DivisionByZero
```

---

## Testing Guidelines

### Writing Tests

- Test files go in `tests/` with `.sfex` extension
- Organize by category (core, oop, io, etc.)
- Include descriptive comments
- Test both success and error cases
- Keep tests focused and minimal

**Example test structure:**

```sfex
# Test: Feature Name - Brief Description

Story:
    Print "=== Test Category ==="
    Print ""

    # Test 1: Basic functionality
    Print "Test 1: Basic case"
    Result is SomeFunction()
    If Result = ExpectedValue:
        Print "✓ Test 1 passed"
    Else:
        Print "✗ Test 1 failed"

    # Test 2: Edge case
    Print "Test 2: Edge case"
    # ... test code ...
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run SFX test suite
cargo run -- run run_tests.sfex

# Run specific test
cargo run -- run tests/core/math.sfex

# Run benchmarks
cargo run --release -- run tests/benchmarks/bench_physics.sfex
```

### Benchmarks

- Benchmarks go in `tests/benchmarks/`
- Measure realistic workloads
- Compare interpreter vs JIT performance
- Document expected performance

---

## Documentation

### Code Comments

- Document public APIs with doc comments (`///`)
- Explain complex algorithms
- Add TODOs for future improvements
- Keep comments up-to-date

### README.md

Update README.md when:
- Adding major features
- Changing installation process
- Updating performance benchmarks
- Adding new examples

---

## License

By contributing to SFX, you agree that your contributions will be licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.

All contributions must include the following copyright notice in new files:

```rust
// Copyright 2025 Temuujin
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
```

---

## Questions?

- **Email:** roriau@gmail.com
- **Issues:** https://github.com/roriau0422/sfex-lang/issues
- **Discussions:** https://github.com/roriau0422/sfex-lang/discussions

---

Thank you for contributing to SFX! Together we're making programming more intuitive and accessible for everyone. 🚀

*"Programming should be as natural as telling a story"* - SFX
