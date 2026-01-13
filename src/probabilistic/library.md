# Provenance Library

Scallop provides **18 different provenance types** covering discrete logic, probabilistic reasoning, and differentiable computation. This reference guide helps you choose the right provenance for your application.

## Overview

Provenances are organized into three categories:

1. **Discrete** (5 types) - For standard logic programming without probabilities
2. **Probabilistic** (6 types) - For reasoning under uncertainty
3. **Differentiable** (7+ types) - For integration with neural networks and gradient-based learning

Each provenance defines how tags (like probabilities) propagate through logical rules, following the provenance semiring framework.

---

## Discrete Provenances

Use discrete provenances when you don't need probabilistic reasoning.

### `unit` - No Tracking

**Description:** Standard DataLog with no provenance tracking. Fastest and most memory-efficient.

**Tag Type:** None (unit type)

**Use When:**
- Pure logic programming
- No need for probabilities or proof tracking
- Maximum performance needed

**CLI:**
``` bash
scli program.scl  # unit is default
```

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="unit")
```

**Example:**
``` scl
rel edge = {(0, 1), (1, 2)}
rel path(a, c) = edge(a, c)
rel path(a, c) = path(a, b), edge(b, c)
query path  // {(0,1), (0,2), (1,2)}
```

---

### `proofs` - Full Derivation Tracking

**Description:** Tracks all possible derivation proofs for each conclusion. Each proof is a set of base facts that together derive the result.

**Tag Type:** `Proofs` (set of sets of fact IDs)

**Use When:**
- Debugging logic programs
- Understanding all derivation paths
- Explainability is critical
- Memory is not constrained

**CLI:**
``` bash
scli --provenance proofs program.scl
```

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="proofs")
```

**Output:** Each tuple comes with all its proofs
```
path: {({0, 1}, (0, 2)), ({0}, (0, 1)), ...}
```

**See:** [Proofs Provenance](proofs.md) for detailed explanation

---

### `boolean` - Boolean Algebra

**Description:** Uses boolean semiring where tags are True/False values.

**Tag Type:** `bool`

**Operations:**
- Addition (OR): `true ∨ false = true`
- Multiplication (AND): `true ∧ false = false`

**Use When:**
- Tracking whether facts exist
- Boolean constraints
- Reachability analysis

**Example:**
``` py
ctx = scallopy.ScallopContext(provenance="boolean")
ctx.add_facts("reliable_edge", [(True, (0, 1)), (False, (1, 2))])
```

---

### `natural` - Natural Numbers

**Description:** Counts using natural numbers. Useful for counting derivations.

**Tag Type:** Natural numbers (0, 1, 2, ...)

**Operations:**
- Addition: Standard addition
- Multiplication: Standard multiplication

**Use When:**
- Counting how many ways something is derived
- Aggregation over counts

---

### `tropical` - Tropical Semiring

**Description:** Min-plus tropical semiring for shortest path problems.

**Tag Type:** Integers

**Operations:**
- Addition: `min(a, b)`
- Multiplication: `a + b`
- Zero: `∞`
- One: `0`

**Use When:**
- Shortest path algorithms
- Cost minimization
- Distance metrics

**Example:**
``` scl
rel edge = {5::(0, 1), 3::(1, 2), 7::(0, 2)}  // Weighted edges
rel shortest_path(a, c) = edge(a, c)
rel shortest_path(a, c) = shortest_path(a, b), edge(b, c)
query shortest_path  // Finds minimum-cost paths
```

---

## Probabilistic Provenances

Use probabilistic provenances for reasoning under uncertainty.

### `minmaxprob` - Conservative Probability Bounds

**Description:** Fast probabilistic bounds using min/max operations. Not probabilistically exact but provides conservative estimates.

**Tag Type:** `f64` (probability between 0.0 and 1.0)

**Operations:**
- Addition (OR): `max(p1, p2)` - Most optimistic
- Multiplication (AND): `min(p1, p2)` - Most pessimistic

**Use When:**
- Fast probabilistic reasoning needed
- Exact probabilities not critical
- Conservative bounds acceptable
- Very large graphs

**CLI:**
``` bash
scli --provenance minmaxprob program.scl
```

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="minmaxprob")
ctx.add_facts("edge", [(0.8, (0, 1)), (0.9, (1, 2))])
```

**Example Output:**
```
path: {0.8::(0, 1), 0.8::(0, 2), 0.9::(1, 2)}
```

**Note:** Not probabilistically accurate! Path (0,2) uses two edges with p=0.8 and p=0.9, but reports min(0.8, 0.9) = 0.8, not the actual 0.72.

---

### `addmultprob` - Add-Mult Probability

**Description:** Sum-product semiring with clamping. Fast but approximate.

**Tag Type:** `f64`

**Operations:**
- Addition (OR): `min(p1 + p2, 1.0)` - Clamped sum
- Multiplication (AND): `p1 * p2`

**Use When:**
- Fast probabilistic approximation
- Probabilities won't sum > 1.0
- Exact computation too expensive

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="addmultprob")
```

**Limitation:** Sum of probabilities can exceed 1.0 before clamping, violating probability axioms.

---

### `topkproofs` - Top-K Proofs with Exact Probability

**Description:** Keeps top-K most probable proofs and computes exact probability using Weighted Model Counting (WMC) via Sentential Decision Diagrams (SDD).

**Tag Type:** `DNFFormula` internally, `f64` output

**Use When:**
- Exact probabilities needed
- Memory efficiency matters
- Only top explanations relevant
- Standard probabilistic reasoning

**CLI:**
``` bash
scli --provenance topkproofs --k 3 program.scl
```

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="topkproofs", k=3)
```

**Parameters:**
- `k`: Number of proofs to keep (default: 3)
- `wmc_with_disjunctions`: Include mutual exclusion in WMC (default: false)

**Example:**
``` py
ctx = scallopy.ScallopContext(provenance="topkproofs", k=5)
ctx.add_facts("edge", [
  (0.9, (0, 1)),
  (0.8, (1, 2)),
  (0.2, (0, 2))
])
ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")
ctx.run()

for (prob, (a, b)) in ctx.relation("path"):
  print(f"Path ({a}, {b}): {prob:.4f}")
# Output:
# Path (0, 1): 0.9000
# Path (0, 2): 0.776  # Exact: 0.72 + 0.2 - 0.72*0.2 (inclusion-exclusion)
# Path (1, 2): 0.8000
```

**Key Features:**
- **WMC** computes exact joint probabilities
- **SDD** enables efficient Boolean formula evaluation
- **Mutual exclusion** supported via disjunctions
- **Top-K** keeps memory bounded

**See:** [Proofs Provenance](proofs.md) for WMC details

---

### `probproofs` - All Proofs with Exact Probability

**Description:** Like `topkproofs` but keeps ALL proofs. More accurate but higher memory.

**Tag Type:** `ProbProofs`

**Use When:**
- Exact probabilities with all derivations
- Memory not constrained
- Complete proof tracking needed

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="probproofs")
```

**Tradeoff:** Higher memory than `topkproofs`, but no loss of proofs.

---

### `samplekproofs` - Sampled K Proofs

**Description:** Samples K proofs probabilistically for unbiased statistical approximation.

**Tag Type:** Sampled proofs

**Use When:**
- Stochastic approximation acceptable
- Memory very limited
- Statistical estimates sufficient

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="samplekproofs", k=10)
```

---

### `topbottomkclauses` - Top-K and Bottom-K Clauses

**Description:** Keeps both top-K and bottom-K clauses for full negation support.

**Tag Type:** Top/bottom clause sets

**Use When:**
- Negation in probabilistic programs
- Need both positive and negative evidence

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="topbottomkclauses", k=3)
```

**Example:**
``` py
import scallopy
import torch

ctx = scallopy.ScallopContext(provenance="difftopbottomkclauses")
ctx.add_relation("obj_color", (int, str))
ctx.add_facts("obj_color", [
  (torch.tensor(0.99), (0, "blue")),
  (torch.tensor(0.01), (0, "green"))
])
ctx.add_rule('num_blue(x) :- x = count(o: obj_color(o, "blue"))')
ctx.run()
```

---

## Differentiable Provenances

Use differentiable provenances for integration with neural networks and gradient-based learning.

### `difftopkproofs` - Differentiable Top-K Proofs

**Description:** `topkproofs` with PyTorch tensor support and gradient computation.

**Tag Type:** `torch.Tensor`

**Use When:**
- Training with gradient descent
- Neurosymbolic AI applications
- End-to-end learning with logic

**Python:**
``` py
import torch
import scallopy

ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
ctx.add_relation("edge", (int, int))
ctx.add_facts("edge", [
  (torch.tensor(0.9, requires_grad=True), (0, 1)),
  (torch.tensor(0.8, requires_grad=True), (1, 2))
])
ctx.add_rule("path(a, c) = edge(a, c)")
ctx.run()

# Results are tensors with gradient support
for (prob_tensor, (a, b)) in ctx.relation("path"):
  loss = (prob_tensor - target) ** 2
  loss.backward()  # Gradients flow back through logic
```

**Key Feature:** Probabilities are PyTorch tensors, enabling backpropagation through logical reasoning.

---

### `difftopkproofsdebug` - With Stable Fact IDs ⭐

**Description:** `difftopkproofs` with **user-provided stable fact IDs** for debugging and traceability. **ONLY provenance supporting stable IDs**.

**Tag Type:** `(torch.Tensor, int)` - probability and fact ID

**Use When:**
- Debugging probabilistic programs
- Fact tracking and retraction needed
- Provenance auditing required
- Building knowledge management systems (HNLE use case)

**Python:**
``` py
import torch
import scallopy

ctx = scallopy.ScallopContext(provenance="difftopkproofsdebug", k=3)
ctx.add_relation("edge", (int, int))

# !!! SPECIAL FACT FORMAT with explicit IDs !!!
ctx.add_facts("edge", [
  ((torch.tensor(0.9), 1), (0, 1)),  # Fact ID = 1
  ((torch.tensor(0.8), 2), (1, 2)),  # Fact ID = 2
  ((torch.tensor(0.2), 3), (0, 2)),  # Fact ID = 3
])

ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")
ctx.run()

# Proofs reference stable fact IDs
for (result, tuple_data) in ctx.relation("path"):
  print(f"Tuple: {tuple_data}, Result: {result}")
```

**Fact ID Requirements:**
- IDs must start from 1
- Must be contiguous (no gaps)
- Must be unique across all facts
- User is responsible for ID management

**Output Includes Proofs:**

When used with `forward()` in modules, returns `(result_tensor, proofs)`:

```python
proofs = [
  [ # Datapoint 1
    [ # Proofs of tuple 1
      [(True, 1), (True, 2)],  # Proof uses fact IDs 1 and 2
    ],
    [ # Proofs of tuple 2
      [(True, 3)],  # Proof uses fact ID 3
    ]
  ]
]
```

**Proof Structure:** `List[List[List[List[Tuple[bool, int]]]]]`
- Batch → Datapoint → Proofs → Proof → Literal
- Each literal: `(is_positive, fact_id)`

**Use Case - HNLE MCP:** Enables fact retraction by stable ID, critical for knowledge management with complex string data.

**Limitations (from FloatWithID research):**
- Display format `0.8 [ID(42)]` exists but cannot be parsed from .scl files
- IDs only exist during API execution (not persisted to .scl)
- Only this provenance type supports user-provided IDs

**See:** [Debugging Proofs](../scallopy/debug_proofs.md) for detailed examples

---

### `diffminmaxprob` - Differentiable Min-Max

**Description:** `minmaxprob` with gradient support.

**Tag Type:** `torch.Tensor`

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="diffminmaxprob")
```

---

### `diffaddmultprob` - Differentiable Add-Mult

**Description:** `addmultprob` with gradient support.

**Tag Type:** `torch.Tensor`

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="diffaddmultprob")
```

---

### `diffsamplekproofs` - Differentiable Sampled Proofs

**Description:** `samplekproofs` with gradient support and unbiased gradient estimates.

**Tag Type:** `torch.Tensor`

**Python:**
``` py
ctx = scallopy.ScallopContext(provenance="diffsamplekproofs", k=10)
```

---

### Additional Differentiable Variants

Scallop also provides Python-based differentiable provenances:

- **`diffaddmultprob2`** - Pure Python implementation of add-mult
- **`diffnandmultprob2`** - NAND-mult semiring in Python
- **`diffmaxmultprob2`** - Max-mult semiring in Python

These are useful for experimentation and custom semiring development.

---

## Provenance Selection Guide

### Decision Tree

**Need gradients for neural networks?**
- → YES: Use `diff*` provenance (differentiable)
  - Need fact ID tracking? → `difftopkproofsdebug`
  - Need exact probability? → `difftopkproofs`
  - Need speed? → `diffminmaxprob` or `diffaddmultprob`
- → NO: Continue below

**Need probabilities?**
- → YES: Use probabilistic provenance
  - Need exact probability? → `topkproofs` (recommended) or `probproofs`
  - Need speed over accuracy? → `minmaxprob` or `addmultprob`
  - Need sampling? → `samplekproofs`
- → NO: Use discrete provenance
  - Need proof tracking? → `proofs`
  - Need boolean logic? → `boolean`
  - Need shortest paths? → `tropical`
  - Need standard logic? → `unit` (fastest)

### Performance Characteristics

| Provenance | Speed | Memory | Probability Accuracy | Gradient Support |
|------------|-------|--------|---------------------|------------------|
| `unit` | ★★★★★ | ★★★★★ | N/A | No |
| `proofs` | ★★☆☆☆ | ★☆☆☆☆ | N/A | No |
| `minmaxprob` | ★★★★★ | ★★★★★ | Bounds only | No |
| `addmultprob` | ★★★★★ | ★★★★★ | Approximate | No |
| `topkproofs` | ★★★☆☆ | ★★★☆☆ | Exact | No |
| `probproofs` | ★★☆☆☆ | ★☆☆☆☆ | Exact | No |
| `difftopkproofs` | ★★★☆☆ | ★★★☆☆ | Exact | Yes |
| `difftopkproofsdebug` | ★★☆☆☆ | ★★☆☆☆ | Exact | Yes |

### Common Use Cases

**Knowledge Graph Reasoning:**
``` py
ctx = scallopy.ScallopContext(provenance="topkproofs", k=5)
```

**Neurosymbolic AI (Training):**
``` py
ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
```

**Fast Probabilistic Queries:**
``` py
ctx = scallopy.ScallopContext(provenance="minmaxprob")
```

**Debugging Logic:**
``` py
ctx = scallopy.ScallopContext(provenance="proofs")
```

**Fact Tracking / Retraction (HNLE):**
``` py
ctx = scallopy.ScallopContext(provenance="difftopkproofsdebug", k=3)
```

**Shortest Path:**
``` py
ctx = scallopy.ScallopContext(provenance="tropical")
```

---

## Configuration Options

### Common Parameters

**k** - Number of proofs to keep (for top-k provenances)
``` py
ctx = scallopy.ScallopContext(provenance="topkproofs", k=5)
```

**wmc_with_disjunctions** - Include mutual exclusion in probability computation
``` py
ctx = scallopy.ScallopContext(
  provenance="topkproofs",
  k=3,
  wmc_with_disjunctions=True  # Respect mutual exclusion
)
```

**train_k / test_k** - Different k values for training vs. testing
``` py
module = scallopy.Module(
  provenance="difftopkproofs",
  train_k=3,  # Keep 3 proofs during training
  test_k=10,  # Keep 10 proofs during testing
  ...
)
```

---

## Summary

- **18 provenance types** covering discrete, probabilistic, and differentiable reasoning
- **Discrete** (`unit`, `proofs`, `boolean`, `natural`, `tropical`) for pure logic
- **Probabilistic** (`minmaxprob`, `addmultprob`, `topkproofs`, etc.) for uncertainty
- **Differentiable** (`diff*` variants) for neural network integration
- **`difftopkproofsdebug`** is special - only one with stable user-provided fact IDs
- **Choose based on**: speed, memory, accuracy, gradients, and tracking needs

---

## Further Reading

- [Provenance](provenance.md) - Provenance semiring framework
- [Proofs](proofs.md) - Understanding derivation proofs
- [Debugging Proofs](../scallopy/debug_proofs.md) - Using `difftopkproofsdebug`
- [Scallopy Provenance](../scallopy/provenance.md) - Python API for provenance configuration
- [Logic and Probability](logic.md) - Combining symbolic and probabilistic reasoning
