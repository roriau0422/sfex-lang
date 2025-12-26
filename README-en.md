# SFX (Situation Framework eXchange)

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://github.com/roriau0422/sfex-lang/actions/workflows/rust.yml/badge.svg)](https://github.com/roriau0422/sfex-lang/actions)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/roriau0422/sfex-lang)](https://github.com/roriau0422/sfex-lang/releases)

A context-oriented programming language I've been building in Rust. The core idea: objects should behave differently based on the current "situation" - like how you act differently at work vs. at home.

```sfex
Story:
    Print "Hello, SFX!"
    
    Numbers is [10, 20, 30]
    Print Numbers[1]  # 10 - yes, 1-based indexing
```

## What is this?

SFX is the first standalone Context-Oriented Programming language. Previous COP implementations (ContextJ, ContextPy, etc.) were all extensions bolted onto existing languages. SFX has native `Situation` and `Switch` syntax built into the language from the ground up.

I started this because I got tired of the ceremony in traditional OOP. Why can't objects just... change behavior based on context? In real life, a `User` object behaves differently when they're an admin vs. a guest. In most languages, you need strategy patterns, dependency injection, or runtime type checks. In SFX:

```sfex
Situation: AdminMode
    Adjust User:
        To GetPermissions:
            Return "admin,write,delete"

Concept: User
    To GetPermissions:
        Return "read"

Story:
    Create User Called Bob
    Print Bob.GetPermissions       # "read"
    
    Switch on AdminMode
    Print Bob.GetPermissions       # "admin,write,delete"
    
    Switch off AdminMode
    Print Bob.GetPermissions       # "read"
```

## Current State

**Working:**
- Lexer/parser with Python-style indentation
- Tree-walking interpreter
- JIT compilation via Cranelift (kicks in after 100 calls)
- Reactive `When` observers
- Standard library: HTTP, WebSocket, TCP, JSON, CSV, XML, HTML, TOML, LLM, File I/O
- Async with `Do in background` and channels
- 1-based indexing, arbitrary precision math

**Newly added:**
- Trace debugger (`sfex debug`)
- Minimal LSP server (stdio diagnostics)
- Project scaffolding (`sfex new`) + package install (`sfex install`)
- Error messages now include line/column hints

## Installation

```bash
git clone https://github.com/roriau0422/sfex-lang.git
cd sfex-lang
cargo build --release
./target/release/sfex run your_script.sfex
```

Requires Rust 1.75+.

## Design Decisions

Some choices I made that might seem weird:

**1-based indexing:** `List[1]` is the first element. Fight me. It's how humans count. Lua does it. R does it. MATLAB does it. You'll survive.

**Arbitrary precision by default:** `0.1 + 0.2 = 0.3` in SFX, not `0.30000000000000004`. If you need speed over precision, use `FastNumber`.

**No null:** Variables default to safe values (0, "", False, []). If you need "absence of value", use `Option` with `Some(x)` or `None`.

**Grapheme-aware strings:** `"👨‍👩‍👧‍👦".Length` is 1, not 7. Because it's one character.

## Syntax Overview

```sfex
# Variables
Name is "Alice"
Age is 25
Items is [1, 2, 3]

# Concepts (like classes)
Concept: Person
    Name, Age
    
    To Greet:
        Print "Hi, I'm " + This.Name

# Control flow
If Age > 18:
    Print "Adult"
Else:
    Print "Minor"

Repeat 10 times:
    Print "Hello"

For each Item in Items:
    Print Item

# Pattern matching
When Score:
    is 100:
        Print "Perfect"
    is 90:
        Print "Great"
    Otherwise:
        Print "OK"
```

## Reactive Observers

This is probably my favorite feature. Define `When` blocks and they fire automatically:

```sfex
Concept: Product
    Price, Tax, Total
    
    When Price changes:
        Set This.Tax to This.Price * 0.1
        Set This.Total to This.Price + This.Tax

Story:
    Create Product Called Phone
    Set Phone.Price to 100
    # Tax is now 10, Total is now 110 - automatically
```

No pub/sub boilerplate. No manual invalidation. It just works.

## Standard Library

| Module | What it does |
|--------|-------------|
| HTTP | GET/POST/PUT/DELETE |
| WebSocket | Bidirectional real-time |
| TCP/UDP | Low-level sockets |
| JSON/XML/HTML/CSV/TOML | Parsing and generation |
| Data | Auto-detect format and parse |
| File | Read/write/stream |
| Env | Environment variables, .env support |
| System | Shell commands |
| Time | Date/time handling |
| Math | Random, trig, rounding |
| LLM | OpenAI API integration |
| Task/Channel | Concurrency primitives |

## Performance

The JIT uses Cranelift. After a function gets called 100 times, it compiles to native code. In my benchmarks on an AMD Ryzen:

- Simple arithmetic loops: ~230M iterations/sec (JIT) vs ~45M (interpreted)
- Fibonacci(30): ~3M calls/sec

Take these numbers with a grain of salt. Microbenchmarks lie. Real-world performance depends on your actual code.

## Why "SFX"?

**S**ituation **F**ramework e**X**change. Also it sounds cool.

## Contributing

This is a solo project but I'd welcome help with:
- Better error messages
- Test coverage
- Documentation
- An LSP implementation would be amazing

File issues at https://github.com/roriau0422/sfex-lang/issues

## License

Apache 2.0

## Contact

Temuujin - roriau@gmail.com

---

*Still early. Things will break. But the core ideas work and I'm actively developing it.*
