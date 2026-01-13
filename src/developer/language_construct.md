# Implementing Language Constructs

This guide explains how to add new language features to Scallop. We'll walk through the process from grammar to runtime execution.

## Overview

Adding a new language feature typically involves these steps:

1. **Grammar** - Define syntax in LALRPOP grammar
2. **AST** - Add AST nodes to represent the construct
3. **Type checking** - Implement type inference/checking
4. **IR generation** - Lower AST to intermediate representation
5. **Execution** - Implement runtime behavior
6. **Testing** - Add comprehensive tests

---

## Step 1: Grammar Definition

Grammar is defined in `/core/src/compiler/front/grammar.lalrpop` using LALRPOP.

### Example: Adding a New Operator

Suppose we want to add a `**` (power) operator:

```lalrpop
// In grammar.lalrpop

pub Expr: Expr = {
    // Existing operators
    <l:Expr> "+" <r:Factor> => Expr::Binary(BinaryOp::Add, Box::new(l), Box::new(r)),
    <l:Expr> "-" <r:Factor> => Expr::Binary(BinaryOp::Sub, Box::new(l), Box::new(r)),

    // New power operator
    <l:Expr> "**" <r:Factor> => Expr::Binary(BinaryOp::Pow, Box::new(l), Box::new(r)),

    Factor,
}
```

### Key Grammar Sections

**Expressions** (`Expr`):
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `and`, `or`, `not`
- Aggregation: `count`, `sum`, `max`, etc.

**Atoms** (`Atom`):
- Predicates: `rel_name(args)`
- Constraints: `x > 5`
- Pattern matching: `case x is Variant(y)`

**Rules** (`Rule`):
- Basic: `head = body`
- Disjunctive: `{ head1; head2 } = body`
- Conjunctive: `head1; head2 = body`

---

## Step 2: AST Nodes

AST definitions are in `/core/src/compiler/front/ast.rs`.

### Adding to AST

```rust
// In ast.rs

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,  // New operator
    // ... other ops
}

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    // ... other expression types
}
```

### AST Best Practices

1. **Use `Box` for recursive types** - Prevents infinite size
2. **Implement `Clone`, `Debug`** - Required for compiler passes
3. **Add `Span` information** - For error reporting
4. **Document semantics** - Explain what the node represents

---

## Step 3: Type Checking

Type checking is in `/core/src/compiler/type_check/`.

### Type Inference

```rust
// In type_check/expr.rs

impl Expr {
    pub fn infer_type(&self, env: &TypeEnv) -> Result<Type, TypeError> {
        match self {
            Expr::Binary(BinaryOp::Add, l, r) => {
                let lt = l.infer_type(env)?;
                let rt = r.infer_type(env)?;
                unify_numeric(lt, rt)
            }
            Expr::Binary(BinaryOp::Pow, l, r) => {
                // New: Type check power operator
                let lt = l.infer_type(env)?;
                let rt = r.infer_type(env)?;
                match (lt, rt) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    (Type::Float, Type::Float) => Ok(Type::Float),
                    (Type::Float, Type::Int) => Ok(Type::Float),
                    _ => Err(TypeError::InvalidPowerOp(lt, rt))
                }
            }
            // ... other cases
        }
    }
}
```

### Type System Components

- **`Type`** - Represents Scallop types (int, float, string, ADTs)
- **`TypeEnv`** - Environment mapping variables to types
- **`TypeError`** - Type checking errors
- **`unify`** - Type unification for inference

---

## Step 4: IR Generation

Lower AST to intermediate representation in `/core/src/compiler/back/`.

### Generating IR

```rust
// In back/compile.rs

impl Compiler {
    fn compile_expr(&mut self, expr: &Expr) -> IRExpr {
        match expr {
            Expr::Binary(BinaryOp::Add, l, r) => {
                let l_ir = self.compile_expr(l);
                let r_ir = self.compile_expr(r);
                IRExpr::Binary(IRBinaryOp::Add, Box::new(l_ir), Box::new(r_ir))
            }
            Expr::Binary(BinaryOp::Pow, l, r) => {
                // New: Compile power operator
                let l_ir = self.compile_expr(l);
                let r_ir = self.compile_expr(r);
                IRExpr::Binary(IRBinaryOp::Pow, Box::new(l_ir), Box::new(r_ir))
            }
            // ... other cases
        }
    }
}
```

### IR Structure

IR is closer to execution than AST:
- **Variables** become explicit
- **Types** are attached
- **Control flow** is explicit
- **Aggregations** are lowered to loops

---

## Step 5: Runtime Execution

Implement execution in `/core/src/runtime/`.

### Adding Runtime Support

```rust
// In runtime/eval.rs

impl Executor {
    fn eval_binary(&self, op: &BinaryOp, l: &Value, r: &Value) -> Result<Value> {
        match op {
            BinaryOp::Add => {
                match (l, r) {
                    (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b)),
                    (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a + b)),
                    _ => Err(RuntimeError::TypeError)
                }
            }
            BinaryOp::Pow => {
                // New: Execute power operator
                match (l, r) {
                    (Value::I32(a), Value::I32(b)) => {
                        Ok(Value::I32(a.pow(*b as u32)))
                    }
                    (Value::F32(a), Value::F32(b)) => {
                        Ok(Value::F32(a.powf(*b)))
                    }
                    _ => Err(RuntimeError::TypeError)
                }
            }
            // ... other cases
        }
    }
}
```

### Runtime Components

- **`Value`** - Runtime value representation
- **`Executor`** - Executes IR
- **`Database`** - Stores relations
- **`ProvenanceContext`** - Manages provenance

---

## Step 6: Testing

Add tests in `/core/tests/integrate/`.

### Integration Test Example

```rust
// In tests/integrate/operators.rs

#[test]
fn test_power_operator() {
    let program = r#"
        rel base = {2, 3, 4}
        rel power(x, y) = base(x), y = x ** 2
        query power
    "#;

    let expected = vec![
        (2, 4),
        (3, 9),
        (4, 16),
    ];

    test_program(program, "power", expected);
}

#[test]
fn test_power_with_float() {
    let program = r#"
        rel x = {2.0, 3.0}
        rel result(x ** 0.5) = x(x)
        query result
    "#;

    let expected = vec![
        1.414,  // sqrt(2)
        1.732,  // sqrt(3)
    ];

    test_program_float(program, "result", expected, 0.01);
}
```

### Test Categories

- **Unit tests** - Test individual components
- **Integration tests** - Test complete programs
- **Type error tests** - Ensure bad programs fail
- **Provenance tests** - Test with different provenances
- **Performance tests** - Benchmark critical operations

---

## Complete Example: Adding `let` Bindings

Let's walk through a complete example: adding local variable bindings.

### 1. Grammar

```lalrpop
// Add to Atom rule
pub Atom: Atom = {
    // Existing patterns...

    // New: let binding
    "let" <var:Name> "=" <val:Expr> "," <body:Atom> => {
        Atom::Let(var, val, Box::new(body))
    },
}
```

### 2. AST

```rust
#[derive(Clone, Debug)]
pub enum Atom {
    // Existing variants...
    Let(String, Expr, Box<Atom>),
}
```

### 3. Type Checking

```rust
impl Atom {
    pub fn type_check(&self, env: &mut TypeEnv) -> Result<(), TypeError> {
        match self {
            Atom::Let(var, val, body) => {
                let val_type = val.infer_type(env)?;
                env.insert(var.clone(), val_type);
                body.type_check(env)?;
                env.remove(var);
                Ok(())
            }
            // ... other cases
        }
    }
}
```

### 4. IR Generation

```rust
impl Compiler {
    fn compile_atom(&mut self, atom: &Atom) -> Vec<IRStmt> {
        match atom {
            Atom::Let(var, val, body) => {
                let val_ir = self.compile_expr(val);
                let var_id = self.fresh_var();

                vec![
                    IRStmt::Assign(var_id, val_ir),
                    IRStmt::Scope {
                        bindings: vec![(var.clone(), var_id)],
                        body: Box::new(self.compile_atom(body)),
                    }
                ]
            }
            // ... other cases
        }
    }
}
```

### 5. Execution

```rust
impl Executor {
    fn eval_stmt(&mut self, stmt: &IRStmt) -> Result<()> {
        match stmt {
            IRStmt::Scope { bindings, body } => {
                // Push new scope
                for (name, var_id) in bindings {
                    let value = self.read_var(*var_id)?;
                    self.env.push(name.clone(), value);
                }

                // Execute body
                self.eval_stmt(body)?;

                // Pop scope
                for (name, _) in bindings {
                    self.env.pop(name);
                }

                Ok(())
            }
            // ... other cases
        }
    }
}
```

### 6. Tests

```rust
#[test]
fn test_let_binding() {
    let program = r#"
        rel edge(0, 1)
        rel edge(1, 2)

        rel result(a, c, d) =
            edge(a, b),
            let x = a + b,
            edge(b, c),
            let y = b + c,
            d = x + y

        query result
    "#;

    let expected = vec![(0, 1, 2), (1, 2, 4)];
    test_program(program, "result", expected);
}
```

---

## Common Pitfalls

### 1. Forgetting Provenance

New operations must handle provenance correctly:

```rust
// ✗ Bad: ignores provenance
fn eval_binary(l: Value, r: Value) -> Value {
    Value::new(l.data + r.data)
}

// ✓ Good: propagates provenance
fn eval_binary(&self, l: TaggedValue, r: TaggedValue) -> TaggedValue {
    let tag = self.provenance.mult(&l.tag, &r.tag);  // Combine tags
    TaggedValue::new(l.value + r.value, tag)
}
```

### 2. Not Handling All Types

Operations must work with all relevant types:

```rust
// ✗ Bad: only handles i32
match (l, r) {
    (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
    _ => panic!("Unexpected types")
}

// ✓ Good: handles all numeric types
match (l, r) {
    (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
    (Value::F32(a), Value::F32(b)) => Value::F32(a + b),
    (Value::I64(a), Value::I64(b)) => Value::I64(a + b),
    _ => Err(TypeError::InvalidOperation)
}
```

### 3. Breaking Semi-Naive Evaluation

New constructs must preserve monotonicity for correctness.

---

## Summary

To add a new language feature:
1. Update grammar in `grammar.lalrpop`
2. Add AST nodes in `ast.rs`
3. Implement type checking in `type_check/`
4. Generate IR in `back/`
5. Implement execution in `runtime/`
6. Write comprehensive tests in `tests/integrate/`

For more details:
- [Developer Guide](index.md) - Architecture overview
- [Bindings](binding.md) - Language bindings
- [LALRPOP Book](http://lalrpop.github.io/lalrpop/) - Parser generator
