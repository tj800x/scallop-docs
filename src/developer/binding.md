# Language Bindings

This guide explains how to create bindings for Scallop in other programming languages. We'll focus on the Python bindings (scallopy) as a reference implementation.

## Overview

Language bindings expose Scallop's Rust API to other languages. The process involves:

1. **FFI Layer** - Foreign Function Interface in Rust
2. **Wrapper Layer** - Language-specific wrapper (Python, C, etc.)
3. **High-Level API** - Idiomatic API for the target language
4. **Integration** - Package and distribute

---

## Python Bindings Architecture

The Python bindings (scallopy) use PyO3 for Rust-Python interop.

### Architecture Layers

```
┌────────────────────────────────────────┐
│   Python User Code                     │
│   ctx = scallopy.ScallopContext()      │
└────────────────┬───────────────────────┘
                 │
┌────────────────▼───────────────────────┐
│   Python API Layer (scallopy/)         │
│   - ScallopContext                     │
│   - Module, Forward                    │
│   - Type conversions                   │
└────────────────┬───────────────────────┘
                 │
┌────────────────▼───────────────────────┐
│   PyO3 Bindings (src/)                 │
│   - #[pyclass], #[pyfunction]          │
│   - Rust ↔ Python conversions          │
└────────────────┬───────────────────────┘
                 │
┌────────────────▼───────────────────────┐
│   Scallop Core (scallop-core)         │
│   - Compiler, Runtime                  │
└────────────────────────────────────────┘
```

---

## PyO3 Basics

PyO3 is a Rust library for Python interop.

### Exposing Rust Structs to Python

```rust
use pyo3::prelude::*;

#[pyclass]
pub struct ScallopContext {
    internal: scallop_core::runtime::Context,
}

#[pymethods]
impl ScallopContext {
    #[new]
    fn new(provenance: Option<String>) -> PyResult<Self> {
        let prov = provenance.unwrap_or("unit".to_string());
        let internal = scallop_core::runtime::Context::new(&prov)?;
        Ok(Self { internal })
    }

    fn add_rule(&mut self, rule: String) -> PyResult<()> {
        self.internal.add_rule(&rule)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn run(&mut self) -> PyResult<()> {
        self.internal.run()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
```

### Module Definition

```rust
#[pymodule]
fn scallopy_internal(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ScallopContext>()?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}

#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

---

## Type Conversions

Converting between Rust and Python types is crucial.

### Rust → Python

```rust
use pyo3::types::{PyList, PyTuple};

impl ScallopContext {
    fn relation(&self, py: Python, name: &str) -> PyResult<PyObject> {
        let tuples = self.internal.relation(name)?;

        // Convert Vec<Tuple> to Python list
        let py_list = PyList::new(py, tuples.iter().map(|tuple| {
            match tuple.arity() {
                1 => tuple.get(0).to_py_object(py),
                _ => {
                    let items: Vec<PyObject> = tuple.iter()
                        .map(|v| v.to_py_object(py))
                        .collect();
                    PyTuple::new(py, items).to_object(py)
                }
            }
        }));

        Ok(py_list.to_object(py))
    }
}
```

### Python → Rust

```rust
impl ScallopContext {
    fn add_facts(&mut self, relation: String, facts: &PyAny) -> PyResult<()> {
        let py_list = facts.downcast::<PyList>()?;

        let rust_facts: Vec<Tuple> = py_list.iter()
            .map(|item| {
                if let Ok(py_tuple) = item.downcast::<PyTuple>() {
                    // Convert Python tuple to Rust Tuple
                    let values: Vec<Value> = py_tuple.iter()
                        .map(|v| python_to_value(v))
                        .collect::<Result<_, _>>()?;
                    Ok(Tuple::from(values))
                } else {
                    // Single value
                    Ok(Tuple::from(vec![python_to_value(item)?]))
                }
            })
            .collect::<Result<_, PyErr>>()?;

        self.internal.add_facts(&relation, rust_facts)?;
        Ok(())
    }
}

fn python_to_value(obj: &PyAny) -> PyResult<Value> {
    if let Ok(i) = obj.extract::<i32>() {
        Ok(Value::I32(i))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Value::F64(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Unsupported type"))
    }
}
```

---

## Handling Errors

Convert Rust errors to Python exceptions properly.

### Error Conversion

```rust
use pyo3::exceptions::{PyValueError, PyRuntimeError};

impl From<scallop_core::Error> for PyErr {
    fn from(err: scallop_core::Error) -> Self {
        match err {
            scallop_core::Error::CompileError(msg) => {
                PyValueError::new_err(format!("Compile error: {}", msg))
            }
            scallop_core::Error::RuntimeError(msg) => {
                PyRuntimeError::new_err(format!("Runtime error: {}", msg))
            }
            scallop_core::Error::TypeError(msg) => {
                PyValueError::new_err(format!("Type error: {}", msg))
            }
            _ => PyRuntimeError::new_err(err.to_string())
        }
    }
}
```

### Using Error Conversion

```rust
#[pymethods]
impl ScallopContext {
    fn add_rule(&mut self, rule: String) -> PyResult<()> {
        self.internal.add_rule(&rule)
            .map_err(|e| e.into())  // Automatically converts to PyErr
    }
}
```

---

## PyTorch Integration

Scallopy integrates with PyTorch for differentiable reasoning.

### Tensor Conversion

```rust
use pyo3::types::PyAny;

fn python_tensor_to_rust(tensor: &PyAny) -> PyResult<Vec<f32>> {
    // Get tensor as numpy array
    let numpy = tensor.call_method0("cpu")?.call_method0("numpy")?;

    // Extract values
    let values: Vec<f32> = numpy.extract()?;
    Ok(values)
}

fn rust_tensor_to_python(py: Python, values: Vec<f32>, shape: Vec<usize>) -> PyResult<PyObject> {
    // Import torch
    let torch = py.import("torch")?;

    // Create tensor
    let tensor = torch.call_method1("tensor", (values,))?
        .call_method1("reshape", (shape,))?;

    Ok(tensor.to_object(py))
}
```

### Gradient Support

```rust
#[pyclass]
pub struct ScallopForward {
    context: ScallopContext,
    provenance: String,
}

#[pymethods]
impl ScallopForward {
    fn forward(&mut self, py: Python, inputs: &PyDict) -> PyResult<PyObject> {
        // Extract input tensors
        let rust_inputs = self.extract_inputs(inputs)?;

        // Run Scallop forward pass
        let outputs = self.context.forward(rust_inputs)?;

        // Convert to PyTorch tensors with gradient support
        self.create_output_tensors(py, outputs)
    }
}
```

---

## Building and Packaging

### Build Configuration

`Cargo.toml`:
```toml
[package]
name = "scallopy"
version = "0.2.5"
edition = "2021"

[lib]
name = "scallopy_internal"
crate-type = ["cdylib"]  # Create dynamic library for Python

[dependencies]
pyo3 = { version = "0.19", features = ["extension-module"] }
scallop-core = { path = "../../core" }
```

### Python Setup

`setup.py` or `pyproject.toml`:
```python
# pyproject.toml
[build-system]
requires = ["maturin>=0.14,<0.15"]
build-backend = "maturin"

[project]
name = "scallopy"
version = "0.2.5"
requires-python = ">=3.8"
dependencies = ["torch>=1.13"]

[tool.maturin]
bindings = "pyo3"
module-name = "scallopy.scallopy_internal"
```

### Building

```bash
# Install maturin
pip install maturin

# Build in debug mode
maturin develop

# Build release wheel
maturin build --release

# Install from wheel
pip install target/wheels/scallopy-*.whl
```

---

## C Bindings

For languages without Rust interop, expose a C API.

### C Header Generation

```rust
// In src/c_api.rs

#[no_mangle]
pub extern "C" fn scallop_context_new(provenance: *const c_char) -> *mut Context {
    let prov_str = unsafe {
        assert!(!provenance.is_null());
        CStr::from_ptr(provenance).to_str().unwrap()
    };

    let context = Box::new(Context::new(prov_str).unwrap());
    Box::into_raw(context)
}

#[no_mangle]
pub extern "C" fn scallop_context_free(ctx: *mut Context) {
    if !ctx.is_null() {
        unsafe { Box::from_raw(ctx) };
    }
}

#[no_mangle]
pub extern "C" fn scallop_add_rule(ctx: *mut Context, rule: *const c_char) -> bool {
    let context = unsafe {
        assert!(!ctx.is_null());
        &mut *ctx
    };

    let rule_str = unsafe {
        assert!(!rule.is_null());
        CStr::from_ptr(rule).to_str().unwrap()
    };

    context.add_rule(rule_str).is_ok()
}
```

### C Header File

```c
// scallop.h

#ifndef SCALLOP_H
#define SCALLOP_H

#include <stdint.h>
#include <stdbool.h>

typedef struct ScallopContext ScallopContext;

ScallopContext* scallop_context_new(const char* provenance);
void scallop_context_free(ScallopContext* ctx);
bool scallop_add_rule(ScallopContext* ctx, const char* rule);
bool scallop_run(ScallopContext* ctx);

#endif
```

---

## Testing Bindings

### Python Tests

```python
# tests/test_context.py

import scallopy

def test_basic_program():
    ctx = scallopy.ScallopContext()
    ctx.add_relation("edge", (int, int))
    ctx.add_facts("edge", [(0, 1), (1, 2)])
    ctx.add_rule("path(a, b) = edge(a, b)")
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")
    ctx.run()

    result = list(ctx.relation("path"))
    assert (0, 1) in result
    assert (1, 2) in result
    assert (0, 2) in result

def test_probabilistic():
    ctx = scallopy.ScallopContext(provenance="minmaxprob")
    ctx.add_relation("edge", (int, int))
    ctx.add_facts("edge", [(0.8, (0, 1)), (0.9, (1, 2))])
    ctx.add_rule("path(a, b) = edge(a, b)")
    ctx.run()

    result = list(ctx.relation("path"))
    assert len(result) == 2
    assert result[0][0] == 0.8  # Probability
    assert result[0][1] == (0, 1)  # Tuple
```

### Integration Tests

```python
# tests/test_pytorch.py

import torch
import scallopy

def test_differentiable_forward():
    sum_2 = scallopy.ScallopForwardFunction(
        program="rel sum_2(a + b) = digit_a(a), digit_b(b)",
        provenance="difftopkproofs",
        input_mappings={"digit_a": list(range(10)), "digit_b": list(range(10))},
        output_mappings={"sum_2": list(range(19))}
    )

    digit_a = torch.randn(16, 10, requires_grad=True)
    digit_b = torch.randn(16, 10, requires_grad=True)

    result = sum_2(digit_a=digit_a, digit_b=digit_b)

    assert result.shape == (16, 19)
    assert result.requires_grad

    # Test gradient flow
    loss = result.sum()
    loss.backward()

    assert digit_a.grad is not None
    assert digit_b.grad is not None
```

---

## Summary

To create language bindings:
1. **Use FFI framework** - PyO3 for Python, cbindgen for C
2. **Convert types carefully** - Handle all Scallop types
3. **Map errors properly** - Convert to target language exceptions
4. **Test thoroughly** - Unit tests, integration tests, examples
5. **Package properly** - Use language-specific tools (maturin, setuptools)

For more details:
- [PyO3 Documentation](https://pyo3.rs/)
- [Developer Guide](index.md) - Architecture overview
- [Language Constructs](language_construct.md) - Implementing features
