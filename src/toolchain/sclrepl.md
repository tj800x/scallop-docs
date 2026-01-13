# Scallop REPL

`sclrepl` is the interactive Read-Eval-Print-Loop for Scallop, allowing you to experiment with Scallop programs interactively.

## Starting the REPL

```bash
sclrepl
```

This starts an interactive session where you can enter Scallop declarations and queries.

---

## Basic Usage

### Entering Declarations

Type Scallop declarations directly:

```
scallop> rel edge = {(0, 1), (1, 2), (2, 3)}
```

### Defining Rules

```
scallop> rel path(a, b) = edge(a, b)
scallop> rel path(a, c) = path(a, b), edge(b, c)
```

### Querying Relations

```
scallop> query path
path: {(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)}
```

---

## REPL Commands

### Special Commands

**`:help`** - Show available commands

```
scallop> :help
```

**`:quit`** or **`:exit`** - Exit the REPL

```
scallop> :quit
```

**`:clear`** - Clear all declarations

```
scallop> :clear
```

**`:provenance <type>`** - Change provenance

```
scallop> :provenance minmaxprob
```

**`:relations`** - List all relations

```
scallop> :relations
```

---

## Interactive Workflow

### Experimentation

```
scallop> rel number = {1, 2, 3, 4, 5}
scallop> rel square(n, n * n) = number(n)
scallop> query square
square: {(1, 1), (2, 4), (3, 9), (4, 16), (5, 25)}
```

### Iterative Development

```
scallop> rel person = {"alice", "bob", "charlie"}
scallop> query person
person: {"alice", "bob", "charlie"}

scallop> rel friend(a, b) = person(a), person(b), a != b
scallop> query friend
friend: {("alice", "bob"), ("alice", "charlie"), ("bob", "alice"), ...}
```

### Probabilistic Reasoning

```
scallop> :provenance minmaxprob
scallop> rel 0.8::edge(0, 1)
scallop> rel 0.9::edge(1, 2)
scallop> rel path(a, b) = edge(a, b)
scallop> query path
path: {0.8::(0, 1), 0.9::(1, 2)}
```

---

## Tips and Tricks

### Multi-line Input

Use `\` for line continuation:

```
scallop> rel complicated_rule(x, y) = \
         some_relation(x), \
         another_relation(y), \
         x > y
```

### Inspecting Relations

```
scallop> :relations
Available relations:
  - edge: (i32, i32)
  - path: (i32, i32)
```

### Quick Testing

```
scallop> rel test = {1, 2, 3}
scallop> query test
test: {1, 2, 3}
scallop> :clear
scallop> # Start fresh
```

---

## Common Use Cases

### Learning Scallop

Quickly test syntax and semantics:

```
scallop> rel fact = {"a", "b", "c"}
scallop> query fact
```

### Prototyping

Develop logic interactively before saving to files:

```
scallop> # Try different rule formulations
scallop> rel version1(x) = ...
scallop> query version1
scallop> # Refine
scallop> rel version2(x) = ...
scallop> query version2
```

### Debugging

Test problematic rules in isolation:

```
scallop> :provenance unit
scallop> # Add minimal facts
scallop> rel edge = {(0, 1)}
scallop> # Test rule
scallop> rel path(a, b) = edge(a, b)
scallop> query path
```

---

## Summary

- **Start**: `sclrepl`
- **Enter** declarations, rules, and queries interactively
- **Commands**: `:help`, `:quit`, `:clear`, `:provenance`, `:relations`
- **Use for**: Learning, prototyping, debugging
- **Multi-line**: Use `\` for continuation

For more details:
- [CLI Overview](cli.md) - All command-line tools
- [scli](scli.md) - Running `.scl` files
- [Language Reference](../language/index.md) - Scallop syntax
