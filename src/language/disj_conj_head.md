# Disjunctive and Conjunctive Heads

Disjunctive and conjunctive heads allow you to express **choices** and **multiple conclusions** in a single rule. This feature is essential for modeling non-deterministic systems, generating alternatives, and encoding logic puzzles.

## What is a Disjunctive Head?

A **disjunctive head** expresses that **at least one** of multiple conclusions must be true, but not necessarily all. This represents a **choice** or **alternative**.

**Syntax:**
```scl
rel { head1(); head2(); ...; headN() } = body
```

**Meaning:** If the body is true, then derive **at least one** of `head1()`, `head2()`, ..., `headN()`.

---

## Basic Disjunctive Head Example

### Coin Flip

Model a fair coin flip where each flip results in either heads or tails:

```scl
// For each coin flip, derive either heads or tails
rel { heads(id); tails(id) } = coin_flip(id)

rel coin_flip = {1, 2, 3}

query heads
query tails
```

**Possible results (non-deterministic):**
```
heads: {1, 2}
tails: {3}
```

Or:
```
heads: {1, 3}
tails: {2}
```

**Interpretation:** For each coin flip, the system chooses between heads and tails. Different runs may produce different results.

---

## Disjunctive Heads in Action

### Boolean Satisfiability (SAT)

A classic application is solving Boolean satisfiability problems:

```scl
// Each variable can be assigned either true or false
rel { assign(x, true); assign(x, false) } = vars(x)

rel vars = {"A", "B"}

// Formula: A ∧ B
rel satisfies_formula = assign("A", true), assign("B", true)

query satisfies_formula
```

**How it works:**
1. For each variable in `vars`, the disjunctive head generates two possibilities: assigned to true OR assigned to false
2. The system explores combinations to find satisfying assignments
3. Only assignments satisfying the formula are derived

### Choice Generation

Generate all possible color assignments:

```scl
// Each object can be colored red, green, or blue
rel { color(obj, "red"); color(obj, "green"); color(obj, "blue") } = object(obj)

rel object = {1, 2}

query color
```

**Result:** Multiple possible worlds where each object has one color.

---

## Combining Disjunction with Constraints

### Coloring with Constraints

```scl
// Each node must be colored
rel { color(n, "red"); color(n, "green"); color(n, "blue") } = node(n)

// Adjacent nodes cannot have the same color
rel valid_coloring() =
  color(n1, c1), color(n2, c2), edge(n1, n2), c1 != c2,
  forall(n: node(n) => color(n, _))

rel node = {1, 2, 3}
rel edge = {(1, 2), (2, 3)}

query color
query valid_coloring
```

**Interpretation:** The system generates color assignments where adjacent nodes have different colors.

---

## Conjunctive Heads

A **conjunctive head** derives **all** conclusions when the body is true.

**Syntax (using semicolon):**
```scl
rel head1(); head2(); ...; headN() = body
```

**Meaning:** If the body is true, then derive **all** of `head1()`, `head2()`, ..., `headN()`.

### Example: Multiple Facts

```scl
// When a person is born, derive both 'alive' and 'young'
rel alive(p); young(p) = born(p)

rel born = {"alice", "bob"}

query alive
query young
```

**Result:**
```
alive: {"alice", "bob"}
young: {"alice", "bob"}
```

Both facts are derived for each person born.

---

## Disjunctive vs Conjunctive

| Feature | Disjunctive `{ a; b; c }` | Conjunctive `a; b; c` |
|---------|--------------------------|----------------------|
| **Meaning** | At least one of a, b, c | All of a, b, c |
| **Choice** | Represents alternatives | No choice |
| **Use case** | SAT, puzzles, choices | Multiple conclusions |

**Disjunctive example:**
```scl
rel { red(x); blue(x) } = object(x)
// Each object is either red OR blue (choice)
```

**Conjunctive example:**
```scl
rel red(x); large(x) = special_object(x)
// Each special object is red AND large (both derived)
```

---

## Advanced: Disjunction with Pattern Matching

Combine disjunctive heads with ADT pattern matching:

```scl
type Formula = Var(String)
             | Not(Formula)
             | And(Formula, Formula)
             | Or(Formula, Formula)

// Each variable gets assigned either true or false
rel { assign(v, true); assign(v, false) } = case bf is Var(v)

// Evaluate the formula
rel eval(bf, r)        = case bf is Var(v), assign(v, r)
rel eval(bf, !r)       = case bf is Not(c), eval(c, r)
rel eval(bf, lr && rr) = case bf is And(lbf, rbf), eval(lbf, lr), eval(rbf, rr)
rel eval(bf, lr || rr) = case bf is Or(lbf, rbf), eval(lbf, lr), eval(rbf, rr)

const MY_FORMULA = Or(Var("A"), Var("B"))

query eval(MY_FORMULA, r)
```

**Result:** The formula is satisfiable when A or B is true.

---

## Use Cases

### 1. Logic Puzzles

```scl
// Sudoku: each cell can contain 1-9
rel { cell(r, c, 1); cell(r, c, 2); ...; cell(r, c, 9) } = position(r, c)
```

### 2. Planning and Scheduling

```scl
// Each task can be assigned to worker A, B, or C
rel { assign(task, "A"); assign(task, "B"); assign(task, "C") } = task(task)
```

### 3. Game State Exploration

```scl
// Player can move up, down, left, or right
rel { move("up"); move("down"); move("left"); move("right") } = can_move()
```

### 4. Configuration Generation

```scl
// Feature can be enabled or disabled
rel { enabled(f); disabled(f) } = feature(f)
```

---

## Semantics and Evaluation

### Non-Determinism

Disjunctive heads introduce **non-determinism** - the system may explore multiple possible worlds:

```scl
rel { a(); b() } = x()
rel x = {1}
```

This creates two possible worlds:
1. World 1: `a: {1}`
2. World 2: `b: {1}`

Or both: `a: {1}, b: {1}` (since "at least one" includes "both")

### With Probabilities

When combined with probabilistic reasoning, disjunctive heads can model uncertainty:

```scl
rel { heads(id); tails(id) } = 0.5::coin_flip(id)
```

Each alternative may have equal or different probabilities.

---

## Limitations

### 1. Cannot Mix Disjunction Types in Same Head

```scl
// ✗ Not allowed: mixing atomic facts with disjunctions
// rel { a(); b() }; c() = x()

// ✓ Use separate rules
rel { a(); b() } = x()
rel c() = x()
```

### 2. Disjunctions Must Be Ground

Variables in disjunctive heads should be bound by the body:

```scl
// ✓ Good: x is bound by object(x)
rel { color(x, "red"); color(x, "blue") } = object(x)

// ✗ Potentially problematic: x unbound
// rel { color(x, "red"); color(x, "blue") } = true
```

---

## Summary

- **Disjunctive heads** `{ a; b; c }` express choices - at least one conclusion
- **Conjunctive heads** `a; b; c` derive all conclusions
- **Use cases**: SAT solving, logic puzzles, planning, configuration
- **Combines with**: ADT pattern matching, probabilistic reasoning, constraints
- **Semantics**: Introduces non-determinism and multiple possible worlds

For more details:
- [Negation](negation.md) - Expressing what is not true
- [Algebraic Data Types](adt_and_entity.md) - Pattern matching with case
- [Probabilistic Programming](../probabilistic/index.md) - Probabilities with choices
