# Installation

This guide will help you install SFX on your system.

## Prerequisites

- **Rust 1.75+** - SFX is written in Rust and requires Cargo to build
- **Git** - For cloning the repository

## Installing Rust

If you don't have Rust installed, get it from [rustup.rs](https://rustup.rs/):

```bash
# Unix/Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download and run rustup-init.exe from https://rustup.rs/
```

## Installing SFX

### From Source (Current Method)

```bash
# Clone the repository
git clone https://github.com/roriau0422/sfex-lang.git
cd sfex-lang

# Build in release mode
cargo build --release

# The sfex binary will be at target/release/sfex
```

### Add to PATH (Optional)

To run `sfex` from anywhere:

**Unix/Linux/macOS:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:/path/to/sfex-lang/target/release"
```

**Windows:**
```powershell
# Add to system PATH:
# Settings → System → About → Advanced system settings → Environment Variables
# Add: C:\path\to\sfex-lang\target\release
```

## Verify Installation

Test that SFX is working:

```bash
./target/release/sfex --version
# Should print: sfex 0.3.2

# Run a test script
./target/release/sfex run tests/core/hello.sfex
```

## Next Steps

- [Quick Start](./quick-start.md) - Learn the basics
- [Your First Program](./first-program.md) - Write your first SFX script
