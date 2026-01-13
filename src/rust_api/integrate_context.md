# IntegrateContext API

## Overview

`IntegrateContext` is the main entry point for embedding Scallop programs in Rust applications. It provides a complete API for compiling Scallop code, adding facts programmatically, executing queries, and retrieving results—all from pure Rust.

The context is generic over two type parameters:
- **`Prov: Provenance`** - Determines the reasoning semantics (standard DataLog, probabilistic, differentiable, etc.)
- **`P: PointerFamily`** - Controls pointer representation (typically `RcFamily` for reference counting)

```rust
pub struct IntegrateContext<Prov: Provenance, P: PointerFamily = RcFamily> {
    // Internal state
}
```

### Comparison to Python API

| Python (scallopy) | Rust (scallop-core) |
|-------------------|---------------------|
| `ScallopContext()` | `IntegrateContext::new(prov)` |
| `ctx.add_program(...)` | `ctx.add_program(...)?` |
| `ctx.run()` | `ctx.run()?` |
| Exception handling | Result types with `?` operator |

## Creating a Context

### Basic Creation

The most common way to create an `IntegrateContext` is with the `new()` method:

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;

let prov_ctx = UnitProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov_ctx);
```

The type parameters can usually be inferred:
```rust
let prov = UnitProvenance::default();
let mut ctx = IntegrateContext::new(prov);  // P defaults to RcFamily
```

### Incremental Execution

For incremental evaluation where facts are added dynamically over time:

```rust
let prov = UnitProvenance::default();
let mut ctx = IntegrateContext::new_incremental(prov);
```

**Incremental mode:**
- Maintains internal state between `run()` calls
- Only recomputes affected parts when facts are added
- More efficient for dynamic updates

### Choosing Provenance Type

The provenance type determines how facts are tagged and combined:

```rust
// Standard DataLog (no provenance tracking)
use scallop_core::runtime::provenance::unit::UnitProvenance;
let prov = UnitProvenance::default();

// Probabilistic reasoning with min-max semiring
use scallop_core::runtime::provenance::min_max_prob::MinMaxProbProvenance;
let prov = MinMaxProbProvenance::default();

// Top-K proofs tracking
use scallop_core::runtime::provenance::top_k_proofs::TopKProofsProvenance;
let prov = TopKProofsProvenance::<RcFamily>::new(3); // Track top 3 proofs
```

See [Provenance Types](provenance.md) for complete details on all available provenance types.

### Choosing Pointer Family

The pointer family controls how internal data structures are reference-counted:

```rust
use scallop_core::runtime::env::{RcFamily, ArcFamily};

// Single-threaded (default, faster)
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

// Thread-safe (for concurrent access)
let mut ctx = IntegrateContext::<_, ArcFamily>::new(prov);
```

**RcFamily** (default): Uses `std::rc::Rc` - faster but not thread-safe
**ArcFamily**: Uses `std::sync::Arc` - thread-safe but slightly slower

## Adding Programs and Rules

### Adding Complete Programs

The `add_program()` method compiles a complete Scallop program from a string:

```rust
ctx.add_program(r#"
    rel edge = {(0, 1), (1, 2), (2, 3)}

    rel path(a, b) = edge(a, b)
    rel path(a, c) = path(a, b), edge(b, c)

    query path
"#)?;
```

**Usage notes:**
- Use raw string literals (`r#"..."#`) to avoid escaping quotes
- Can include relation declarations, rules, queries, type definitions
- Returns `Result<(), IntegrateError>` - use `?` to propagate errors
- Multiple calls append to existing program

### Adding Relation Declarations

Declare relation types explicitly:

```rust
ctx.add_relation("edge(i32, i32)")?;
ctx.add_relation("node(i32, String)")?;
ctx.add_relation("weighted_edge(i32, i32, f64)")?;
```

**When to use:**
- When adding facts programmatically (before `add_facts()`)
- To enforce type constraints
- For better error messages

### Adding Individual Rules

Add single rules incrementally:

```rust
ctx.add_rule("path(a, b) = edge(a, b)")?;
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")?;
```

Equivalent to:
```rust
ctx.add_program(r#"
    rel path(a, b) = edge(a, b)
    rel path(a, c) = path(a, b), edge(b, c)
"#)?;
```

### Error Handling

All compilation methods return `Result<_, IntegrateError>`:

```rust
use scallop_core::integrate::IntegrateError;

match ctx.add_program("rel invalid syntax!") {
    Ok(_) => println!("Success"),
    Err(IntegrateError::Compile(errors)) => {
        eprintln!("Compilation failed:");
        for err in errors {
            eprintln!("  {}", err);
        }
    }
    Err(e) => eprintln!("Other error: {:?}", e),
}
```

**Common error types:**
- `IntegrateError::Compile` - Syntax or type errors
- `IntegrateError::Runtime` - Execution errors
- `IntegrateError::Front` - Front-end compilation errors

## Adding Facts Programmatically

### Basic Fact Insertion

Add facts to existing relations using `add_facts()`:

```rust
use scallop_core::common::tuple::Tuple;

// Declare the relation first
ctx.add_relation("edge(i32, i32)")?;

// Add facts without tags (standard DataLog)
ctx.add_facts("edge", vec![
    (None, Tuple::from((0i32, 1i32))),
    (None, Tuple::from((1i32, 2i32))),
    (None, Tuple::from((2i32, 3i32))),
], false)?;
```

**Parameters:**
- `predicate: &str` - Relation name
- `facts: Vec<(Option<Tag>, Tuple)>` - List of (tag, tuple) pairs
- `type_check: bool` - Whether to validate types (false for performance)

### Creating Tuples

Tuples can be created from Rust values using the `From` trait:

```rust
// From tuple
let t1: Tuple = (0i32, 1i32).into();

// From explicit values
let t2 = Tuple::from((0i32, "hello", 3.14));

// Manual construction
use scallop_core::common::value::Value;
let t3 = Tuple::from(vec![
    Value::I32(0),
    Value::String("world".to_string()),
    Value::F64(2.71),
]);
```

### Type Checking

Enable type checking to validate tuples against relation schemas:

```rust
ctx.add_relation("edge(i32, i32)")?;

// This will fail with type_check = true
ctx.add_facts("edge", vec![
    (None, Tuple::from(("string", 42))),  // Wrong type!
], true)?;  // Errors with TypeError
```

**Recommendations:**
- Use `type_check = false` for performance in tight loops
- Use `type_check = true` during development for safety
- Always ensure types match the declared relation schema

### Adding Facts with Probabilities

For probabilistic provenances, tag facts with probabilities:

```rust
use scallop_core::runtime::provenance::min_max_prob::MinMaxProbProvenance;

let prov = MinMaxProbProvenance::default();
let mut ctx = IntegrateContext::new(prov);

ctx.add_relation("edge(i32, i32)")?;
ctx.add_facts("edge", vec![
    (Some(0.8.into()), Tuple::from((0i32, 1i32))),
    (Some(0.9.into()), Tuple::from((1i32, 2i32))),
    (Some(0.7.into()), Tuple::from((2i32, 3i32))),
], false)?;
```

The tag type (`Some(0.8.into())`) automatically converts to the provenance's `InputTag` type.

### Complete Example

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;
use scallop_core::common::tuple::Tuple;

fn main() -> Result<(), IntegrateError> {
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Declare relations
    ctx.add_relation("node(i32, String)")?;
    ctx.add_relation("edge(i32, i32)")?;

    // Add node facts
    ctx.add_facts("node", vec![
        (None, Tuple::from((0i32, "Alice"))),
        (None, Tuple::from((1i32, "Bob"))),
        (None, Tuple::from((2i32, "Charlie"))),
    ], false)?;

    // Add edge facts
    ctx.add_facts("edge", vec![
        (None, Tuple::from((0i32, 1i32))),
        (None, Tuple::from((1i32, 2i32))),
    ], false)?;

    // Add rules
    ctx.add_rule("query node")?;
    ctx.add_rule("query edge")?;

    Ok(())
}
```

## Executing Programs

### Basic Execution

Execute the program to fixpoint with `run()`:

```rust
ctx.run()?;
```

**What happens:**
1. Compiles any new rules/facts since last run
2. Executes the Scallop program to fixpoint
3. Stores results internally for querying

**Returns:**
- `Ok(())` if execution succeeded
- `Err(IntegrateError::Runtime(_))` if execution failed

### Incremental Execution

For incremental contexts, multiple `run()` calls only recompute affected parts:

```rust
let mut ctx = IntegrateContext::new_incremental(prov);

ctx.add_relation("edge(i32, i32)")?;
ctx.add_rule("path(a, b) = edge(a, b)")?;
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")?;

// Initial facts and run
ctx.add_facts("edge", vec![
    (None, (0i32, 1i32).into()),
    (None, (1i32, 2i32).into()),
], false)?;
ctx.run()?;

println!("Initial results: {}",
    ctx.computed_relation_ref("path").unwrap().len());

// Add more facts and re-run (incremental update)
ctx.add_facts("edge", vec![
    (None, (2i32, 3i32).into()),
], false)?;
ctx.run()?;

println!("Updated results: {}",
    ctx.computed_relation_ref("path").unwrap().len());
```

### Iteration Limits

Control recursion depth with iteration limits:

```rust
// Set maximum iterations
ctx.set_iter_limit(100);
ctx.run()?;

// Remove limit (default: unlimited)
ctx.remove_iter_limit();
ctx.run()?;
```

**Use cases:**
- Preventing infinite loops in recursive rules
- Testing convergence behavior
- Performance benchmarking

## Querying Results

### Getting Result Collections

Retrieve computed relations using `computed_relation_ref()`:

```rust
let path_relation = ctx.computed_relation_ref("path")
    .ok_or("Relation 'path' not found")?;

println!("Found {} path tuples", path_relation.len());
```

**Returns:**
- `Some(&DynamicOutputCollection<Prov>)` if relation exists and is computed
- `None` if relation doesn't exist or hasn't been queried

### Iterating Over Results

Each collection provides an iterator over elements:

```rust
let path = ctx.computed_relation_ref("path").unwrap();

for elem in path.iter() {
    println!("Tag: {}, Tuple: {:?}", elem.tag, elem.tuple);
}
```

**Element structure:**
```rust
pub struct DynamicElement<Prov: Provenance> {
    pub tag: Prov::OutputTag,  // Probability, proof, etc.
    pub tuple: Tuple,           // The fact tuple
}
```

### Extracting Values from Tuples

Access tuple elements and convert to Rust types:

```rust
let path = ctx.computed_relation_ref("path").unwrap();

for elem in path.iter() {
    // Pattern match on tuple elements
    if let (Some(Value::I32(from)), Some(Value::I32(to))) =
        (elem.tuple.get(0), elem.tuple.get(1)) {
        println!("Path from {} to {}", from, to);
    }
}
```

**Available Value variants:**
```rust
use scallop_core::common::value::Value;

match value {
    Value::I32(n) => println!("Integer: {}", n),
    Value::String(s) => println!("String: {}", s),
    Value::F64(f) => println!("Float: {}", f),
    Value::Bool(b) => println!("Boolean: {}", b),
    Value::Char(c) => println!("Char: {}", c),
    // ... and many more
    _ => println!("Other type"),
}
```

### Complete Querying Example

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::min_max_prob::MinMaxProbProvenance;
use scallop_core::runtime::env::RcFamily;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    let prov = MinMaxProbProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    ctx.add_program(r#"
        rel edge = {
            0.8::(0, 1),
            0.9::(1, 2),
            0.7::(2, 3)
        }
        rel path(a, b) = edge(a, b)
        rel path(a, c) = path(a, b), edge(b, c)
        query path
    "#)?;

    ctx.run()?;

    // Get results
    let path = ctx.computed_relation_ref("path")
        .ok_or("Path relation not found")?;

    println!("Probabilistic paths:");
    for elem in path.iter() {
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (elem.tuple.get(0), elem.tuple.get(1)) {
            println!("  {} -> {}: probability = {}", from, to, elem.tag);
        }
    }

    Ok(())
}
```

**Output:**
```
Probabilistic paths:
  0 -> 1: probability = 0.8
  1 -> 2: probability = 0.9
  2 -> 3: probability = 0.7
  0 -> 2: probability = 0.8
  1 -> 3: probability = 0.7
  0 -> 3: probability = 0.7
```

## Configuration Options

### Debug Modes

Enable detailed output for different compilation stages:

```rust
// Front-end debugging (parsing, type checking)
ctx.set_debug_front(true);

// Back-end debugging (RAM generation)
ctx.set_debug_back(true);

// RAM execution trace
ctx.set_debug_ram(true);
```

**Output goes to stdout:**
- Front debug: AST, type information, relation schemas
- Back debug: Back-IR, RAM program
- RAM debug: Execution trace, iteration counts

### Iteration Control

Configure recursion limits:

```rust
// Set maximum iterations (prevents infinite loops)
ctx.set_iter_limit(100);

// Remove iteration limit (default: unlimited)
ctx.remove_iter_limit();
```

### Early Discard

Optimize by discarding facts with zero tags early:

```rust
ctx.set_early_discard(true);
```

**When to use:**
- Provenance types where tag = 0 means "impossible" (probabilities, proofs)
- Large programs with many zero-probability derivations
- Memory-constrained environments

**When NOT to use:**
- Standard DataLog (UnitProvenance)
- Provenances where 0 is meaningful

### Configuration Example

```rust
fn create_optimized_context() -> IntegrateContext<MinMaxProbProvenance> {
    let prov = MinMaxProbProvenance::default();
    let mut ctx = IntegrateContext::new(prov);

    // Enable optimizations
    ctx.set_early_discard(true);
    ctx.set_iter_limit(1000);

    // Enable debugging for development
    #[cfg(debug_assertions)]
    {
        ctx.set_debug_front(true);
        ctx.set_debug_ram(true);
    }

    ctx
}
```

## Error Handling

### IntegrateError Enum

All IntegrateContext methods that can fail return `Result<T, IntegrateError>`:

```rust
pub enum IntegrateError {
    Compile(Vec<CompileError>),
    Front(FrontCompileError),
    Runtime(RuntimeError),
}
```

### Common Error Patterns

**Compilation errors:**
```rust
match ctx.add_program("rel invalid!") {
    Ok(_) => {},
    Err(IntegrateError::Compile(errors)) => {
        for err in errors {
            eprintln!("Compile error: {}", err);
        }
    }
    Err(e) => eprintln!("Other error: {:?}", e),
}
```

**Runtime errors:**
```rust
match ctx.run() {
    Ok(_) => {},
    Err(IntegrateError::Runtime(err)) => {
        eprintln!("Runtime error: {:?}", err);
    }
    Err(e) => eprintln!("Other error: {:?}", e),
}
```

### Using the ? Operator

Most code can simply use `?` to propagate errors:

```rust
fn setup_context() -> Result<IntegrateContext<UnitProvenance>, IntegrateError> {
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::new(prov);

    ctx.add_relation("edge(i32, i32)")?;
    ctx.add_rule("path(a, b) = edge(a, b)")?;

    Ok(ctx)
}

fn main() {
    match setup_context() {
        Ok(ctx) => println!("Context created successfully"),
        Err(e) => eprintln!("Failed to create context: {:?}", e),
    }
}
```

### Type Errors

Type mismatches when adding facts:

```rust
ctx.add_relation("edge(i32, i32)")?;

// This will error if type_check = true
match ctx.add_facts("edge", vec![
    (None, Tuple::from(("not", "integers"))),
], true) {
    Ok(_) => {},
    Err(IntegrateError::Runtime(RuntimeError::Database(
        DatabaseError::TypeError { relation, relation_type, tuple }
    ))) => {
        eprintln!("Type error in relation '{}':", relation);
        eprintln!("  Expected: {}", relation_type);
        eprintln!("  Got: {:?}", tuple);
    }
    Err(e) => eprintln!("Other error: {:?}", e),
}
```

## Complete Example

Putting it all together:

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;
use scallop_core::common::tuple::Tuple;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    // Create context
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Configure
    ctx.set_iter_limit(100);

    // Add program
    ctx.add_program(r#"
        rel edge = {(0, 1), (1, 2), (2, 3)}
        rel path(a, b) = edge(a, b)
        rel path(a, c) = path(a, b), edge(b, c)
        query path
    "#)?;

    // Execute
    ctx.run()?;

    // Query results
    let path = ctx.computed_relation_ref("path")
        .ok_or("Path relation not found")?;

    println!("Paths found: {}", path.len());
    for elem in path.iter() {
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (elem.tuple.get(0), elem.tuple.get(1)) {
            println!("  {} -> {}", from, to);
        }
    }

    Ok(())
}
```

## Next Steps

- **[Foreign Functions](foreign_functions.md)** - Extend Scallop with custom Rust functions
- **[Foreign Predicates](foreign_predicates.md)** - Create fact generators in Rust
- **[Provenance Types](provenance.md)** - Deep dive into reasoning semantics
- **[Getting Started](getting_started.md)** - Quick start guide and examples
