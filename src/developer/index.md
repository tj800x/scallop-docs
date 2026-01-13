# Developer Guide

This section covers the internal architecture of Scallop for contributors and developers who want to understand or extend the system.

## Overview

Scallop is implemented in Rust and consists of several major components:

- **Compiler** - Parses Scallop programs and produces intermediate representations
- **Runtime** - Executes programs using various provenance semirings
- **Bindings** - Python (scallopy), C, and other language integrations
- **Utils** - Common utilities, data structures, and algorithms

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     User Programs                       │
│  .scl files, Python API calls, CLI commands             │
└───────────────────┬─────────────────────────────────────┘
                    │
        ┌───────────┴──────────┐
        │                      │
        ▼                      ▼
┌───────────────┐      ┌──────────────┐
│   Compiler    │      │   Bindings   │
│  (Front-end)  │      │  (scallopy)  │
└───────┬───────┘      └──────┬───────┘
        │                     │
        ▼                     ▼
┌──────────────────────────────────────┐
│          Runtime Engine               │
│  - Provenance semirings               │
│  - Execution strategies               │
│  - Storage and indexing               │
└──────────────────────────────────────┘
```

### Component Overview

**Compiler (`core/src/compiler/`)**
- Lexer and parser (LALRPOP-based)
- Type inference and checking
- Intermediate representation (IR) generation
- Query planning and optimization

**Runtime (`core/src/runtime/`)**
- Provenance semiring framework
- Execution engine (semi-naive evaluation, stratification)
- Storage backend (relations, tuples, indexes)
- Foreign function/predicate interface

**Bindings (`etc/scallopy/`, C bindings)**
- Python API (scallopy)
- PyTorch integration
- Foreign function registration

**Utils (`core/src/utils/`)**
- Data structures (B-trees, tries, etc.)
- SDD (Sentential Decision Diagram) for WMC
- Type system utilities

---

## Code Organization

### Repository Structure

```
scallop/
├── core/                    # Core Scallop implementation (Rust)
│   ├── src/
│   │   ├── compiler/        # Front-end compilation
│   │   ├── runtime/         # Execution engine
│   │   ├── common/          # Shared types and utilities
│   │   ├── integrate/       # Integration layer
│   │   └── utils/           # Helper utilities
│   └── tests/               # Integration tests
├── etc/
│   ├── scallopy/            # Python bindings
│   ├── scli/                # CLI tools
│   └── sclrepl/             # REPL
├── doc/                     # Documentation (mdBook)
└── examples/                # Example programs
```

### Key Modules

**Compiler Modules:**
- `compiler/front/` - Front-end (parsing, AST)
- `compiler/type_check/` - Type inference and checking
- `compiler/back/` - Back-end (IR generation)
- `compiler/ram/` - RAM (Relational Algebra Machine) compilation

**Runtime Modules:**
- `runtime/provenance/` - Provenance semiring implementations
- `runtime/database/` - Storage and indexing
- `runtime/monitor/` - Execution monitoring
- `runtime/env/` - Execution environment

**Common Modules:**
- `common/expr/` - Expression types
- `common/foreign_function/` - Foreign function interface
- `common/tuple/` - Tuple types and operations
- `common/value/` - Value types

---

## Key Concepts for Contributors

### 1. Provenance Semirings

Scallop's core abstraction is the **provenance semiring**:

```rust
pub trait Provenance {
    type InputTag;        // What users provide
    type OutputTag;       // What users get back
    type Tag;             // Internal representation

    fn tagging_fn(&self, input_tag: Self::InputTag) -> Self::Tag;
    fn recover_fn(&self, tag: &Self::Tag) -> Self::OutputTag;

    fn add(&self, t1: &Self::Tag, t2: &Self::Tag) -> Self::Tag;  // OR operation
    fn mult(&self, t1: &Self::Tag, t2: &Self::Tag) -> Self::Tag; // AND operation
    fn negate(&self, tag: &Self::Tag) -> Self::Tag;              // NOT operation
}
```

All 18 provenance types implement this trait.

### 2. Compilation Pipeline

```
.scl source
    ↓ Parse (LALRPOP)
AST (Abstract Syntax Tree)
    ↓ Type inference
Typed AST
    ↓ Lower to IR
Front IR
    ↓ Transform
Back IR
    ↓ Compile
RAM Program
    ↓ Execute
Results
```

### 3. Execution Model

Scallop uses **semi-naive evaluation** for recursive rules:
1. Start with base facts (iteration 0)
2. Apply rules to derive new facts (iteration i)
3. Only use facts from iteration i-1 (semi-naive)
4. Repeat until fixpoint (no new facts)

### 4. Storage Backend

Relations are stored in efficient data structures:
- **Extensional relations** (base facts) - stored as sorted vectors
- **Intensional relations** (derived) - computed on demand or materialized
- **Indexes** - B-trees for fast lookups

---

## Building from Source

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- Cargo (comes with Rust)
- Python 3.8+ (for scallopy bindings)
- PyTorch (optional, for ML integration)

### Build Steps

```bash
# Clone repository
git clone https://github.com/scallop-lang/scallop.git
cd scallop

# Build CLI tools
cd core
cargo build --release

# Binaries in target/release/
./target/release/scli --version

# Build Python bindings
cd ../etc/scallopy
pip install -e .
```

### Running Tests

```bash
# Core tests
cd core
cargo test

# Integration tests
cargo test --test integrate

# Python tests
cd ../etc/scallopy
python -m pytest tests/
```

---

## Contributing

### Getting Started

1. **Read the codebase** - Start with `/core/src/lib.rs`
2. **Run examples** - `cargo run --example <name>`
3. **Add tests** - All new features need tests
4. **Follow conventions** - Match existing code style

### Code Style

- Follow Rust conventions (rustfmt)
- Document public APIs
- Write integration tests for user-facing features
- Keep PRs focused and atomic

### Testing Guidelines

**Unit tests** - In the same file as the code:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Test code
    }
}
```

**Integration tests** - In `core/tests/integrate/`:
```rust
#[test]
fn test_new_language_feature() {
    let program = r#"
        rel edge = {(0, 1), (1, 2)}
        rel path(a, b) = edge(a, b)
        query path
    "#;
    // Test execution
}
```

---

## Next Steps

- [Language Constructs](language_construct.md) - Implementing new language features
- [Bindings](binding.md) - Adding language bindings
- [Contributing Guide](https://github.com/scallop-lang/scallop/blob/master/CONTRIBUTING.md)

For questions, join the Scallop community or open an issue on GitHub.
