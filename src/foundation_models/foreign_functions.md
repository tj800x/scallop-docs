# Foreign Functions

Foreign functions are **pure computational functions** that extend Scallop with external capabilities. They allow plugins to provide custom operations that can be called directly within Scallop programs using the `$function_name(args)` syntax.

## What are Foreign Functions?

### Definition

A foreign function is a **deterministic computation** that:
- Takes one or more input values
- Returns a single output value
- Has no side effects (pure function)
- Can fail gracefully (e.g., divide-by-zero returns no result)

### Syntax

Foreign functions are invoked with a dollar sign prefix:

```scl
rel result = {$function_name(arg1, arg2, ...)}
rel computed(x, y) = data(x), y = $function_name(x)
```

### Key Characteristics

**Pure computation:**
- Same inputs always produce same outputs
- No randomness, no external state modification
- Can be memoized for efficiency

**Type-safe:**
- Arguments and return types are declared
- Type checking at compile time
- Automatic type conversion where possible

**Partial functions:**
- May fail on some inputs (division by zero, index out of bounds)
- Failed computations are silently dropped from results
- No exceptions propagated to Scallop runtime

## Using Foreign Functions

### Basic Usage

Foreign functions are typically provided by plugins and become available after plugin loading:

```python
import scallopy

# Create context and load plugins
ctx = scallopy.ScallopContext()
plugin_registry = scallopy.PluginRegistry()
plugin_registry.load_plugins_from_entry_points()
plugin_registry.load_into_ctx(ctx)

# Now foreign functions from plugins are available
ctx.add_program("""
  rel image_path = {"photo.jpg"}
  rel loaded(img) = image_path(path), img = $load_image(path)
  query loaded
""")
ctx.run()
```

### In Scallop Programs

**As value generators:**
```scl
rel images = {
  $load_image("photo1.jpg"),
  $load_image("photo2.jpg"),
  $load_image("photo3.jpg")
}
```

**In rule bodies:**
```scl
rel processed(path, result) =
  image_paths(path),
  img = $load_image(path),
  result = $apply_filter(img, "blur")
```

**With aggregations:**
```scl
rel total = {$sum(x) | values(x)}
rel concat_all = {$string_join(s, ", ") | strings(s)}
```

### Type Conversion

Scallop automatically converts between compatible types:

| Scallop Type | Python Type | Notes |
|--------------|-------------|-------|
| `i8`, `i16`, `i32`, `i64`, `isize` | `int` | Integer family |
| `u8`, `u16`, `u32`, `u64`, `usize` | `int` | Unsigned integers |
| `f32`, `f64` | `float` | Floating point |
| `String` | `str` | Text |
| `bool` | `bool` | Boolean |
| `Tensor` | `torch.Tensor` | PyTorch tensors |

### Error Handling

Functions that fail produce no output:

```scl
rel indices = {0, 1, 2, 5, 10}
rel chars(i, c) = indices(i), c = $string_char_at("hello", i)
query chars

// Result: {(0, 'h'), (1, 'e'), (2, 'l')}
// Indices 5 and 10 are out of bounds and silently dropped
```

## Argument Types

Foreign functions support a rich type system for arguments and return values.

### Basic Types

| Python Type | Scallop Types | Description | Example |
|-------------|---------------|-------------|---------|
| `int` | `i8`, `i16`, `i32`, `i64`, `isize` | Signed integers | `42`, `-100` |
| `int` | `u8`, `u16`, `u32`, `u64`, `usize` | Unsigned integers | `255`, `1000` |
| `float` | `f32`, `f64` | Floating point | `3.14`, `2.718` |
| `str` | `String` | Text strings | `"hello"` |
| `bool` | `bool` | Boolean values | `true`, `false` |
| `torch.Tensor` | `Tensor` | PyTorch tensors | Images, embeddings |

### Type Annotations

**Always use explicit type annotations:**

```python
# ✓ Correct: explicit types
@scallopy.foreign_function
def add(x: int, y: int) -> int:
    return x + y

# ✗ Wrong: missing annotations (will fail)
@scallopy.foreign_function
def add(x, y):
    return x + y
```

### Automatic Type Conversion

Scallop performs automatic conversions between compatible types:

**Integer conversions:**
```python
@scallopy.foreign_function
def process(x: int) -> int:
    return x * 2

# Works with any Scallop integer type:
# i32(5) → 10
# u64(3) → 6
```

**Float conversions:**
```python
@scallopy.foreign_function
def square_root(x: float) -> float:
    import math
    return math.sqrt(x)

# Accepts f32 or f64:
# f32(16.0) → 4.0
# f64(25.0) → 5.0
```

**String handling:**
```python
@scallopy.foreign_function
def uppercase(s: str) -> str:
    return s.upper()

# String("hello") → "HELLO"
```

### Tensor Types

PyTorch tensors are first-class types in Scallop:

```python
import torch
import scallopy

@scallopy.foreign_function
def normalize_image(img: scallopy.Tensor) -> scallopy.Tensor:
    """Normalize image tensor to [0, 1] range."""
    tensor = img.float()
    return tensor / 255.0

@scallopy.foreign_function
def tensor_shape(img: scallopy.Tensor) -> str:
    """Get tensor shape as string."""
    return str(tuple(img.shape))
```

**Usage:**
```scl
rel image = {$load_image("photo.jpg")}
rel normalized(n) = image(img), n = $normalize_image(img)
rel shape(s) = image(img), s = $tensor_shape(img)
```

### Generic Types

Use generic type parameters for functions that work with multiple types:

```python
T = scallopy.ScallopGenericTypeParam(scallopy.Number)

@scallopy.foreign_function
def maximum(*values: T) -> T:
    """Return maximum of any numeric type."""
    return max(values)

# Works with any numeric type:
# $maximum(1, 5, 3) → 5 (integers)
# $maximum(1.5, 2.7, 0.3) → 2.7 (floats)
```

**Built-in generic constraints:**
- `scallopy.Number` - Any numeric type (int or float)
- `scallopy.Any` - Any Scallop type

## Optional Arguments

Foreign functions support optional arguments with default values.

### Basic Optional Arguments

```python
@scallopy.foreign_function
def greet(name: str, title: str = "Mr./Ms.") -> str:
    """Greet someone with optional title."""
    return f"Hello, {title} {name}!"
```

**Scallop usage:**
```scl
// With default title
rel greeting1 = {$greet("Smith")}
// Result: {"Hello, Mr./Ms. Smith!"}

// With custom title
rel greeting2 = {$greet("Johnson", "Dr.")}
// Result: {"Hello, Dr. Johnson!"}
```

### Multiple Optional Arguments

```python
@scallopy.foreign_function
def format_number(
    value: float,
    decimals: int = 2,
    prefix: str = "",
    suffix: str = ""
) -> str:
    """Format number with optional prefix, suffix, and precision."""
    formatted = f"{value:.{decimals}f}"
    return f"{prefix}{formatted}{suffix}"
```

**Scallop usage:**
```scl
rel numbers = {3.14159, 2.71828}

// Just value (all defaults)
rel simple(n, s) = numbers(n), s = $format_number(n)
// Result: {(3.14159, "3.14"), (2.71828, "2.72")}

// With precision
rel precise(n, s) = numbers(n), s = $format_number(n, 4)
// Result: {(3.14159, "3.1416"), (2.71828, "2.7183")}

// With all options
rel fancy(n, s) = numbers(n), s = $format_number(n, 2, "$", " USD")
// Result: {(3.14159, "$3.14 USD"), (2.71828, "$2.72 USD")}
```

### Optional with None

Use `None` as default for truly optional parameters:

```python
from typing import Optional

@scallopy.foreign_function
def fetch_or_default(key: str, default: Optional[str] = None) -> str:
    """Fetch value or return default."""
    if key in STORAGE:
        return STORAGE[key]
    return default if default is not None else "N/A"
```

**Scallop usage:**
```scl
rel keys = {"existing_key", "missing_key"}

// Without default
rel results1(k, v) = keys(k), v = $fetch_or_default(k)
// Result: {("existing_key", "value"), ("missing_key", "N/A")}

// With default
rel results2(k, v) = keys(k), v = $fetch_or_default(k, "DEFAULT")
// Result: {("existing_key", "value"), ("missing_key", "DEFAULT")}
```

## Variable Arguments

Foreign functions can accept variable numbers of arguments using `*args`.

### Basic Variable Arguments

```python
@scallopy.foreign_function
def sum_all(*args: int) -> int:
    """Sum any number of integers."""
    return sum(args)
```

**Scallop usage:**
```scl
// Different numbers of arguments
rel sum2 = {$sum_all(1, 2)}           // 3
rel sum3 = {$sum_all(1, 2, 3)}        // 6
rel sum5 = {$sum_all(1, 2, 3, 4, 5)}  // 15
```

### String Concatenation

```python
@scallopy.foreign_function
def concat(*strings: str) -> str:
    """Concatenate any number of strings."""
    return "".join(strings)
```

**Scallop usage:**
```scl
rel greeting = {$concat("Hello", ", ", "World", "!")}
// Result: {"Hello, World!"}

rel path = {$concat("/", "usr", "/", "local", "/", "bin")}
// Result: {"/usr/local/bin"}
```

### Variable Arguments with Separator

```python
@scallopy.foreign_function
def join_with(separator: str, *parts: str) -> str:
    """Join strings with specified separator."""
    return separator.join(parts)
```

**Scallop usage:**
```scl
rel csv = {$join_with(",", "apple", "banana", "cherry")}
// Result: {"apple,banana,cherry"}

rel path = {$join_with("/", "home", "user", "documents")}
// Result: {"home/user/documents"}
```

### Mixed Fixed and Variable Arguments

```python
@scallopy.foreign_function
def weighted_average(weight: float, *values: float) -> float:
    """Compute weighted average."""
    if not values:
        return 0.0
    return sum(v * weight for v in values) / len(values)
```

**Scallop usage:**
```scl
rel numbers = {1.0, 2.0, 3.0, 4.0, 5.0}
rel weighted(w, avg) = w = 0.8, avg = $weighted_average(w, 1.0, 2.0, 3.0)
// Result: {(0.8, 1.6)}
```

### Generic Variable Arguments

```python
T = scallopy.ScallopGenericTypeParam(scallopy.Number)

@scallopy.foreign_function
def min_value(*values: T) -> T:
    """Find minimum of any numeric type."""
    return min(values)

@scallopy.foreign_function
def max_value(*values: T) -> T:
    """Find maximum of any numeric type."""
    return max(values)
```

**Scallop usage:**
```scl
rel int_min = {$min_value(5, 2, 8, 1, 9)}      // 1
rel float_max = {$max_value(1.5, 3.2, 0.7)}    // 3.2
```

## Error Handling in Foreign Functions

Foreign functions should handle errors gracefully to maintain Scallop's declarative semantics.

### Exception Handling

When a foreign function raises an exception, Scallop **drops that computation** from results:

```python
@scallopy.foreign_function
def safe_divide(a: float, b: float) -> float:
    """Divide with zero-check."""
    if b == 0:
        raise ValueError("Division by zero")
    return a / b
```

**Scallop behavior:**
```scl
rel operations = {(10.0, 2.0), (15.0, 3.0), (8.0, 0.0), (20.0, 4.0)}
rel results(a, b, r) = operations(a, b), r = $safe_divide(a, b)
query results

// Result: {(10.0, 2.0, 5.0), (15.0, 3.0, 5.0), (20.0, 4.0, 5.0)}
// (8.0, 0.0) is silently dropped - no error message
```

### Graceful Degradation

Return default values instead of raising exceptions when appropriate:

```python
@scallopy.foreign_function
def safe_index(lst: str, idx: int) -> str:
    """Get character at index, return empty string if out of bounds."""
    try:
        return lst[idx]
    except IndexError:
        return ""  # Graceful fallback
```

**Scallop usage:**
```scl
rel text = {"hello"}
rel indices = {0, 1, 2, 10, 20}
rel chars(i, c) = text(t), indices(i), c = $safe_index(t, i)

// Result: {(0, "h"), (1, "e"), (2, "l"), (10, ""), (20, "")}
// Out-of-bounds indices return empty string instead of failing
```

### Partial Functions

Some functions are inherently partial (undefined for some inputs). Use exceptions to signal undefined cases:

```python
@scallopy.foreign_function
def sqrt(x: float) -> float:
    """Square root - undefined for negative numbers."""
    import math
    if x < 0:
        raise ValueError("Cannot take square root of negative number")
    return math.sqrt(x)
```

**Scallop behavior:**
```scl
rel numbers = {16.0, 25.0, -9.0, 36.0}
rel roots(n, r) = numbers(n), r = $sqrt(n)

// Result: {(16.0, 4.0), (25.0, 5.0), (36.0, 6.0)}
// -9.0 is dropped (undefined)
```

### Input Validation

Validate inputs and raise exceptions for invalid cases:

```python
@scallopy.foreign_function
def parse_age(s: str) -> int:
    """Parse age string, must be valid integer."""
    try:
        age = int(s)
        if age < 0 or age > 150:
            raise ValueError(f"Invalid age: {age}")
        return age
    except ValueError as e:
        raise ValueError(f"Cannot parse age from '{s}': {e}")
```

**Scallop usage:**
```scl
rel age_strings = {"25", "30", "invalid", "200", "45"}
rel ages(s, a) = age_strings(s), a = $parse_age(s)

// Result: {("25", 25), ("30", 30), ("45", 45)}
// "invalid" (not a number) and "200" (out of range) are dropped
```

### Logging Errors

Log errors for debugging while still maintaining graceful behavior:

```python
import logging

@scallopy.foreign_function
def fetch_data(url: str) -> str:
    """Fetch data from URL with error logging."""
    import requests
    try:
        response = requests.get(url, timeout=5)
        response.raise_for_status()
        return response.text
    except requests.RequestException as e:
        logging.error(f"Failed to fetch {url}: {e}")
        raise  # Re-raise to drop from Scallop results
```

**Behavior:**
- Successful fetches return data
- Failed fetches are logged and dropped from results
- Scallop program continues execution

### Best Practices for Error Handling

**✓ Good practices:**

```python
# 1. Clear error messages
@scallopy.foreign_function
def validate_email(email: str) -> bool:
    if "@" not in email:
        raise ValueError(f"Invalid email format: {email}")
    return True

# 2. Explicit None checks
@scallopy.foreign_function
def safe_operation(value: Optional[str]) -> str:
    if value is None:
        raise ValueError("Value cannot be None")
    return value.upper()

# 3. Type validation
@scallopy.foreign_function
def process_positive(x: int) -> int:
    if x <= 0:
        raise ValueError(f"Expected positive integer, got {x}")
    return x * 2
```

**✗ Avoid:**

```python
# Don't silence all errors
@scallopy.foreign_function
def bad_function(x: str) -> str:
    try:
        return risky_operation(x)
    except:  # Too broad!
        return ""  # Hides real problems

# Don't use print() for errors
@scallopy.foreign_function
def bad_logging(x: int) -> int:
    if x < 0:
        print("Error: negative value")  # User won't see this
        raise ValueError("Negative value")
    return x
```

### Error Recovery Patterns

**Pattern 1: Try multiple strategies**
```python
@scallopy.foreign_function
def flexible_parse(s: str) -> float:
    """Try multiple parsing strategies."""
    # Strategy 1: Direct float conversion
    try:
        return float(s)
    except ValueError:
        pass

    # Strategy 2: Remove commas
    try:
        return float(s.replace(",", ""))
    except ValueError:
        pass

    # Strategy 3: Extract first number
    import re
    match = re.search(r'-?\d+\.?\d*', s)
    if match:
        return float(match.group())

    raise ValueError(f"Cannot parse number from '{s}'")
```

**Pattern 2: Fallback values**
```python
@scallopy.foreign_function
def get_or_default(key: str, default: str = "UNKNOWN") -> str:
    """Get value with fallback."""
    if key in DATABASE:
        return DATABASE[key]
    return default  # No exception, returns default
```

**Pattern 3: Validation before computation**
```python
@scallopy.foreign_function
def safe_compute(x: int, y: int) -> int:
    """Compute with pre-validation."""
    # Validate inputs first
    if x < 0 or y < 0:
        raise ValueError("Inputs must be non-negative")
    if x + y > 1000000:
        raise ValueError("Result would be too large")

    # Safe to compute
    return expensive_operation(x, y)
```

## Examples from Plugins

### OpenCV Plugin: Image I/O

The OpenCV plugin provides several foreign functions for image manipulation:

**Loading images:**
```python
@scallopy.foreign_function
def load_image(image_dir: str) -> scallopy.Tensor:
    from PIL import Image
    import torch, numpy

    image = Image.open(image_dir).convert("RGB")
    image_tensor = torch.tensor(numpy.asarray(image))
    return image_tensor
```

**Usage:**
```scl
rel image_paths = {"cat.jpg", "dog.jpg", "bird.jpg"}
rel images(path, img) = image_paths(path), img = $load_image(path)
query images
```

**Cropping images:**
```python
@scallopy.foreign_function
def crop_image(
    img: scallopy.Tensor,
    bbox_x: scallopy.u32,
    bbox_y: scallopy.u32,
    bbox_w: scallopy.u32,
    bbox_h: scallopy.u32,
    loc: str = None
) -> scallopy.Tensor:
    # Crop implementation with optional location modifier
    # ...
    return img[y1:y2, x1:x2, :]
```

**Usage:**
```scl
rel original = {$load_image("photo.jpg")}
rel face_region(cropped) = original(img), cropped = $crop_image(img, 100, 50, 200, 200)
rel enlarged(result) = face_region(img), result = $crop_image(img, 0, 0, 300, 300, "enlarge(1.5)")
```

### GPT Plugin: Text Generation

The GPT plugin provides a simple text-to-text foreign function:

**Implementation:**
```python
@scallopy.foreign_function
def gpt(prompt: str) -> str:
    if prompt in STORAGE:  # Memoization
        return STORAGE[prompt]

    response = openai.ChatCompletion.create(
        model="gpt-3.5-turbo",
        messages=[{"role": "user", "content": prompt}],
        temperature=0.0,
    )
    result = response["choices"][0]["message"]["content"].strip()
    STORAGE[prompt] = result
    return result
```

**Usage:**
```scl
rel questions = {
  "What is the capital of France?",
  "Translate to Spanish: Hello",
  "What is 15 * 24?"
}

rel answers(q, a) = questions(q), a = $gpt(q)
query answers

// Expected output (mock when API key not set):
// answers: {
//   ("What is the capital of France?", "Paris"),
//   ("Translate to Spanish: Hello", "Hola"),
//   ("What is 15 * 24?", "360")
// }
```

### Built-in Mathematical Functions

Scallop includes many built-in foreign functions:

```scl
rel numbers = {-5, 0, 3, 7}

// Absolute value
rel abs_vals(x, y) = numbers(x), y = $abs(x)
// Result: {(-5, 5), (0, 0), (3, 3), (7, 7)}

// String formatting
rel formatted(s) = numbers(x), s = $format("Value: {}", x)
// Result: {("Value: -5"), ("Value: 0"), ...}

// Hash function
rel hashes(x, h) = numbers(x), h = $hash(x)
```

## Creating Foreign Functions in Plugins

### Basic Foreign Function

To create a foreign function in a plugin:

```python
import scallopy

class MyPlugin(scallopy.Plugin):
    def __init__(self):
        super().__init__("my_plugin")

    def load_into_ctx(self, ctx):
        # Define and register foreign function
        @scallopy.foreign_function
        def double(x: int) -> int:
            return x * 2

        ctx.register_foreign_function(double)
```

### With Optional Arguments

```python
@scallopy.foreign_function
def greet(name: str, title: str = "Mr./Ms.") -> str:
    return f"Hello, {title} {name}!"

# Can be called as:
# $greet("Smith") → "Hello, Mr./Ms. Smith!"
# $greet("Smith", "Dr.") → "Hello, Dr. Smith!"
```

### With Variable Arguments

```python
@scallopy.foreign_function
def my_sum(*args: int) -> int:
    return sum(args)

# Can be called with any number of arguments:
# $my_sum(1, 2) → 3
# $my_sum(1, 2, 3, 4, 5) → 15
```

### With Generic Types

```python
T = scallopy.ScallopGenericTypeParam(scallopy.Number)

@scallopy.foreign_function
def maximum(*values: T) -> T:
    return max(values)

# Works with any numeric type:
# $maximum(1, 5, 3) → 5 (integers)
# $maximum(1.5, 2.7, 0.3) → 2.7 (floats)
```

### Error Handling

Foreign functions should handle errors gracefully:

```python
@scallopy.foreign_function
def safe_divide(a: float, b: float) -> float:
    if b == 0:
        raise ValueError("Division by zero")  # Handled by Scallop
    return a / b
```

When the function raises an exception, Scallop drops that computation:

```scl
rel operations = {(10, 2), (15, 3), (8, 0)}
rel results(a, b, r) = operations(a, b), r = $safe_divide(a, b)
query results

// Result: {(10, 2, 5.0), (15, 3, 5.0)}
// (8, 0) is dropped due to division by zero
```

## Best Practices

### Memoization for Expensive Operations

Cache results of expensive computations:

```python
CACHE = {}

@scallopy.foreign_function
def expensive_operation(x: str) -> str:
    if x not in CACHE:
        # Expensive computation here
        CACHE[x] = compute_result(x)
    return CACHE[x]
```

### Lazy Loading

Load heavy dependencies only when needed:

```python
_MODEL = None

@scallopy.foreign_function
def use_model(input: str) -> str:
    global _MODEL
    if _MODEL is None:
        import heavy_ml_library
        _MODEL = heavy_ml_library.load_model()
    return _MODEL.predict(input)
```

### Type Safety

Always annotate types explicitly:

```python
# ✓ Good: explicit types
@scallopy.foreign_function
def process(x: int, y: str) -> bool:
    return len(y) > x

# ✗ Bad: missing type annotations
def process(x, y):  # Will fail at registration
    return len(y) > x
```

## Next Steps

- **[Foreign Predicates](foreign_predicates.md)** - Learn about multi-valued fact generation
- **[Foreign Attributes](foreign_attributes.md)** - Metaprogramming with decorators
- **[GPT Plugin](openai_gpt.md)** - Complete example of LLM integration
- **[Create Your Own Plugin](create_your_own_plugin.md)** - Build custom plugins

For more details on the language-level foreign function syntax, see [Foreign Functions (Language)](../language/foreign_functions.md).
