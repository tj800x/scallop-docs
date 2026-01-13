# Complex Reasoning Example

This example demonstrates advanced provenance tracking with TopKProofsProvenance for explainable probabilistic reasoning.

## What This Example Demonstrates

- Using `TopKProofsProvenance` for proof tracking
- Understanding derivation proofs (how facts are derived)
- Weighted Model Counting (WMC) for probability computation
- Combining multiple alternative derivations
- Explainability in probabilistic reasoning

## The Program

```scl
// Probabilistic edges with fact IDs
rel 0.8::(0)::edge(0, 1)  // fact_id: 0
rel 0.9::(1)::edge(1, 2)  // fact_id: 1
rel 0.7::(2)::edge(2, 3)  // fact_id: 2
rel 0.6::(3)::edge(0, 2)  // fact_id: 3 (shortcut)

// Transitive closure
rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
```

## TopKProofs Provenance

**What it tracks:**
- **Proofs** - Sets of fact IDs that together derive a conclusion
- **Top-K** - Only keeps most probable K proofs (configurable)
- **WMC** - Computes exact probability from proof DNF formula

**Example:**
```
path(0, 2) has two proofs:
  Proof 1: {fact_3}           // Direct edge
  Proof 2: {fact_0, fact_1}   // Via node 1

Probability via WMC:
  P = P(fact_3) + P(fact_0 ∧ fact_1) - P(fact_3 ∧ fact_0 ∧ fact_1)
  P = 0.6 + (0.8 × 0.9) - (0.6 × 0.8 × 0.9)
  P = 0.6 + 0.72 - 0.432
  P = 0.888
```

## Expected Output

```
=== Complex Reasoning Example ===

Using TopKProofsProvenance:
  - Tracks derivation proofs (how facts are derived)
  - Computes probabilities via Weighted Model Counting
  - Keeps top-K most probable proofs

Adding probabilistic edges:
  edge(0, 1) with prob 0.8 [fact_id: 0]
  edge(1, 2) with prob 0.9 [fact_id: 1]
  edge(2, 3) with prob 0.7 [fact_id: 2]
  edge(0, 2) with prob 0.6 [fact_id: 3] (shortcut)

Rules defined:
  path(a, b) = edge(a, b)
  path(a, c) = path(a, b), edge(b, c)
  long_path(a, d) = path(a, b), path(b, c), path(c, d)

Executing...
Done

=== Paths with Probabilities ===
path(0, 1) = 0.8000
path(1, 2) = 0.9000
path(2, 3) = 0.7000
path(0, 2) = 0.8880  // Two derivations:
                           //   1. Direct: 0.6
                           //   2. Via 1: 0.8 × 0.9 = 0.72
                           //   Combined via WMC: ~0.85
path(1, 3) = 0.6300
path(0, 3) = 0.9168  // Multiple paths:
                           //   Best: via 1,2 = 0.8 × 0.9 × 0.7

=== Long Paths (3+ hops) ===
No long paths found (graph too small)

=== Understanding Proofs ===

TopKProofsProvenance tracks:
  1. Which facts were used in each derivation
  2. Multiple alternative derivations (proofs)
  3. Combines them using Weighted Model Counting

Example for path(0, 2):
  Proof 1: Uses fact_id 3 (direct edge 0→2)
    Probability: 0.6
  Proof 2: Uses fact_ids {0, 1} (edges 0→1, 1→2)
    Probability: 0.8 × 0.9 = 0.72
  Combined (inclusion-exclusion):
    0.6 + 0.72 - (0.6 × 0.72) ≈ 0.888

=== Example Complete ===

Key Takeaways:
  - Proofs track derivation history
  - WMC computes exact probabilities from proofs
  - TopK keeps most probable explanations
  - Useful for explainability and debugging
```

## Running This Example

```bash
cargo run
```

## Key Concepts

### Creating TopKProofs Context

```rust
use scallop_core::runtime::provenance::probabilistic::top_k_proofs::TopKProofsProvenance;

let prov = TopKProofsProvenance::<RcFamily>::new(3);  // Top-3 proofs
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);
```

**Parameter K:**
- K = 1: Only track best proof (fastest)
- K = 3: Track top-3 proofs (good balance)
- K = ∞: Track all proofs (most accurate, slowest)

### Adding Facts with IDs

```rust
ctx.add_facts("edge", vec![
    (Some((0.8, 0).into()), Tuple::from((0, 1))),  // (prob, fact_id)
    (Some((0.9, 1).into()), Tuple::from((1, 2))),
], false)?;
```

**Fact ID:**
- Uniquely identifies each input fact
- Used in proofs to track derivations
- Can be any usize value

### Proof Structure

A **proof** is a set of fact IDs:

```
Proof = {fact_id₁, fact_id₂, ..., fact_idₙ}
```

**Example:**
```
path(0, 3) derived from:
  edge(0, 1) [fact_0] ∧ edge(1, 2) [fact_1] ∧ edge(2, 3) [fact_2]
Proof: {0, 1, 2}
```

### DNF Formula

Multiple proofs form a **Disjunctive Normal Form** (DNF):

```
(fact_0 ∧ fact_1) ∨ (fact_3)
```

Represents two alternative ways to derive the same fact.

### Weighted Model Counting (WMC)

**Given:** DNF formula and fact probabilities

**Compute:** Exact probability using inclusion-exclusion

**Algorithm:**
1. Build SDD (Sentential Decision Diagram) from DNF
2. Evaluate SDD with probability semiring
3. Apply inclusion-exclusion for overlapping proofs

**Example:**
```
Formula: (f₀ ∧ f₁) ∨ f₃
Probabilities: P(f₀)=0.8, P(f₁)=0.9, P(f₃)=0.6

WMC:
  P((f₀ ∧ f₁) ∨ f₃)
  = P(f₀ ∧ f₁) + P(f₃) - P(f₀ ∧ f₁ ∧ f₃)
  = (0.8 × 0.9) + 0.6 - (0.8 × 0.9 × 0.6)
  = 0.72 + 0.6 - 0.432
  = 0.888
```

## Comparison with Other Provenances

### TopKProofs vs MinMaxProb

| Aspect | TopKProofs | MinMaxProb |
|--------|------------|------------|
| Probability | Exact (via WMC) | Approximate (min/max) |
| Explainability | Full proofs | No proofs |
| Performance | Slower (WMC overhead) | Faster (simple ops) |
| Memory | Stores DNF formulas | Just f64 |

**Example:**
```
path(0, 2) with two derivations:
  1. Direct: 0.6
  2. Via 1: 0.72

TopKProofs: 0.888 (exact via inclusion-exclusion)
MinMaxProb: 0.72 (just max)
```

### When to Use TopKProofs

**Use TopKProofs when:**
- Need exact probabilities
- Need explanation/provenance
- Debugging derivations
- Auditing reasoning
- Trust/safety critical applications

**Don't use when:**
- Speed is critical
- Approximate probabilities sufficient
- Memory constrained
- Very large datasets

## Real-World Applications

### 1. Explainable AI

```rust
// Which facts led to this conclusion?
let result = ctx.computed_relation_ref("diagnosis")?;
for elem in result.iter() {
    // elem.tag contains probability + proof information
    println!("Diagnosis: {:?}", elem.tuple);
    println!("Confidence: {:.2}", elem.tag);
    // Access proofs for explanation
}
```

### 2. Debugging Complex Rules

```rust
// Why did this fact get derived?
// Examine proofs to see which rules fired
```

### 3. Sensitivity Analysis

```rust
// How does changing a fact's probability affect results?
// Track which facts appear in top proofs
```

## Advanced: Accessing Proofs Programmatically

**Note:** Direct proof access requires deeper integration with provenance API.

```rust
// Conceptual example (actual API may differ)
let path = ctx.computed_relation_ref("path")?;
for elem in path.iter() {
    // elem.tag is f64 (probability from WMC)
    // Proofs are internal to provenance context
    // For debug access, use DiffTopKProofsDebugProvenance
}
```

## Next Steps

- **[basic_datalog](../basic_datalog/)** - Start with basics
- **[probabilistic_reasoning](../probabilistic_reasoning/)** - Simpler probabilistic reasoning
- **[Provenance Types Guide](../../../doc/src/rust_api/provenance.md)** - All provenance types

## Related Documentation

- [Provenance Types](../../../doc/src/rust_api/provenance.md)
- [Getting Started Guide](../../../doc/src/rust_api/getting_started.md)
- [IntegrateContext API](../../../doc/src/rust_api/integrate_context.md)
- [Research Paper: Scallop](https://arxiv.org/abs/2304.04812)
