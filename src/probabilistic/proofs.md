# Proofs Provenance

Proofs are fundamental to understanding how Scallop derives conclusions. When Scallop computes a result, it doesn't just give you an answer - it can also tell you *why* that answer exists by tracking the derivation proofs.

## What are Proofs?

In Scallop, a **proof** is a set of base facts that, when combined together through rules, derive a conclusion. Think of it as the "evidence" or "reasoning chain" that supports a derived fact.

### Simple Example

Consider this simple graph program:

``` scl
rel edge = {(0, 1), (1, 2), (0, 2)}
rel path(a, c) = edge(a, c)
rel path(a, c) = path(a, b), edge(b, c)
query path(0, 2)
```

The fact `path(0, 2)` can be derived in **two different ways**:

1. **Proof 1**: Directly from `edge(0, 2)` (using the first rule)
2. **Proof 2**: From `edge(0, 1)` + `edge(1, 2)` (using the second rule)

Each proof is a set of **fact IDs** - unique identifiers for the base facts:
- Let's say `edge(0, 1)` is fact ID 0
- `edge(1, 2)` is fact ID 1
- `edge(0, 2)` is fact ID 2

Then:
- **Proof 1** = `{2}` (just uses fact 2)
- **Proof 2** = `{0, 1}` (uses facts 0 and 1 together)

---

## Proofs Provenance

The `proofs` provenance tracks **all possible derivation paths** for each conclusion.

### Enabling Proofs Tracking

In Scallop CLI:

``` bash
scli --provenance proofs program.scl
```

In Python:

``` py
import scallopy

ctx = scallopy.ScallopContext(provenance="proofs")
ctx.add_relation("edge", (int, int))
ctx.add_facts("edge", [(0, 1), (1, 2), (0, 2)])
ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")
ctx.run()

# Each result includes its proofs
for (proofs_obj, (start, end)) in ctx.relation("path"):
  print(f"Path ({start}, {end}): {proofs_obj}")
```

### Understanding Proof Structure

Proofs are represented as a **set of sets of fact IDs**:

```
Proofs = { {fact_id₁, fact_id₂}, {fact_id₃}, ... }
```

- The outer set represents **alternative derivations** (disjunction - OR)
- Each inner set represents **facts used together** (conjunction - AND)

**Example interpretation:**
```
Proofs = { {0, 1}, {2} }
```

This means: "The conclusion can be derived by using (fact 0 AND fact 1) OR (fact 2 alone)"

### Multiple Proofs for the Same Tuple

When a tuple can be derived in multiple ways, all proofs are tracked:

``` scl
rel edge = {(0, 1), (1, 2), (0, 2), (0, 3), (1, 3), (2, 3)}
rel path(a, c) = edge(a, c)
rel path(a, c) = path(a, b), edge(b, c)
```

For `path(0, 3)`, there might be many proofs:
- Direct: `{edge(0,3)}`
- Via 1: `{edge(0,1), edge(1,3)}`
- Via 2: `{edge(0,2), edge(2,3)}`
- Via 1→2: `{edge(0,1), edge(1,2), edge(2,3)}`

The `proofs` provenance tracks **all** of them.

---

## Top-K Proofs

Tracking all proofs can be expensive - there might be exponentially many derivations! The `topkproofs` provenance provides a **memory-efficient alternative** by keeping only the **top-K most probable proofs**.

### Why Top-K?

Consider a graph with many paths. A conclusion might have thousands of alternative derivations. In practice, we often only care about the most likely explanations.

### Using Top-K Proofs

In CLI:

``` bash
scli --provenance topkproofs --k 3 program.scl
```

In Python:

``` py
ctx = scallopy.ScallopContext(provenance="topkproofs", k=3)
```

The `k` parameter controls how many proofs to keep. With `k=3`, Scallop maintains the 3 most probable derivation paths for each conclusion.

### Top-K with Probabilities

Here's where Top-K shines - with probabilistic facts:

``` scl
rel edge = {
  0.9::(0, 1),  // High confidence edge
  0.8::(1, 2),  // High confidence edge
  0.2::(0, 2),  // Low confidence edge
  0.7::(1, 3)
}

rel path(a, c) = edge(a, c)
rel path(a, c) = path(a, b), edge(b, c)

query path(0, 2)
```

For `path(0, 2)`, there are two proofs:
1. Proof via 1: `{edge(0,1), edge(1,2)}` with probability 0.9 × 0.8 = 0.72
2. Direct proof: `{edge(0,2)}` with probability 0.2

With `topkproofs` and `k=1`, only the most probable proof (via node 1) would be kept.

### Exact Probability via WMC

Top-K proofs use **Weighted Model Counting (WMC)** to compute exact probabilities from the kept proofs. The proofs are converted to a Boolean formula and evaluated using a Sentential Decision Diagram (SDD) for efficient computation.

**Example:**
- Proofs: `{{0, 1}, {2}}`
- Boolean formula: `(f₀ ∧ f₁) ∨ f₂`
- With probabilities: `P(f₀)=0.9, P(f₁)=0.8, P(f₂)=0.2`
- WMC computes: `P = 0.72 + 0.2 - (0.72 × 0.2) = 0.776`

The inclusion-exclusion principle ensures we don't double-count when proofs overlap.

---

## DNF Formula Representation

Internally, Scallop represents proofs as **Disjunctive Normal Form (DNF)** formulas.

### What is DNF?

A DNF formula is a disjunction (OR) of conjunctions (AND):

```
Formula = (lit₁ ∧ lit₂ ∧ ...) ∨ (lit₃ ∧ lit₄ ∧ ...) ∨ ...
          \_____________/     \_____________/
              Clause 1            Clause 2
```

Each clause is a conjunction of literals (fact IDs).

### Example

Proofs `{{0, 1}, {2}}` becomes:

```
DNF = (fact_0 ∧ fact_1) ∨ fact_2
```

This structure makes it efficient to:
1. Combine proofs from different rules
2. Compute probabilities via WMC
3. Handle negation and disjunctions

### Operations on Proofs

Scallop's provenance framework defines how proofs combine:

**Addition (Disjunction)**: Merge alternative derivations
```
{fact_0} + {fact_1} = {fact_0, fact_1}
```

**Multiplication (Conjunction)**: Combine proofs from rule bodies
```
{{0}} × {{1}} = {{0, 1}}
{{0, 1}} × {{2}} = {{0, 1, 2}}
```

These operations follow semiring properties, making the system mathematically principled.

---

## Proofs vs. Other Provenances

Different provenance types have different tradeoffs:

| Provenance | Tracks Proofs? | Memory | Exact Probability | Use Case |
|------------|----------------|---------|-------------------|----------|
| `unit` | No | Minimal | N/A | Standard DataLog, no tracking |
| `proofs` | Yes, all | High | No | Full derivation tracking |
| `topkproofs` | Yes, top-K | Medium | Yes (via WMC) | Probabilistic reasoning with efficiency |
| `minmaxprob` | No | Minimal | No (bounds only) | Fast probabilistic bounds |
| `addmultprob` | No | Minimal | No (approximate) | Fast probabilistic approximation |

**Choose `proofs` when:**
- You need to understand all derivation paths
- Memory is not a concern
- You're debugging logic

**Choose `topkproofs` when:**
- You have probabilistic facts
- You need exact probabilities
- Memory efficiency matters
- You care about top explanations

**Choose `minmaxprob` when:**
- You need fast probabilistic bounds
- Exact probabilities aren't critical
- Memory is very limited

---

## Proof Debugging

For advanced proof debugging, Scallop provides `difftopkproofsdebug` provenance that exposes fact IDs and full proof structures. See [Debugging Proofs](../scallopy/debug_proofs.md) for details.

### Example: Finding Which Facts Were Used

``` py
import torch
import scallopy

ctx = scallopy.ScallopContext(provenance="difftopkproofsdebug", k=3)
ctx.add_relation("edge", (int, int))

# Add facts with explicit IDs
ctx.add_facts("edge", [
  ((torch.tensor(0.9), 1), (0, 1)),  # Fact ID 1
  ((torch.tensor(0.8), 2), (1, 2)),  # Fact ID 2
  ((torch.tensor(0.2), 3), (0, 2)),  # Fact ID 3
])

ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")
ctx.run()

# Results include fact IDs in proofs
for (result, tuple_data) in ctx.relation("path"):
  print(f"Tuple: {tuple_data}, Result: {result}")
  # Result contains probability and proofs with fact IDs
```

The proofs will show exactly which fact IDs were used to derive each path.

---

## Common Patterns

### Counting Proofs

Want to know how many ways something can be derived?

``` scl
rel count_derivations(n) = n = count(p: path(0, 2) with proof p)
```

(Note: This is conceptual - actual implementation depends on provenance support)

### Filtering by Proof Confidence

With probabilistic proofs, you can filter for high-confidence derivations:

``` scl
rel high_confidence_path(a, b) = path(a, b) and confidence(a, b) > 0.9
```

### Analyzing Derivation Depth

By tracking proofs, you can analyze how "deep" a derivation is (how many facts it uses):

- Proofs with 1 fact = direct facts
- Proofs with 2 facts = one-hop derivations
- Proofs with N facts = (N-1)-hop derivations

---

## Summary

- **Proofs** = sets of fact IDs that together derive a conclusion
- **`proofs` provenance** = tracks all derivation paths
- **`topkproofs` provenance** = keeps top-K most probable proofs for efficiency
- **DNF formulas** = internal representation enabling efficient computation
- **WMC** = algorithm for computing exact probabilities from proofs
- **Fact IDs** = enable traceability and debugging

Understanding proofs is key to:
1. Debugging your Scallop programs
2. Understanding why conclusions are drawn
3. Optimizing probabilistic reasoning
4. Building explainable AI systems

---

## Further Reading

- [Provenance](provenance.md) - The provenance semiring framework
- [Provenance Library](library.md) - All provenance types explained
- [Debugging Proofs](../scallopy/debug_proofs.md) - Advanced proof debugging with fact IDs
- [Logic and Probability](logic.md) - Combining symbolic and probabilistic reasoning
