# Custom Types

Custom types allow you to define domain-specific types that make your Scallop programs more readable, maintainable, and type-safe. Scallop supports two kinds of custom types: **type aliases** and **algebraic data types (ADTs)**.

## Type Aliases

Type aliases create alternative names for existing types, improving code readability without changing behavior.

### Basic Type Aliases

```scl
type UserId = i32
type Username = String
type Score = f32
```

**Usage:**
```scl
type user(id: UserId, name: Username, score: Score)

rel user = {(1, "alice", 95.5), (2, "bob", 87.0)}
```

### Why Use Type Aliases?

**Without type aliases:**
```scl
type edge(i32, i32)
type distance(i32, i32, f32)
```

**With type aliases:**
```scl
type NodeId = i32
type Distance = f32

type edge(from: NodeId, to: NodeId)
type distance(from: NodeId, to: NodeId, dist: Distance)
```

The second version is more self-documenting - you immediately understand what each field represents.

---

## Algebraic Data Types (ADTs)

ADTs allow you to define complex structured data with variants (sum types) and fields (product types). See [Algebraic Data Types and Entities](adt_and_entity.md) for comprehensive coverage.

### Quick ADT Syntax

```scl
type TypeName = Variant1(Type1, Type2, ...)
              | Variant2(Type3, Type4, ...)
              | TerminalVariant()
```

### Example: Traffic Light States

```scl
type TrafficLight = Red()
                  | Yellow()
                  | Green()

const CURRENT_STATE = Red()

rel is_safe_to_cross(state) = case state is Green()
rel must_stop(state) = case state is Red()
```

### Example: Coordinates

```scl
type Coordinate = Cartesian(f32, f32)
                | Polar(f32, f32)

const ORIGIN = Cartesian(0.0, 0.0)
const ANGLE_0 = Polar(1.0, 0.0)

rel is_origin(coord) = case coord is Cartesian(x, y), x == 0.0, y == 0.0
```

---

## Domain-Specific Types for Clarity

### Example: Game Actions

Instead of using strings or integers to represent actions:

```scl
// ✗ Poor: using strings
rel action = {"move_up", "move_down", "move_left", "move_right"}

// ✓ Better: using ADT
type Action = MoveUp()
            | MoveDown()
            | MoveLeft()
            | MoveRight()

rel legal_action(action) = ...
rel execute(action, result) = ...
```

### Example: Error Handling

```scl
type Result = Success(i32)
            | Error(String)

rel divide(a, b, result) =
  a > 0, b > 0, result = Success(a / b)

rel divide(a, b, result) =
  b == 0, result = Error("Division by zero")
```

---

## Type Safety Benefits

### Catching Errors at Compile Time

**Without custom types:**
```scl
rel edge(0, 1)  // Valid
rel edge("a", "b")  // Also valid - but maybe not intended!
```

**With custom types:**
```scl
type NodeId = i32
type edge(from: NodeId, to: NodeId)

rel edge(0, 1)  // Valid
rel edge("a", "b")  // Compile error: type mismatch
```

### Preventing Mixed Units

```scl
type Meters = f32
type Seconds = f32
type Velocity = f32  // meters per second

type position(time: Seconds, distance: Meters)
type velocity(value: Velocity)

// Can't accidentally use distance where velocity is expected
rel speed(v) = velocity(v)
// rel speed(d) = position(_, d)  // Type error if uncommented
```

---

## Enum-Style Types

For representing a fixed set of values, ADTs work like enums:

### Days of Week

```scl
type Day = Monday()
         | Tuesday()
         | Wednesday()
         | Thursday()
         | Friday()
         | Saturday()
         | Sunday()

rel is_weekend(day) = case day is Saturday() or case day is Sunday()

const TODAY = Monday()
query is_weekend(TODAY)
// Result: false
```

### HTTP Status Codes

```scl
type Status = OK()
            | NotFound()
            | ServerError()
            | Unauthorized()

rel is_success(status) = case status is OK()
rel is_client_error(status) = case status is NotFound() or case status is Unauthorized()
```

---

## Best Practices

### 1. Use Type Aliases for Clarity

```scl
// ✓ Good: self-documenting
type Timestamp = i64
type UserId = i32
type MessageId = i32

type message(id: MessageId, sender: UserId, timestamp: Timestamp)

// ✗ Poor: unclear what integers represent
type message(i32, i32, i64)
```

### 2. Use ADTs for Structured Variants

```scl
// ✓ Good: clear variants
type Shape = Circle(f32)           // radius
           | Rectangle(f32, f32)   // width, height
           | Triangle(f32, f32, f32)  // three sides

// ✗ Poor: using strings to represent shape type
type shape(String, f32, f32, f32)  // Unclear meaning
```

### 3. Keep Variant Names Unique

```scl
// ✗ Bad: name collision
type IntList = Nil() | Cons(i32, IntList)
type BoolList = Nil() | Cons(bool, BoolList)  // Error: Nil and Cons already defined

// ✓ Good: unique names
type IntList = IntNil() | IntCons(i32, IntList)
type BoolList = BoolNil() | BoolCons(bool, BoolList)
```

### 4. Name Types Descriptively

```scl
// ✓ Good names
type EmailAddress = String
type PhoneNumber = String
type PostalCode = String

// ✗ Poor names
type StrA = String
type StrB = String
type StrC = String
```

---

## Limitations

### No Generic Types

Scallop does not currently support generic/parametric types. Each type must be defined explicitly:

```scl
// ✗ Not supported: generic List<T>
// type List<T> = Nil() | Cons(T, List<T>)

// ✓ Supported: specific types
type IntList = IntNil() | IntCons(i32, IntList)
type StringList = StrNil() | StrCons(String, StringList)
```

### Variant Names Must Be Globally Unique

Variant names cannot be reused across different ADTs:

```scl
// ✗ Error: None used in two types
type Option = Some(i32) | None()
type Result = OK(i32) | None()  // Compile error

// ✓ Fixed: unique variant names
type Option = Some(i32) | NoneOpt()
type Result = OK(i32) | NoneRes()
```

---

## Summary

- **Type aliases** create readable names for existing types
- **ADTs** define custom structured data with variants
- **Type safety** catches errors at compile time
- **Domain modeling** makes programs self-documenting
- **Best practices**: Use descriptive names, avoid collisions, prefer ADTs for variants

For more details:
- [Algebraic Data Types and Entities](adt_and_entity.md) - Comprehensive ADT guide
- [Value Types](value_type.md) - Built-in types
- [Constants](constants.md) - Named constants with custom types
