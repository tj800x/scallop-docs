# Crash Course

Welcome to Scallop! This crash course will get you started with Scallop in about 15 minutes. You'll learn the basics of logic programming, probabilistic reasoning, and Python integration.

## What is Scallop?

Scallop is a DataLog-based language that combines three powerful paradigms:

1. **Logic Programming**: Write declarative rules to derive new facts from existing ones
2. **Probabilistic Reasoning**: Attach probabilities to facts and track uncertainty through computations
3. **Differentiable Computing**: Integrate with machine learning frameworks like PyTorch for neurosymbolic AI

Scallop is built on a **Provenance Semiring** framework that tracks how conclusions are derived. This means you can not only compute answers, but also understand *why* those answers exist and *how probable* they are.

**Common use cases:**
- Knowledge graph reasoning
- Probabilistic databases
- Neurosymbolic AI (combining neural networks with symbolic logic)
- Program analysis
- Question answering with uncertainty

Let's dive in!

---

## Installation

Before you begin, make sure you have Scallop installed:

### For Command-Line Programs

Install the Scallop CLI tools (`scli`, `sclrepl`):

**From binary releases:**
```bash
# Download from https://github.com/scallop-lang/scallop/releases
# Or build from source:
git clone https://github.com/scallop-lang/scallop.git
cd scallop
cargo build --release
# Binaries in target/release/
```

**Verify installation:**
```bash
scli --version
# Output: scli 0.2.5
```

### For Python Integration

Install scallopy for Python:

```bash
pip install scallopy
```

**Verify installation:**
```python
import scallopy
print(scallopy.__version__)
```

For complete installation instructions, see [Scallop CLI](toolchain/cli.md) and [Getting Started with Scallopy](scallopy/getting_started.md).

---

## Your First Scallop Program

The best way to learn is by example. Let's start with a classic problem: computing the transitive closure of a graph.

### The Problem

Suppose we have a graph with edges connecting nodes:
- Node 0 connects to node 1
- Node 1 connects to node 2
- Node 2 connects to node 3

We want to find all paths in this graph (not just direct edges).

### The Scallop Solution

Create a file called `edge_path.scl`:

``` scl
rel edge = {(0, 1), (1, 2), (2, 3)}

rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b) and edge(b, c)

query path
```

Let's break this down line by line:

**Line 1:** We declare facts about edges using set notation. This defines three edges in our graph.

**Line 3:** The first rule says "there's a path from `a` to `b` if there's an edge from `a` to `b`". This handles direct connections.

**Line 4:** The second rule says "there's a path from `a` to `c` if there's a path from `a` to `b` AND an edge from `b` to `c`". This is the recursive case that builds longer paths.

**Line 6:** We query all paths to see the results.

### Running the Program

Save the file and run it with the Scallop interpreter:

``` bash
scli edge_path.scl
```

You'll see the output:

```
path: {(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)}
```

Scallop found all six paths in the graph! Notice how it computed the transitive closure automatically using the recursive rules.

### Key Concepts

- **Relations**: Like `edge` and `path` - they hold sets of tuples
- **Facts**: Individual data points like `(0, 1)`
- **Rules**: Logical statements with `=` that derive new facts
- **Queries**: Ask Scallop to show you the results

---

## Adding Probabilities

Now let's make things more interesting by adding probabilities. We'll model rolling two dice and computing the maximum value.

### Probabilistic Facts

Create a file called `double_dice.scl`:

``` scl
rel first_dice = {
  0.166::1;
  0.166::2;
  0.166::3;
  0.166::4;
  0.166::5;
  0.166::6;
}

rel second_dice = {
  0.166::1;
  0.166::2;
  0.166::3;
  0.166::4;
  0.166::5;
  0.166::6;
}

rel result(x) = first_dice(x), x > 3
rel result(y > x ? y : x) = first_dice(x), x <= 3, second_dice(y)

query result
```

### Understanding the Syntax

**Probabilistic facts** use the `::` operator: `probability::value`

**Semicolons** (`;`) indicate mutual exclusion - the die can only show one number at a time. This is called an **annotated disjunction**.

**The logic:**
- If the first die shows > 3, that's our result (we don't need the second die)
- If the first die shows ≤ 3, we take the maximum of both dice

### Running with Probabilities

``` bash
scli --provenance minmaxprob double_dice.scl
```

The `--provenance minmaxprob` flag tells Scallop to track probabilities using the min-max provenance (a conservative probability bound).

You'll see results like:

```
result: {0.166::(4), 0.166::(5), 0.416::(6), 0.083::(3), ...}
```

Each result has a probability! For example, getting a 6 has probability ~0.416 (41.6%).

### Key Probabilistic Concepts

- **Tagged facts**: `probability::fact` attaches probabilities to data
- **Annotated disjunctions**: `;` separator for mutually exclusive alternatives
- **Provenance**: The tracking method (we'll explore more types later)

---

## Python Integration

Scallop really shines when integrated with Python for machine learning applications. Let's see how to use the Python API.

### Setting Up

First, install scallopy:

``` bash
pip install scallopy
```

### Your First Python Program

Create `edge_path_prob.py`:

``` py
from scallopy import ScallopContext

# Create a context with probabilistic reasoning
ctx = ScallopContext(provenance="minmaxprob")

# Define the relation schema
ctx.add_relation("edge", (int, int))

# Add probabilistic facts
ctx.add_facts("edge", [
  (0.1, (0, 1)),  # 10% chance of edge 0→1
  (0.2, (1, 2)),  # 20% chance of edge 1→2
  (0.3, (2, 3)),  # 30% chance of edge 2→3
])

# Add rules
ctx.add_rule("path(a, c) = edge(a, c)")
ctx.add_rule("path(a, c) = edge(a, b), path(b, c)")

# Run the program
ctx.run()

# Inspect results
for (probability, (start, end)) in ctx.relation("path"):
  print(f"Path {start}→{end}: probability {probability:.3f}")
```

Run it:

``` bash
python edge_path_prob.py
```

Output:

```
Path 0→1: probability 0.100
Path 1→2: probability 0.200
Path 2→3: probability 0.300
Path 0→2: probability 0.200
Path 1→3: probability 0.300
Path 0→3: probability 0.300
```

### Understanding the API

**ScallopContext** is the main interface for Scallop in Python:

- `ScallopContext(provenance="...")` - Create a context with specified provenance
- `ctx.add_relation(name, types)` - Declare a relation's schema
- `ctx.add_facts(relation, [(prob, tuple), ...])` - Add probabilistic facts
- `ctx.add_rule(rule_string)` - Add logical rules
- `ctx.run()` - Execute the program
- `ctx.relation(name)` - Get results as a list of `(probability, tuple)` pairs

### PyTorch Integration

Scallop can integrate directly with PyTorch for differentiable reasoning! Here's a taste:

``` py
import torch
import scallopy

# Create a differentiable module
sum_2 = scallopy.Module(
  provenance="difftopkproofs",  # Differentiable provenance
  program="rel sum_2(a + b) = digit_a(a) and digit_b(b)",
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)

# Use it in a neural network
class MNISTAdder(torch.nn.Module):
  def __init__(self):
    super().__init__()
    self.digit_classifier = torch.nn.Linear(784, 10)  # Neural digit classifier
    self.scallop_reasoner = sum_2  # Symbolic addition

  def forward(self, img1, img2):
    digit1_probs = torch.softmax(self.digit_classifier(img1), dim=-1)
    digit2_probs = torch.softmax(self.digit_classifier(img2), dim=-1)
    sum_probs = self.scallop_reasoner(digit_a=digit1_probs, digit_b=digit2_probs)
    return sum_probs
```

The neural network learns to classify digits, and Scallop handles the logical reasoning (addition) - all with gradient flow for end-to-end training!

---

## Next Steps

Congratulations! You've learned the basics of Scallop. Here's where to go next:

### Learn More Language Features

- **[Language Reference](language/index.md)** - Comprehensive guide to Scallop's syntax
  - [Relations and Facts](language/relation.md) - Data modeling
  - [Writing Rules](language/rules.md) - Logic programming patterns
  - [Recursive Rules](language/recursion.md) - Powerful recursive reasoning
  - [Aggregations](language/aggregation.md) - count, sum, min, max, and more
  - [Algebraic Data Types](language/adt_and_entity.md) - Structured data and pattern matching

### Dive Into Probabilistic Programming

- **[Probabilistic Programming](probabilistic/index.md)** - Master uncertainty reasoning
  - [Provenance](probabilistic/provenance.md) - How probability tracking works
  - [Proofs](probabilistic/proofs.md) - Understanding derivations
  - [Provenance Library](probabilistic/library.md) - All 18 provenance types explained
  - [Logic and Probability](probabilistic/logic.md) - Combining symbolic and probabilistic reasoning

### Python and Machine Learning

- **[Scallopy](scallopy/index.md)** - Python API and PyTorch integration
  - [Getting Started](scallopy/getting_started.md) - Setup and basics
  - [Scallop Context](scallopy/context.md) - The core API
  - [Creating Modules](scallopy/module.md) - PyTorch integration
  - [Configuring Provenance](scallopy/provenance.md) - Probability tracking in Python
  - [Debugging Proofs](scallopy/debug_proofs.md) - Understanding derivations

### Tools and CLI

- **[Scallop CLI](toolchain/scli.md)** - Command-line interpreter
- **[Scallop REPL](toolchain/sclrepl.md)** - Interactive exploration

### Example Programs

Check out the examples directory for more:
- `/examples/datalog/` - Classic logic programming examples
- `/etc/scallopy/examples/` - Python integration examples

### Getting Help

- **Documentation**: You're reading it!
- **GitHub**: [github.com/scallop-lang/scallop](https://github.com/scallop-lang/scallop)
- **Paper**: [PLDI 2023 paper](https://dl.acm.org/doi/10.1145/3591280) on Scallop's foundations

---

## Quick Reference

### Basic Syntax

``` scl
// Facts
rel edge(0, 1)
rel edge = {(0, 1), (1, 2), (2, 3)}

// Probabilistic facts
rel 0.8::reliable_edge(0, 1)
rel color = {0.7::red; 0.3::blue}  // Mutually exclusive

// Rules
rel path(a, b) = edge(a, b)
rel path(a, c) = path(a, b) and edge(b, c)

// Queries
query path
query path(0, x)  // Specific query
```

### CLI Commands

``` bash
scli program.scl                           # Run program
scli --provenance minmaxprob program.scl   # With provenance
scli --provenance topkproofs --k 5 prog.scl  # Top-5 proofs
sclrepl                                    # Start REPL
```

### Python API

``` py
import scallopy

# Context API
ctx = scallopy.ScallopContext(provenance="minmaxprob")
ctx.add_relation("edge", (int, int))
ctx.add_facts("edge", [(0.8, (0, 1))])
ctx.add_rule("path(a, b) = edge(a, b)")
ctx.run()
results = ctx.relation("path")

# Module API (for PyTorch)
module = scallopy.Module(
  provenance="difftopkproofs",
  program="...",
  input_mappings={...},
  output_mapping=(...)
)
output = module(input1=tensor1, input2=tensor2)
```

Happy programming with Scallop!
