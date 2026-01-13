# Basic DataLog Example

This example demonstrates the fundamentals of using the Scallop Rust API for standard DataLog reasoning.

## What This Example Demonstrates

- Creating an `IntegrateContext` with `UnitProvenance`
- Adding a Scallop program with facts and rules
- Executing the program with `run()`
- Querying results with `computed_relation_ref()`
- Iterating over tuples and extracting values

## The Program

```scl
// Define edges
rel edge = {(0, 1), (1, 2), (2, 3), (3, 4)}

// Transitive closure
rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
```

**Logic:**
- `path(a, b)` holds if there's an edge from `a` to `b`
- `path(a, c)` holds if there's a path from `a` to `b` and an edge from `b` to `c`
- This computes the transitive closure (all reachable pairs)

## Expected Output

```
=== Basic DataLog Example ===

Program loaded successfully
Program executed

Results for path relation:
Total tuples: 10

  path(0, 1)
  path(1, 2)
  path(2, 3)
  path(3, 4)
  path(0, 2)
  path(1, 3)
  path(2, 4)
  path(0, 3)
  path(1, 4)
  path(0, 4)

=== Example Complete ===
```

## Running This Example

```bash
cargo run
```

## Key Concepts

### IntegrateContext

The main entry point for embedding Scallop in Rust:

```rust
let prov = UnitProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);
```

**Generic parameters:**
- `Prov: Provenance` - Determines reasoning semantics (here: `UnitProvenance` for standard DataLog)
- `P: PointerFamily` - Memory management (here: `RcFamily` for reference counting)

### UnitProvenance

Standard DataLog semantics with no provenance tracking:
- No tags on facts
- No probabilities or counts
- Pure logical reasoning

### Adding Programs

```rust
ctx.add_program(r#"
    rel edge = {(0, 1), (1, 2)}
    rel path(a, b) = edge(a, b)
"#)?;
```

Programs can include:
- Fact declarations (`rel edge = {...}`)
- Rules (`rel path(a, c) = ...`)
- Queries (`query path`)

### Querying Results

```rust
let results = ctx.computed_relation_ref("path")?;
for elem in results.iter() {
    println!("{:?}", elem.tuple);
}
```

Results are `DynamicElement` containing:
- `tuple` - The tuple values
- `tag` - Provenance tag (Unit for this example)

### Value Extraction

```rust
if let (Some(Value::I32(from)), Some(Value::I32(to))) =
    (elem.tuple.get(0), elem.tuple.get(1))
{
    println!("path({}, {})", from, to);
}
```

Pattern match on `Value` enum to extract typed values.

## Next Steps

- **[probabilistic_reasoning](../probabilistic_reasoning/)** - Add probabilities to facts
- **[foreign_functions](../foreign_functions/)** - Extend with custom Rust functions
- **[IntegrateContext API](../../../doc/src/rust_api/integrate_context.md)** - Complete API reference

## Related Documentation

- [Getting Started Guide](../../../doc/src/rust_api/getting_started.md)
- [Provenance Types](../../../doc/src/rust_api/provenance.md)
