# Scallop Language Reference

Scallop is a DataLog-based logic programming language extended with powerful features for modern applications. This section covers the core language constructs and advanced features.

## Overview

Scallop extends traditional DataLog with:
- **Probabilistic reasoning** - Attach probabilities to facts and track uncertainty
- **Algebraic data types** - Define structured data with sum and product types
- **Aggregations** - Compute count, sum, max, min, and custom aggregations
- **Negation** - Express what is *not* true
- **Disjunctive heads** - Represent choices and alternatives
- **Foreign functions** - Integrate Python and external computation
- **Magic set transformation** - Optimize query evaluation

---

## Core Language Features

### Relations and Facts

Relations are the fundamental data structure in Scallop. Learn about declaring relations, adding facts, and data types:

- [Relations](relation.md) - Declaring and using relations
- [Value Types](value_type.md) - Scallop's type system (integers, floats, strings, etc.)
- [Constants](constants.md) - Named constants for readability

### Rules and Logic

Rules define how to derive new facts from existing facts using logical inference:

- [Rules](rules.md) - Basic rule syntax and patterns
- [Recursion](recursion.md) - Recursive rules for transitive closure, paths, etc.
- [Negation](negation.md) - Expressing negative conditions
- [Queries](query.md) - Extracting results

### Advanced Data Types

Scallop supports sophisticated type systems for structuring data:

- [Algebraic Data Types (ADTs)](adt_and_entity.md) - Sum types, product types, pattern matching
- [Custom Types](custom_type.md) - Defining domain-specific types
- [Entities](adt_and_entity.md#entities) - Content-addressable values

### Aggregations and Computation

Compute derived values from collections of facts:

- [Aggregation](aggregation.md) - count, sum, max, min, and custom aggregators
- [Foreign Functions](foreign_functions.md) - Call Python functions from Scallop
- [Foreign Predicates](foreign_predicates.md) - Implement predicates in Python

### Probability and Provenance

Track uncertainty and trace how conclusions are derived:

- [Tags and Provenance](provenance.md) - Attaching metadata to facts
- [Probabilistic Programming](../probabilistic/index.md) - Full probabilistic reasoning guide

### Advanced Features

Push the boundaries of logic programming:

- [Disjunctive and Conjunctive Heads](disj_conj_head.md) - Express choices in rule heads
- [Magic Set Transformation](magic_set.md) - Query optimization technique
- [Loading CSV](loading_csv.md) - Import data from CSV files

---

## Language Philosophy

### Declarative Programming

Scallop programs describe **what** to compute, not **how** to compute it:

```scl
// What: Define transitive closure
rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

// Scallop figures out how to compute it efficiently
```

### Set Semantics

Relations are **sets** of tuples - order doesn't matter, duplicates are eliminated:

```scl
rel numbers = {1, 2, 3}
rel numbers = {3, 2, 1}  // Same as above
rel numbers = {1, 1, 2}  // Duplicate 1 is ignored
```

### Monotonic Reasoning

Facts can only be added, never removed (except with negation). This enables efficient incremental computation.

---

## Syntax Quick Reference

### Relation Declaration

```scl
// Declare relation with types
type edge(from: i32, to: i32)

// Declare and add facts
rel edge = {(0, 1), (1, 2), (2, 3)}
```

### Rules

```scl
// Basic rule
rel path(a, b) = edge(a, b)

// Rule with multiple conditions
rel path(a, c) = path(a, b), edge(b, c)

// Rule with constraint
rel adult(name) = person(name, age), age >= 18
```

### Aggregation

```scl
// Count elements
rel total(n) = n = count(x: numbers(x))

// Sum values
rel sum_ages(s) = s = sum(age: person(_, age))

// Max value
rel oldest(max_age) = max_age = max(age: person(_, age))
```

### Disjunctive Head

```scl
// Express choices
rel { heads(); tails() } = coin_flip()
```

### Pattern Matching

```scl
// Match on ADT variants
rel is_leaf(t) = case t is Leaf(_)
rel left_child(t, l) = case t is Node(l, _)
```

---

## Example Programs

### Transitive Closure

```scl
rel edge = {(0, 1), (1, 2), (2, 3)}

rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
// Result: {(0,1), (0,2), (0,3), (1,2), (1,3), (2,3)}
```

### Probabilistic Graph

```scl
rel edge = {0.8::(0, 1), 0.9::(1, 2)}

rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b), edge(b, c)

query path
// Result: {0.8::(0,1), 0.9::(1,2), 0.72::(0,2)}
```

### Family Relations

```scl
rel parent = {("alice", "bob"), ("alice", "charlie"), ("bob", "diana")}

rel ancestor(a, d) = parent(a, d)
rel ancestor(a, d) = ancestor(a, p), parent(p, d)

rel sibling(a, b) = parent(p, a), parent(p, b), a != b

query sibling
// Result: {("bob", "charlie"), ("charlie", "bob")}
```

---

## Language Tools

- **scli** - Run `.scl` programs from the command line ([CLI Guide](../toolchain/scli.md))
- **sclrepl** - Interactive REPL for experimentation ([REPL Guide](../toolchain/sclrepl.md))
- **scallopy** - Python integration for ML applications ([Python Guide](../scallopy/index.md))

---

## Further Reading

- [Crash Course](../crash_course.md) - Quick introduction to Scallop
- [Probabilistic Programming](../probabilistic/index.md) - Reasoning under uncertainty
- [Python Integration](../scallopy/index.md) - Using Scallop with PyTorch
- [Reference Guide](reference_guide.md) - Comprehensive syntax reference
