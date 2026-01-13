# Configuring Provenance

Provenance in Scallopy determines how Scallop tracks and computes probabilities during execution. This guide shows you how to configure provenance in Python and work with probabilistic facts using the `ScallopContext` API.

## Setting Provenance in Python

When creating a `ScallopContext`, you specify the provenance type as a string parameter.

### Basic Provenance Configuration

```python
import scallopy

# Standard DataLog (no probability tracking)
ctx = scallopy.ScallopContext(provenance="unit")

# Min-max probability bounds (fast, conservative)
ctx = scallopy.ScallopContext(provenance="minmaxprob")

# Add-mult probability (fast, approximate)
ctx = scallopy.ScallopContext(provenance="addmultprob")

# Top-K proofs with exact probability
ctx = scallopy.ScallopContext(provenance="topkproofs", k=3)
```

### Available Provenance Types

The `provenance` parameter accepts the following values:

**Discrete Provenances** (no probability):
- `"unit"` - Standard DataLog, no provenance tracking
- `"proofs"` - Collect all derivation proofs
- `"tropical"` - Min-add semiring (positive integers + infinity)
- `"boolean"` - Boolean logic tracking
- `"natural"` - Natural number semiring

**Probabilistic Provenances**:
- `"minmaxprob"` - Min-max probability bounds (fast, conservative)
- `"addmultprob"` - Add-mult probability (fast, approximate)
- `"topkproofs"` - Top-K most probable proofs with exact probability via WMC
- `"probproofs"` - All proofs with exact probability
- `"topbottomkclauses"` - Top/bottom-K clauses for negation and aggregation

**Differentiable Provenances** (for PyTorch integration):
- `"difftopkproofs"` - Differentiable top-K proofs
- `"difftopkproofsdebug"` - With stable fact IDs for debugging
- `"diffminmaxprob"` - Differentiable min-max probability
- `"diffaddmultprob"` - Differentiable add-mult probability
- `"diffsamplekproofs"` - Differentiable unbiased sampling of K proofs
- `"difftopbottomkclauses"` - Differentiable top/bottom-K with full negation support

### The K Parameter

Provenances like `topkproofs` and `difftopkproofs` require a `k` parameter specifying how many proofs to track:

```python
# Keep top 5 most probable proofs for each derived fact
ctx = scallopy.ScallopContext(provenance="topkproofs", k=5)
```

**Tradeoff:**
- **Larger K** = More memory, more accurate probabilities, slower execution
- **Smaller K** = Less memory, approximate probabilities, faster execution

**Typical values:** k=3 for development, k=5-10 for production

### Train vs. Test K

For machine learning applications, you can use different K values during training and testing:

```python
ctx = scallopy.ScallopContext(
  provenance="difftopkproofs",
  train_k=3,  # Smaller K during training (faster)
  test_k=10   # Larger K during inference (more accurate)
)
```

---

## Adding Probabilistic Facts

Once you've configured provenance, you add facts with probabilities using the `add_facts()` method.

### Basic Probabilistic Facts

The most common format is a list of `(probability, tuple)` pairs:

```python
import scallopy

ctx = scallopy.ScallopContext(provenance="minmaxprob")
ctx.add_relation("edge", (int, int))

# Add facts with probabilities
ctx.add_facts("edge", [
  (0.1, (0, 1)),  # 10% probability
  (0.2, (1, 2)),  # 20% probability
  (0.3, (2, 3)),  # 30% probability
])

ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = edge(a, b), path(b, c)")
ctx.run()

# Inspect results with probabilities
for (prob, (start, end)) in ctx.relation("path"):
  print(f"Path {start}→{end}: probability {prob:.3f}")
```

**Output:**
```
Path 0→1: probability 0.100
Path 1→2: probability 0.200
Path 2→3: probability 0.300
Path 0→2: probability 0.200
Path 1→3: probability 0.300
Path 0→3: probability 0.300
```

### Facts Without Probabilities

If provenance supports it, you can omit probabilities (defaults to 1.0):

```python
ctx.add_facts("edge", [
  (0, 1),  # Implicitly probability 1.0
  (1, 2),
])
```

### Mutual Exclusion (Disjunctions)

When facts are **mutually exclusive** (only one can be true), specify disjunctions to get correct probabilities:

```python
ctx = scallopy.ScallopContext(provenance="topkproofs", k=3)
ctx.add_relation("color", (int, str))

# Object 0 can be blue OR green (mutually exclusive)
ctx.add_facts("color", [
  (0.7, (0, "blue")),
  (0.3, (0, "green")),
], disjunctions=[
  [0, 1]  # Indices 0 and 1 are mutually exclusive
])

# Object 1 can be red OR yellow
ctx.add_facts("color", [
  (0.6, (1, "red")),
  (0.4, (1, "yellow")),
], disjunctions=[
  [2, 3]  # Indices 2 and 3 are mutually exclusive
])
```

**Important:** The `disjunctions` parameter uses **indices** into the facts list being added. Each disjunction is a list of indices that are mutually exclusive.

**Multiple disjunction groups:**
```python
ctx.add_facts("relation", [
  (0.5, (0,)),  # Index 0
  (0.5, (1,)),  # Index 1
  (0.3, (2,)),  # Index 2
  (0.7, (3,)),  # Index 3
], disjunctions=[
  [0, 1],  # Facts 0 and 1 are mutually exclusive
  [2, 3],  # Facts 2 and 3 are mutually exclusive
])
```

### Loading Facts from CSV

For large datasets, load facts directly from CSV files:

```python
ctx = scallopy.ScallopContext(provenance="minmaxprob")

# Load CSV with implicit probability 1.0
ctx.add_relation("edge", (int, int), load_csv="edges.csv")
```

**CSV format with probabilities:**
```csv
0.9,0,1
0.8,1,2
0.7,2,3
```

```python
# First column is probability
ctx.add_relation("edge", (int, int), load_csv="edges_prob.csv")
```

For advanced CSV options, see [ScallopContext documentation](context.md#loading-csv).

---

## Differentiable Provenance

Differentiable provenances integrate with PyTorch, enabling gradient-based learning over symbolic reasoning.

### Basic Setup with PyTorch

```python
import torch
import scallopy

ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
ctx.add_relation("edge", (int, int))

# Add facts with torch tensors
ctx.add_facts("edge", [
  (torch.tensor(0.9), (0, 1)),
  (torch.tensor(0.8), (1, 2)),
  (torch.tensor(0.7), (2, 3)),
])

ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = edge(a, b), path(b, c)")
ctx.run()

# Results are tensors with gradients
for (prob_tensor, (start, end)) in ctx.relation("path"):
  print(f"Path {start}→{end}: {prob_tensor}")
```

### Forward Functions for Neural Networks

The most common pattern is using `forward_function()` to create differentiable modules:

```python
import torch
import scallopy

# Create context and define program
ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
ctx.add_relation("digit_1", int, range(10))
ctx.add_relation("digit_2", int, range(10))
ctx.add_rule("sum_2(a + b) = digit_1(a) and digit_2(b)")

# Create forward function
forward = ctx.forward_function("sum_2", list(range(19)))

# Use in training loop
digit_1 = torch.softmax(torch.randn((16, 10), requires_grad=True), dim=1)
digit_2 = torch.softmax(torch.randn((16, 10), requires_grad=True), dim=1)

# Forward pass through Scallop
sum_2 = forward(digit_1=digit_1, digit_2=digit_2)

# Backward pass computes gradients
loss = torch.nn.BCELoss()(sum_2, ground_truth)
loss.backward()

# Gradients flow back to digit_1 and digit_2
```

### Disjunctions with PyTorch

Mutual exclusion works with differentiable provenances too:

```python
ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
ctx.add_relation("obj_color", (int, str))

# Object colors with mutual exclusion
ctx.add_facts("obj_color", [
  (torch.tensor(0.99), (0, "blue")),
  (torch.tensor(0.01), (0, "green")),
], disjunctions=[[0, 1]])

ctx.add_facts("obj_color", [
  (torch.tensor(0.86), (1, "blue")),
  (torch.tensor(0.14), (1, "green")),
], disjunctions=[[0, 1]])
```

### Stable Fact IDs for Debugging

The `difftopkproofsdebug` provenance supports user-provided stable fact IDs for tracking and debugging:

```python
import torch
import scallopy

ctx = scallopy.ScallopContext(provenance="difftopkproofsdebug", k=3)
ctx.add_relation("edge", (int, int))

# Add facts with stable IDs
ctx.add_facts("edge", [
  ((torch.tensor(0.9), 1), (0, 1)),  # Fact ID = 1
  ((torch.tensor(0.8), 2), (1, 2)),  # Fact ID = 2
  ((torch.tensor(0.7), 3), (2, 3)),  # Fact ID = 3
])
```

**Fact ID requirements:**
- Start from 1 (not 0)
- Contiguous (no gaps)
- Unique across all facts

**Use cases:**
- Debugging: Trace which facts contributed to conclusions
- HNLE MCP: Retract facts by stable ID
- Provenance auditing: Track data lineage

For detailed usage, see [Debugging Probabilistic Programs](../probabilistic/debug.md).

---

## Custom Provenance

For advanced use cases, you can implement custom provenance semirings in Python.

### Built-in Python Provenances

Scallopy includes Python-implemented provenances:

```python
ctx = scallopy.ScallopContext(provenance="diffaddmultprob2")
# Equivalent to diffaddmultprob but implemented in Python
```

Available Python provenances:
- `"diffaddmultprob2"` - Add-mult probability
- `"diffnandmultprob2"` - NAND-mult probability
- `"diffmaxmultprob2"` - Max-mult probability

### Custom Provenance Objects

You can create custom provenance semirings by subclassing `ScallopProvenance`:

```python
from scallopy import ScallopProvenance

class MyCustomProvenance(ScallopProvenance):
  def __init__(self):
    super().__init__()

  def base(self, tag):
    # Define base tagging
    return tag

  def add(self, tag1, tag2):
    # Define disjunction (OR)
    return tag1 + tag2

  def mult(self, tag1, tag2):
    # Define conjunction (AND)
    return tag1 * tag2

# Use custom provenance
ctx = scallopy.ScallopContext(custom_provenance=MyCustomProvenance())
```

**Provenance semiring operations:**
- `base(tag)` - Tag a base fact
- `add(t1, t2)` - Combine alternative derivations (disjunction)
- `mult(t1, t2)` - Combine rule body atoms (conjunction)
- `negate(tag)` - Negate a tag (optional, for negation support)

---

## Common Patterns

### Pattern 1: Choosing the Right Provenance

**Decision tree:**

1. **Do you need probabilities?**
   - No → Use `"unit"` (fastest)
   - Yes → Continue

2. **Do you need gradients for ML?**
   - No → Continue to #3
   - Yes → Continue to #4

3. **Non-differentiable probabilistic:**
   - Fast bounds okay? → `"minmaxprob"`
   - Need exact probability? → `"topkproofs"` with k=5-10
   - Need all proofs? → `"probproofs"` (expensive)

4. **Differentiable for ML:**
   - Standard case → `"difftopkproofs"` with k=3-5
   - Need negation/aggregation → `"difftopbottomkclauses"`
   - Debugging needed → `"difftopkproofsdebug"`

### Pattern 2: Incremental Fact Addition

Add facts incrementally during execution:

```python
ctx = scallopy.ScallopContext(provenance="minmaxprob")
ctx.add_relation("edge", (int, int))

# Add initial facts
ctx.add_facts("edge", [(0.9, (0, 1))])
ctx.add_rule("path(a, c) = edge(a, c)")
ctx.run()

# Add more facts later
ctx.add_facts("edge", [(0.8, (1, 2))])
ctx.run()  # Re-run with new facts
```

### Pattern 3: Batched Facts with Input Mappings

For ML applications, use input mappings to define the domain:

```python
ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)

# Define relation with input mapping (domain)
ctx.add_relation("digit", int, input_mapping=range(10))

# Add batched facts (probability distribution over domain)
digit_probs = torch.softmax(torch.randn(10), dim=0)
ctx.add_facts("digit", digit_probs)
```

**Input mapping** defines the expected domain, and facts can be provided as:
- Tensor of shape `(domain_size,)` - probability distribution
- List of tuples - sparse facts

### Pattern 4: Provenance for Different Reasoning Tasks

**Knowledge graph reasoning:**
```python
ctx = scallopy.ScallopContext(provenance="topkproofs", k=10)
# Need exact probabilities for multi-hop reasoning
```

**Neurosymbolic learning:**
```python
ctx = scallopy.ScallopContext(provenance="difftopkproofs", k=3)
# Gradients for training, K=3 for efficiency
```

**Approximate inference:**
```python
ctx = scallopy.ScallopContext(provenance="minmaxprob")
# Fast bounds for large-scale applications
```

**Debugging and explanation:**
```python
ctx = scallopy.ScallopContext(provenance="difftopkproofsdebug", k=5)
# Stable fact IDs for tracing derivations
```

### Pattern 5: WMC Optimization

Enable WMC with disjunctions for better probability computation:

```python
ctx = scallopy.ScallopContext(
  provenance="topkproofs",
  k=5,
  wmc_with_disjunctions=True  # Better handling of disjunctions
)
```

This improves probability computation when you have many mutually exclusive facts.

---

## Summary

- **Set provenance** when creating `ScallopContext(provenance="...")`
- **18 provenance types** available: discrete, probabilistic, differentiable
- **Add probabilistic facts** with `add_facts(relation, [(prob, tuple), ...])`
- **Mutual exclusion** specified via `disjunctions` parameter
- **Differentiable provenances** integrate with PyTorch for gradient-based learning
- **Stable fact IDs** available in `difftopkproofsdebug` for debugging
- **Custom provenances** possible by subclassing `ScallopProvenance`

For more details:
- [Provenance Concepts](../probabilistic/provenance.md) - Theory and semiring framework
- [Provenance Library](../probabilistic/library.md) - All 18 provenance types explained
- [Debugging Proofs](../probabilistic/debug.md) - Using difftopkproofsdebug
- [ScallopContext API](context.md) - Complete API reference
- [Creating Modules](module.md) - PyTorch integration for ML
