# Foreign Predicates

Foreign predicates are **fact generators** that extend Scallop by dynamically producing facts based on input patterns. Unlike foreign functions that return single values, foreign predicates can yield multiple results and support flexible bound/free variable patterns.

## What are Foreign Predicates?

### Definition

A foreign predicate is a **Python generator function** that:
- Takes input arguments (bound variables)
- Yields zero or more facts (bound + free variables)
- Can produce probabilistic facts with tags
- Supports pattern-based querying

### Syntax

Foreign predicates are called like regular Scallop predicates:

```scl
rel result(input, output) = data(input), my_predicate(input, output)
rel check = data(x) and my_predicate(x)  // Boolean predicate
```

### Foreign Predicates vs Foreign Functions

| Feature | Foreign Function | Foreign Predicate |
|---------|------------------|-------------------|
| **Invocation** | `$function(args)` | `predicate(args)` |
| **Returns** | Single value | Zero or more facts |
| **Pattern** | All inputs bound | Supports bound/free patterns |
| **Use case** | Pure computation | Fact generation, search |
| **Tag** | Inherits from context | Per-fact probability |

### Key Characteristics

**Multi-valued:**
- Can yield multiple results for a single input
- Empty results are valid (no facts generated)
- Each result is independent

**Pattern-driven:**
- Supports multiple calling patterns (bb, bf, fb, ff)
- "b" = bound (input provided), "f" = free (output to generate)
- Example: `gpt(input, output)` supports `bb` (check) and `bf` (generate)

**Probabilistic:**
- Each yielded fact has a probability/tag
- Tags integrate with Scallop's provenance system
- Enables fuzzy matching and uncertain reasoning

## Using Foreign Predicates

### Basic Usage

Foreign predicates become available after plugin loading:

```python
import scallopy

# Create context with provenance
ctx = scallopy.ScallopContext(provenance="minmaxprob")

# Load plugins (e.g., GPT plugin)
plugin_registry = scallopy.PluginRegistry()
plugin_registry.load_plugins_from_entry_points()
plugin_registry.load_into_ctx(ctx)

# Now use foreign predicates from plugins
ctx.add_program("""
  rel question = {"What is the capital of France?"}
  rel answer(q, a) = question(q), gpt(q, a)
  query answer
""")
ctx.run()
```

### Calling Patterns

**Bound-Free (bf) - Generation:**
```scl
// Input is bound, output is free - generate answers
rel questions = {"What is 2+2?", "What is the capital of Spain?"}
rel answers(q, a) = questions(q), gpt(q, a)
```

**Bound-Bound (bb) - Verification:**
```scl
// Both bound - check if answer is correct
rel question_answer_pairs = {
  ("What is 2+2?", "4"),
  ("What is 2+2?", "5")
}
rel verified(q, a) = question_answer_pairs(q, a), gpt(q, a)
// Only ("What is 2+2?", "4") passes verification
```

**Multiple results:**
```scl
// Foreign predicate can yield multiple answers
rel query = {"What are some French cities?"}
rel cities(c) = query(q), list_cities(q, c)
// Yields: Paris, Lyon, Marseille, Nice, ...
```

### Integration with Rules

**Chaining predicates:**
```scl
rel document = {"The quick brown fox jumps over the lazy dog"}
rel extracted_entity(doc, entity) = document(doc), extract_entities(doc, entity)
rel classified(entity, type) = extracted_entity(_, entity), classify_entity(entity, type)
```

**Filtering results:**
```scl
rel candidates(x) = source(s), generate_options(s, x)
rel filtered(x) = candidates(x), validate(x)  // Boolean predicate
```

**Aggregation:**
```scl
rel all_answers = {a | question(q), gpt(q, a)}
rel num_unique_answers(n) = n = count(a: all_answers(a))
```

## Examples from Plugins

### GPT Plugin: Text Generation

The GPT plugin provides a foreign predicate that queries the OpenAI API:

**Implementation:**
```python
@scallopy.foreign_predicate
def gpt(s: str) -> scallopy.Facts[None, str]:
    # Check memoization cache
    if s in STORAGE:
        response = STORAGE[s]
    else:
        # Call OpenAI API
        response = openai.ChatCompletion.create(
            model="gpt-3.5-turbo",
            messages=[{"role": "user", "content": s}],
            temperature=0.0,
        )
        STORAGE[s] = response

    # Iterate through all choices (typically 1)
    for choice in response["choices"]:
        result = choice["message"]["content"].strip()
        yield (result,)
```

**Usage:**
```scl
rel questions = {
  "What is the capital of France?",
  "Translate to Spanish: Good morning",
  "List three colors"
}

rel qa(question, answer) = questions(question), gpt(question, answer)
query qa

// Expected output (mock when API key not set):
// qa: {
//   ("What is the capital of France?", "Paris"),
//   ("Translate to Spanish: Good morning", "Buenos días"),
//   ("List three colors", "Red, blue, green")
// }
```

**Multiple outputs:**
```scl
// Configure GPT to return multiple answers
rel question = {"What are some programming languages?"}
rel languages(q, lang) = question(q), gpt_multi(q, lang)

// With n=3 completions, yields multiple facts:
// languages: {
//   ("What are some programming languages?", "Python, Java, C++"),
//   ("What are some programming languages?", "JavaScript, Ruby, Go"),
//   ("What are some programming languages?", "Rust, Swift, Kotlin")
// }
```

### Custom Example: Semantic Similarity

```python
@scallopy.foreign_predicate
def string_semantic_eq(s1: str, s2: str) -> scallopy.Facts[float, Tuple]:
    """Fuzzy string matching for kinship terms"""
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
```

**Usage:**
```scl
rel kinship = {
  ("alice", "mom", "bob"),
  ("alice", "mother", "casey"),
  ("david", "father", "emma")
}

rel parent(person, child) =
  kinship(person, relation, child),
  string_semantic_eq(relation, "mother")

rel parent(person, child) =
  kinship(person, relation, child),
  string_semantic_eq(relation, "father")

query parent

// Result (with probabilities):
// parent: {
//   0.99::("alice", "bob"),      // "mom" ~= "mother" with prob 0.99
//   1.0::("alice", "casey"),     // "mother" == "mother" exactly
//   1.0::("david", "emma")       // "father" == "father" exactly
// }
```

### Custom Example: Divisor Generation

```python
@scallopy.foreign_predicate
def divisors(n: int) -> scallopy.Facts[float, Tuple[int]]:
    """Generate all divisors of n"""
    for i in range(1, n + 1):
        if n % i == 0:
            yield (1.0, (i,))
```

**Usage:**
```scl
rel numbers = {12, 15, 20}
rel has_divisor(num, div) = numbers(num), divisors(num, div)
query has_divisor

// Result:
// has_divisor: {
//   (12, 1), (12, 2), (12, 3), (12, 4), (12, 6), (12, 12),
//   (15, 1), (15, 3), (15, 5), (15, 15),
//   (20, 1), (20, 2), (20, 4), (20, 5), (20, 10), (20, 20)
// }
```

## Creating Foreign Predicates in Plugins

### Basic Foreign Predicate

To create a foreign predicate in a plugin:

```python
import scallopy
from scallopy import Facts
from typing import Tuple

class MyPlugin(scallopy.Plugin):
    def __init__(self):
        super().__init__("my_plugin")

    def load_into_ctx(self, ctx):
        @scallopy.foreign_predicate
        def my_pred(x: int) -> Facts[float, Tuple[int]]:
            # Generate facts
            yield (1.0, (x * 2,))
            yield (1.0, (x * 3,))

        ctx.register_foreign_predicate(my_pred)
```

### Type Signature

The return type must be `Facts[TagType, TupleType]`:

```python
from scallopy import Facts
from typing import Tuple

# Single output column
def single_output(x: int) -> Facts[float, Tuple[int]]:
    yield (1.0, (x * 2,))

# Multiple output columns
def multi_output(x: int) -> Facts[float, Tuple[int, str]]:
    yield (1.0, (x, "even" if x % 2 == 0 else "odd"))

# Boolean predicate (empty tuple)
def is_prime(n: int) -> Facts[float, Tuple]:
    if check_prime(n):
        yield (1.0, ())
```

### Yielding Facts

Use `yield` to produce facts lazily:

```python
@scallopy.foreign_predicate
def range_values(start: int, end: int) -> Facts[float, Tuple[int]]:
    for i in range(start, end + 1):
        yield (1.0, (i,))

# Generates facts: 1, 2, 3, 4, 5 for range_values(1, 5)
```

### Probabilistic Facts

Tag values integrate with provenance:

```python
@scallopy.foreign_predicate
def fuzzy_match(s1: str, s2: str) -> Facts[float, Tuple]:
    # Calculate similarity score
    similarity = compute_similarity(s1, s2)

    # Only yield if similarity is high enough
    if similarity > 0.5:
        yield (similarity, ())

# Usage:
# rel similar(a, b) = strings(a), strings(b), fuzzy_match(a, b)
```

### Error Handling

Handle errors gracefully by not yielding:

```python
@scallopy.foreign_predicate
def safe_operation(x: int) -> Facts[float, Tuple[int]]:
    try:
        result = risky_computation(x)
        yield (1.0, (result,))
    except Exception as e:
        # Don't yield - fact doesn't exist for this input
        pass
```

### Memoization

Cache expensive computations:

```python
CACHE = {}

@scallopy.foreign_predicate
def expensive_pred(x: str) -> Facts[float, Tuple[str]]:
    if x not in CACHE:
        CACHE[x] = expensive_api_call(x)

    for result in CACHE[x]:
        yield (1.0, (result,))
```

## Best Practices

### Use Predicates for Multi-Valued Results

**✓ Good - Multiple results with predicate:**
```python
@foreign_predicate
def get_synonyms(word: str) -> Facts[float, Tuple[str]]:
    for synonym in lookup_synonyms(word):
        yield (1.0, (synonym,))
```

**✗ Bad - Single result with function:**
```python
@foreign_function
def get_synonyms(word: str) -> str:
    return ", ".join(lookup_synonyms(word))  # Returns string, loses structure
```

### Pattern Validation

Document and validate expected patterns:

```python
@scallopy.foreign_predicate
def my_pred(x: int, y: int) -> Facts[float, Tuple[int]]:
    """
    Supports patterns:
    - (bound, bound) → (free): Given x and y, compute result
    - (bound, free) → not supported (would require search)
    """
    result = x + y
    yield (1.0, (result,))
```

### Lazy Evaluation

Take advantage of generator laziness:

```python
@scallopy.foreign_predicate
def infinite_sequence(start: int) -> Facts[float, Tuple[int]]:
    # This won't run forever - Scallop only pulls what it needs
    i = start
    while True:
        yield (1.0, (i,))
        i += 1
```

## Next Steps

- **[Foreign Functions](foreign_functions.md)** - Single-valued computations
- **[Foreign Attributes](foreign_attributes.md)** - Metaprogramming decorators
- **[GPT Plugin](openai_gpt.md)** - Complete LLM integration example
- **[Create Your Own Plugin](create_your_own_plugin.md)** - Build custom plugins

For detailed Python API documentation, see [Foreign Predicates (Python API)](../scallopy/foreign_predicate.md).
