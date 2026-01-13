# Types

Scallop has a rich type system for specifying relation schemas. This guide covers how to use Scallop types in Python via the `scallopy` API.

## Python to Scallop Type Mapping

When declaring relations in Python, common Python types are automatically mapped to Scallop types:

| Python Type | Scallop Type | Description |
|-------------|--------------|-------------|
| `int` | `i32` | 32-bit signed integer |
| `bool` | `bool` | Boolean value |
| `float` | `f32` | 32-bit floating point |
| `str` | `String` | Unicode string |

**Example:**
```python
ctx.add_relation("person", (str, int, bool))
# Translates to: person(String, i32, bool)
```

---

## Native Scallop Types

For more control, use Scallop's native types directly.

### Integer Types

**Signed integers:**
- `i8` - 8-bit signed integer (-128 to 127)
- `i16` - 16-bit signed integer
- `i32` - 32-bit signed integer (default for Python `int`)
- `i64` - 64-bit signed integer
- `i128` - 128-bit signed integer
- `isize` - Pointer-sized signed integer

**Unsigned integers:**
- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`

### Floating Point Types

- `f32` - 32-bit floating point
- `f64` - 64-bit floating point (double precision)

### Text Types

- `char` - Single Unicode character
- `String` - Variable-length string
- `Symbol` - Interned string (efficient for repeated strings)

### Special Types

- `bool` - Boolean values
- `DateTime` - Date and time values
- `Duration` - Time durations
- `Entity` - Algebraic data type entities (content-addressable)
- `Tensor` - PyTorch tensor values

---

## Using Types in Python

### Automatic Mapping

Python types automatically map to Scallop types:

```python
import scallopy

ctx = scallopy.ScallopContext()

# Python types are automatically converted
ctx.add_relation("edge", (int, int))        # → (i32, i32)
ctx.add_relation("weight", (int, float))     # → (i32, f32)
ctx.add_relation("name", (int, str))         # → (i32, String)
ctx.add_relation("flag", bool)                # → bool
```

### Explicit Scallop Types

Use `scallopy` types for explicit control:

```python
import scallopy

ctx.add_relation("my_rel", (scallopy.usize, scallopy.f64, scallopy.i32))
```

**All available types:**
```python
# Signed integers
scallopy.i8, scallopy.i16, scallopy.i32, scallopy.i64, scallopy.i128, scallopy.isize

# Unsigned integers
scallopy.u8, scallopy.u16, scallopy.u32, scallopy.u64, scallopy.u128, scallopy.usize

# Floating point
scallopy.f32, scallopy.f64

# Other primitives
scallopy.bool, scallopy.char, scallopy.String

# Special types
scallopy.Symbol, scallopy.DateTime, scallopy.Duration
scallopy.Entity, scallopy.Tensor
```

---

## Advanced Types

### Type Families

Scallop supports generic types for polymorphic predicates:

```python
from scallopy import value_types

# Generic number type
def add(a: value_types.Number, b: value_types.Number) -> value_types.Number:
  ...
```

**Available type families:**
- `Any` - Any type
- `Number` - Any numeric type
- `Integer` - Any integer type
- `SignedInteger` - Signed integers (i8, i16, i32, i64, i128, isize)
- `UnsignedInteger` - Unsigned integers (u8, u16, u32, u64, u128, usize)
- `Float` - Floating point types (f32, f64)

### Special Types

**Tensor:**
```python
import scallopy

ctx.add_relation("embedding", (int, scallopy.Tensor))
# For neural network integration
```

**DateTime and Duration:**
```python
from scallopy import DateTime, Duration

ctx.add_relation("event", (String, DateTime))
ctx.add_relation("duration", (String, Duration))
```

**Entity:**
```python
# For ADT (Algebraic Data Type) entities
ctx.add_relation("object", scallopy.Entity)
```

---

## Summary

- **Python → Scallop** automatic mapping: `int→i32`, `float→f32`, `str→String`, `bool→bool`
- **Explicit types** via `scallopy.i64`, `scallopy.usize`, etc.
- **String types** for custom or non-standard types
- **Type families** for generic programming: `Integer`, `Float`, `Number`
- **Special types**: `Tensor` (PyTorch), `DateTime`, `Duration`, `Entity`
- **Single-column shorthand**: Use type directly, no tuple needed
- **Always check schema** with error messages if types don't match

For more details:
- [ScallopContext](context.md) - Type usage in add_relation
- [Foreign Predicates](foreign_predicate.md) - Type annotations for predicates
- [Scallop Language Types](../language/value_type.md) - Full type system reference
