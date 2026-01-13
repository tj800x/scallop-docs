# Foreign Functions Example

This example demonstrates implementing custom Rust functions that extend Scallop's capabilities.

## What This Example Demonstrates

- Implementing the `ForeignFunction` trait
- String manipulation functions (`string_length`, `uppercase`)
- Numeric operations (`abs`, `max`)
- Registering functions with `IntegrateContext`
- Using foreign functions in Scallop programs with `$function(args)` syntax

## Foreign Functions Implemented

### 1. string_length - String Length

```rust
$string_length(s: String) -> USize
```

Returns the length of a string.

**Implementation highlights:**
- Single argument of type `String`
- Returns `USize`
- Extracts value with pattern matching

### 2. uppercase - String Uppercase

```rust
$uppercase(s: String) -> String
```

Converts a string to uppercase.

**Implementation highlights:**
- String-to-string transformation
- Uses Rust's `.to_uppercase()` method

### 3. abs - Absolute Value

```rust
$abs(n: i32) -> i32
```

Returns the absolute value of an integer.

**Implementation highlights:**
- Integer arithmetic
- Single argument and return value

### 4. max - Maximum of Two Integers

```rust
$max(a: i32, b: i32) -> i32
```

Returns the larger of two integers.

**Implementation highlights:**
- Two arguments (binary operation)
- Demonstrates multi-argument functions

## Expected Output

```
=== Foreign Functions Example ===

Registering foreign functions:
  - string_length(String) -> USize
  - uppercase(String) -> String
  - abs(i32) -> i32
  - max(i32, i32) -> i32

Program loaded
Program executed

String Lengths:
  "hello" has length 5
  "world" has length 5
  "scallop" has length 7

Uppercase Conversions:
  "hello" -> "HELLO"
  "world" -> "WORLD"
  "scallop" -> "SCALLOP"

Absolute Values:
  abs(-5) = 5
  abs(10) = 10
  abs(-3) = 3
  abs(7) = 7

Pair Maximums (sample):
  max(-5, -3) = -3
  max(-5, 7) = 7
  max(-5, 10) = 10
  max(-3, 7) = 7
  max(-3, 10) = 10

=== Example Complete ===
```

## Running This Example

```bash
cargo run
```

## Key Concepts

### The ForeignFunction Trait

```rust
pub trait ForeignFunction: DynClone {
    fn name(&self) -> String;
    fn num_static_arguments(&self) -> usize;
    fn static_argument_type(&self, i: usize) -> ForeignFunctionParameterType;
    fn return_type(&self) -> ForeignFunctionParameterType;
    fn execute(&self, args: Vec<Value>) -> Option<Value>;
}
```

**Required implementations:**
1. **name()** - Unique function name
2. **num_static_arguments()** - Number of required arguments
3. **static_argument_type(i)** - Type of i-th argument
4. **return_type()** - Type of return value
5. **execute(args)** - Function logic

### Implementing a Simple Function

```rust
#[derive(Clone)]
pub struct StringLength;

impl ForeignFunction for StringLength {
    fn name(&self) -> String {
        "string_length".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        1  // Takes one argument
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
            None  // Type mismatch
        }
    }
}
```

### Registering Functions

```rust
ctx.register_foreign_function(StringLength);
ctx.register_foreign_function(IntMax);
// ... etc
```

Functions must be registered **before** adding programs that use them.

### Using in Scallop Programs

```scl
rel words = {"hello", "world"}
rel lengths(w, len) = words(w), len = $string_length(w)
```

**Syntax:**
- Prefix with `$`
- Call like regular function: `$function_name(args)`
- Assign result to variable

### Error Handling

```rust
fn execute(&self, args: Vec<Value>) -> Option<Value> {
    if let Value::String(s) = &args[0] {
        Some(Value::USize(s.len()))
    } else {
        None  // Type error - argument is not a String
    }
}
```

Return `None` for:
- Type mismatches
- Invalid arguments
- Computation errors

## Implementation Pattern

**Step-by-step:**

1. **Define struct** (must be Clone):
   ```rust
   #[derive(Clone)]
   pub struct MyFunction;
   ```

2. **Implement ForeignFunction**:
   ```rust
   impl ForeignFunction for MyFunction {
       // ... required methods
   }
   ```

3. **Register with context**:
   ```rust
   ctx.register_foreign_function(MyFunction);
   ```

4. **Use in Scallop**:
   ```scl
   rel result = $my_function(arg)
   ```

## Next Steps

- **[foreign_predicates](../foreign_predicates/)** - Non-deterministic fact generators
- **[complex_reasoning](../complex_reasoning/)** - Combine functions with proofs
- **[Foreign Functions Guide](../../../doc/src/rust_api/foreign_functions.md)** - Complete API reference

## Related Documentation

- [Foreign Functions API](../../../doc/src/rust_api/foreign_functions.md)
- [Getting Started Guide](../../../doc/src/rust_api/getting_started.md)
- [IntegrateContext API](../../../doc/src/rust_api/integrate_context.md)
