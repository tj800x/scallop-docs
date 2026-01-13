# Debugging Probabilistic Programs

Probabilistic reasoning adds complexity to logic programs. When probabilities don't match expectations or conclusions seem wrong, you need tools to understand what's happening. This guide covers debugging techniques for Scallop's probabilistic features.

## Why Debug Probabilistic Programs?

Debugging probabilistic programs presents unique challenges:

### Common Issues

**1. Unexpected Probabilities**
``` scl
rel 0.9::reliable(0, 1)
rel 0.8::reliable(1, 2)
rel path(a, c) = reliable(a, b), reliable(b, c)
query path(0, 2)
// Expected: 0.72, Got: Something else?
```

Why might the probability be wrong?
- Wrong provenance choice (minmaxprob vs. topkproofs)
- Mutual exclusion not specified
- Missing facts or rules
- Incorrect probability formula

**2. Missing Derivations**
```
Why isn't this tuple in my results?
```

Possible causes:
- Facts filtered due to low probability
- Top-K limit excluding less probable proofs
- Logical error in rules
- Type mismatches

**3. Performance Bottlenecks**
```
Program runs very slowly on probabilistic data
```

Could be:
- Too many proofs being tracked
- Wrong provenance for the task
- Inefficient rule structure
- Memory exhaustion from proof explosion

**4. Unexplainable Results**
```
How did Scallop arrive at this conclusion?
```

Need to see:
- Which facts were used
- How probabilities combined
- Alternative derivation paths

### The Solution: Debug Provenances

Scallop provides **debug provenance modes** that expose the internal reasoning:
- **Fact IDs**: Unique identifiers for each base fact
- **Proofs**: Sets of fact IDs that derive each conclusion
- **Probability breakdowns**: How values were computed

---

## Using Debug Provenances

The `difftopkproofsdebug` provenance is specifically designed for debugging. It provides:

1. **Stable Fact IDs**: User-controlled identifiers for facts
2. **Full Proof Structures**: See exactly which facts derive each conclusion
3. **Probability Tracking**: Understand how probabilities propagate
4. **Gradient Support**: Works with PyTorch for neurosymbolic debugging

### Basic Setup

``` py
import torch
import scallopy

# Create debug context
ctx = scallopy.ScallopContext(provenance="difftopkproofsdebug", k=3)

# Define relations
ctx.add_relation("edge", (int, int))

# Add facts with EXPLICIT FACT IDs
ctx.add_facts("edge", [
  ((torch.tensor(0.9), 1), (0, 1)),  # Fact ID 1
  ((torch.tensor(0.8), 2), (1, 2)),  # Fact ID 2
  ((torch.tensor(0.2), 3), (0, 2)),  # Fact ID 3
])

# Add rules
ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")

# Run
ctx.run()

# Inspect results
for (result, (a, b)) in ctx.relation("path"):
  print(f"Path ({a}, {b}): {result}")
```

### Understanding the Output

When using `difftopkproofsdebug` with `forward()` in modules, you get back `(result_tensor, proofs)`:

``` py
(result_tensor, proofs) = module(input_a=facts_a, input_b=facts_b)
```

**Result Tensor**: Standard probability outputs
```python
result_tensor = torch.tensor([
  [0.09, 0.8119, 0.09],  # Probabilities for each output tuple
])
```

**Proofs Structure**: Nested lists showing which fact IDs were used
```python
proofs = [
  [ # Datapoint 1
    [ # Tuple 1 proofs
      [(True, 1), (True, 3)],  # First proof uses facts 1 and 3
      [(True, 2), (True, 4)],  # Second proof uses facts 2 and 4
    ],
    [ # Tuple 2 proofs
      [(True, 5)],  # Uses fact 5 alone
    ]
  ]
]
```

**Proof Format:** `List[List[List[List[Tuple[bool, int]]]]]`
- **Batch** → **Datapoint** → **Tuple Proofs** → **Individual Proof** → **Literal**
- Each literal: `(is_positive, fact_id)`
  - `is_positive=True`: Fact is required (positive literal)
  - `is_positive=False`: Fact must be absent (negative literal, with negation)
  - `fact_id`: The user-provided fact ID

### Complete Debugging Example

``` py
import torch
import scallopy

# Setup
sum_2 = scallopy.Module(
  provenance="difftopkproofsdebug",
  k=3,
  program="rel sum_2(a + b) = digit_a(a) and digit_b(b)",
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)

# Prepare facts with IDs
# Fact IDs: Digit A=1 → ID 1, Digit A=2 → ID 2
#           Digit B=1 → ID 3, Digit B=2 → ID 4
digit_a = [
  [((torch.tensor(0.1), 1), (1,)), ((torch.tensor(0.9), 2), (2,))],
]
digit_b = [
  [((torch.tensor(0.9), 3), (1,)), ((torch.tensor(0.1), 4), (2,))],
]

# Run and get proofs
(result_tensor, proofs) = sum_2(digit_a=digit_a, digit_b=digit_b)

print("Results:", result_tensor)
# Results: tensor([[0.09, 0.8119, 0.09]])

print("\nProofs:")
for datapoint_idx, datapoint_proofs in enumerate(proofs):
  print(f"  Datapoint {datapoint_idx}:")
  for tuple_idx, tuple_proofs in enumerate(datapoint_proofs):
    print(f"    Tuple {tuple_idx} (sum={tuple_idx+2}):")
    for proof in tuple_proofs:
      fact_ids = [fact_id for (is_pos, fact_id) in proof if is_pos]
      print(f"      Proof: Facts {fact_ids}")

# Output:
# Datapoint 0:
#   Tuple 0 (sum=2):
#     Proof: Facts [1, 3]  # 1+1=2
#   Tuple 1 (sum=3):
#     Proof: Facts [1, 4]  # 1+2=3
#     Proof: Facts [2, 3]  # 2+1=3
#   Tuple 2 (sum=4):
#     Proof: Facts [2, 4]  # 2+2=4
```

---

## Stable Fact IDs

A key feature of `difftopkproofsdebug` is **user-provided stable fact IDs**. This is the **ONLY provenance type** that supports this feature.

### Why Stable IDs?

**Problem:** Without stable IDs, Scallop assigns fact IDs sequentially (0, 1, 2, ...) in order of insertion. If you add or remove facts, all IDs shift, breaking traceability.

**Solution:** With `difftopkproofsdebug`, YOU control the fact IDs. This enables:

1. **Fact Retraction**: Remove specific facts by ID later
2. **Traceability**: Track which source data contributed to conclusions
3. **Provenance Auditing**: Maintain stable references across program runs
4. **Complex Data Handling**: Works with facts containing newlines, quotes, etc.

### Fact ID Requirements

When using stable IDs, you must follow these rules:

1. **Start from 1**: IDs begin at 1 (not 0)
2. **Contiguous**: No gaps (1, 2, 3, 4, ..., N)
3. **Unique**: Each fact gets a different ID
4. **User-Managed**: You're responsible for avoiding collisions

**Example:**
``` py
# CORRECT: IDs are 1, 2, 3 (contiguous, starting from 1)
ctx.add_facts("edge", [
  ((torch.tensor(0.9), 1), (0, 1)),
  ((torch.tensor(0.8), 2), (1, 2)),
  ((torch.tensor(0.2), 3), (0, 2)),
])

# INCORRECT: IDs skip (1, 3, 5) or start from 0
ctx.add_facts("edge", [
  ((torch.tensor(0.9), 0), (0, 1)),  # ❌ Starts from 0
  ((torch.tensor(0.8), 2), (1, 2)),  # ❌ Skips 1
])
```

### ID Management Strategy

**For small programs:**
```python
fact_id = 1
for fact_data in my_facts:
  facts_with_ids.append(((torch.tensor(prob), fact_id), fact_data))
  fact_id += 1
```

**For large systems (HNLE pattern):**
```python
class FactIDManager:
  def __init__(self):
    self.next_id = 1
    self.id_to_fact = {}  # Reverse mapping

  def allocate_id(self, fact_content):
    fact_id = self.next_id
    self.next_id += 1
    self.id_to_fact[fact_id] = fact_content
    return fact_id

  def retract(self, fact_id):
    del self.id_to_fact[fact_id]
    # Note: IDs are not reused
```

### Use Case: HNLE Knowledge Management

The HNLE (Hyperion kNowledge Learning Engine) MCP server uses stable fact IDs for:

**Asserting facts with complex strings:**
``` py
# Agent asserts a fact with newlines and special characters
fact_content = "rel edge(0, 1, \"complex\nstring\nwith\nquotes\")"
fact_id = manager.allocate_id(fact_content)

ctx.add_facts("edge", [
  ((torch.tensor(0.8), fact_id), (0, 1, fact_content))
])
# Returns: {"stable_id": fact_id}
```

**Querying with provenance:**
``` py
results = ctx.relation("path")
# Each result includes fact IDs in proofs
# Can trace back: "path(0, 2) was derived using facts 1 and 3"
```

**Retracting by stable ID:**
``` py
# Later, retract without reconstructing the complex string
manager.retract(fact_id)
# Remove from Scallop context using the stable ID
```

**Key Advantage:** No need to escape or reconstruct complex strings with newlines, quotes, etc. Just use the stable ID.

### Limitations (from FloatWithID Research)

**Important caveats about stable IDs:**

1. **API-Only**: IDs exist during execution but are NOT persisted to `.scl` files
2. **Display Format**: Scallop can display `0.8 [ID(42)]` but **cannot parse it back**
3. **Single Provenance**: Only `difftopkproofsdebug` supports user IDs
4. **Manual Management**: You must track ID mappings externally if needed across sessions

**Example - What doesn't work:**
``` scl
// Writing to .scl file:
rel <42>0.8::edge(0, 1)  // ❌ Syntax not supported

// Reading from display:
"0.8 [ID(42)]"  // ✓ Can display
// ❌ Parser cannot read this back - IDs lost on reload
```

**Workaround for Persistence:** Maintain external mapping file
``` json
{
  "fact_id_mapping": {
    "1": {"relation": "edge", "tuple": "(0, 1)", "prob": 0.9},
    "2": {"relation": "edge", "tuple": "(1, 2)", "prob": 0.8}
  }
}
```

---

## Common Debugging Patterns

### Pattern 1: Finding Which Facts Matter

**Problem:** A conclusion has surprisingly high/low probability. Which input facts are responsible?

**Solution:**
``` py
# Run with debug provenance
ctx = scallopy.ScallopContext(provenance="difftopkproofsdebug", k=5)
# ... add facts with IDs, run program ...

# Examine proofs for specific tuple
for (result, tuple_data) in ctx.relation("suspicious_relation"):
  if tuple_data == target_tuple:
    # Extract fact IDs from proofs (when using Module.forward)
    print(f"Tuple {tuple_data} derived from facts: {proofs}")
```

### Pattern 2: Comparing Provenance Behaviors

**Problem:** Different provenances give different results. Why?

**Solution:** Run with multiple provenances and compare
``` py
for prov in ["minmaxprob", "topkproofs", "probproofs"]:
  ctx = scallopy.ScallopContext(provenance=prov, k=5)
  # ... same facts and rules ...
  ctx.run()
  print(f"{prov}: {list(ctx.relation('result'))}")
```

**Common finding:** `minmaxprob` gives bounds, `topkproofs` gives exact probability.

### Pattern 3: Identifying Filtered Derivations

**Problem:** Expected more results. Are they being filtered by Top-K?

**Solution:** Increase K and see if results appear
``` py
for k in [1, 3, 5, 10]:
  ctx = scallopy.ScallopContext(provenance="topkproofs", k=k)
  # ... add facts and rules ...
  ctx.run()
  print(f"k={k}: {len(list(ctx.relation('result')))} results")
```

If count increases with K, you're hitting the proof limit.

### Pattern 4: Debugging Probability Computation

**Problem:** Don't understand why probability is X.

**Solution:** Check WMC formula
``` py
# With difftopkproofsdebug, examine proofs structure
# Example: Proofs {{1,2}, {3}}
# Formula: (fact_1 ∧ fact_2) ∨ fact_3
# Probability: P(1)*P(2) + P(3) - P(1)*P(2)*P(3)

# Manually compute
p1, p2, p3 = 0.9, 0.8, 0.2
p_result = p1*p2 + p3 - p1*p2*p3
print(f"Manual: {p_result}")  # Compare to Scallop output
```

### Pattern 5: Detecting Mutual Exclusion Issues

**Problem:** Probabilities sum to more than 1.0

**Solution:** Check if mutual exclusion is specified
``` scl
// WRONG: No mutual exclusion
rel color = {0.7::red, 0.6::blue}  // Sums to 1.3!

// CORRECT: Mutual exclusion with semicolon
rel color = {0.7::red; 0.3::blue}  // Sums to 1.0
```

**In Python with disjunctions:**
``` py
ctx.add_facts("color", [
  (0.7, "red"),
  (0.3, "blue")
], disjunctions=[[0, 1]])  # Facts 0 and 1 are mutually exclusive
```

---

## Debugging Checklist

When your probabilistic program doesn't behave as expected:

- [ ] **Check provenance choice**: Is it appropriate for your task?
  - Need exact probability? → `topkproofs` or `probproofs`
  - Fast bounds okay? → `minmaxprob`

- [ ] **Verify fact probabilities**: Do input facts have correct values?
  - Print all input facts before running
  - Check for typos in probabilities

- [ ] **Check mutual exclusion**: Are annotated disjunctions specified?
  - Use `;` separator in .scl files
  - Use `disjunctions` parameter in Python

- [ ] **Inspect K parameter**: Are proofs being cut off?
  - Try increasing K
  - Check if more results appear

- [ ] **Enable debug mode**: Use `difftopkproofsdebug` to see proofs
  - Add stable fact IDs
  - Examine which facts contribute to conclusions

- [ ] **Validate rules**: Are logical rules correct?
  - Test with `unit` provenance first
  - Verify basic logic before adding probabilities

- [ ] **Check for type errors**: Do tuple types match relation schemas?
  - Use `add_relation` to declare schemas
  - Verify tuple arities

---

## Summary

- **Debug provenances** expose internal reasoning
- **`difftopkproofsdebug`** provides fact IDs and full proof structures
- **Stable fact IDs** enable traceability and retraction
- **Only API feature**: IDs not persisted to .scl files
- **Common patterns**: Compare provenances, check K limits, inspect proofs
- **Use for**: HNLE knowledge management, explainable AI, debugging complex probabilistic programs

---

## Further Reading

- [Debugging Proofs (Python API)](../scallopy/debug_proofs.md) - Detailed Python examples
- [Proofs Provenance](proofs.md) - Understanding derivation proofs
- [Provenance Library](library.md) - All provenance types
- [Scallopy Provenance](../scallopy/provenance.md) - Python API for provenance configuration
