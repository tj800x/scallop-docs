# REPL

The Scallop REPL (Read-Eval-Print-Loop) provides an interactive environment for experimenting with Scallop programs.

## Quick Start

```bash
sclrepl
```

## What is a REPL?

A REPL is an interactive programming environment that:
- **Reads** your input (declarations, rules, queries)
- **Evaluates** the Scallop code
- **Prints** the results
- **Loops** back to read more input

## Basic Example

```
scallop> rel number = {1, 2, 3}
scallop> rel double(n, n * 2) = number(n)
scallop> query double
double: {(1, 2), (2, 4), (3, 6)}
```

## Why Use the REPL?

**Learning** - Experiment with syntax and semantics interactively

**Prototyping** - Develop logic incrementally before saving to files

**Debugging** - Test rules in isolation to identify issues

**Quick calculations** - Run one-off computations without creating files

## Common Commands

- `:help` - Show available commands
- `:quit` - Exit the REPL
- `:clear` - Reset all declarations
- `:provenance <type>` - Change provenance mode

## For More Details

See the [sclrepl documentation](sclrepl.md) for complete REPL usage including special commands, workflows, and advanced features.
