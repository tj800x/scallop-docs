# Creating Modules

Scallop modules are PyTorch-compatible components that wrap Scallop programs for seamless integration with neural networks. They enable end-to-end differentiable neurosymbolic learning by combining neural perception with symbolic reasoning.

## API Overview

Scallop provides two primary APIs for creating differentiable modules:

- **`scallopy.Module`** - High-level PyTorch `nn.Module` wrapper
- **`scallopy.ScallopForwardFunction`** - Functional interface for forward passes

Both APIs provide the same functionality. Use `ScallopForwardFunction` for functional-style code or when you need fine-grained control. This guide uses both interchangeably.

**Note:** All examples in this guide work with both APIs. Replace `scallopy.Module(...)` with `scallopy.ScallopForwardFunction(...)` as needed.

## What is a Scallop Module?

A Scallop module is a `torch.nn.Module` subclass that:
- Wraps a Scallop program (relations, rules, and facts)
- Accepts PyTorch tensors as probabilistic input facts
- Performs symbolic reasoning via Scallop's execution engine
- Returns PyTorch tensors as output with gradient support
- Integrates seamlessly into neural network architectures

### Key Benefits

**1. Declarative Logic**: Express reasoning symbolically instead of learning it from data
```python
# Instead of training a neural network to learn addition...
# ...declare it symbolically:
"sum_2(a + b) = digit_a(a) and digit_b(b)"
```

**2. Gradient Flow**: Backpropagation works through symbolic reasoning
```python
loss.backward()  # Gradients flow through Scallop module
```

**3. Interpretability**: Logic is explicit and human-readable
```python
# Rules are visible: you know exactly what the model is doing
ctx.add_rule("path(a, c) = edge(a, b), path(b, c)")
```

**4. Sample Efficiency**: Inject domain knowledge instead of learning everything
```python
# Neural network learns perception (digit classification)
# Scallop handles symbolic reasoning (multi-digit arithmetic)
```

---

## Important: Program Requirements

### Type Declarations Required

When creating modules, your Scallop program **must include type declarations** for input relations:

```python
# ✓ Correct - includes type declarations
sum_2 = scallopy.Module(
  program="""
    type digit_a(usize), digit_b(usize)
    rel sum_2(a + b) = digit_a(a) and digit_b(b)
  """,
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)

# ✗ Incorrect - missing type declarations
sum_2 = scallopy.Module(
  program="rel sum_2(a + b) = digit_a(a) and digit_b(b)",  # Will cause warnings
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)
```

**Why?** Type declarations help Scallop's compiler optimize the program and ensure type safety.

### Batch Dimension Required

Input tensors **must have a batch dimension** as the first axis:

```python
# ✓ Correct - shape: (batch_size, num_classes)
digit_a = torch.randn(16, 10)  # Batch of 16, 10 digit classes
digit_b = torch.randn(16, 10)
result = sum_2(digit_a=digit_a, digit_b=digit_b)
# result.shape: (16, 19)

# ✗ Incorrect - missing batch dimension
digit_a = torch.randn(10)  # Shape error!
result = sum_2(digit_a=digit_a, digit_b=digit_b)
```

**Why?** Scallop processes batches of inputs for efficiency. Even for single examples, use `tensor.unsqueeze(0)` to add batch dimension.

---

## Creating Basic Modules

There are three ways to create a Scallop module:

### Method 1: Inline Program String

The most common approach for simple programs:

```python
import scallopy

sum_2 = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="""
    type digit_a(usize), digit_b(usize)
    rel sum_2(a + b) = digit_a(a) and digit_b(b)
  """,
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)
```

**When to use**: Short programs (< 20 lines), quick prototyping

### Method 2: External File

For larger programs, load from a `.scl` file:

```python
# File: reasoning.scl contains your Scallop program
module = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  file="reasoning.scl",
  input_mappings={"input_rel": range(100)},
  output_mapping=("output_rel", range(50))
)
```

**When to use**: Large programs, code reuse, version control

### Method 3: Programmatic Construction

Build the program piece by piece:

```python
module = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  relations=[
    ("digit_1", (int,)),
    ("digit_2", (int,)),
  ],
  rules=[
    "sum_2(a + b) = digit_1(a) and digit_2(b)",
    "mult_2(a * b) = digit_1(a) and digit_2(b)",
  ],
  input_mappings={
    "digit_1": range(10),
    "digit_2": range(10),
  },
  output_mappings={
    "sum_2": range(19),
    "mult_2": range(100),
  }
)
```

**When to use**: Dynamic program generation, conditional logic structure

---

## Input and Output Mappings

Mappings define how PyTorch tensors are converted to/from Scallop facts.

### Input Mappings

Input mappings specify the **domain** of input relations - the set of possible values:

```python
input_mappings={
  "digit": range(10),  # Domain: digits 0-9
  "color": ["red", "green", "blue"],  # Domain: three colors
}
```

**How it works:**

When you call the module with a tensor:
```python
digit_probs = torch.tensor([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.1])
result = module(digit=digit_probs)
```

Scallop interprets this as:
```
digit(0) with probability 0.1
digit(1) with probability 0.0
...
digit(8) with probability 0.8
digit(9) with probability 0.1
```

**Formats:**

1. **Range**: `range(n)` for integers 0 to n-1
   ```python
   input_mappings={"digit": range(10)}
   ```

2. **List**: Explicit enumeration
   ```python
   input_mappings={"color": ["red", "green", "blue"]}
   ```

3. **List of tuples**: For multi-column relations
   ```python
   input_mappings={
     "edge": [(0,1), (1,2), (2,3), (0,3)]
   }
   ```

### Output Mappings

Output mappings specify how to extract results from Scallop relations:

**Single output:**
```python
output_mapping=("sum_2", range(19))
# Extracts sum_2(0), sum_2(1), ..., sum_2(18)
# Returns tensor of shape (batch_size, 19)
```

**Multiple outputs:**
```python
output_mappings={
  "sum_2": range(19),
  "mult_2": range(100),
}
# Returns dictionary: {"sum_2": tensor1, "mult_2": tensor2}
```

**Tuple outputs:**
```python
output_mapping=("path", [(0,1), (0,2), (1,2)])
# Extracts path(0,1), path(0,2), path(1,2)
# Returns tensor of shape (batch_size, 3)
```

---

## Forward Pass

Once created, use the module like any PyTorch component.

### Basic Forward Pass

```python
import torch
import scallopy

# Create module
sum_2 = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="rel sum_2(a + b) = digit_a(a) and digit_b(b)",
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)

# Prepare input: probability distributions over digits
digit_a = torch.tensor([
  [0.0, 0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],  # Mostly 1
  [0.0, 0.0, 0.0, 0.8, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0],  # Mostly 3
])
digit_b = torch.tensor([
  [0.0, 0.0, 0.8, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],  # Mostly 2
  [0.0, 0.0, 0.0, 0.0, 0.9, 0.1, 0.0, 0.0, 0.0, 0.0],  # Mostly 4
])

# Forward pass
result = sum_2(digit_a=digit_a, digit_b=digit_b)

# Result shape: (2, 19) - batch of 2, sums 0-18
print(result)
# Output:
# tensor([[0.0, 0.0, 0.0, 0.72, 0.18, ...],   # 1+2=3 (prob 0.72)
#         [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.72, ...]])  # 3+4=7
```

### Gradient Computation

Gradients flow through the Scallop module:

```python
# Inputs with gradients
digit_a = torch.randn(16, 10, requires_grad=True)
digit_a = torch.softmax(digit_a, dim=1)

digit_b = torch.randn(16, 10, requires_grad=True)
digit_b = torch.softmax(digit_b, dim=1)

# Forward pass
sum_result = sum_2(digit_a=digit_a, digit_b=digit_b)

# Compute loss
ground_truth = torch.zeros(16, 19)
ground_truth[:, 5] = 1.0  # Target: sum should be 5

loss_fn = torch.nn.BCELoss()
loss = loss_fn(sum_result, ground_truth)

# Backward pass
loss.backward()

# Gradients are computed for digit_a and digit_b
assert digit_a.grad is not None
assert digit_b.grad is not None
```

### Multiple Outputs

When using `output_mappings` (plural), the module returns a dictionary:

```python
module = scallopy.Module(
  provenance="diffaddmultprob",
  program="""
    rel sum_2(a + b) = digit_1(a) and digit_2(b)
    rel mult_2(a * b) = digit_1(a) and digit_2(b)
  """,
  input_mappings={"digit_1": range(10), "digit_2": range(10)},
  output_mappings={
    "sum_2": range(20),
    "mult_2": range(100),
  }
)

digit_1 = torch.randn(16, 10)
digit_2 = torch.randn(16, 10)

result = module(digit_1=digit_1, digit_2=digit_2)

# Result is a dictionary
print(result["sum_2"].shape)   # (16, 20)
print(result["mult_2"].shape)  # (16, 100)
```

---

## Integration with Neural Networks

Scallop modules compose naturally with PyTorch layers.

### Pattern 1: Symbolic Reasoning Layer

Use Scallop as a reasoning component in a larger network:

```python
import torch
import torch.nn as nn
import scallopy

class DigitAdder(nn.Module):
  def __init__(self):
    super().__init__()

    # Neural perception: classify digits from images
    self.digit_classifier = nn.Sequential(
      nn.Linear(784, 128),
      nn.ReLU(),
      nn.Linear(128, 10),
    )

    # Symbolic reasoning: add digits
    self.adder = scallopy.Module(
      provenance="difftopkproofs",
      k=3,
      program="rel sum(a + b) = digit_a(a) and digit_b(b)",
      input_mappings={"digit_a": range(10), "digit_b": range(10)},
      output_mapping=("sum", range(19))
    )

  def forward(self, img_a, img_b):
    # Neural: classify digits
    logits_a = self.digit_classifier(img_a)
    logits_b = self.digit_classifier(img_b)

    probs_a = torch.softmax(logits_a, dim=1)
    probs_b = torch.softmax(logits_b, dim=1)

    # Symbolic: add them
    sum_probs = self.adder(digit_a=probs_a, digit_b=probs_b)

    return sum_probs

# Training loop
model = DigitAdder()
optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)
loss_fn = nn.CrossEntropyLoss()

for img_a, img_b, target_sum in dataloader:
  optimizer.zero_grad()

  sum_probs = model(img_a, img_b)
  loss = loss_fn(sum_probs, target_sum)

  loss.backward()
  optimizer.step()
```

### Pattern 2: Knowledge Graph Reasoning

Inject symbolic knowledge into perception:

```python
class KnowledgeEnhancedClassifier(nn.Module):
  def __init__(self):
    super().__init__()

    # Neural perception
    self.encoder = nn.Linear(input_dim, 64)

    # Symbolic reasoning with domain knowledge
    self.reasoner = scallopy.Module(
      provenance="difftopkproofs",
      k=5,
      program="""
        rel parent(p, c) = raw_parent(p, c)
        rel ancestor(a, d) = parent(a, d)
        rel ancestor(a, d) = ancestor(a, c), parent(c, d)
        rel sibling(a, b) = parent(p, a), parent(p, b), a != b
      """,
      input_mappings={"raw_parent": parent_pairs},
      output_mapping=("sibling", sibling_pairs)
    )

  def forward(self, features):
    encoded = self.encoder(features)
    parent_probs = torch.softmax(encoded, dim=1)

    sibling_probs = self.reasoner(raw_parent=parent_probs)
    return sibling_probs
```

### Pattern 3: Multi-Task Learning

Use multiple output relations for joint predictions:

```python
classifier = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="""
    rel is_animal(x) = is_cat(x)
    rel is_animal(x) = is_dog(x)
    rel has_tail(x) = is_animal(x), not is_human(x)
    rel can_fly(x) = is_bird(x)
  """,
  input_mappings={
    "is_cat": range(100),
    "is_dog": range(100),
    "is_bird": range(100),
    "is_human": range(100),
  },
  output_mappings={
    "is_animal": range(100),
    "has_tail": range(100),
    "can_fly": range(100),
  }
)

# Single forward pass computes all outputs jointly
outputs = classifier(is_cat=cat_probs, is_dog=dog_probs, ...)
animal_pred = outputs["is_animal"]
tail_pred = outputs["has_tail"]
fly_pred = outputs["can_fly"]
```

---

## Common Patterns

### Pattern 1: Train/Test K Configuration

Use smaller K during training for speed, larger K during inference for accuracy:

```python
model = scallopy.Module(
  provenance="difftopkproofs",
  train_k=3,   # Fast during training
  test_k=10,   # Accurate during inference
  program=program_str,
  input_mappings=input_maps,
  output_mapping=output_map
)

# Automatically uses train_k during training
model.train()
output = model(input_data)

# Automatically uses test_k during evaluation
model.eval()
output = model(input_data)
```

### Pattern 2: JIT Compilation

Enable JIT compilation for faster execution:

```python
model = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program=program_str,
  input_mappings=input_maps,
  output_mapping=output_map,
  jit=True,
  jit_name="my_model"  # Optional: for caching
)
```

**Benefits:**
- First run compiles the model
- Subsequent runs use cached compiled version
- Significant speedup for repeated executions

### Pattern 3: Sparse Gradients

For large domains with sparse activations:

```python
# Create forward function with sparse Jacobian
ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
ctx.add_relation("input", int, range(1000))
ctx.add_rule("output(x * 2) = input(x)")

forward = ctx.forward_function(
  "output",
  range(2000),
  sparse_jacobian=True  # Use sparse gradients
)

# Gradients are now sparse tensors
input_data = torch.randn(1000, requires_grad=True)
output = forward(input=input_data)
loss = output.sum()
loss.backward()  # Efficient sparse gradient computation
```

### Pattern 4: Non-Probabilistic Inputs

Some inputs don't need probability tracking:

```python
model = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="""
    rel weighted_sum(x * w + y * w) = value(x), value(y), weight(w)
  """,
  input_mappings={
    "value": range(10),
    "weight": [0.5, 1.0, 1.5, 2.0],
  },
  non_probabilistic=["weight"],  # Weights are fixed
  output_mapping=("weighted_sum", range(100))
)
```

### Pattern 5: Iteration Limits

Control recursion depth:

```python
model = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="""
    rel path(a, b) = edge(a, b)
    rel path(a, c) = path(a, b), edge(b, c)
  """,
  input_mappings={"edge": edge_pairs},
  output_mapping=("path", path_pairs),
  iter_limit=10  # Maximum 10 iterations of recursion
)
```

---

## Summary

- **Scallop modules** integrate symbolic reasoning into PyTorch neural networks
- **Three creation methods**: inline program, external file, programmatic
- **Input mappings** define domains; tensors represent probability distributions
- **Output mappings** extract results; support single or multiple outputs
- **Gradient flow** enables end-to-end differentiable neurosymbolic learning
- **Common patterns**: train/test K, JIT compilation, sparse gradients, non-probabilistic inputs

For more details:
- [Module Input](module_input.md) - Deep dive into input mappings and formats
- [Module Output](module_output.md) - Deep dive into output mappings and formats
- [Configuring Provenance](provenance.md) - Choosing the right provenance
- [Save and Load](save_and_load.md) - Serializing trained models
- [ScallopContext](context.md) - Lower-level API for custom workflows
