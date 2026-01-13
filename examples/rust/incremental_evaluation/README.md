# Incremental Evaluation Example

This example demonstrates Scallop's incremental evaluation capabilities for efficient dynamic updates.

## What This Example Demonstrates

- Creating incremental contexts with `new_incremental()`
- Adding facts incrementally over multiple rounds
- Re-running evaluation after updates
- Observing efficient incremental computation
- Understanding when incremental mode is beneficial

## The Program Flow

```rust
// 1. Create incremental context
let mut ctx = IntegrateContext::<_, RcFamily>::new_incremental(prov);

// 2. Define rules once
ctx.add_rule("path(a, b) = edge(a, b)")?;
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")?;

// 3. Round 1: Initial facts
ctx.add_facts("edge", vec![(None, (0, 1).into()), (None, (1, 2).into())], false)?;
ctx.run()?;

// 4. Round 2: Add more facts
ctx.add_facts("edge", vec![(None, (2, 3).into()), (None, (3, 4).into())], false)?;
ctx.run()?;  // Incremental update!

// 5. Round 3: Add more facts
ctx.add_facts("edge", vec![(None, (0, 3).into())], false)?;
ctx.run()?;  // Another incremental update!
```

## Expected Output

```
=== Incremental Evaluation Example ===

Created incremental context
Rules defined

=== Round 1: Initial Facts ===
Added edges: (0, 1), (1, 2)
Evaluation complete
Paths found: 3
  path(0, 1)
  path(1, 2)
  path(0, 2)

=== Round 2: Add More Edges ===
Added edges: (2, 3), (3, 4)
Incremental evaluation complete
Total paths now: 10
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

=== Round 3: Add Shortcut Edge ===
Added edge: (0, 3) [shortcut]
Incremental evaluation complete
Final path count: 10
(Note: count may not change - shortcut provides alternative derivation)

Sample paths:
  path(0, 1)
  path(1, 2)
  path(2, 3)
  path(3, 4)
  path(0, 2)
  path(1, 3)
  path(2, 4)
  path(0, 3)

=== Example Complete ===

Key Point:
  - new_incremental() enables efficient updates
  - Only affected facts are recomputed
  - Ideal for dynamic datasets
```

## Running This Example

```bash
cargo run
```

## Key Concepts

### Incremental vs Standard Context

**Standard context:**
```rust
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);
// Re-runs everything from scratch on each run()
```

**Incremental context:**
```rust
let mut ctx = IntegrateContext::<_, RcFamily>::new_incremental(prov);
// Only recomputes affected facts on each run()
```

### When to Use Incremental Mode

**Use incremental when:**
- Facts are added over time
- Dataset changes frequently
- You need to query after each update
- Re-running from scratch would be expensive

**Example use cases:**
- Real-time data streams
- Interactive applications
- Dynamic knowledge bases
- Simulation systems with evolving state

**Don't use incremental when:**
- Facts are static (added once)
- Complete re-evaluation is fast
- Memory overhead is a concern (incremental stores more state)

### How Incremental Evaluation Works

1. **First run:** Computes all facts normally
2. **Add new facts:** Marks affected relations as "dirty"
3. **Second run:**
   - Only recomputes relations that depend on new facts
   - Reuses unchanged results from previous run
   - Produces final output efficiently

**Example:**
```
Round 1: edge(0,1), edge(1,2)
  → Computes: path(0,1), path(1,2), path(0,2)

Round 2: ADD edge(2,3), edge(3,4)
  → Only computes NEW paths using new edges
  → Reuses: path(0,1), path(1,2), path(0,2)
  → Adds: path(2,3), path(3,4), path(1,3), path(2,4), path(0,3), path(1,4), path(0,4)
```

### Performance Benefits

For a graph with:
- **N** existing nodes
- **M** existing edges
- **k** new edges added

**Standard evaluation:**
- Complexity: O((N+k)² × (M+k))
- Re-processes everything

**Incremental evaluation:**
- Complexity: O(k × N × M) (roughly)
- Only processes new facts and their effects

**Speedup:** Can be 10-100x faster for small updates to large datasets

## Limitations and Considerations

### Memory Overhead

Incremental mode stores:
- All previous facts
- Intermediate computation state
- Dependency tracking metadata

**Trade-off:** Speed vs memory

### Monotonicity Requirement

Incremental evaluation assumes facts are **added only** (monotonic).

**Not supported:**
- Fact deletion
- Fact modification
- Non-monotonic negation

For these, use standard (non-incremental) mode or recreate context.

### When Incremental Doesn't Help

If updates affect **most** of the dataset:
- Incremental overhead > benefit
- Better to use standard mode

**Example:**
```rust
// Bad for incremental: 90% of graph changes
ctx.add_facts("edge", massive_update_vec, false)?;
```

## Real-World Example: Streaming Data

```rust
use std::time::Instant;

fn process_stream(data_stream: impl Iterator<Item = (i32, i32)>) -> Result<(), IntegrateError> {
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new_incremental(prov);

    ctx.add_relation("edge(i32, i32)")?;
    ctx.add_rule("path(a, b) = edge(a, b)")?;
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")?;

    for (batch_num, chunk) in data_stream.enumerate().chunks(100) {
        let start = Instant::now();

        // Add batch of facts
        let facts: Vec<_> = chunk.map(|(_, (a, b))| {
            (None, Tuple::from((a, b)))
        }).collect();

        ctx.add_facts("edge", facts, false)?;
        ctx.run()?;

        let elapsed = start.elapsed();
        let path_count = ctx.computed_relation_ref("path")?.len();

        println!("Batch {}: {} paths in {:?}", batch_num, path_count, elapsed);
    }

    Ok(())
}
```

## Next Steps

- **[complex_reasoning](../complex_reasoning/)** - Advanced provenance with proofs
- **[IntegrateContext API](../../../doc/src/rust_api/integrate_context.md)** - More configuration options
- **[Getting Started Guide](../../../doc/src/rust_api/getting_started.md)** - Other examples

## Related Documentation

- [IntegrateContext API](../../../doc/src/rust_api/integrate_context.md)
- [Getting Started Guide](../../../doc/src/rust_api/getting_started.md)
- [Provenance Types](../../../doc/src/rust_api/provenance.md)
