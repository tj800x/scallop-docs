# Tags and Provenance

Provenance in Scallop tracks **how** facts are derived and **what metadata** they carry. Tags are the mechanism for attaching this metadata to facts. This page covers the language-level syntax for tags and provenance; for the complete guide to provenance types and their behavior, see [Probabilistic Programming](../probabilistic/index.md).

## What is Provenance?

**Provenance** is metadata that tracks:
- **Where** facts come from (which input facts contributed)
- **How probable** facts are (uncertainty quantification)
- **What derivations** led to facts (proof tracking)

Every fact in Scallop has an associated **tag** that stores this provenance information.

---

## Tag Syntax

Tags are attached to facts using the `::` operator:

**Basic syntax:**
```scl
rel TAG::fact
```

### Probabilistic Tags

The most common use of tags is for probabilities:

```scl
rel 0.8::edge(0, 1)
rel 0.9::edge(1, 2)
rel 0.7::edge(2, 3)
```

**Interpretation:** Edge (0,1) exists with probability 0.8.

### Set Notation with Tags

```scl
rel edge = {0.8::(0, 1), 0.9::(1, 2), 0.7::(2, 3)}
```

**Disjunctive facts (batch syntax):**
```scl
rel digit = {
  0.166::1;
  0.166::2;
  0.166::3;
  0.166::4;
  0.166::5;
  0.166::6
}
```

The semicolon `;` indicates mutual exclusion - exactly one is true.

---

## Tag Propagation

When rules derive new facts, tags are automatically combined according to the provenance type.

### Example: Transitive Closure with Probabilities

```scl
rel edge = {0.8::(0, 1), 0.9::(1, 2)}

rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
```

**With `--provenance minmaxprob`:**
```
path: {0.8::(0, 1), 0.9::(1, 2), 0.72::(0, 2)}
```

**How `(0, 2)` got 0.72:**
- Derived from: `path(0, 1)` (prob 0.8) AND `edge(1, 2)` (prob 0.9)
- Min-max probability: min(0.8, 0.9) = 0.72 (or 0.8 * 0.9 depending on semiring)

---

## Provenance Types

Different provenance types interpret tags differently. Set via command-line or Python API.

### Unit Provenance (Default)

No tags, standard Datalog:

```bash
scli program.scl
```

```scl
rel edge = {(0, 1), (1, 2)}
rel path(a, b) = edge(a, b)
query path
// Result: {(0, 1), (1, 2)}
```

### Proofs Provenance

Tracks which facts contributed to each derivation:

```bash
scli --provenance proofs program.scl
```

```scl
rel edge = {(0, 1), (1, 2), (2, 3)}
rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
```

**Result:**
```
path: {
  {{0}}::(0, 1),
  {{0, 1}}::(0, 2),
  {{0, 1, 2}}::(0, 3),
  {{1}}::(1, 2),
  {{1, 2}}::(1, 3),
  {{2}}::(2, 3)
}
```

Each set shows which input fact IDs were used.

### Min-Max Probability

Combines probabilities using min (AND) and max (OR):

```bash
scli --provenance minmaxprob program.scl
```

```scl
rel edge = {0.8::(0, 1), 0.9::(1, 2), 0.5::(0, 2)}

rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
```

**Result:**
```
path: {0.8::(0, 1), 0.9::(1, 2), 0.74::(0, 2)}
```

Path (0,2) has two derivations:
- Direct edge: 0.5
- Via (0,1,2): min(0.8, 0.9) = 0.8
- Combined: max(0.5, 0.8) = 0.8 (or more sophisticated via WMC)

### Top-K Proofs

Keeps only the K most probable derivations:

```bash
scli --provenance topkproofs -k 3 program.scl
```

Useful for large search spaces where tracking all proofs is expensive.

---

## Tag Syntax Details

### Omitting Tags

Facts without tags get a default tag (usually 1.0 or unit):

```scl
rel edge(0, 1)  // Implicitly: 1.0::edge(0, 1) in probabilistic mode
```

### Tags in Rules

You can explicitly tag derived facts:

```scl
rel 0.5::uncertain_path(a, b) = edge(a, b), maybe_accessible(a)
```

### Combining Tags

Rules with multiple conditions combine tags automatically:

```scl
rel 0.8::edge(0, 1)
rel 0.9::edge(1, 2)

// Tags combined: 0.8 * 0.9 = 0.72 (or min/max depending on provenance)
rel path(a, c) = edge(a, b), edge(b, c)
```

---

## Exclusive Disjunctions

The semicolon `;` in batch syntax indicates **mutual exclusion**:

```scl
rel color(1) = {0.4::"red"; 0.3::"blue"; 0.3::"green"}
```

**Meaning:** Exactly one color is true (probabilities sum to 1.0).

**Contrast with comma `,` (independent):**
```scl
rel color(1) = {0.4::"red", 0.3::"blue", 0.3::"green"}
```

**Meaning:** All colors could be true simultaneously.

---

## Provenance in CLI

### Specifying Provenance

```bash
# Default (unit)
scli program.scl

# Proofs
scli --provenance proofs program.scl

# Min-max probability
scli --provenance minmaxprob program.scl

# Top-K proofs
scli --provenance topkproofs -k 5 program.scl

# Add-mult probability
scli --provenance addmultprob program.scl
```

### Differentiable Provenance (Python only)

```python
import scallopy

ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
# Enables gradient computation for machine learning
```

---

## Common Patterns

### Pattern 1: Uncertain Facts

```scl
rel 0.9::parent("alice", "bob")
rel 0.8::parent("bob", "charlie")

rel ancestor(a, d) = parent(a, d)
rel ancestor(a, d) = ancestor(a, p), parent(p, d)

query ancestor("alice", "charlie")
// Result: 0.72::("alice", "charlie")
```

### Pattern 2: Confidence Scores

```scl
rel 0.95::detected_object(1, "car")
rel 0.87::detected_object(2, "person")
rel 0.62::detected_object(3, "bicycle")

rel high_confidence(id, obj) = detected_object(id, obj), prob > 0.9
```

### Pattern 3: Combining Evidence

```scl
rel 0.8::witness_a_says_guilty()
rel 0.6::witness_b_says_guilty()
rel 0.9::forensic_evidence_says_guilty()

rel strong_case() =
  witness_a_says_guilty(),
  witness_b_says_guilty(),
  forensic_evidence_says_guilty()
// Combined probability depends on provenance type
```

---

## Debugging Provenance

### Inspecting Tags

Use proofs provenance to see derivations:

```bash
scli --provenance proofs program.scl
```

### Tracing Derivations

Use debug flags to monitor tag propagation:

```bash
scli --debug-tag program.scl
```

Shows how tags flow through rules during execution.

---

## Summary

- **Tags** attach metadata (probabilities, proofs) to facts using `::`
- **Provenance types** determine how tags are combined
- **Syntax:** `TAG::fact` or `{TAG::fact1, TAG::fact2}`
- **Exclusive disjunction:** Use `;` for mutually exclusive choices
- **CLI:** Use `--provenance TYPE` to specify provenance
- **Common use:** Probabilistic reasoning, uncertainty tracking, proof tracking

For more details:
- [Probabilistic Programming](../probabilistic/index.md) - Complete provenance guide
- [Provenance Library](../probabilistic/library.md) - All 18 provenance types
- [Proofs](../probabilistic/proofs.md) - Understanding derivation tracking
- [Debug](../probabilistic/debug.md) - Debugging probabilistic programs
