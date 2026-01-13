# Scallop Interpreter (scli)

`scli` is the Scallop interpreter for running Scallop programs from `.scl` files. It supports probabilistic reasoning, debugging, and query execution.

## Basic Usage

```bash
scli <input-file>
```

**Example:**
```bash
scli program.scl
```

---

## Options

### Provenance Configuration

**`--provenance` / `-p`** - Set the provenance type

```bash
scli --provenance minmaxprob program.scl
scli -p topkproofs program.scl
```

Available provenances: `unit`, `proofs`, `minmaxprob`, `addmultprob`, `topkproofs`, `probproofs`, etc.

Default: `unit` (discrete DataLog)

**`--top-k` / `-k`** - Set K value for top-K provenances

```bash
scli --provenance topkproofs --top-k 5 program.scl
scli -p topkproofs -k 10 program.scl
```

Default: `3`

### Query Options

**`--query` / `-q`** - Query a specific relation

```bash
scli --query path program.scl
```

Without this flag, all `query` declarations in the file are executed.

**`--output-all`** - Output all relations (including hidden ones)

```bash
scli --output-all program.scl
```

### Execution Control

**`--iter-limit`** - Set iteration limit for recursion

```bash
scli --iter-limit 100 program.scl
```

Useful for preventing infinite loops in recursive programs.

**`--stop-at-goal`** - Stop when goal relation is derived

```bash
scli --stop-at-goal program.scl
```

Terminates execution as soon as the goal relation has facts.

**`--no-early-discard`** - Disable early discarding

```bash
scli --no-early-discard program.scl
```

Keeps all intermediate results instead of discarding low-probability facts.

### Optimization Options

**`--do-not-remove-unused-relations`** - Keep unused relations

```bash
scli --do-not-remove-unused-relations program.scl
```

By default, relations not used in queries are removed for efficiency.

**`--wmc-with-disjunctions`** - Use WMC for disjunctions

```bash
scli --wmc-with-disjunctions program.scl
```

Enables weighted model counting with disjunctive facts for better probability computation.

**`--scheduler`** - Set execution scheduler

```bash
scli --scheduler <scheduler-type> program.scl
```

Controls execution order of rules.

### Debugging Options

**`--debug` / `-d`** - Enable general debugging

```bash
scli --debug program.scl
```

Prints execution information and intermediate states.

**`--debug-front`** - Debug front-end IR

```bash
scli --debug-front program.scl
```

Shows intermediate representation after parsing.

**`--debug-back`** - Debug back-end IR

```bash
scli --debug-back program.scl
```

Shows intermediate representation before execution.

**`--debug-ram`** - Debug RAM program

```bash
scli --debug-ram program.scl
```

Shows the compiled RAM (Relational Algebra Machine) program.

**`--debug-runtime`** - Monitor runtime execution

```bash
scli --debug-runtime program.scl
```

Prints detailed execution traces.

**`--debug-tag`** - Monitor tag propagation

```bash
scli --debug-tag program.scl
```

Shows how provenance tags propagate through execution.

### Other Options

**`--version` / `-V`** - Print version

```bash
scli --version
```

**`--help` / `-h`** - Print help

```bash
scli --help
```

---

## Examples

### Basic Execution

```bash
# Run simple program
scli edge_path.scl
```

### Probabilistic Reasoning

```bash
# Run with min-max probability
scli --provenance minmaxprob uncertain_graph.scl

# Run with top-K proofs
scli -p topkproofs -k 5 uncertain_graph.scl
```

### Query Specific Relation

```bash
# Only output the 'result' relation
scli --query result computation.scl
```

### Debugging

```bash
# Debug execution
scli --debug program.scl

# Monitor runtime with tag propagation
scli --debug-runtime --debug-tag program.scl
```

### Performance Tuning

```bash
# Limit recursion depth
scli --iter-limit 50 recursive_program.scl

# Use WMC optimization
scli --wmc-with-disjunctions --provenance topkproofs program.scl
```

---

## Common Patterns

### Development

```bash
# Quick test
scli test.scl

# With debugging
scli --debug test.scl
```

### Testing Different Provenances

```bash
# Compare results
scli --provenance unit program.scl
scli --provenance minmaxprob program.scl
scli --provenance topkproofs -k 5 program.scl
```

### Production

```bash
# Optimized execution
scli --provenance topkproofs -k 10 \
     --wmc-with-disjunctions \
     --iter-limit 1000 \
     production.scl
```

---

## Summary

- **Basic**: `scli program.scl`
- **Provenance**: `-p <type>` and `-k <value>`
- **Query**: `-q <relation>` for specific output
- **Debug**: `--debug*` flags for troubleshooting
- **Optimize**: `--iter-limit`, `--wmc-with-disjunctions`
- **Version**: `scli --version` (current: 0.2.5)

For more details:
- [CLI Overview](cli.md) - All command-line tools
- [Probabilistic Programming](../probabilistic/index.md) - Provenance types
- [Language Reference](../language/index.md) - Scallop syntax
