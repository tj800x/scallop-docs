# Module Input

Input mappings define how PyTorch tensors are converted into Scallop facts. They specify the **domain** (set of possible values) of input relations, allowing you to pass probability distributions as tensors and have them automatically converted to probabilistic facts.

## What is an Input Mapping?

An input mapping establishes a correspondence between tensor indices and Scallop tuples:

```python
input_mappings={"digit": range(10)}
```

This mapping says: "The `digit` relation has domain 0-9, and a tensor of shape `(10,)` represents probabilities for each digit."

**Example:**
```python
# Tensor: [0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.1]
# Interpreted as:
# digit(0) with probability 0.1
# digit(8) with probability 0.8
# digit(9) with probability 0.1
```

---

## Input Mapping Formats

Scallop supports multiple formats for defining input mappings:

### Format 1: Range (Simple Integer Domain)

The most common format for integer-valued relations:

```python
input_mappings={"digit": range(10)}
# Domain: digit(0), digit(1), ..., digit(9)
# Expected tensor shape: (10,) or (batch_size, 10)
```

**Properties:**
- `kind`: `"list"`
- `shape`: `(10,)`
- `dimension`: `1`
- `is_singleton`: `True` (single-column relation)

### Format 2: List (Explicit Enumeration)

For arbitrary values:

```python
input_mappings={
  "color": ["red", "green", "blue"]
}
# Domain: color(red), color(green), color(blue)
# Expected tensor shape: (3,)
```

**With tuples:**
```python
input_mappings={
  "edge": [(0,1), (1,2), (2,3), (0,3)]
}
# Domain: edge(0,1), edge(1,2), edge(2,3), edge(0,3)
# Expected tensor shape: (4,)
```

**Properties:**
- `kind`: `"list"`
- `shape`: `(len(list),)`
- `is_singleton`: `False` if tuples, `True` if values

### Format 3: Dictionary (Multi-Dimensional)

For relations with multiple columns, use a dictionary mapping dimension indices to value lists:

```python
input_mappings={
  "edge": {
    0: range(5),  # First column: nodes 0-4
    1: range(5),  # Second column: nodes 0-4
  }
}
# Domain: all pairs (i, j) where i, j ∈ {0, 1, 2, 3, 4}
# Expected tensor shape: (5, 5) - 25 possible edges
```

**Mixed types:**
```python
input_mappings={
  "likes": {
    0: ["alice", "bob", "charlie"],
    1: ["pizza", "salad", "burger"],
  }
}
# Domain: likes(person, food) for all person-food combinations
# Expected tensor shape: (3, 3)
```

**Properties:**
- `kind`: `"dict"`
- `shape`: `(len(dim0), len(dim1), ...)`
- `dimension`: Number of dimensions
- `is_singleton`: `False`

### Format 4: Tuple (Fixed Constant)

For a single fixed tuple:

```python
input_mappings={
  "start_node": (0,)
}
# Domain: start_node(0) only
# Expected tensor shape: ()
```

**Properties:**
- `kind`: `"tuple"`
- `shape`: `()`
- `dimension`: `0`

### Format 5: Value (Single Constant)

For a single value:

```python
input_mappings={
  "threshold": 0.5
}
# Domain: threshold(0.5) only
# Expected tensor shape: ()
```

**Properties:**
- `kind`: `"value"`
- `shape`: `()`
- `dimension`: `0`
- `is_singleton`: `True`

---

## Tensor Shapes and Batching

Input mappings automatically handle batching.

### Single Example

If tensor shape matches the mapping shape exactly, it's treated as a single example:

```python
im = scallopy.InputMapping(range(10))
tensor = torch.randn(10)  # Single probability distribution

facts = im.process_tensor(tensor, batched=False)
# Returns: list of 10 facts
```

### Batched Input

If tensor has an extra leading dimension, it's treated as a batch:

```python
im = scallopy.InputMapping(range(10))
tensor = torch.randn(16, 10)  # Batch of 16 distributions

facts = im.process_tensor(tensor, batched=False)
# Returns: list of 16 lists, each with 10 facts
```

### Multi-Dimensional Mappings

For multi-dimensional mappings, the tensor shape must match:

```python
im = scallopy.InputMapping({0: range(5), 1: range(3)})
tensor = torch.randn(5, 3)  # Single example
# OR
tensor = torch.randn(16, 5, 3)  # Batch of 16

facts = im.process_tensor(tensor)
```

---

## Sparse Inputs

For large domains, you often don't want to include all facts. Scallop provides filtering mechanisms:

### Retain Top-K

Keep only the K highest-probability facts:

```python
input_mappings={
  "digit": scallopy.InputMapping(
    range(10),
    retain_k=3  # Keep only top 3 digits
  )
}

# Tensor: [0.05, 0.02, 0.30, 0.01, 0.40, 0.03, 0.10, 0.02, 0.05, 0.02]
# After retain_k=3: only facts for indices 4 (0.40), 2 (0.30), 6 (0.10) are kept
```

**With multi-dimensional mappings:**
```python
input_mappings={
  "edge": scallopy.InputMapping(
    {0: range(10), 1: range(10)},
    retain_k=5,  # Keep only top 5 edges across all 100 possibilities
  )
}
```

**Per-dimension sampling:**
```python
input_mappings={
  "edge": scallopy.InputMapping(
    {0: range(10), 1: range(10)},
    retain_k=2,
    sample_dim=1,  # Keep top 2 for each value of dimension 1
  )
}
# Result: 10 * 2 = 20 facts (top 2 destinations for each source)
```

### Retain Threshold

Keep only facts above a probability threshold:

```python
input_mappings={
  "digit": scallopy.InputMapping(
    range(10),
    retain_threshold=0.1  # Only keep probabilities > 0.1
  )
}

# Tensor: [0.05, 0.02, 0.30, 0.01, 0.40, 0.03, 0.10, 0.02, 0.05, 0.02]
# After threshold: only facts for indices 2 (0.30), 4 (0.40) are kept
# Note: 0.10 is NOT kept (must be strictly greater than threshold)
```

### Categorical Sampling

Instead of deterministic top-K, sample K facts according to their probabilities:

```python
input_mappings={
  "digit": scallopy.InputMapping(
    range(10),
    retain_k=3,
    sample_strategy="categorical"  # Stochastic sampling
  )
}
# Each forward pass samples 3 different digits based on probabilities
```

---

## Disjunctions in Input Mappings

When facts are mutually exclusive, mark them as disjunctive:

### Global Disjunction

All facts in the relation are mutually exclusive:

```python
input_mappings={
  "digit": scallopy.InputMapping(
    range(10),
    disjunctive=True
  )
}
# All 10 digit facts share one mutual exclusion ID
```

### Per-Dimension Disjunction

Mutual exclusion along a specific dimension:

```python
input_mappings={
  "color": scallopy.InputMapping(
    {0: range(3), 1: ["red", "green", "blue"]},
    disjunctive_dim=1  # Each object has mutually exclusive colors
  )
}
# color(0, red), color(0, green), color(0, blue) are mutually exclusive
# color(1, red), color(1, green), color(1, blue) are mutually exclusive
# But color(0, red) and color(1, red) are NOT mutually exclusive
```

---

## Complete Example

Putting it all together:

```python
import torch
import scallopy

# Create module with complex input mappings
module = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="""
    // Classify objects
    rel class(o, c) = color(o, col), shape(o, sh), classifier(col, sh, c)
  """,
  input_mappings={
    # Simple list
    "color": scallopy.InputMapping(
      {0: range(10), 1: ["red", "green", "blue"]},
      disjunctive_dim=1,  # Each object has one color
      retain_k=1,  # Keep most likely color per object
      sample_dim=1,
    ),

    # Multi-dimensional with threshold
    "shape": scallopy.InputMapping(
      {0: range(10), 1: ["circle", "square", "triangle"]},
      disjunctive_dim=1,
      retain_threshold=0.2,  # Only confident shapes
    ),

    # Fixed classifier (non-probabilistic)
    "classifier": [
      ("red", "circle", "apple"),
      ("green", "circle", "lime"),
      # ... more rules
    ],
  },
  output_mapping=("class", [(i, c) for i in range(10) for c in ["apple", "lime"]])
)

# Use with batched tensors
color_probs = torch.softmax(torch.randn(16, 10, 3), dim=2)
shape_probs = torch.softmax(torch.randn(16, 10, 3), dim=2)

result = module(color=color_probs, shape=shape_probs)
# Result shape: (16, 20) - batch of 16, (object, class) pairs
```

---

## Summary

- **Input mappings** define the domain of input relations
- **Five formats**: range, list, dict, tuple, value
- **Batching** is automatic - extra leading dimension = batch
- **Sparse inputs** via `retain_k`, `retain_threshold`, or `sample_strategy`
- **Disjunctions** mark mutually exclusive facts (global or per-dimension)
- **Properties**: `kind`, `shape`, `dimension`, `is_singleton`

For more details:
- [Creating Modules](module.md) - Overview of Scallop modules
- [Module Output](module_output.md) - Output mappings
- [Configuring Provenance](provenance.md) - Probability tracking
