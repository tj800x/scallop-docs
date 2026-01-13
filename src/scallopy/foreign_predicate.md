# Foreign Predicates

Foreign predicates allow you to implement Scallop predicates in Python, enabling custom logic, external data sources, and integration with Python libraries.

## What is a Foreign Predicate?

A **foreign predicate** is a Python function that Scallop can call during execution to generate facts dynamically. Instead of declaring facts statically, foreign predicates compute facts on-the-fly based on input arguments.

**Use cases:**
- **Custom logic**: Implement complex computations not expressible in Scallop
- **External data**: Query databases, APIs, or files during reasoning
- **Python libraries**: Use NumPy, scikit-learn, or other Python tools
- **Semantic similarity**: Fuzzy matching, embeddings, neural networks

---

## Important: Required Imports and Type Signature

### Must Import from scallopy

Foreign predicates require specific imports from the scallopy package:

```python
# ✓ Correct - import required types
from scallopy import foreign_predicate, Facts
from typing import Tuple

@foreign_predicate
def string_length(s: str) -> Facts[float, Tuple[int]]:
  yield (1.0, (len(s),))

# ✗ Incorrect - missing imports
import scallopy

@scallopy.foreign_predicate  # Will fail
def string_length(s: str) -> int:  # Wrong return type
  return len(s)  # Wrong - must yield
```

### Return Type Must Be Facts Generator

The return type **must be** `Facts[TagType, TupleType]` and use `yield`, not `return`:

```python
# ✓ Correct - yields Facts
def my_predicate(x: int) -> Facts[float, Tuple[int]]:
  yield (1.0, (x * 2,))

# ✗ Incorrect - returns value directly
def my_predicate(x: int) -> int:
  return x * 2  # Error: "Return type must be Facts"
```

---

## Basic Usage

### Defining a Foreign Predicate

Use the `@foreign_predicate` decorator:

```python
from scallopy import foreign_predicate, Facts
from typing import Tuple

@foreign_predicate
def string_length(s: str) -> Facts[float, Tuple[int]]:
  length = len(s)
  yield (1.0, (length,))  # (probability, tuple)
```

**Anatomy:**
- **Decorator**: `@foreign_predicate` marks the function
- **Type hints**: Input parameters are typed (e.g., `s: str`)
- **Return type**: `Facts[TagType, TupleType]` - generator of (tag, tuple) pairs
- **Yield**: Produce facts lazily using `yield` (not `return`)

### Registering with Context

```python
import scallopy

ctx = scallopy.ScallopContext(provenance="minmaxprob")

# Register the foreign predicate
ctx.register_foreign_predicate(string_length)

# Use in rules
ctx.add_relation("word", str)
ctx.add_facts("word", [("apple",), ("banana",), ("cat",)])
ctx.add_rule("length(w, l) = word(w) and string_length(w, l)")
ctx.run()

# Results
for (prob, (word, length)) in ctx.relation("length"):
  print(f"{word}: {length} letters (prob={prob})")
```

**Output:**
```
apple: 5 letters (prob=1.0)
banana: 6 letters (prob=1.0)
cat: 3 letters (prob=1.0)
```

---

## Type Annotations

Foreign predicates require proper type hints for Scallop to understand the interface.

### Supported Types

**Primitive types:**
- `int` → `i32`
- `float` → `f32`
- `bool` → `bool`
- `str` → `String`

**Scallop types:**
- `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- `f32`, `f64`
- `char`, `bool`, `String`

### Input Arguments

Input argument types define what Scallop passes to your function:

```python
@foreign_predicate
def add(x: int, y: int) -> Facts[float, Tuple[int]]:
  result = x + y
  yield (1.0, (result,))
```

### Output Types

The `Facts` type annotation specifies:
1. **Tag type** (first parameter): Probability type (usually `float`)
2. **Tuple type** (second parameter): Output tuple structure

**Single-column output:**
```python
def length(s: str) -> Facts[float, Tuple[int]]:
  yield (1.0, (len(s),))
```

**Multi-column output:**
```python
def split_name(full: str) -> Facts[float, Tuple[str, str]]:
  parts = full.split(" ")
  if len(parts) == 2:
    yield (1.0, (parts[0], parts[1]))
```

**Empty tuple (boolean predicate):**
```python
def is_palindrome(s: str) -> Facts[float, Tuple]:
  if s == s[::-1]:
    yield (1.0, ())  # Empty tuple = just a boolean check
```

---

## Yielding Facts

Foreign predicates use `yield` to produce facts lazily.

### Single Fact

```python
@foreign_predicate
def square(x: int) -> Facts[float, Tuple[int]]:
  yield (1.0, (x * x,))
```

### Multiple Facts

```python
@foreign_predicate
def divisors(n: int) -> Facts[float, Tuple[int]]:
  for i in range(1, n + 1):
    if n % i == 0:
      yield (1.0, (i,))

# Usage in Scallop:
# divisors(12, x) generates: x ∈ {1, 2, 3, 4, 6, 12}
```

### Probabilistic Facts

```python
@foreign_predicate
def semantic_similar(s1: str, s2: str) -> Facts[float, Tuple]:
  # Use embedding similarity, edit distance, etc.
  similarity = compute_similarity(s1, s2)
  if similarity > 0.5:
    yield (similarity, ())
```

### Conditional Facts

```python
@foreign_predicate
def classify_age(age: int) -> Facts[float, Tuple[str]]:
  if age < 18:
    yield (1.0, ("minor",))
  elif age < 65:
    yield (1.0, ("adult",))
  else:
    yield (1.0, ("senior",))
```

---

## Complete Example

Here's a realistic example using foreign predicates for semantic similarity:

```python
from typing import Tuple
import scallopy
from scallopy import foreign_predicate, Facts

# Foreign predicate for semantic equivalence
@foreign_predicate
def string_semantic_eq(s1: str, s2: str) -> Facts[float, Tuple]:
  """Check if two strings are semantically equivalent"""
  equivalents = {
    ("mom", "mother"): 0.99,
    ("mom", "mom"): 1.0,
    ("mother", "mother"): 1.0,
    ("dad", "father"): 0.99,
    ("dad", "dad"): 1.0,
    ("father", "father"): 1.0,
  }

  if (s1, s2) in equivalents:
    yield (equivalents[(s1, s2)], ())

# Create context and register
ctx = scallopy.ScallopContext(provenance="minmaxprob")
ctx.register_foreign_predicate(string_semantic_eq)

# Add kinship data with varied terminology
ctx.add_relation("kinship", (str, str, str))
ctx.add_facts("kinship", [
  (1.0, ("alice", "mom", "bob")),
  (1.0, ("alice", "mother", "casey")),
  (1.0, ("david", "father", "emma")),
])

# Define rules using foreign predicate
ctx.add_rule("""
  parent(person, child) =
    kinship(person, relation, child) and
    string_semantic_eq(relation, "mother")
""")

ctx.add_rule("""
  parent(person, child) =
    kinship(person, relation, child) and
    string_semantic_eq(relation, "father")
""")

ctx.add_rule("""
  sibling(a, b) =
    parent(p, a) and parent(p, b) and a != b
""")

ctx.run()

# Results
print("Parents:")
for (prob, (person, child)) in ctx.relation("parent"):
  print(f"  {person} is parent of {child} (prob={prob})")

print("\nSiblings:")
for (prob, (a, b)) in ctx.relation("sibling"):
  print(f"  {a} and {b} are siblings (prob={prob})")
```

**Output:**
```
Parents:
  alice is parent of bob (prob=0.99)
  alice is parent of casey (prob=1.0)
  david is parent of emma (prob=1.0)

Siblings:
  bob and casey are siblings (prob=0.99)
  casey and bob are siblings (prob=0.99)
```

---

## Advanced Patterns

### Pattern 1: External Data Source

Query a database during reasoning:

```python
import sqlite3

@foreign_predicate
def lookup_price(product: str) -> Facts[float, Tuple[float]]:
  conn = sqlite3.connect("products.db")
  cursor = conn.execute("SELECT price FROM products WHERE name = ?", (product,))
  row = cursor.fetchone()
  if row:
    yield (1.0, (row[0],))
  conn.close()
```

### Pattern 2: Python Library Integration

Use NumPy for numerical operations:

```python
import numpy as np

@foreign_predicate
def cosine_similarity(vec_id1: int, vec_id2: int) -> Facts[float, Tuple[float]]:
  vec1 = embeddings[vec_id1]
  vec2 = embeddings[vec_id2]
  similarity = np.dot(vec1, vec2) / (np.linalg.norm(vec1) * np.linalg.norm(vec2))
  yield (1.0, (float(similarity),))
```

### Pattern 3: Caching Results

Avoid redundant computation:

```python
from functools import lru_cache

@lru_cache(maxsize=1000)
def _compute_expensive(x: int) -> int:
  # Expensive computation
  return expensive_function(x)

@foreign_predicate
def cached_predicate(x: int) -> Facts[float, Tuple[int]]:
  result = _compute_expensive(x)
  yield (1.0, (result,))
```

### Pattern 4: Error Handling

Handle errors gracefully:

```python
@foreign_predicate
def safe_divide(a: float, b: float) -> Facts[float, Tuple[float]]:
  try:
    result = a / b
    yield (1.0, (result,))
  except ZeroDivisionError:
    # Don't yield anything - fact doesn't exist
    pass
```

---

## Summary

- **Foreign predicates** implement Scallop predicates in Python
- **`@foreign_predicate` decorator** marks functions
- **Type annotations** required for inputs and outputs
- **`Facts[float, Tuple[...]]`** return type with generator
- **`yield`** produces facts lazily
- **`ctx.register_foreign_predicate()`** registers with context
- **Use cases**: custom logic, external data, Python libraries

For more details:
- [Foreign Functions](foreign_function.md) - Similar but for functional computations
- [ScallopContext](context.md) - Context API for registration
- [Creating Modules](module.md) - Using foreign predicates in modules
