# Module Output

Output mappings define how Scallop results are extracted and converted back into PyTorch tensors. They specify which tuples from output relations should be included in the final tensor and in what order.

## What is an Output Mapping?

An output mapping specifies **which facts to extract** from a Scallop relation and **how to arrange them** in the output tensor.

```python
output_mapping=("sum_2", range(19))
```

This says: "Extract facts `sum_2(0)`, `sum_2(1)`, ..., `sum_2(18)` and return them as a tensor of shape `(19,)` (or `(batch_size, 19)` if batched)."

**Flow:**
```
Scallop execution → Relations with facts → Output mapping → Tensor
```

---

## Single Output

Use `output_mapping` (singular) when your module produces one output relation.

### Format: Tuple Notation

```python
output_mapping=("relation_name", mapping_list)
```

**Example 1: Simple integer range**
```python
module = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="rel sum_2(a + b) = digit_a(a) and digit_b(b)",
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)

result = module(digit_a=probs_a, digit_b=probs_b)
# Result shape: (batch_size, 19)
# result[:, 0] = probability of sum_2(0)
# result[:, 1] = probability of sum_2(1)
# ...
# result[:, 18] = probability of sum_2(18)
```

### Format: List of Values

```python
output_mapping=("relation_name", [value1, value2, ...])
```

**Example 2: Explicit list**
```python
module = scallopy.Module(
  ...
  output_mapping=("color", ["red", "green", "blue"])
)

result = module(...)
# Result shape: (batch_size, 3)
# result[:, 0] = probability of color("red")
# result[:, 1] = probability of color("green")
# result[:, 2] = probability of color("blue")
```

### Format: List of Tuples

For multi-column relations:

```python
output_mapping=("relation_name", [(tuple1), (tuple2), ...])
```

**Example 3: Multi-column relation**
```python
# Extract specific paths
output_mapping=("path", [(0,1), (0,2), (1,2), (2,3)])

result = module(...)
# Result shape: (batch_size, 4)
# result[:, 0] = probability of path(0, 1)
# result[:, 1] = probability of path(0, 2)
# result[:, 2] = probability of path(1, 2)
# result[:, 3] = probability of path(2, 3)
```

**Example 4: All pairs**
```python
# Generate all pairs programmatically
all_pairs = [(i, j) for i in range(5) for j in range(5)]
output_mapping=("edge", all_pairs)

result = module(...)
# Result shape: (batch_size, 25)
```

---

## Multiple Outputs

Use `output_mappings` (plural) when your module produces multiple output relations.

### Format: Dictionary

```python
output_mappings={
  "relation1": mapping1,
  "relation2": mapping2,
  ...
}
```

**Example:**
```python
module = scallopy.Module(
  provenance="diffaddmultprob",
  program="""
    rel sum_2(a + b) = digit_1(a) and digit_2(b)
    rel mult_2(a * b) = digit_1(a) and digit_2(b)
  """,
  input_mappings={
    "digit_1": range(10),
    "digit_2": range(10),
  },
  output_mappings={
    "sum_2": range(20),    # 0-19
    "mult_2": range(100),  # 0-99
  }
)

result = module(digit_1=probs_1, digit_2=probs_2)

# Result is a DICTIONARY
print(result["sum_2"].shape)   # (batch_size, 20)
print(result["mult_2"].shape)  # (batch_size, 100)
```

### Accessing Multiple Outputs

```python
# Forward pass
outputs = module(input_a=tensor_a, input_b=tensor_b)

# Access individual outputs
sum_probs = outputs["sum_2"]
mult_probs = outputs["mult_2"]

# Compute losses separately
loss_sum = criterion(sum_probs, target_sum)
loss_mult = criterion(mult_probs, target_mult)

total_loss = loss_sum + loss_mult
total_loss.backward()
```

---

## Output Formats

### Format 1: Range

Most common for integer-valued relations:

```python
output_mapping=("digit", range(10))
# Extracts: digit(0), digit(1), ..., digit(9)
```

### Format 2: List of Values

For explicit enumeration:

```python
output_mapping=("color", ["red", "green", "blue", "yellow"])
# Extracts: color("red"), color("green"), color("blue"), color("yellow")
```

### Format 3: List of Tuples

For multi-column relations:

```python
# Binary relation
output_mapping=("edge", [(0,1), (1,2), (2,3)])

# Ternary relation
output_mapping=("triple", [(a, b, c) for a in range(3) for b in range(3) for c in range(3)])
```

### Format 4: None (No Output Mapping)

When you don't need tensor output:

```python
output_mapping=None
# Module runs Scallop but doesn't extract results as tensor
# Useful for intermediate computations or side effects
```

---

## Advanced Patterns

### Pattern 1: Filtering Relevant Outputs

Only extract the outputs you care about:

```python
# Don't need all 100 paths, just specific ones
relevant_paths = [(0, 3), (1, 4), (2, 5)]
output_mapping=("path", relevant_paths)
```

### Pattern 2: Multi-Task Learning

Different outputs for different tasks:

```python
module = scallopy.Module(
  ...
  output_mappings={
    "classification": ["cat", "dog", "bird"],
    "has_tail": [True, False],
    "can_fly": [True, False],
  }
)

outputs = module(...)
class_pred = outputs["classification"]
tail_pred = outputs["has_tail"]
fly_pred = outputs["can_fly"]
```

### Pattern 3: Hierarchical Outputs

Extract results at multiple levels:

```python
module = scallopy.Module(
  ...
  program="""
    rel direct_neighbor(a, b) = edge(a, b)
    rel two_hop(a, c) = edge(a, b), edge(b, c)
    rel three_hop(a, d) = two_hop(a, c), edge(c, d)
  """,
  output_mappings={
    "direct_neighbor": pairs,
    "two_hop": pairs,
    "three_hop": pairs,
  }
)
```

### Pattern 4: Dynamic Output Generation

Generate output mappings programmatically:

```python
# Generate all combinations
num_objects = 10
object_pairs = [(i, j) for i in range(num_objects) for j in range(num_objects) if i != j]

module = scallopy.Module(
  ...
  output_mapping=("similarity", object_pairs)
)
```

---

## Output Tensor Properties

### Shape

**Single output:**
- Without batch: `(num_tuples,)`
- With batch: `(batch_size, num_tuples)`

**Multiple outputs:**
- Returns a dictionary where each value has shape `(batch_size, num_tuples_for_that_relation)`

### Values

Each element is the **probability** of that tuple existing:
- `0.0` = tuple doesn't exist
- `1.0` = tuple certainly exists
- `0.0 < p < 1.0` = tuple exists with probability p

### Gradients

If inputs have `requires_grad=True`, output tensors will have gradients:

```python
digit_probs = torch.softmax(
  torch.randn(16, 10, requires_grad=True),
  dim=1
)

result = module(digit=digit_probs)
loss = criterion(result, target)
loss.backward()

# Gradients flow back to digit_probs
assert digit_probs.grad is not None
```

---

## Complete Example

```python
import torch
import scallopy

# Multi-output module for knowledge graph reasoning
module = scallopy.Module(
  provenance="difftopkproofs",
  k=5,
  program="""
    // Input relations
    type parent(person, person)
    type sibling(person, person)

    // Derived relations
    rel ancestor(a, d) = parent(a, d)
    rel ancestor(a, d) = ancestor(a, c), parent(c, d)
    rel cousin(a, b) = parent(pa, a), parent(pb, b), sibling(pa, pb)
    rel related(a, b) = ancestor(a, b)
    rel related(a, b) = cousin(a, b)
  """,
  input_mappings={
    "parent": person_pairs,
    "sibling": person_pairs,
  },
  output_mappings={
    "ancestor": person_pairs,
    "cousin": person_pairs,
    "related": person_pairs,
  }
)

# Forward pass
parent_probs = torch.softmax(torch.randn(8, len(person_pairs)), dim=1)
sibling_probs = torch.softmax(torch.randn(8, len(person_pairs)), dim=1)

outputs = module(parent=parent_probs, sibling=sibling_probs)

# Three output tensors
ancestor_probs = outputs["ancestor"]  # Shape: (8, len(person_pairs))
cousin_probs = outputs["cousin"]      # Shape: (8, len(person_pairs))
related_probs = outputs["related"]    # Shape: (8, len(person_pairs))

# Compute multi-task loss
loss = (
  criterion(ancestor_probs, ancestor_target) +
  criterion(cousin_probs, cousin_target) +
  criterion(related_probs, related_target)
)

loss.backward()
```

---

## Summary

- **Output mappings** extract Scallop results as PyTorch tensors
- **Single output**: `output_mapping=("relation", list)`
- **Multiple outputs**: `output_mappings={"rel1": list1, "rel2": list2}`
- **Formats**: range, list of values, list of tuples
- **Tensor shape**: `(batch_size, num_tuples)` with probability values
- **Gradients**: Flow back to inputs for end-to-end learning

For more details:
- [Creating Modules](module.md) - Overview of Scallop modules
- [Module Input](module_input.md) - Input mappings
- [Configuring Provenance](provenance.md) - Probability tracking
