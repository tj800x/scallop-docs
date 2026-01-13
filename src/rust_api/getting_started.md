# Getting Started with Scallop Rust API

This guide introduces the `scallop-core` Rust library for embedding Scallop programs in Rust applications.

## Overview

The `scallop-core` crate provides a complete Rust API for:
- **Compiling and executing** Scallop programs
- **Registering foreign functions** and predicates in pure Rust
- **Provenance tracking** with various semiring types
- **Incremental evaluation** for efficient updates
- **Runtime configuration** and debugging

Unlike the Python bindings (scallopy), the Rust API gives direct access to Scallop's core runtime without serialization overhead.

## Installation

Add `scallop-core` to your `Cargo.toml`:

```toml
[dependencies]
scallop-core = { path = "../path/to/scallop/core" }
# Or from crates.io when published:
# scallop-core = "0.2.5"
```

**Note:** Requires nightly Rust due to unstable features:
- `min_specialization`
- `extract_if`
- `hash_extract_if`
- `proc_macro_span`

Set your toolchain:
```bash
rustup default nightly
```

## Quick Start

### Example 1: Basic Program

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;

fn main() {
    // Create context with unit provenance (standard DataLog)
    let prov_ctx = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov_ctx);

    // Add Scallop program
    ctx.add_program(r#"
        rel edge = {(0, 1), (1, 2), (2, 3)}
        rel path(a, b) = edge(a, b)
        rel path(a, c) = path(a, b), edge(b, c)
        query path
    "#).unwrap();

    // Execute
    ctx.run().unwrap();

    // Get results
    let path_relation = ctx.computed_relation_ref("path").unwrap();
    for tuple in path_relation.iter() {
        println!("{:?}", tuple);
    }
}
```

**Output:**
```
(0, 1)
(1, 2)
(2, 3)
(0, 2)
(1, 3)
(0, 3)
```

### Example 2: Adding Facts Programmatically

```rust
use scallop_core::common::tuple::Tuple;

fn main() {
    let prov_ctx = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov_ctx);

    // Declare relation type
    ctx.add_relation("edge(i32, i32)").unwrap();

    // Add facts programmatically
    ctx.add_facts(
        "edge",
        vec![
            (None, Tuple::from((0i32, 1i32))),
            (None, Tuple::from((1i32, 2i32))),
            (None, Tuple::from((2i32, 3i32))),
        ],
        false, // type_check
    ).unwrap();

    // Add rules
    ctx.add_rule("path(a, b) = edge(a, b)").unwrap();
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)").unwrap();

    // Execute
    ctx.run().unwrap();

    // Query
    let path = ctx.computed_relation_ref("path").unwrap();
    println!("Path tuples: {}", path.len());
}
```

### Example 3: Probabilistic Reasoning

```rust
use scallop_core::runtime::provenance::min_max_prob::MinMaxProbProvenance;

fn main() {
    // Use min-max probability provenance
    let prov_ctx = MinMaxProbProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov_ctx);

    ctx.add_relation("edge(i32, i32)").unwrap();

    // Add facts with probabilities
    ctx.add_facts(
        "edge",
        vec![
            (Some(0.8.into()), Tuple::from((0i32, 1i32))),
            (Some(0.9.into()), Tuple::from((1i32, 2i32))),
            (Some(0.7.into()), Tuple::from((2i32, 3i32))),
        ],
        false,
    ).unwrap();

    ctx.add_rule("path(a, b) = edge(a, b)").unwrap();
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)").unwrap();

    ctx.run().unwrap();

    // Results include probabilities
    let path = ctx.computed_relation_ref("path").unwrap();
    for elem in path.iter() {
        println!("Probability: {}, Tuple: {:?}", elem.tag, elem.tuple);
    }
}
```

**Output:**
```
Probability: 0.8, Tuple: (0, 1)
Probability: 0.9, Tuple: (1, 2)
Probability: 0.7, Tuple: (2, 3)
Probability: 0.72, Tuple: (0, 2)    // min(0.8, 0.9)
Probability: 0.63, Tuple: (1, 3)    // min(0.9, 0.7)
Probability: 0.56, Tuple: (0, 3)    // min(0.72, 0.7)
```

## Core Concepts

### IntegrateContext

The main entry point for Scallop programs. Generic over:
- **Provenance type** (`Prov`) - defines reasoning semantics
- **Pointer family** (`P`) - typically `RcFamily` for reference counting

```rust
pub struct IntegrateContext<Prov: Provenance, P: PointerFamily> {
    // Internal compiler and runtime state
}
```

**Key methods:**
- `add_program(&mut self, program: &str)` - Add complete Scallop program
- `add_relation(&mut self, decl: &str)` - Declare relation type
- `add_rule(&mut self, rule: &str)` - Add single rule
- `add_facts(&mut self, rel: &str, facts: Vec<_>)` - Add fact tuples
- `run(&mut self)` - Execute program
- `computed_relation_ref(&mut self, name: &str)` - Get query results

### Provenance Types

Provenance determines **how facts are tagged** and **how tags combine**:

| Provenance | Use Case | Tag Type | Semantics |
|------------|----------|----------|-----------|
| `UnitProvenance` | Standard DataLog | `Unit` | No tracking |
| `BooleanProvenance` | Negation-as-failure | `bool` | Boolean algebra |
| `NaturalProvenance` | Counting | `usize` | Cardinality |
| `MinMaxProbProvenance` | Probabilistic | `f64` | Min/max on paths |
| `AddMultProbProvenance` | Probabilistic | `f64` | Add/mult on paths |
| `TopKProofsProvenance` | Proof tracking | `DNFFormula` | Top-K most probable proofs |
| `ProbProofsProvenance` | Complete proofs | `Proofs` | All proofs + probabilities |

### Tuples and Values

Facts are represented as `Tuple` containing `Value` elements:

```rust
use scallop_core::common::value::Value;
use scallop_core::common::tuple::Tuple;

// Create a tuple (0, "hello", 3.14)
let tuple = Tuple::from((
    Value::I32(0),
    Value::String("hello".to_string()),
    Value::F64(3.14),
));

// Or use From trait
let tuple: Tuple = (0i32, "hello", 3.14).into();
```

**Value types:**
- `I8`, `I16`, `I32`, `I64`, `I128`, `ISize` - Signed integers
- `U8`, `U16`, `U32`, `U64`, `U128`, `USize` - Unsigned integers
- `F32`, `F64` - Floating point
- `Bool` - Boolean
- `Char` - Character
- `String` - UTF-8 string
- `Symbol` - Interned symbol
- `Entity` - Algebraic data type value

## Common Patterns

### Pattern 1: Incremental Evaluation

```rust
fn main() {
    let prov_ctx = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new_incremental(prov_ctx);

    // Initial program
    ctx.add_relation("edge(i32, i32)").unwrap();
    ctx.add_rule("path(a, b) = edge(a, b)").unwrap();
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)").unwrap();

    // Initial facts
    ctx.add_facts("edge", vec![
        (None, (0i32, 1i32).into()),
        (None, (1i32, 2i32).into()),
    ], false).unwrap();

    ctx.run().unwrap();
    println!("Initial path count: {}", ctx.computed_relation_ref("path").unwrap().len());

    // Add more facts incrementally
    ctx.add_facts("edge", vec![
        (None, (2i32, 3i32).into()),
    ], false).unwrap();

    ctx.run().unwrap();
    println!("Updated path count: {}", ctx.computed_relation_ref("path").unwrap().len());
}
```

### Pattern 2: Query-Driven Execution

```rust
fn main() {
    let prov_ctx = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov_ctx);

    ctx.add_program(r#"
        rel edge = {(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)}
        rel path(a, b) = edge(a, b)
        rel path(a, c) = path(a, b), edge(b, c)

        // Only query paths from node 0
        query path(0, x)
    "#).unwrap();

    ctx.run().unwrap();

    // Only paths starting from 0 are computed
    let result = ctx.computed_relation_ref("path").unwrap();
    for elem in result.iter() {
        println!("{:?}", elem.tuple);
    }
}
```

### Pattern 3: Error Handling

```rust
use scallop_core::integrate::IntegrateError;

fn run_scallop_program(program: &str) -> Result<(), IntegrateError> {
    let prov_ctx = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov_ctx);

    // All operations return Result
    ctx.add_program(program)?;
    ctx.run()?;

    let result = ctx.computed_relation_ref("result")
        .ok_or_else(|| IntegrateError::RelationNotFound("result".to_string()))?;

    println!("Result: {} tuples", result.len());
    Ok(())
}

fn main() {
    match run_scallop_program("rel result = {1, 2, 3}") {
        Ok(_) => println!("Success"),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

## Configuration Options

### Debugging

Enable debug output for different compilation stages:

```rust
ctx.set_debug_front(true);   // Front-end (parsing, type checking)
ctx.set_debug_back(true);    // Back-end (RAM generation)
ctx.set_debug_ram(true);     // RAM execution trace
```

### Iteration Limits

Control recursion depth:

```rust
ctx.set_iter_limit(100);     // Maximum 100 iterations
ctx.remove_iter_limit();     // Unlimited (default)
```

### Early Discard

Optimize by discarding zero-tagged facts:

```rust
ctx.set_early_discard(true);  // Discard facts with tag=0 early
```

## Building and Running

### As a Library Dependency

```toml
[dependencies]
scallop-core = { path = "../scallop/core" }
```

```rust
// src/main.rs
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;

fn main() {
    let prov_ctx = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov_ctx);

    ctx.add_program(r#"
        rel answer = {42}
        query answer
    "#).unwrap();

    ctx.run().unwrap();

    let result = ctx.computed_relation_ref("answer").unwrap();
    for elem in result.iter() {
        println!("Answer: {:?}", elem.tuple);
    }
}
```

**Build and run:**
```bash
cargo build
cargo run
```

### Development Setup

```bash
# Clone Scallop repository
git clone https://github.com/scallop-lang/scallop.git
cd scallop

# Create example project
cargo new --bin my-scallop-app
cd my-scallop-app

# Add dependency (edit Cargo.toml)
[dependencies]
scallop-core = { path = "../core" }

# Build with nightly
rustup default nightly
cargo build
```

## Next Steps

- **[IntegrateContext API](integrate_context.md)** - Complete API reference
- **[Foreign Functions](foreign_functions.md)** - Register custom Rust functions
- **[Foreign Predicates](foreign_predicates.md)** - Create extensional predicates
- **[Provenance Types](provenance.md)** - Deep dive into reasoning semantics
- **[Examples](../examples/rust/)** - Complete working examples

## Common Issues

### Nightly Rust Required

**Error:**
```
error[E0658]: use of unstable library feature 'min_specialization'
```

**Solution:**
```bash
rustup default nightly
```

### Missing Relation

**Error:**
```
IntegrateError::RelationNotFound("path")
```

**Solution:** Ensure relation is declared or computed:
```rust
ctx.add_relation("path(i32, i32)").unwrap();  // Declare first
// Or add query to compute it
ctx.add_rule("query path").unwrap();
```

### Type Mismatch

**Error:**
```
TypeError: Expected i32, got String
```

**Solution:** Match Value types to relation declarations:
```rust
ctx.add_relation("edge(i32, i32)").unwrap();  // Declare types
ctx.add_facts("edge", vec![
    (None, Tuple::from((0i32, 1i32))),  // Use i32, not usize
], false).unwrap();
```

## Resources

- **[Scallop Repository](https://github.com/scallop-lang/scallop)** - Source code
- **[Core Tests](https://github.com/scallop-lang/scallop/tree/master/core/tests/integrate)** - Integration test examples
- **[Language Guide](../language/index.md)** - Scallop language reference
- **[Rust API Docs](https://docs.rs/scallop-core)** - Generated rustdoc (when published)
