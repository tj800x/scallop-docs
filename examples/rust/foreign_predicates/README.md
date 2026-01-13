# Foreign Predicates Example

This example demonstrates implementing custom fact generators using the ForeignPredicate trait.

## What This Example Demonstrates

- Implementing the `ForeignPredicate` trait
- Binding patterns: **bf** (bounded-free) and **ff** (free-free)
- Generating multiple results from a single call
- External data integration (simulated CSV loading)
- Using predicates in Scallop programs

## Foreign Predicates Implemented

### 1. range - Integer Range Generator

```
range(n, i) [bf pattern]
```

Generates integers from 0 to n-1.

**Binding pattern bf:**
- `n` is bounded (input)
- `i` is free (output)

**Usage:** `range(5, i)` generates `{(5, 0), (5, 1), (5, 2), (5, 3), (5, 4)}`

### 2. string_chars - String Character Splitter

```
string_chars(s, c) [bf pattern]
```

Splits a string into individual characters.

**Binding pattern bf:**
- `s` is bounded (input string)
- `c` is free (output character)

**Usage:** `string_chars("hello", c)` generates `{("hello", 'h'), ("hello", 'e'), ...}`

### 3. csv_data - External Data Loader

```
csv_data(name, age, role) [ff pattern]
```

Loads data from simulated CSV (in real use, would read from file).

**Binding pattern ff:**
- All arguments are free (outputs)
- Generates all rows when called

**Usage:** `csv_data(name, age, role)` generates all employee records

## Expected Output

```
=== Foreign Predicates Example ===

Registering foreign predicates:
  - range(n, i) [bf pattern]
  - string_chars(s, c) [bf pattern]
  - csv_data(name, age, role) [ff pattern]

Program loaded
Program executed

Sequences:
  sequence(3, 0)  sequence(3, 1)  sequence(3, 2)
  sequence(5, 0)  sequence(5, 1)  sequence(5, 2)  sequence(5, 3)  sequence(5, 4)
  sequence(7, 0)  sequence(7, 1)  sequence(7, 2)  sequence(7, 3)  sequence(7, 4)  sequence(7, 5)  sequence(7, 6)

Letters:
  "hello" contains 'h'
  "hello" contains 'e'
  "hello" contains 'l'
  "hello" contains 'l'
  "hello" contains 'o'
  "world" contains 'w'
  "world" contains 'o'
  "world" contains 'r'
  "world" contains 'l'
  "world" contains 'd'

Employees (from CSV):
  Alice, age 30, Engineer
  Bob, age 25, Designer
  Charlie, age 35, Manager
  Diana, age 28, Analyst

Senior Employees (age >= 30):
  Alice
  Charlie

=== Example Complete ===
```

## Running This Example

```bash
cargo run
```

## Key Concepts

### The ForeignPredicate Trait

```rust
pub trait ForeignPredicate: DynClone {
    fn name(&self) -> String;
    fn arity(&self) -> usize;
    fn argument_type(&self, i: usize) -> ValueType;
    fn num_bounded(&self) -> usize;
    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)>;
}
```

**Key differences from ForeignFunction:**
- Returns `Vec` of tuples (multiple results)
- Has `num_bounded()` for binding patterns
- Takes only bounded arguments, returns full tuples

### Binding Patterns

**bf pattern (bounded-free):**
```rust
fn num_bounded(&self) -> usize { 1 }  // First arg bounded

// Scallop calls: range(5, i)
// bounded = [Value::I32(5)]
// Returns: [(tag, [Value::I32(5), Value::I32(0)]),
//           (tag, [Value::I32(5), Value::I32(1)]), ...]
```

**ff pattern (free-free):**
```rust
fn num_bounded(&self) -> usize { 0 }  // No bounded args

// Scallop calls: csv_data(name, age, role)
// bounded = []
// Returns: all data rows
```

### Implementing a Simple Predicate

```rust
#[derive(Clone)]
pub struct Range;

impl ForeignPredicate for Range {
    fn name(&self) -> String {
        "range".to_string()
    }

    fn arity(&self) -> usize {
        2  // Total arguments
    }

    fn argument_type(&self, i: usize) -> ValueType {
        ValueType::I32
    }

    fn num_bounded(&self) -> usize {
        1  // First argument is input
    }

    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        if let Value::I32(n) = &bounded[0] {
            // Generate range [0, n)
            (0..*n).map(|i| {
                (
                    DynamicInputTag::None,
                    vec![bounded[0].clone(), Value::I32(i)]
                )
            }).collect()
        } else {
            vec![]
        }
    }
}
```

### Return Value Structure

```rust
Vec<(DynamicInputTag, Vec<Value>)>
     ↑                  ↑
     Tag (prob/count)   Complete tuple (bounded + free)
```

**Important:** Returned tuples must include **all** arguments (bounded + free).

### Using in Scallop Programs

```scl
rel sizes = {3, 5, 7}
rel sequence(n, i) = sizes(n), range(n, i)
```

**Execution:**
- For each `n` in `sizes`
- Call `range(n, i)` with `n` bounded
- Generate results for each `i` in [0, n)

## Implementation Patterns

### Pattern 1: Generator (bf)

```rust
// Generate sequence of values from bounded input
fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
    if let Value::I32(n) = &bounded[0] {
        (0..*n).map(|i| {
            (DynamicInputTag::None, vec![bounded[0].clone(), Value::I32(i)])
        }).collect()
    } else {
        vec![]
    }
}
```

### Pattern 2: Data Source (ff)

```rust
// Load all data when called
fn evaluate(&self, _bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
    self.data.iter().map(|(a, b, c)| {
        (
            DynamicInputTag::None,
            vec![Value::from(a), Value::from(b), Value::from(c)]
        )
    }).collect()
}
```

### Pattern 3: Transformation (bf)

```rust
// Transform bounded input into multiple outputs
fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
    if let Value::String(s) = &bounded[0] {
        s.chars().map(|c| {
            (DynamicInputTag::None, vec![bounded[0].clone(), Value::Char(c)])
        }).collect()
    } else {
        vec![]
    }
}
```

## Real-World Use Cases

### Database Query

```rust
#[derive(Clone)]
pub struct SQLQuery { /* connection */ }

impl ForeignPredicate for SQLQuery {
    fn evaluate(&self, bounded: &[Value]) -> Vec<...> {
        // Execute: SELECT * FROM table WHERE id = bounded[0]
        // Return rows as tuples
    }
}
```

### File Reader

```rust
#[derive(Clone)]
pub struct ReadCSV { path: String }

impl ForeignPredicate for ReadCSV {
    fn evaluate(&self, _bounded: &[Value]) -> Vec<...> {
        // Read CSV file
        // Parse rows
        // Return as tuples
    }
}
```

### API Call

```rust
#[derive(Clone)]
pub struct RestAPI;

impl ForeignPredicate for RestAPI {
    fn evaluate(&self, bounded: &[Value]) -> Vec<...> {
        // HTTP GET request with bounded[0] as endpoint
        // Parse JSON response
        // Return fields as tuples
    }
}
```

## Next Steps

- **[incremental_evaluation](../incremental_evaluation/)** - Dynamic updates
- **[complex_reasoning](../complex_reasoning/)** - Combine predicates with proofs
- **[Foreign Predicates Guide](../../../doc/src/rust_api/foreign_predicates.md)** - Complete API reference

## Related Documentation

- [Foreign Predicates API](../../../doc/src/rust_api/foreign_predicates.md)
- [Foreign Functions API](../../../doc/src/rust_api/foreign_functions.md)
- [Getting Started Guide](../../../doc/src/rust_api/getting_started.md)
