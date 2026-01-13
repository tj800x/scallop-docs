# Scallop Compiler (sclc)

`sclc` is the Scallop compiler for compiling Scallop programs into optimized executables or intermediate representations.

## Status

**Note:** The Scallop compiler is currently under development. This documentation describes planned features.

## Basic Usage

```bash
sclc [OPTIONS] <input-file>
```

**Example:**
```bash
sclc program.scl -o program
```

---

## Planned Features

### Compilation to Native Code

Compile Scallop programs to standalone executables:

```bash
sclc program.scl -o program
./program
```

### Intermediate Representations

Generate IR for inspection and optimization:

```bash
# Generate LLVM IR
sclc --emit-llvm program.scl

# Generate assembly
sclc --emit-asm program.scl

# Generate object file
sclc --emit-obj program.scl
```

### Optimization Levels

Control optimization:

```bash
# No optimization (fast compile)
sclc -O0 program.scl

# Basic optimization
sclc -O1 program.scl

# Full optimization (default)
sclc -O2 program.scl

# Aggressive optimization
sclc -O3 program.scl
```

### Static Analysis

Analyze programs without execution:

```bash
# Type checking
sclc --check program.scl

# Unused relation detection
sclc --warn-unused program.scl

# Complexity analysis
sclc --analyze program.scl
```

---

## Use Cases

### Production Deployment

Compile programs for production use:

```bash
sclc -O3 --static production.scl -o prod_binary
```

### Cross-Platform Builds

Target different platforms:

```bash
sclc --target x86_64-linux program.scl
sclc --target aarch64-macos program.scl
```

### Library Generation

Create reusable libraries:

```bash
sclc --lib program.scl -o libscallop_logic.a
```

---

## Current Alternative

While `sclc` is under development, use:

**For execution:** Use `scli` to run programs

```bash
scli program.scl
```

**For Python integration:** Use `scallopy`

```python
import scallopy
ctx = scallopy.ScallopContext()
# ...
```

**For optimization:** Use `scli` flags

```bash
scli --wmc-with-disjunctions \
     --iter-limit 1000 \
     program.scl
```

---

## Summary

- **Status**: Under development
- **Goal**: Compile Scallop to native code
- **Current**: Use `scli` for execution
- **Future**: Standalone executables, optimization, static analysis

For more details:
- [CLI Overview](cli.md) - All command-line tools
- [scli](scli.md) - Current execution method
- [GitHub Issues](https://github.com/scallop-lang/scallop/issues) - Track compiler development
