# Rust Examples

This page provides an overview of the Rust example projects included with Scallop.

## Location

All Rust examples are located in the repository at:
```
scallop/examples/rust/
```

## Available Examples

The following 6 example projects demonstrate different aspects of the Scallop Rust API:

### 1. basic_datalog - Getting Started

**What it demonstrates:**
- Creating an IntegrateContext
- Adding Scallop programs
- Running queries
- Iterating over results

**Difficulty:** ⭐ Beginner

**Run:**
```bash
cd examples/rust/basic_datalog
cargo run
```

---

### 2. probabilistic_reasoning - Probabilistic Queries

**What it demonstrates:**
- Using MinMaxProbProvenance
- Adding facts with probabilities
- Interpreting confidence scores
- Probabilistic transitive closure

**Difficulty:** ⭐⭐ Intermediate

**Run:**
```bash
cd examples/rust/probabilistic_reasoning
cargo run
```

---

### 3. foreign_functions - Custom Rust Functions

**What it demonstrates:**
- Implementing the ForeignFunction trait
- String manipulation functions
- Numeric operations
- Registering functions with IntegrateContext

**Difficulty:** ⭐⭐ Intermediate

**Run:**
```bash
cd examples/rust/foreign_functions
cargo run
```

---

### 4. foreign_predicates - Fact Generators

**What it demonstrates:**
- Implementing the ForeignPredicate trait
- Binding patterns (bf, ff)
- Generating multiple results
- External data integration

**Difficulty:** ⭐⭐⭐ Advanced

**Run:**
```bash
cd examples/rust/foreign_predicates
cargo run
```

---

### 5. incremental_evaluation - Dynamic Updates

**What it demonstrates:**
- Creating incremental contexts
- Adding facts incrementally
- Re-running after updates
- Efficient incremental computation

**Difficulty:** ⭐⭐ Intermediate

**Run:**
```bash
cd examples/rust/incremental_evaluation
cargo run
```

---

### 6. complex_reasoning - Advanced Provenance

**What it demonstrates:**
- TopKProofsProvenance for proof tracking
- Extracting derivation proofs
- Weighted Model Counting
- Complex reasoning patterns

**Difficulty:** ⭐⭐⭐ Advanced

**Run:**
```bash
cd examples/rust/complex_reasoning
cargo run
```

---

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

## Building All Examples

From the `examples/rust/` directory:

```bash
for example in basic_datalog probabilistic_reasoning foreign_functions foreign_predicates incremental_evaluation complex_reasoning; do
    echo "Building $example..."
    cd $example && cargo build && cd ..
done
```

## Running All Examples

From the `examples/rust/` directory:

```bash
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
use scallop_core::runtime::provenance::discrete::unit::UnitProvenance;
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
scallop-core = { path = "../../../core" }
```

### Type Inference Issues

If you encounter type inference errors with generics, specify types explicitly:

```rust
// Instead of:
let mut ctx = IntegrateContext::new(prov);

// Use:
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);
```

## Next Steps

After exploring the examples:

- **[Getting Started Guide](getting_started.md)** - Introduction to Scallop Rust API
- **[IntegrateContext API](integrate_context.md)** - Complete API reference
- **[Foreign Functions](foreign_functions.md)** - Custom Rust functions
- **[Foreign Predicates](foreign_predicates.md)** - Fact generators
- **[Provenance Types](provenance.md)** - Reasoning semantics

## Contributing

Have an example you'd like to add? Contributions welcome!

1. Create a new directory under `examples/rust/`
2. Follow the standard project structure
3. Add entry to this documentation
4. Ensure it builds with `cargo build`
5. Submit a pull request

## Resources

- **[Scallop Repository](https://github.com/scallop-lang/scallop)** - Main repository
- **[Scallop Language Guide](../language/index.md)** - Scallop language reference
- **[Python API Examples](../scallopy/getting_started.md)** - Python bindings examples
