# Foreign Predicates

This guide covers implementing **foreign predicates** in Rust to extend Scallop with custom fact generators.

## Overview

Foreign predicates are **non-deterministic relations** that generate facts dynamically at runtime. Unlike foreign functions (which are pure and deterministic), foreign predicates can:

- **Yield multiple results** for a single input
- **Generate facts from external sources** (databases, files, APIs)
- **Support different input/output modes** via binding patterns
- **Tag results with probabilities** for provenance tracking

**Comparison to Foreign Functions:**

| Feature | Foreign Functions | Foreign Predicates |
|---------|-------------------|-------------------|
| Determinism | Pure, deterministic | Non-deterministic |
| Results | Single value | Multiple tuples |
| Use case | Computation | Fact generation |
| Example | `$string_length(s)` | `range(n, i)` |

**Comparison to Python API:**

The Rust `ForeignPredicate` trait corresponds to Python's `@foreign_predicate` decorator:

```python
# Python
@foreign_predicate(name="range", output_arg_types=[int])
def range_pred(n: int) -> Facts[float, Tuple[int, int]]:
    for i in range(n):
        yield (1.0, (n, i))
```

```rust
// Rust equivalent (shown later in this guide)
impl ForeignPredicate for Range {
    fn name(&self) -> String { "range".to_string() }
    fn arity(&self) -> usize { 2 }
    fn num_bounded(&self) -> usize { 1 }
    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        // Implementation
    }
}
```

---

## The ForeignPredicate Trait

### Trait Definition

```rust
pub trait ForeignPredicate: DynClone {
    /// Name of the predicate
    fn name(&self) -> String;

    /// Total number of arguments
    fn arity(&self) -> usize;

    /// Type of the i-th argument
    fn argument_type(&self, i: usize) -> ValueType;

    /// Number of bounded (input) arguments
    fn num_bounded(&self) -> usize;

    /// Number of free (output) arguments (computed)
    fn num_free(&self) -> usize {
        self.arity() - self.num_bounded()
    }

    /// Evaluate predicate with bounded arguments, yield free arguments
    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)>;

    /// Optional: evaluate with all arguments provided (for validation)
    fn evaluate_with_all_arguments(&self, args: &[Value]) -> Vec<DynamicInputTag> {
        vec![]  // Default: no validation
    }
}
```

**Key Points:**

- **`name()`** - Predicate name used in Scallop programs
- **`arity()`** - Total number of arguments (bounded + free)
- **`argument_type(i)`** - ValueType for each argument position
- **`num_bounded()`** - How many arguments are inputs
- **`evaluate(bounded)`** - Core method that generates results

### Return Type: Tagged Tuples

The `evaluate()` method returns:

```rust
Vec<(DynamicInputTag, Vec<Value>)>
```

**Structure:**
- **Outer Vec** - Multiple results (non-deterministic)
- **DynamicInputTag** - Probability or ID for provenance tracking
- **Vec\<Value\>** - Complete tuple (bounded + free arguments)

**Example:**
```rust
vec![
    (DynamicInputTag::None, vec![Value::I32(5), Value::I32(0)]),
    (DynamicInputTag::None, vec![Value::I32(5), Value::I32(1)]),
    (DynamicInputTag::None, vec![Value::I32(5), Value::I32(2)]),
]
// Three results from range(5, i): (5, 0), (5, 1), (5, 2)
```

---

## Binding Patterns

Foreign predicates support **different input/output modes** based on which arguments are bounded (input) vs free (output).

### Binding Pattern Notation

| Pattern | Meaning | Example Call | Description |
|---------|---------|--------------|-------------|
| `bb` | Both bounded | `pred(5, 10)` | Both arguments provided |
| `bf` | First bounded, second free | `pred(5, x)` | First is input, second is output |
| `fb` | First free, second bounded | `pred(x, 10)` | First is output, second is input |
| `ff` | Both free | `pred(x, y)` | Generate all pairs |

**In Scallop programs:**
```scl
// Binding pattern bf: n is bounded, i is free
rel result(n, i) = n in {5, 10}, range(n, i)
// Calls: range(5, i) and range(10, i)

// Binding pattern bb: both bounded (for validation)
rel check = range(5, 3)
// Calls: range(5, 3) - checks if (5, 3) is valid
```

### How Scallop Determines Binding Patterns

Scallop analyzes the query to determine which arguments are **bounded** (known values) vs **free** (variables):

```rust
// In foreign predicate implementation:
fn num_bounded(&self) -> usize { 1 }  // First argument is bounded

// Scallop automatically determines:
// - Call with n=5: bounded = [Value::I32(5)]
// - Predicate returns: [(tag, [Value::I32(5), Value::I32(0)]), ...]
```

**Important:** The `bounded` slice in `evaluate()` contains **only the bounded arguments**, but the returned tuple must contain **all arguments** (bounded + free).

---

## Implementing Simple Predicates

### Example 1: Range Generator (Pattern: bf)

Generates integers from 0 to n-1 for a given n.

```rust
use scallop_core::common::foreign_predicate::*;
use scallop_core::common::value::*;
use scallop_core::common::input_tag::DynamicInputTag;

#[derive(Clone)]
pub struct Range;

impl ForeignPredicate for Range {
    fn name(&self) -> String {
        "range".to_string()
    }

    fn arity(&self) -> usize {
        2  // (n, i)
    }

    fn argument_type(&self, i: usize) -> ValueType {
        ValueType::I32  // Both arguments are i32
    }

    fn num_bounded(&self) -> usize {
        1  // First argument (n) is bounded
    }

    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        // Extract bounded argument
        if let Value::I32(n) = &bounded[0] {
            // Generate range [0, n)
            (0..*n).map(|i| {
                (
                    DynamicInputTag::None,
                    vec![Value::I32(*n), Value::I32(i)]  // Full tuple: (n, i)
                )
            }).collect()
        } else {
            vec![]  // Type mismatch
        }
    }
}
```

**Usage in Scallop:**
```scl
rel numbers(n, i) = n in {5, 10}, range(n, i)
query numbers

// Results:
// (5, 0), (5, 1), (5, 2), (5, 3), (5, 4)
// (10, 0), (10, 1), ..., (10, 9)
```

**Register with IntegrateContext:**
```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;

let prov = UnitProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

ctx.register_foreign_predicate(Range);

ctx.add_program(r#"
    rel numbers(n, i) = n in {5, 10}, range(n, i)
    query numbers
"#).unwrap();

ctx.run().unwrap();

let numbers = ctx.computed_relation_ref("numbers").unwrap();
for elem in numbers.iter() {
    println!("{:?}", elem.tuple);
}
```

### Example 2: String Splitter (Pattern: bf, Multiple Results)

Splits a string into individual characters.

```rust
#[derive(Clone)]
pub struct StringChars;

impl ForeignPredicate for StringChars {
    fn name(&self) -> String {
        "string_chars".to_string()
    }

    fn arity(&self) -> usize {
        2  // (string, char)
    }

    fn argument_type(&self, i: usize) -> ValueType {
        if i == 0 {
            ValueType::String
        } else {
            ValueType::Char
        }
    }

    fn num_bounded(&self) -> usize {
        1  // First argument (string) is bounded
    }

    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        if let Value::String(s) = &bounded[0] {
            s.chars().map(|c| {
                (
                    DynamicInputTag::None,
                    vec![bounded[0].clone(), Value::Char(c)]
                )
            }).collect()
        } else {
            vec![]
        }
    }
}
```

**Usage:**
```scl
rel word = {"hello", "world"}
rel letters(w, c) = word(w), string_chars(w, c)
query letters

// Results:
// ("hello", 'h'), ("hello", 'e'), ("hello", 'l'), ("hello", 'l'), ("hello", 'o')
// ("world", 'w'), ("world", 'o'), ("world", 'r'), ("world", 'l'), ("world", 'd')
```

---

## Multiple Binding Patterns

Some predicates support **different binding patterns** for bidirectional lookup.

### Example: Key-Value Store (Patterns: bf, fb, ff)

```rust
use std::collections::HashMap;

#[derive(Clone)]
pub struct Lookup {
    data: HashMap<String, String>,
}

impl Lookup {
    pub fn new() -> Self {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());
        data.insert("age".to_string(), "30".to_string());
        data.insert("city".to_string(), "NYC".to_string());
        Self { data }
    }
}

impl ForeignPredicate for Lookup {
    fn name(&self) -> String {
        "lookup".to_string()
    }

    fn arity(&self) -> usize {
        2  // (key, value)
    }

    fn argument_type(&self, _: usize) -> ValueType {
        ValueType::String
    }

    fn num_bounded(&self) -> usize {
        1  // Can be either first or second argument
    }

    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        // Note: This simplified example only handles bf pattern
        // For multiple patterns, you'd need to track which argument is bounded

        if let Value::String(key) = &bounded[0] {
            // Pattern bf: key → value
            if let Some(value) = self.data.get(key) {
                vec![(
                    DynamicInputTag::None,
                    vec![bounded[0].clone(), Value::String(value.clone())]
                )]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}
```

**Note:** Full multi-pattern support requires tracking which arguments are bounded. In practice, you might implement separate predicates for different patterns or use Scallop's built-in pattern matching.

**Usage:**
```scl
rel keys = {"name", "age"}
rel values(k, v) = keys(k), lookup(k, v)
query values

// Results:
// ("name", "Alice")
// ("age", "30")
```

---

## Tagging Facts with Probabilities

Foreign predicates can **tag results with probabilities** for provenance tracking.

### Using DynamicInputTag Variants

```rust
pub enum DynamicInputTag {
    None,                              // No tag (unit provenance)
    Bool(bool),                        // Boolean tag
    Natural(usize),                    // Natural number tag
    Float(f64),                        // Probability tag
    // ... other variants
}
```

### Example: Probabilistic Results

```rust
#[derive(Clone)]
pub struct WeatherForecast;

impl ForeignPredicate for WeatherForecast {
    fn name(&self) -> String {
        "forecast".to_string()
    }

    fn arity(&self) -> usize {
        2  // (city, weather)
    }

    fn argument_type(&self, _: usize) -> ValueType {
        ValueType::String
    }

    fn num_bounded(&self) -> usize {
        1  // City is bounded
    }

    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        if let Value::String(city) = &bounded[0] {
            match city.as_str() {
                "NYC" => vec![
                    (DynamicInputTag::Float(0.7), vec![bounded[0].clone(), Value::String("sunny".into())]),
                    (DynamicInputTag::Float(0.2), vec![bounded[0].clone(), Value::String("rainy".into())]),
                    (DynamicInputTag::Float(0.1), vec![bounded[0].clone(), Value::String("cloudy".into())]),
                ],
                "LA" => vec![
                    (DynamicInputTag::Float(0.9), vec![bounded[0].clone(), Value::String("sunny".into())]),
                    (DynamicInputTag::Float(0.1), vec![bounded[0].clone(), Value::String("cloudy".into())]),
                ],
                _ => vec![]
            }
        } else {
            vec![]
        }
    }
}
```

**Usage with Probabilistic Provenance:**
```rust
use scallop_core::runtime::provenance::min_max_prob::MinMaxProbProvenance;

let prov = MinMaxProbProvenance::default();
let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

ctx.register_foreign_predicate(WeatherForecast);

ctx.add_program(r#"
    rel city = {"NYC", "LA"}
    rel weather(c, w) = city(c), forecast(c, w)
    query weather
"#).unwrap();

ctx.run().unwrap();

let weather = ctx.computed_relation_ref("weather").unwrap();
for elem in weather.iter() {
    println!("Probability: {}, Tuple: {:?}", elem.tag, elem.tuple);
}
```

**Output:**
```
Probability: 0.7, Tuple: ("NYC", "sunny")
Probability: 0.2, Tuple: ("NYC", "rainy")
Probability: 0.1, Tuple: ("NYC", "cloudy")
Probability: 0.9, Tuple: ("LA", "sunny")
Probability: 0.1, Tuple: ("LA", "cloudy")
```

---

## Complete Working Example

Here's a full program demonstrating foreign predicates with file I/O.

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;
use scallop_core::common::foreign_predicate::*;
use scallop_core::common::value::*;
use scallop_core::common::input_tag::DynamicInputTag;

// Foreign predicate: read CSV file
#[derive(Clone)]
pub struct ReadCSV {
    data: Vec<(String, i32, String)>,
}

impl ReadCSV {
    pub fn new() -> Self {
        // Simulated CSV data: (name, age, city)
        Self {
            data: vec![
                ("Alice".into(), 30, "NYC".into()),
                ("Bob".into(), 25, "LA".into()),
                ("Charlie".into(), 35, "Chicago".into()),
            ]
        }
    }
}

impl ForeignPredicate for ReadCSV {
    fn name(&self) -> String {
        "read_csv".to_string()
    }

    fn arity(&self) -> usize {
        3  // (name, age, city)
    }

    fn argument_type(&self, i: usize) -> ValueType {
        match i {
            0 => ValueType::String,  // name
            1 => ValueType::I32,     // age
            2 => ValueType::String,  // city
            _ => panic!("Invalid argument index"),
        }
    }

    fn num_bounded(&self) -> usize {
        0  // All free (ff pattern)
    }

    fn evaluate(&self, _bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        self.data.iter().map(|(name, age, city)| {
            (
                DynamicInputTag::None,
                vec![
                    Value::String(name.clone()),
                    Value::I32(*age),
                    Value::String(city.clone()),
                ]
            )
        }).collect()
    }
}

fn main() -> Result<(), IntegrateError> {
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Register foreign predicates
    ctx.register_foreign_predicate(Range);
    ctx.register_foreign_predicate(ReadCSV::new());

    ctx.add_program(r#"
        // Load data from CSV
        rel person(name, age, city) = read_csv(name, age, city)

        // Find adults
        rel adult(name) = person(name, age, city) and age >= 30

        // Generate ID range
        rel ids(n, id) = n in {3}, range(n, id)

        query person
        query adult
        query ids
    "#)?;

    ctx.run()?;

    // Display results
    println!("People:");
    let person = ctx.computed_relation_ref("person")?;
    for elem in person.iter() {
        println!("  {:?}", elem.tuple);
    }

    println!("\nAdults:");
    let adult = ctx.computed_relation_ref("adult")?;
    for elem in adult.iter() {
        println!("  {:?}", elem.tuple);
    }

    println!("\nIDs:");
    let ids = ctx.computed_relation_ref("ids")?;
    for elem in ids.iter() {
        println!("  {:?}", elem.tuple);
    }

    Ok(())
}
```

**Expected Output:**
```
People:
  ("Alice", 30, "NYC")
  ("Bob", 25, "LA")
  ("Charlie", 35, "Chicago")

Adults:
  ("Alice")
  ("Charlie")

IDs:
  (3, 0)
  (3, 1)
  (3, 2)
```

---

## Best Practices

### 1. Type Safety

Always validate argument types before processing:

```rust
fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
    // Good: Type check
    if let Value::I32(n) = &bounded[0] {
        // Process
    } else {
        return vec![];  // Type mismatch
    }
}
```

### 2. Return Complete Tuples

The returned tuples must include **all arguments** (bounded + free):

```rust
// Predicate: range(n, i) with arity=2, num_bounded=1
fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
    if let Value::I32(n) = &bounded[0] {
        (0..*n).map(|i| {
            (
                DynamicInputTag::None,
                vec![
                    bounded[0].clone(),  // Include bounded argument (n)
                    Value::I32(i)        // Add free argument (i)
                ]
            )
        }).collect()
    } else {
        vec![]
    }
}
```

### 3. Use Appropriate Tags

Match tag type to provenance:

```rust
// For UnitProvenance
DynamicInputTag::None

// For probabilistic provenance
DynamicInputTag::Float(0.8)

// For counting provenance
DynamicInputTag::Natural(5)
```

### 4. Handle Empty Results

Return empty vec for invalid inputs or no results:

```rust
fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
    if let Value::String(key) = &bounded[0] {
        if let Some(value) = self.lookup(key) {
            vec![/* result */]
        } else {
            vec![]  // Key not found
        }
    } else {
        vec![]  // Type mismatch
    }
}
```

### 5. Clone Bounded Arguments

When including bounded arguments in results, clone them:

```rust
vec![
    bounded[0].clone(),  // Clone bounded argument
    Value::String(result)  // Add free argument
]
```

---

## Common Patterns

### Pattern 1: Database Query

```rust
#[derive(Clone)]
pub struct SQLQuery {
    // Connection pool, etc.
}

impl ForeignPredicate for SQLQuery {
    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        if let Value::String(table) = &bounded[0] {
            // Execute: SELECT * FROM table
            // Return rows as tuples
        }
        vec![]
    }
}
```

### Pattern 2: File Reader

```rust
#[derive(Clone)]
pub struct ReadLines {
    path: String,
}

impl ForeignPredicate for ReadLines {
    fn evaluate(&self, _bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        std::fs::read_to_string(&self.path)
            .ok()
            .map(|content| {
                content.lines().enumerate().map(|(i, line)| {
                    (
                        DynamicInputTag::None,
                        vec![Value::USize(i), Value::String(line.to_string())]
                    )
                }).collect()
            })
            .unwrap_or_else(Vec::new)
    }
}
```

### Pattern 3: API Call

```rust
#[derive(Clone)]
pub struct RestAPI;

impl ForeignPredicate for RestAPI {
    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        if let Value::String(endpoint) = &bounded[0] {
            // HTTP GET request
            // Parse JSON response
            // Return fields as tuples
        }
        vec![]
    }
}
```

---

## Next Steps

- **[Provenance Types](provenance.md)** - Deep dive into reasoning semantics and tagging
- **[Rust Examples](../examples/rust/)** - Complete working examples
- **[IntegrateContext API](integrate_context.md)** - Registering and using predicates

## Resources

- **Trait Definition:** `scallop-core/src/common/foreign_predicate.rs`
- **Test Examples:** `scallop-core/tests/integrate/adt.rs`
- **Python API Comparison:** [Foreign Predicates (Python)](../scallopy/foreign_predicate.md)
