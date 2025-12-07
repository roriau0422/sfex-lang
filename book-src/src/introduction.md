# Introduction

> **"Programming the way humans think, not the way computers think"**

Welcome to SFX (Situation Framework eXchange) - a beginner-friendly, context-oriented programming language designed for the 2025 AI era. With **JIT compilation**, **reactive observers**, and **mathematical honesty**, SFX makes programming intuitive and powerful.

## What is SFX?

SFX is a programming language that fixes 50 years of accumulated lies in programming:

| The Lie | The Truth (SFX) |
|---------|----------------|
| `0.1 + 0.2 ≠ 0.3` | **0.1 + 0.2 = 0.3** (arbitrary precision) |
| List[0] is first | **List[1] is first** (1-based indexing) |
| Null pointer errors | **No null** (safe defaults: 0, "", False, []) |
| "👨‍👩‍👧‍👦".Length = 7 | **"👨‍👩‍👧‍👦".Length = 1** (grapheme clustering) |
| Objects are static | **Context-oriented** (Situations modify behavior) |
| Manual cache updates | **Reactive observers** (self-healing data) |

## Your First SFX Program

```sfex
Story:
    Print "Hello, SFX!"

    # Mathematical honesty - no float errors!
    Result is 0.1 + 0.2
    Print Result  # 0.3 ✓

    # 1-based indexing - List[1] is first
    Numbers is [10, 20, 30]
    First is Numbers[1]  # 10 ✓
    Print "First number: " + First
```

## Key Features at a Glance

- **JIT Compilation**: Automatic 2-5x performance boost with Cranelift
- **Reactive Observers**: Self-healing data that maintains consistency automatically
- **Context-Oriented**: Objects behave differently in different Situations
- **Mathematical Honesty**: Arbitrary precision arithmetic by default
- **1-Based Indexing**: Lists start at 1, not 0
- **No Null Pointers**: Safe defaults prevent null reference errors
- **Grapheme Clustering**: Emoji counted correctly as single characters
- **Powerful Standard Library**: Data parsing, networking, concurrency, LLM integration

## Current Version

**Version 0.3.2** - All core features complete including:
- Core language with interpreter
- Full standard library (21 modules)
- JIT compilation with trace optimization
- Context system (Situations)
- Reactive When observers
- Concurrency (Tasks, Channels)
- LLM integration

## What's Next?

Ready to get started? Check out:
- [Installation](./getting-started/installation.md) - Set up SFX on your system
- [Quick Start](./getting-started/quick-start.md) - Learn the basics in 5 minutes
- [Your First Program](./getting-started/first-program.md) - Write your first SFX application
- [Why SFX?](./core/why-sfx.md) - Understand the philosophy behind SFX

## License

SFX is open source under the Apache License 2.0.

---

*"Programming should be as natural as telling a story"* - SFX
