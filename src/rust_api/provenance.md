# Provenance Types

This guide covers Scallop's **provenance tracking system** - a powerful semiring-based framework for reasoning with different semantics.

## Overview

Provenance determines **how facts are tagged** and **how tags combine** during reasoning. Scallop's unified execution engine can switch between discrete logic, probabilistic reasoning, and differentiable computation while maintaining **full traceability** of derivations.

### What is Provenance?

In Scallop, every fact has an associated **tag** that tracks metadata:

```rust
// Without provenance (standard DataLog)
edge(0, 1)

// With probabilistic provenance
0.8::edge(0, 1)  // 80% confidence

// With counting provenance
5::edge(0, 1)    // Appears 5 times
```

**Key insight:** The same Scallop program can execute with different provenance types to answer different questions:
- **Unit** - "Does this fact hold?" (true/false)
- **Natural** - "How many derivations exist?" (count)
- **MinMaxProb** - "What's the confidence?" (probability)
- **TopKProofs** - "What are the top-K explanations?" (proofs + probability)

### The Three-Stage Tag Flow

```
Input Facts              Runtime Execution        Output Results
    ↓                           ↓                        ↓
InputTag ──tagging_fn()→ Tag ──operations→ Tag ──recover_fn()→ OutputTag
```

**Example:**
```rust
// Input: User provides probability
InputTag = 0.8

// Internal: Converted to provenance tag
Tag = 0.8  // For MinMaxProbProvenance

// Output: Result displayed to user
OutputTag = 0.56  // After combining: 0.8 * 0.7 = 0.56
```

---

## The Provenance Trait

### Trait Definition

```rust
pub trait Provenance: Clone + 'static {
    /// The input tag space (what users provide)
    type InputTag: Clone + Debug + StaticInputTag;

    /// The internal tag space (used during execution)
    type Tag: Tag;

    /// The output tag space (what users see in results)
    type OutputTag: Clone + Debug + Display;

    /// Name of the provenance
    fn name(&self) -> String;

    /// Convert input tag to internal tag
    fn tagging_fn(&self, ext_tag: Self::InputTag) -> Self::Tag;

    /// Convert optional input tag (None → one())
    fn tagging_optional_fn(&self, ext_tag: Option<Self::InputTag>) -> Self::Tag {
        match ext_tag {
            Some(et) => self.tagging_fn(et),
            None => self.one(),
        }
    }

    /// Convert internal tag to output tag
    fn recover_fn(&self, t: &Self::Tag) -> Self::OutputTag;

    /// Check if a fact should be discarded
    fn discard(&self, t: &Self::Tag) -> bool;

    /// Zero element (disjunction identity)
    fn zero(&self) -> Self::Tag;

    /// One element (conjunction identity)
    fn one(&self) -> Self::Tag;

    /// Add operation (disjunction, OR)
    fn add(&self, t1: &Self::Tag, t2: &Self::Tag) -> Self::Tag;

    /// Multiply operation (conjunction, AND)
    fn mult(&self, t1: &Self::Tag, t2: &Self::Tag) -> Self::Tag;

    /// Negate operation (NOT)
    fn negate(&self, t: &Self::Tag) -> Option<Self::Tag> {
        None  // Default: negation not supported
    }

    /// Check if tag has saturated (convergence)
    fn saturated(&self, t_old: &Self::Tag, t_new: &Self::Tag) -> bool;

    /// Get weight of a tag (for ranking)
    fn weight(&self, tag: &Self::Tag) -> f64 {
        1.0  // Default: all tags equally weighted
    }
}
```

### Associated Types

**InputTag** - What users provide when adding facts:
```rust
ctx.add_facts("edge", vec![
    (Some(0.8.into()), (0, 1).into()),  // InputTag = f64 for MinMaxProb
], false)?;
```

**Tag** - Internal representation during execution:
```rust
// MinMaxProbProvenance uses f64 as Tag
type Tag = f64;

// TopKProofsProvenance uses DNFFormula
type Tag = Rc<DNFFormula>;
```

**OutputTag** - What users see in results:
```rust
for elem in results.iter() {
    println!("Tag: {}, Tuple: {:?}", elem.tag, elem.tuple);
    // elem.tag is OutputTag type
}
```

---

## Available Provenance Types

### Discrete Provenances

#### UnitProvenance - Standard DataLog

**No tracking** - Classic logical reasoning.

```rust
use scallop_core::runtime::provenance::discrete::unit::UnitProvenance;

impl Provenance for UnitProvenance {
    type InputTag = ();
    type Tag = Unit;
    type OutputTag = Unit;

    fn add(&self, _t1: &Unit, _t2: &Unit) -> Unit { Unit }  // OR
    fn mult(&self, _t1: &Unit, _t2: &Unit) -> Unit { Unit } // AND
}
```

**Use case:** Traditional DataLog queries without metadata.

```rust
let prov = UnitProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

ctx.add_facts("edge", vec![
    (None, (0, 1).into()),  // No tag
    (None, (1, 2).into()),
], false)?;
```

#### BooleanProvenance - Boolean Algebra

**Boolean tags** - Negation-as-failure support.

```rust
type InputTag = bool;
type Tag = bool;
type OutputTag = bool;

// Semiring operations:
fn add(&self, t1: &bool, t2: &bool) -> bool { *t1 || *t2 }   // OR
fn mult(&self, t1: &bool, t2: &bool) -> bool { *t1 && *t2 }  // AND
fn negate(&self, t: &bool) -> Option<bool> { Some(!*t) }     // NOT
```

**Use case:** Programs with negation.

#### NaturalProvenance - Counting

**Count multiplicity** - Track number of derivations.

```rust
type InputTag = usize;
type Tag = usize;
type OutputTag = usize;

// Semiring operations:
fn add(&self, t1: &usize, t2: &usize) -> usize { t1 + t2 }   // Sum
fn mult(&self, t1: &usize, t2: &usize) -> usize { t1 * t2 }  // Product
```

**Use case:** Cardinality queries, bag semantics.

```rust
let prov = NaturalProvenance::default();
// Fact appears 3 times
ctx.add_facts("edge", vec![(Some(3), (0, 1).into())], false)?;
```

#### ProofsProvenance - Derivation Tracking

**Proof trees** - Track all derivation paths (no probabilities).

```rust
type InputTag = Exclusion;
type Tag = Rc<Proofs>;
type OutputTag = Rc<Proofs>;
```

**Use case:** Debugging, explainability without probabilities.

### Probabilistic Provenances

#### MinMaxProbProvenance - Probabilistic (Min-Max Semiring)

**Probability tracking** with min-max semantics.

```rust
use scallop_core::runtime::provenance::probabilistic::min_max_prob::MinMaxProbProvenance;

impl Provenance for MinMaxProbProvenance {
    type InputTag = f64;
    type Tag = f64;
    type OutputTag = f64;

    fn add(&self, t1: &f64, t2: &f64) -> f64 {
        t1.max(*t2)  // Best alternative (OR)
    }

    fn mult(&self, t1: &f64, t2: &f64) -> f64 {
        t1.min(*t2)  // Weakest link (AND)
    }

    fn negate(&self, p: &f64) -> Option<f64> {
        Some(1.0 - p)  // Complement probability
    }
}
```

**Semiring intuition:**
- `add` (OR) = take maximum probability (best alternative)
- `mult` (AND) = take minimum probability (weakest link)

**Use case:** Fuzzy logic, confidence propagation.

```rust
let prov = MinMaxProbProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

ctx.add_facts("edge", vec![
    (Some(0.8.into()), (0, 1).into()),
    (Some(0.7.into()), (1, 2).into()),
], false)?;

// path(0, 2) derives with prob = min(0.8, 0.7) = 0.7
```

#### AddMultProbProvenance - Probabilistic (Add-Mult Semiring)

**Independent events** probability.

```rust
fn add(&self, t1: &f64, t2: &f64) -> f64 {
    t1 + t2 - (t1 * t2)  // Inclusion-exclusion (OR)
}

fn mult(&self, t1: &f64, t2: &f64) -> f64 {
    t1 * t2  // Independent events (AND)
}
```

**Semiring intuition:**
- `add` (OR) = inclusion-exclusion principle
- `mult` (AND) = independent probability multiplication

**Use case:** Statistical reasoning, independent events.

#### TopKProofsProvenance - Top-K Most Probable Proofs

**Track top-K derivation proofs** with probabilities.

```rust
type InputTag = InputExclusiveProb;
type Tag = Rc<DNFFormula>;
type OutputTag = f64;
```

**Internally tracks:**
- DNF formula representing proof combinations
- Computes probability via Weighted Model Counting (WMC)
- Returns top-K most probable derivations

**Use case:** Explanation generation, ranking derivations.

```rust
use scallop_core::runtime::provenance::probabilistic::top_k_proofs::TopKProofsProvenance;

let prov = TopKProofsProvenance::<RcFamily>::new(3);  // Top-3 proofs
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

// Results include probability computed from top proofs
```

#### ProbProofsProvenance - Exact Probability with All Proofs

**Complete proof tracking** with exact probabilities.

```rust
type InputTag = ProbProofs<RcFamily>;
type Tag = Rc<ProbProofs<RcFamily>>;
type OutputTag = f64;
```

**Tracks all derivation paths** and computes exact probability via SDD (Sentential Decision Diagram).

**Use case:** Exact probabilistic reasoning, complete explanations.

### Differentiable Provenances

These provenances support **gradient computation** for integration with machine learning frameworks like PyTorch.

#### DiffTopKProofsProvenance\<T\> - Differentiable Top-K

**Backpropagation support** for neural-symbolic integration.

```rust
type InputTag = InputExclusiveDiffProb<T>;
type Tag = Rc<DNFFormula>;
type OutputTag = (f64, Vec<T>);  // (probability, gradients)
```

**External tag `T`:** Typically a PyTorch tensor for gradient tracking.

**Use case:** Neural-symbolic learning, gradient-based optimization.

#### DiffTopKProofsDebugProvenance\<T\> - Differentiable with Proofs

**Debug variant** that returns proofs alongside gradients.

```rust
type InputTag = InputExclusiveDiffProbWithID<T>;
type OutputTag = (f64, Vec<T>, Vec<Proofs>);  // (prob, gradients, proofs)
```

**Unique feature:** Supports **user-provided stable IDs** for facts.

**Use case:** Debugging neural-symbolic systems, stable fact identification.

---

## Using Different Provenances

### Creating a Context with Provenance

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::*;
use scallop_core::runtime::env::RcFamily;

// Standard DataLog
let prov = UnitProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

// Probabilistic (min-max)
let prov = MinMaxProbProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

// Top-3 proofs
let prov = TopKProofsProvenance::<RcFamily>::new(3);
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);
```

### Adding Facts with Tags

```rust
use scallop_core::common::tuple::Tuple;

// Unit provenance (no tag)
ctx.add_facts("edge", vec![
    (None, Tuple::from((0i32, 1i32))),
], false)?;

// Probabilistic provenance
ctx.add_facts("edge", vec![
    (Some(0.8.into()), Tuple::from((0i32, 1i32))),
    (Some(0.9.into()), Tuple::from((1i32, 2i32))),
], false)?;

// Natural provenance (count)
ctx.add_facts("edge", vec![
    (Some(5usize), Tuple::from((0i32, 1i32))),  // Appears 5 times
], false)?;
```

### Interpreting Output Tags

```rust
ctx.run()?;

let results = ctx.computed_relation_ref("path")?;
for elem in results.iter() {
    match provenance_type {
        "unit" => {
            println!("Tuple: {:?}", elem.tuple);
            // elem.tag is Unit (no info)
        }
        "minmaxprob" => {
            println!("Probability: {}, Tuple: {:?}", elem.tag, elem.tuple);
            // elem.tag is f64
        }
        "natural" => {
            println!("Count: {}, Tuple: {:?}", elem.tag, elem.tuple);
            // elem.tag is usize
        }
        _ => {}
    }
}
```

---

## Semiring Operations

Provenance forms a **semiring** with two operations:

### Addition (Disjunction, OR)

Combines **alternative derivations** of the same fact.

```scl
rel path(0, 2) :- edge(0, 1), edge(1, 2)  // Derivation 1
rel path(0, 2) :- edge(0, 2)               // Derivation 2
```

**How provenances combine alternatives:**

| Provenance | `add(t1, t2)` | Example |
|------------|---------------|---------|
| Unit | `Unit` | No change |
| Boolean | `t1 ∨ t2` | `true ∨ false = true` |
| Natural | `t1 + t2` | `3 + 5 = 8` |
| MinMaxProb | `max(t1, t2)` | `max(0.8, 0.6) = 0.8` |
| AddMultProb | `t1 + t2 - t1*t2` | `0.8 + 0.6 - 0.48 = 0.92` |

### Multiplication (Conjunction, AND)

Combines **dependent facts** in a rule body.

```scl
rel path(a, c) :- edge(a, b), edge(b, c)  // Both facts needed
```

**How provenances combine conjunctions:**

| Provenance | `mult(t1, t2)` | Example |
|------------|----------------|---------|
| Unit | `Unit` | No change |
| Boolean | `t1 ∧ t2` | `true ∧ false = false` |
| Natural | `t1 * t2` | `3 * 5 = 15` |
| MinMaxProb | `min(t1, t2)` | `min(0.8, 0.9) = 0.8` |
| AddMultProb | `t1 * t2` | `0.8 * 0.9 = 0.72` |

### Identity Elements

Every semiring has **zero** (additive identity) and **one** (multiplicative identity):

| Provenance | Zero | One |
|------------|------|-----|
| Unit | `Unit` | `Unit` |
| Boolean | `false` | `true` |
| Natural | `0` | `1` |
| MinMaxProb | `0.0` | `1.0` |

**Properties:**
```
add(t, zero) = t
mult(t, one) = t
```

### Example: MinMaxProb Semiring

```scl
rel 0.8::edge(0, 1)
rel 0.9::edge(1, 2)
rel 0.7::edge(0, 2)

rel path(a, b) = edge(a, b)
rel path(a, c) = edge(a, b), edge(b, c)

query path(0, 2)
```

**Derivation:**
1. **Direct path:** `edge(0, 2)` → probability = `0.7`
2. **Indirect path:** `edge(0, 1) ∧ edge(1, 2)` → probability = `min(0.8, 0.9) = 0.8`
3. **Combine alternatives:** `add(0.7, 0.8) = max(0.7, 0.8) = 0.8`

**Result:** `path(0, 2)` has probability **0.8**

---

## Complete Example: Probabilistic Path Finding

Here's a full program demonstrating probabilistic reasoning.

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::probabilistic::min_max_prob::MinMaxProbProvenance;
use scallop_core::runtime::env::RcFamily;
use scallop_core::common::tuple::Tuple;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    // Create context with min-max probability provenance
    let prov = MinMaxProbProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Define schema
    ctx.add_relation("edge(i32, i32)")?;

    // Add probabilistic edges
    ctx.add_facts("edge", vec![
        (Some(0.8.into()), Tuple::from((0i32, 1i32))),  // 80% confidence
        (Some(0.9.into()), Tuple::from((1i32, 2i32))),  // 90% confidence
        (Some(0.7.into()), Tuple::from((2i32, 3i32))),  // 70% confidence
        (Some(0.6.into()), Tuple::from((0i32, 2i32))),  // 60% confidence (shortcut)
    ], false)?;

    // Define transitive closure
    ctx.add_rule("path(a, b) = edge(a, b)")?;
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")?;

    // Execute
    ctx.run()?;

    // Query paths with probabilities
    let path = ctx.computed_relation_ref("path")?;

    println!("Probabilistic Paths:");
    for elem in path.iter() {
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (elem.tuple.get(0), elem.tuple.get(1))
        {
            println!("  path({}, {}) with confidence: {:.2}", from, to, elem.tag);
        }
    }

    Ok(())
}
```

**Expected Output:**
```
Probabilistic Paths:
  path(0, 1) with confidence: 0.80
  path(1, 2) with confidence: 0.90
  path(2, 3) with confidence: 0.70
  path(0, 2) with confidence: 0.80  // max(0.6 direct, 0.8 via 1)
  path(1, 3) with confidence: 0.70  // min(0.9, 0.7)
  path(0, 3) with confidence: 0.70  // multiple paths, best is 0.70
```

**Explanation:**
- `path(0, 2)` has two derivations:
  1. Direct edge with prob 0.6
  2. Via node 1: min(0.8, 0.9) = 0.8
  3. Combined: max(0.6, 0.8) = **0.8**

- `path(0, 3)` has multiple paths:
  1. Via 1, 2: min(0.8, 0.9, 0.7) = 0.7
  2. Via 2: min(0.8, 0.7) = 0.7 (using shortcut)
  3. Combined: max(0.7, 0.7) = **0.7**

---

## Comparing Probabilistic Provenances

### MinMaxProb vs AddMultProb vs TopKProofs

```rust
// Same input facts
let facts = vec![
    (Some(0.8.into()), (0, 1).into()),
    (Some(0.9.into()), (1, 2).into()),
];

// Program: path(a, c) :- edge(a, b), edge(b, c)
```

**Results for `path(0, 2)`:**

| Provenance | Probability | Interpretation |
|------------|-------------|----------------|
| MinMaxProb | 0.8 | Weakest link: min(0.8, 0.9) |
| AddMultProb | 0.72 | Independent: 0.8 × 0.9 |
| TopKProofs | 0.72 | Via WMC (equivalent to AddMultProb for single proof) |

**When to use each:**

- **MinMaxProb** - Fuzzy logic, confidence propagation, when conjunction means "limited by weakest"
- **AddMultProb** - Statistical independence, Bayesian reasoning
- **TopKProofs** - When you need explanations and multiple derivation paths
- **ProbProofs** - When you need exact probabilities with all proof trees

---

## Advanced: Weighted Model Counting (WMC)

For proof-based provenances (TopKProofs, ProbProofs), probabilities are computed via **Weighted Model Counting** over Boolean formulas.

### How It Works

1. **Proofs to Formula:**
   ```
   Proofs: {{fact_0, fact_1}, {fact_2}}
   Formula: (f₀ ∧ f₁) ∨ f₂
   ```

2. **Build SDD (Sentential Decision Diagram)** for efficient computation

3. **Evaluate with probability semiring:**
   ```
   f₀ = 0.8, f₁ = 0.9, f₂ = 0.5
   WMC = (0.8 × 0.9) + 0.5 - (0.8 × 0.9 × 0.5)
       = 0.72 + 0.5 - 0.36
       = 0.86
   ```

**Key insight:** Proof-based provenances use **inclusion-exclusion** to compute exact probabilities from potentially overlapping proofs.

---

## Provenance Selection Guide

### Quick Reference

| Use Case | Recommended Provenance | Why |
|----------|------------------------|-----|
| Standard DataLog | `UnitProvenance` | No overhead, classic semantics |
| Counting derivations | `NaturalProvenance` | Track multiplicity |
| Fuzzy logic / confidence | `MinMaxProbProvenance` | Simple, efficient |
| Statistical reasoning | `AddMultProbProvenance` | Models independence |
| Need explanations | `TopKProofsProvenance` | Provides proof trees |
| Exact probabilities | `ProbProofsProvenance` | Complete computation |
| ML integration | `DiffTopKProofsProvenance` | Gradient support |
| Debugging proofs | `DiffTopKProofsDebugProvenance` | Full observability |

### Performance Considerations

**Computational cost (low to high):**
1. UnitProvenance - No overhead
2. NaturalProvenance, MinMaxProbProvenance - Simple arithmetic
3. AddMultProbProvenance - Inclusion-exclusion
4. TopKProofsProvenance - WMC with top-K pruning
5. ProbProofsProvenance - Full WMC (most expensive)
6. DiffTopKProofsProvenance - WMC + gradient computation

**Memory usage:**
- Unit, Boolean, Natural - Minimal (single value)
- Probabilistic (f64) - 8 bytes per fact
- Proof-based - Stores DNF formulas (can be large)

---

## Next Steps

- **[Foreign Functions](foreign_functions.md)** - Extend Scallop with custom computations
- **[Foreign Predicates](foreign_predicates.md)** - Tag facts with probabilities
- **[Rust Examples](../examples/rust/)** - See provenance in action
- **[Getting Started](getting_started.md)** - Basic examples with different provenances

## Resources

- **Trait Definition:** `scallop-core/src/runtime/provenance/provenance.rs`
- **Implementations:** `scallop-core/src/runtime/provenance/{discrete,probabilistic,differentiable}/`
- **Research Paper:** [Scallop: A Language for Neurosymbolic Programming](https://arxiv.org/abs/2304.04812)
- **WMC Background:** [Weighted Model Counting](https://en.wikipedia.org/wiki/Model_counting)
