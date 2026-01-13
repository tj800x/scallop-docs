# Scallop Rust API Examples

This directory contains complete working examples demonstrating the Scallop Rust API (`scallop-core`).

## Prerequisites

**Nightly Rust Required:**
```bash
rustup default nightly
```

Scallop requires nightly Rust due to unstable features:
- `min_specialization`
- `extract_if`
- `hash_extract_if`
- `proc_macro_span`

## Examples

### 1. [basic_datalog](basic_datalog/) - Getting Started

**What it demonstrates:**
- Creating an IntegrateContext
- Adding Scallop programs
- Running queries
- Iterating over results

**Run:**
```bash
cd basic_datalog
cargo run
```

**Difficulty:** ⭐ Beginner

---

### 2. [probabilistic_reasoning](probabilistic_reasoning/) - Probabilistic Queries

**What it demonstrates:**
- Using MinMaxProbProvenance
- Adding facts with probabilities
- Interpreting confidence scores
- Probabilistic transitive closure

**Run:**
```bash
cd probabilistic_reasoning
cargo run
```

**Difficulty:** ⭐⭐ Intermediate

---

### 3. [foreign_functions](foreign_functions/) - Custom Rust Functions

**What it demonstrates:**
- Implementing the ForeignFunction trait
- String manipulation functions
- Numeric operations
- Registering functions with IntegrateContext

**Run:**
```bash
cd foreign_functions
cargo run
```

**Difficulty:** ⭐⭐ Intermediate

---

### 4. [foreign_predicates](foreign_predicates/) - Fact Generators

**What it demonstrates:**
- Implementing the ForeignPredicate trait
- Binding patterns (bf, ff)
- Generating multiple results
- External data integration

**Run:**
```bash
cd foreign_predicates
cargo run
```

**Difficulty:** ⭐⭐⭐ Advanced

---

### 5. [incremental_evaluation](incremental_evaluation/) - Dynamic Updates

**What it demonstrates:**
- Creating incremental contexts
- Adding facts incrementally
- Re-running after updates
- Efficient incremental computation

**Run:**
```bash
cd incremental_evaluation
cargo run
```

**Difficulty:** ⭐⭐ Intermediate

---

### 6. [complex_reasoning](complex_reasoning/) - Advanced Provenance

**What it demonstrates:**
- TopKProofsProvenance for proof tracking
- Extracting derivation proofs
- Weighted Model Counting
- Complex reasoning patterns

**Run:**
```bash
cd complex_reasoning
cargo run
```

**Difficulty:** ⭐⭐⭐ Advanced

---

## Building All Examples

```bash
# From this directory
for example in basic_datalog probabilistic_reasoning foreign_functions foreign_predicates incremental_evaluation complex_reasoning; do
    echo "Building $example..."
    cd $example && cargo build && cd ..
done
```

## Running All Examples

```bash
# From this directory
for example in basic_datalog probabilistic_reasoning foreign_functions foreign_predicates incremental_evaluation complex_reasoning; do
    echo "=== Running $example ==="
    cd $example && cargo run && cd ..
    echo ""
done
```

## Project Structure

Each example follows this structure:

```
example_name/
├── Cargo.toml          # Dependencies and project config
├── src/
│   └── main.rs         # Example code with comments
└── README.md           # Example-specific documentation
```

## Common Patterns

### Creating a Context

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;

let prov = UnitProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);
```

### Adding a Program

```rust
ctx.add_program(r#"
    rel edge = {(0, 1), (1, 2)}
    rel path(a, b) = edge(a, b)
    query path
"#)?;
```

### Running and Querying

```rust
ctx.run()?;

let results = ctx.computed_relation_ref("path")?;
for elem in results.iter() {
    println!("{:?}", elem.tuple);
}
```

## Documentation

- **[Getting Started Guide](../../doc/src/rust_api/getting_started.md)** - Introduction to Scallop Rust API
- **[IntegrateContext API](../../doc/src/rust_api/integrate_context.md)** - Complete API reference
- **[Foreign Functions](../../doc/src/rust_api/foreign_functions.md)** - Custom Rust functions
- **[Foreign Predicates](../../doc/src/rust_api/foreign_predicates.md)** - Fact generators
- **[Provenance Types](../../doc/src/rust_api/provenance.md)** - Reasoning semantics

## Troubleshooting

### Nightly Rust Error

**Error:**
```
error[E0658]: use of unstable library feature 'min_specialization'
```

**Solution:**
```bash
rustup default nightly
```

### Missing scallop-core Dependency

**Error:**
```
error: failed to load manifest for dependency `scallop-core`
```

**Solution:** Update `Cargo.toml` with correct path:
```toml
[dependencies]
scallop-core = { path = "../../core" }
```

### Type Inference Issues

If you encounter type inference errors with generics, specify types explicitly:

```rust
// Instead of:
let mut ctx = IntegrateContext::new(prov);

// Use:
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);
```

## Contributing

Have an example you'd like to add? Contributions welcome!

1. Create a new directory under `examples/rust/`
2. Follow the standard project structure
3. Add entry to this README
4. Ensure it builds with `cargo build`
5. Submit a pull request

## Resources

- **[Scallop Repository](https://github.com/scallop-lang/scallop)** - Main repository
- **[Scallop Language Guide](../../doc/src/language/index.md)** - Scallop language reference
- **[Python API Examples](../python/)** - Python bindings examples
- **[Plugin Examples](../plugins/)** - Foundation model plugins
