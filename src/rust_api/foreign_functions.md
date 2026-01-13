# Foreign Functions

## Overview

Foreign functions extend Scallop with pure, deterministic computations implemented in Rust. They allow you to call Rust code from within Scallop programs, enabling operations that Scallop cannot express natively—such as string manipulation, mathematical operations, or domain-specific calculations.

**Key characteristics:**
- **Pure functions** - No side effects; same input always produces same output
- **Deterministic** - Single output value for any given input
- **Type-safe** - Static type checking enforced at compile time
- **Partial functions** - Can return `None` to indicate failure

### Use Cases

- String operations (length, concatenation, formatting)
- Mathematical functions (abs, max, min, trigonometry)
- Type conversions (string to int, etc.)
- Domain-specific operations (hashing, encoding, etc.)

### Comparison to Python

| Python `@foreign_function` | Rust `ForeignFunction` trait |
|----------------------------|------------------------------|
| Simple decorator | Trait implementation |
| Type annotations optional | Type system required |
| Runtime type checking | Compile-time type checking |
| Return value or None | `Option<Value>` return type |

**Example usage in Scallop:**
```scl
// After registering a foreign function
rel lengths(s, len) = strings(s), len = $string_length(s)
rel max_val(a, b, m) = numbers(a, b), m = $max(a, b)
```

## The ForeignFunction Trait

The `ForeignFunction` trait defines the interface for all foreign functions:

```rust
use scallop_core::common::foreign_function::*;
use scallop_core::common::value::Value;

pub trait ForeignFunction: DynClone {
    // Required methods
    fn name(&self) -> String;
    fn return_type(&self) -> ForeignFunctionParameterType;
    fn execute(&self, args: Vec<Value>) -> Option<Value>;

    // Optional methods (with defaults)
    fn num_generic_types(&self) -> usize { 0 }
    fn generic_type_family(&self, i: usize) -> TypeFamily { ... }
    fn num_static_arguments(&self) -> usize { 0 }
    fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType { ... }
    fn num_optional_arguments(&self) -> usize { 0 }
    fn optional_argument_type(&self, i: usize) -> ForeignFunctionParameterType { ... }
    fn has_variable_arguments(&self) -> bool { false }
    fn variable_argument_type(&self) -> ForeignFunctionParameterType { ... }
}
```

### Required Methods

**`name(&self) -> String`**
Returns the function name as it appears in Scallop programs (without the `$` prefix):

```rust
fn name(&self) -> String {
    "string_length".to_string()
}
```

Used in Scallop as: `$string_length(s)`

**`return_type(&self) -> ForeignFunctionParameterType`**
Specifies the return value type:

```rust
fn return_type(&self) -> ForeignFunctionParameterType {
    ForeignFunctionParameterType::BaseType(ValueType::USize)
}
```

**`execute(&self, args: Vec<Value>) -> Option<Value>`**
The actual computation logic:

```rust
fn execute(&self, args: Vec<Value>) -> Option<Value> {
    if let Value::String(s) = &args[0] {
        Some(Value::USize(s.len()))
    } else {
        None
    }
}
```

- **Input:** Vector of `Value` objects (arguments)
- **Output:** `Some(Value)` on success, `None` on error

### Optional Methods (Type System)

**Static arguments** (required parameters):
```rust
fn num_static_arguments(&self) -> usize { 2 }  // e.g., $max(a, b)

fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
    match i {
        0 => ForeignFunctionParameterType::Generic(0),
        1 => ForeignFunctionParameterType::Generic(0),
        _ => panic!("Invalid argument index"),
    }
}
```

**Optional arguments:**
```rust
fn num_optional_arguments(&self) -> usize { 1 }  // e.g., $substring(s, start, end?)

fn optional_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
    assert_eq!(i, 0);
    ForeignFunctionParameterType::BaseType(ValueType::USize)
}
```

**Variable arguments:**
```rust
fn has_variable_arguments(&self) -> bool { true }  // e.g., $concat(strs...)

fn variable_argument_type(&self) -> ForeignFunctionParameterType {
    ForeignFunctionParameterType::BaseType(ValueType::String)
}
```

## Parameter Type System

The `ForeignFunctionParameterType` enum describes argument and return types:

```rust
pub enum ForeignFunctionParameterType {
    /// A generic type parameter (e.g., T0, T1)
    Generic(usize),

    /// A type family (Integer, Float, String, etc.)
    TypeFamily(TypeFamily),

    /// A concrete base type (i32, String, f64, etc.)
    BaseType(ValueType),
}
```

### BaseType - Concrete Types

Use `BaseType` for specific, fixed types:

```rust
use scallop_core::common::value_type::ValueType;

// i32 type
ForeignFunctionParameterType::BaseType(ValueType::I32)

// String type
ForeignFunctionParameterType::BaseType(ValueType::String)

// f64 type
ForeignFunctionParameterType::BaseType(ValueType::F64)
```

**Available ValueTypes:**
- Integers: `I8`, `I16`, `I32`, `I64`, `I128`, `ISize`
- Unsigned: `U8`, `U16`, `U32`, `U64`, `U128`, `USize`
- Floats: `F32`, `F64`
- Others: `Bool`, `Char`, `String`, `Symbol`

### TypeFamily - Type Groups

Use `TypeFamily` when a function works with multiple related types:

```rust
use scallop_core::common::type_family::TypeFamily;

// Works with any integer type
ForeignFunctionParameterType::TypeFamily(TypeFamily::Integer)

// Works with any numeric type (integers + floats)
ForeignFunctionParameterType::TypeFamily(TypeFamily::Number)

// Works with any type
ForeignFunctionParameterType::TypeFamily(TypeFamily::Any)
```

**Available TypeFamilies:**
- `TypeFamily::Integer` - All integer types (signed and unsigned)
- `TypeFamily::SignedInteger` - Only signed integers
- `TypeFamily::UnsignedInteger` - Only unsigned integers
- `TypeFamily::Float` - F32 and F64
- `TypeFamily::Number` - All numeric types
- `TypeFamily::String` - String and Symbol
- `TypeFamily::Any` - Any type

### Generic - Parameterized Types

Use `Generic(id)` for type parameters that maintain consistency across arguments:

```rust
// Function signature: $max<T: Number>(a: T, b: T) -> T
fn num_generic_types(&self) -> usize { 1 }  // One type parameter T

fn generic_type_family(&self, i: usize) -> TypeFamily {
    assert_eq!(i, 0);
    TypeFamily::Number  // T must be a Number
}

fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
    ForeignFunctionParameterType::Generic(0)  // Both args use T
}

fn return_type(&self) -> ForeignFunctionParameterType {
    ForeignFunctionParameterType::Generic(0)  // Return type is also T
}
```

**Type parameter rules:**
- Generic IDs start at 0
- Return type can only be `Generic` or `BaseType` (not `TypeFamily`)
- All generic types must be used in arguments

## Implementing Simple Functions

### Step-by-Step: String Length Function

Let's implement `$string_length(String) -> usize`:

**Step 1: Create the struct**
```rust
#[derive(Clone)]
pub struct StringLength;
```

**Step 2: Implement the trait**
```rust
use scallop_core::common::foreign_function::*;
use scallop_core::common::value::Value;
use scallop_core::common::value_type::ValueType;

impl ForeignFunction for StringLength {
    fn name(&self) -> String {
        "string_length".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        1  // Takes one argument
    }

    fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
        assert_eq!(i, 0);
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::USize)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        if let Value::String(s) = &args[0] {
            Some(Value::USize(s.len()))
        } else {
            None  // Type mismatch (shouldn't happen if types are correct)
        }
    }
}
```

**Step 3: Register with context**
```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;

fn main() -> Result<(), IntegrateError> {
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Register the function
    ctx.register_foreign_function(StringLength)?;

    // Use it in Scallop
    ctx.add_program(r#"
        rel words = {"hello", "world", "rust"}
        rel lengths(w, len) = words(w), len = $string_length(w)
        query lengths
    "#)?;

    ctx.run()?;

    // Print results
    let results = ctx.computed_relation_ref("lengths").unwrap();
    for elem in results.iter() {
        println!("{:?}", elem.tuple);
    }

    Ok(())
}
```

**Output:**
```
("hello", 5)
("world", 5)
("rust", 4)
```

### Example: Integer Addition

A simple function that adds two integers:

```rust
#[derive(Clone)]
pub struct Add;

impl ForeignFunction for Add {
    fn name(&self) -> String {
        "add".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        2
    }

    fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::I32)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::I32)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        if let (Value::I32(a), Value::I32(b)) = (&args[0], &args[1]) {
            Some(Value::I32(a + b))
        } else {
            None
        }
    }
}
```

**Usage:**
```scl
rel numbers = {(1, 2), (3, 4), (5, 6)}
rel sums(a, b, sum) = numbers(a, b), sum = $add(a, b)
query sums
```

**Result:**
```
(1, 2, 3)
(3, 4, 7)
(5, 6, 11)
```

## Generic Functions

Generic functions work with multiple types while maintaining type consistency:

### Example: Max Function

Implements `$max<T: Number>(a: T, b: T) -> T`:

```rust
use scallop_core::common::foreign_function::*;
use scallop_core::common::value::Value;
use scallop_core::common::type_family::TypeFamily;

#[derive(Clone)]
pub struct Max;

impl ForeignFunction for Max {
    fn name(&self) -> String {
        "max".to_string()
    }

    fn num_generic_types(&self) -> usize {
        1  // One type parameter T
    }

    fn generic_type_family(&self, i: usize) -> TypeFamily {
        assert_eq!(i, 0);
        TypeFamily::Number  // T must be a number
    }

    fn num_static_arguments(&self) -> usize {
        2  // Two arguments
    }

    fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
        // Both arguments have type T (Generic(0))
        ForeignFunctionParameterType::Generic(0)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        // Return type is also T
        ForeignFunctionParameterType::Generic(0)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        // Handle all numeric types
        match (&args[0], &args[1]) {
            (Value::I32(a), Value::I32(b)) => Some(Value::I32(*a.max(b))),
            (Value::I64(a), Value::I64(b)) => Some(Value::I64(*a.max(b))),
            (Value::F64(a), Value::F64(b)) => Some(Value::F64(a.max(*b))),
            (Value::U32(a), Value::U32(b)) => Some(Value::U32(*a.max(b))),
            // Add more types as needed...
            _ => None,
        }
    }
}
```

**Type safety in action:**
```scl
rel int_pairs = {(5, 10), (20, 15)}
rel float_pairs = {(3.14, 2.71), (1.41, 1.73)}

// Valid: both args are i32
rel int_max(a, b, m) = int_pairs(a, b), m = $max(a, b)

// Valid: both args are f64
rel float_max(a, b, m) = float_pairs(a, b), m = $max(a, b)

// Invalid: mixing types would fail at compile time
// rel mixed(a, b, m) = int_pairs(a, _), float_pairs(_, b), m = $max(a, b)
```

### Example: Fibonacci (Generic Integer)

Works with any integer type:

```rust
#[derive(Clone)]
pub struct Fib;

impl ForeignFunction for Fib {
    fn name(&self) -> String {
        "fib".to_string()
    }

    fn num_generic_types(&self) -> usize {
        1
    }

    fn generic_type_family(&self, i: usize) -> TypeFamily {
        assert_eq!(i, 0);
        TypeFamily::Integer  // Only integers, not floats
    }

    fn num_static_arguments(&self) -> usize {
        1
    }

    fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
        assert_eq!(i, 0);
        ForeignFunctionParameterType::Generic(0)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::Generic(0)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        match &args[0] {
            Value::I32(n) => compute_fib(*n).map(Value::I32),
            Value::I64(n) => compute_fib(*n).map(Value::I64),
            Value::U32(n) => compute_fib(*n).map(Value::U32),
            // ... handle other integer types
            _ => None,
        }
    }
}

fn compute_fib<T: num_traits::PrimInt>(n: T) -> Option<T> {
    // Fibonacci implementation for generic integer type
    // ...
}
```

## Optional and Variable Arguments

### Optional Arguments

Functions can have optional parameters that default if not provided:

**Example: Substring with optional end**

`$substring(s: String, start: usize, end: usize?) -> String`

```rust
#[derive(Clone)]
pub struct Substring;

impl ForeignFunction for Substring {
    fn name(&self) -> String {
        "substring".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        2  // s and start are required
    }

    fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
        match i {
            0 => ForeignFunctionParameterType::BaseType(ValueType::String),
            1 => ForeignFunctionParameterType::BaseType(ValueType::USize),
            _ => panic!("Invalid argument index"),
        }
    }

    fn num_optional_arguments(&self) -> usize {
        1  // end is optional
    }

    fn optional_argument_type(&self, i: usize) -> ForeignFunctionParameterType {
        assert_eq!(i, 0);
        ForeignFunctionParameterType::BaseType(ValueType::USize)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        let s = if let Value::String(s) = &args[0] {
            s
        } else {
            return None;
        };

        let start = if let Value::USize(start) = args[1] {
            start
        } else {
            return None;
        };

        let end = if args.len() > 2 {
            if let Value::USize(end) = args[2] {
                end
            } else {
                return None;
            }
        } else {
            s.len()  // Default: to end of string
        };

        Some(Value::String(s.get(start..end)?.to_string()))
    }
}
```

**Usage:**
```scl
rel text = {"hello world"}

// With both arguments
rel part1(t, sub) = text(t), sub = $substring(t, 0, 5)  // "hello"

// With optional argument omitted
rel part2(t, sub) = text(t), sub = $substring(t, 6)     // "world"
```

### Variable Arguments

Functions that accept unlimited arguments:

**Example: String concatenation**

`$concat(strings: String...) -> String`

```rust
#[derive(Clone)]
pub struct Concat;

impl ForeignFunction for Concat {
    fn name(&self) -> String {
        "concat".to_string()
    }

    fn has_variable_arguments(&self) -> bool {
        true  // Accept any number of arguments
    }

    fn variable_argument_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        let mut result = String::new();

        for arg in args {
            if let Value::String(s) = arg {
                result.push_str(&s);
            } else {
                return None;
            }
        }

        Some(Value::String(result))
    }
}
```

**Usage:**
```scl
rel parts = {("Hello", " "), ("world", "!")}

// Concat multiple arguments
rel message = {$concat("Hello", " ", "world", "!")}  // "Hello world!"

// Variable number of arguments
rel concat2(a, b) = parts(a, b), _ = $concat(a, b)
rel concat4(a, b, c, d) = parts(a, b), parts(c, d), _ = $concat(a, b, c, d)
```

**Note:** Optional and variable arguments cannot coexist in the same function.

## Error Handling

### Returning None for Errors

When `execute()` encounters an error, return `None`:

```rust
fn execute(&self, args: Vec<Value>) -> Option<Value> {
    // Type check
    let n = if let Value::I32(n) = args[0] {
        n
    } else {
        return None;  // Wrong type
    };

    // Validation
    if n < 0 {
        return None;  // Invalid input (negative factorial)
    }

    // Computation that might fail
    let result = compute_factorial(n)?;  // Propagate None on overflow

    Some(Value::I32(result))
}
```

### When to Return None

- **Type mismatch:** Arguments don't match expected types (shouldn't happen if trait is correct)
- **Invalid input:** Mathematically invalid (sqrt of negative, division by zero)
- **Computation error:** Overflow, underflow, out of range
- **External failure:** I/O error, resource unavailable (avoid in pure functions!)

### Panic vs. None

**Use `None`:**
- Invalid inputs that can occur during normal operation
- Computation failures (overflow, domain errors)
- Partial functions (e.g., division by zero)

**Use `panic!`:**
- Programming errors (wrong trait implementation)
- Internal invariant violations
- Invalid argument indices in trait methods

**Example:**
```rust
fn execute(&self, args: Vec<Value>) -> Option<Value> {
    if let Value::I32(n) = args[0] {
        if n < 0 {
            None  // Graceful: negative input is user error
        } else {
            Some(Value::I32(n * 2))
        }
    } else {
        // Should never happen if types are correct
        panic!("Type system violated!")
    }
}
```

### Error Propagation

In `execute()`, use `?` to propagate `None` from fallible operations:

```rust
fn execute(&self, args: Vec<Value>) -> Option<Value> {
    let s = if let Value::String(s) = &args[0] {
        s
    } else {
        return None;
    };

    // Parse string to int (returns Option)
    let n: i32 = s.parse().ok()?;  // ? propagates None on failure

    // More operations...
    let result = some_fallible_op(n)?;

    Some(Value::I32(result))
}
```

## Complete Working Example

Here's a complete example demonstrating multiple foreign functions:

```rust
use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::runtime::env::RcFamily;
use scallop_core::common::foreign_function::*;
use scallop_core::common::value::Value;
use scallop_core::common::value_type::ValueType;
use scallop_core::common::type_family::TypeFamily;

// Function 1: String length
#[derive(Clone)]
struct StrLen;

impl ForeignFunction for StrLen {
    fn name(&self) -> String {
        "str_len".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        1
    }

    fn static_argument_type(&self, _i: usize) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::USize)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        if let Value::String(s) = &args[0] {
            Some(Value::USize(s.len()))
        } else {
            None
        }
    }
}

// Function 2: Max of two numbers
#[derive(Clone)]
struct Max;

impl ForeignFunction for Max {
    fn name(&self) -> String {
        "max".to_string()
    }

    fn num_generic_types(&self) -> usize {
        1
    }

    fn generic_type_family(&self, _i: usize) -> TypeFamily {
        TypeFamily::Number
    }

    fn num_static_arguments(&self) -> usize {
        2
    }

    fn static_argument_type(&self, _i: usize) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::Generic(0)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::Generic(0)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        match (&args[0], &args[1]) {
            (Value::I32(a), Value::I32(b)) => Some(Value::I32(*a.max(b))),
            (Value::F64(a), Value::F64(b)) => Some(Value::F64(a.max(*b))),
            _ => None,
        }
    }
}

fn main() -> Result<(), IntegrateError> {
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Register foreign functions
    ctx.register_foreign_function(StrLen)?;
    ctx.register_foreign_function(Max)?;

    // Add Scallop program
    ctx.add_program(r#"
        rel words = {"hello", "world", "rust", "scallop"}
        rel numbers = {(5, 10), (20, 15), (8, 12)}

        // Use string length function
        rel word_lengths(w, len) = words(w), len = $str_len(w)

        // Use max function
        rel maximums(a, b, m) = numbers(a, b), m = $max(a, b)

        query word_lengths
        query maximums
    "#)?;

    // Run the program
    ctx.run()?;

    // Query and display results
    println!("Word lengths:");
    let word_lengths = ctx.computed_relation_ref("word_lengths").unwrap();
    for elem in word_lengths.iter() {
        println!("  {:?}", elem.tuple);
    }

    println!("\nMaximums:");
    let maximums = ctx.computed_relation_ref("maximums").unwrap();
    for elem in maximums.iter() {
        println!("  {:?}", elem.tuple);
    }

    Ok(())
}
```

**Output:**
```
Word lengths:
  ("hello", 5)
  ("world", 5)
  ("rust", 4)
  ("scallop", 7)

Maximums:
  (5, 10, 10)
  (20, 15, 20)
  (8, 12, 12)
```

## Next Steps

- **[Foreign Predicates](foreign_predicates.md)** - Create non-deterministic fact generators
- **[IntegrateContext API](integrate_context.md)** - Register and use foreign functions
- **[Getting Started](getting_started.md)** - Quick start guide
- **[Examples](../examples/rust/foreign_functions/)** - Complete working examples
