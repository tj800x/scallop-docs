# Scallop CLI

The Scallop toolchain provides command-line tools for working with Scallop programs. These tools enable you to run programs, experiment interactively, and compile Scallop code.

## Available Tools

Scallop includes three main command-line tools:

### scli - Scallop Interpreter

The primary tool for running Scallop programs from `.scl` files.

```bash
scli program.scl
```

**Use cases:**
- Execute Scallop programs
- Test and debug logic
- Run with different provenances
- Query specific relations

[Full documentation →](scli.md)

### sclrepl - Interactive REPL

An interactive Read-Eval-Print-Loop for experimenting with Scallop.

```bash
sclrepl
```

**Use cases:**
- Interactive exploration
- Quick prototyping
- Learning Scallop syntax
- Testing small programs

[Full documentation →](sclrepl.md)

### sclc - Scallop Compiler

Compiles Scallop programs (future feature).

```bash
sclc program.scl
```

**Use cases:**
- Compile to standalone executables
- Optimize performance
- Generate intermediate representations

[Full documentation →](sclc.md)

---

## Installation

### From Binary Releases

Download prebuilt binaries from the [GitHub releases page](https://github.com/scallop-lang/scallop/releases):

```bash
# Download and extract
tar -xzf scallop-<version>-<platform>.tar.gz

# Move to PATH
sudo mv scli sclrepl sclc /usr/local/bin/
```

### From Source

Build from source using Cargo:

```bash
# Clone repository
git clone https://github.com/scallop-lang/scallop.git
cd scallop

# Build release binaries
cargo build --release

# Binaries in target/release/
./target/release/scli --version
```

### Verify Installation

```bash
scli --version
# Output: scli 0.2.5
```

---

## Quick Start

### Running Your First Program

Create a file `hello.scl`:

```scallop
rel greeting = {"Hello", "Bonjour", "Hola"}
rel target = {"World", "Monde", "Mundo"}
rel message(g, t) = greeting(g), target(t)

query message
```

Run it:

```bash
scli hello.scl
```

Output:
```
message: {("Hello", "World"), ("Hello", "Monde"), ...}
```

### With Probabilistic Reasoning

Create `prob_example.scl`:

```scallop
rel 0.9::reliable_edge(0, 1)
rel 0.8::reliable_edge(1, 2)
rel path(a, b) = reliable_edge(a, b)
rel path(a, c) = path(a, b), reliable_edge(b, c)

query path
```

Run with provenance:

```bash
scli --provenance minmaxprob prob_example.scl
```

Output:
```
path: {0.9::(0, 1), 0.8::(1, 2), 0.8::(0, 2)}
```

---

## Common Workflows

### Development Workflow

1. **Write** - Create `.scl` file
2. **Test** - Run with `scli`
3. **Debug** - Add `--debug` flags
4. **Iterate** - Modify and re-run

### Experimentation Workflow

1. **Explore** - Use `sclrepl` for quick tests
2. **Prototype** - Develop logic interactively
3. **Save** - Export to `.scl` file
4. **Run** - Execute with `scli`

### Production Workflow

1. **Develop** - Write and test programs
2. **Integrate** - Use Python API (scallopy)
3. **Deploy** - Embed in applications
4. **Monitor** - Use debug flags if needed

---

## Summary

- **Three tools**: `scli` (interpreter), `sclrepl` (REPL), `sclc` (compiler)
- **scli** is the main tool for running `.scl` programs
- **sclrepl** provides interactive exploration
- **sclc** compiles programs (future)
- **Install** from releases or build from source

For detailed usage:
- [scli Documentation](scli.md) - Interpreter options and flags
- [sclrepl Documentation](sclrepl.md) - Interactive REPL guide
- [sclc Documentation](sclc.md) - Compiler usage
