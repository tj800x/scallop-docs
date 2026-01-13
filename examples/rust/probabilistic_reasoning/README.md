# Probabilistic Reasoning Example

This example demonstrates probabilistic reasoning using the MinMaxProbProvenance semiring.

## What This Example Demonstrates

- Creating an `IntegrateContext` with `MinMaxProbProvenance`
- Adding facts with probabilities using `add_facts()`
- Understanding min-max semiring semantics
- Propagating confidence through transitive closure
- Interpreting probabilistic query results

## The Program

```scl
// Probabilistic edges with confidence scores
rel 0.9::edge(0, 1)
rel 0.8::edge(1, 2)
rel 0.7::edge(2, 3)
rel 0.6::edge(0, 2)  // Shortcut with lower confidence

// Transitive closure
rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
```

## MinMaxProb Semiring

The MinMaxProbProvenance uses:

**Addition (OR) - Best Alternative:**
```
add(p1, p2) = max(p1, p2)
```

**Multiplication (AND) - Weakest Link:**
```
mult(p1, p2) = min(p1, p2)
```

**Intuition:**
- When multiple derivations exist, take the **highest** probability (best path)
- When combining facts in a conjunction, take the **lowest** probability (weakest link)

## Expected Output

```
=== Probabilistic Reasoning Example ===

Using MinMaxProbProvenance:
  - add(p1, p2) = max(p1, p2)  // Best alternative
  - mult(p1, p2) = min(p1, p2) // Weakest link

Adding probabilistic edges:
  edge(0, 1) with probability 0.9
  edge(1, 2) with probability 0.8
  edge(2, 3) with probability 0.7
  edge(0, 2) with probability 0.6 (shortcut)

Rules defined:
  path(a, b) = edge(a, b)
  path(a, c) = path(a, b), edge(b, c)

Executing...
Done

Probabilistic Paths:
(Showing how probabilities propagate)

  path(0, 1) with confidence: 0.90  // Direct edge: 0.9
  path(1, 2) with confidence: 0.80  // Direct edge: 0.8
  path(2, 3) with confidence: 0.70  // Direct edge: 0.7
  path(0, 2) with confidence: 0.80  // max(0.6 direct, min(0.9, 0.8) via 1) = max(0.6, 0.8) = 0.8
  path(1, 3) with confidence: 0.70  // min(0.8, 0.7) = 0.7
  path(0, 3) with confidence: 0.70  // Best path via 1,2: min(0.9, 0.8, 0.7) = 0.7

=== Example Complete ===
```

## Key Derivations Explained

### path(0, 2) = 0.80

Two derivations:
1. **Direct edge:** `edge(0, 2)` with prob 0.6
2. **Via node 1:** `edge(0, 1) ∧ edge(1, 2)` = `min(0.9, 0.8)` = 0.8

Combined: `max(0.6, 0.8)` = **0.8**

### path(0, 3) = 0.70

Multiple paths:
1. Via 1,2: `min(0.9, 0.8, 0.7)` = 0.7
2. Via 2 (using shortcut): `min(0.8, 0.7)` = 0.7

Best: **0.7**

## Running This Example

```bash
cargo run
```

## Key Concepts

### Adding Facts with Probabilities

```rust
ctx.add_facts("edge", vec![
    (Some(0.9.into()), Tuple::from((0i32, 1i32))),
    (Some(0.8.into()), Tuple::from((1i32, 2i32))),
], false)?;
```

**Parameters:**
- `Some(prob.into())` - Wrap probability in Option and convert to InputTag
- `false` - Skip type checking (already declared relation)

### Accessing Probabilities in Results

```rust
for elem in path.iter() {
    println!("Probability: {}", elem.tag);  // elem.tag is f64
    println!("Tuple: {:?}", elem.tuple);
}
```

For MinMaxProbProvenance:
- `elem.tag` is `f64` (the probability)
- Range: 0.0 (impossible) to 1.0 (certain)

### When to Use MinMaxProb

**Good for:**
- Fuzzy logic
- Confidence propagation
- "Weakest link" reasoning
- When conjunctions represent sequential dependencies

**Example use cases:**
- Network reliability (path strength = weakest link)
- Multi-step processes (overall confidence = least confident step)
- Information flow (confidence degradation)

## Comparison with Other Provenances

| Provenance | path(0, 2) | Interpretation |
|------------|------------|----------------|
| **MinMaxProb** | 0.80 | Best path has weakest link 0.8 |
| AddMultProb | 0.82 | Probability via inclusion-exclusion |
| Unit | Unit | Path exists (no probability) |

## Next Steps

- **[foreign_functions](../foreign_functions/)** - Add custom computations
- **[complex_reasoning](../complex_reasoning/)** - TopKProofs for explanations
- **[Provenance Types Guide](../../../doc/src/rust_api/provenance.md)** - Deep dive into semirings

## Related Documentation

- [Getting Started Guide](../../../doc/src/rust_api/getting_started.md)
- [Provenance Types](../../../doc/src/rust_api/provenance.md)
- [IntegrateContext API](../../../doc/src/rust_api/integrate_context.md)
