# CodeQL Plugin

The CodeQL plugin integrates GitHub's CodeQL static analysis engine into Scallop, enabling code pattern detection, security analysis, and software engineering queries within logical programs.

## Overview

The CodeQL plugin provides a **single foreign attribute** for database extraction:

- **`@codeql_database`** - Extract CodeQL database information into Scallop relations

### Key Features

- **Static code analysis**: Query code structure and patterns
- **Security analysis**: Detect vulnerabilities and code smells
- **Local execution**: Runs CodeQL CLI locally, no API required
- **Java support**: Currently supports Java codebases
- **Extensible**: Add custom CodeQL queries

### Use Cases

- **Vulnerability detection**: Find security issues in code
- **Code pattern analysis**: Identify coding patterns
- **Dependency tracking**: Analyze method and class relationships
- **Data flow analysis**: Track data through the codebase
- **Software metrics**: Compute complexity and quality metrics

## Installation

###Prerequisites

**1. Install CodeQL CLI:**

```bash
# Download from GitHub
curl -L https://github.com/github/codeql-cli-binaries/releases/latest/download/codeql-osx64.zip -o codeql.zip
unzip codeql.zip
mv codeql /usr/local/bin/

# Verify installation
codeql --version
```

**2. Install CodeQL plugin:**

```bash
# Install plugin
cd /path/to/scallop
make -C etc/scallopy-plugins develop-codeql

# Or with pip
cd etc/scallopy-plugins/codeql
pip install -e .
```

**3. Set CodeQL path:**

```bash
export CODEQL_PATH="/usr/local/bin/codeql"
```

### Creating a CodeQL Database

Before using the plugin, create a CodeQL database for your project:

```bash
# For Java projects
codeql database create my-java-db \
  --language=java \
  --source-root=/path/to/java/project

# For Python projects
codeql database create my-python-db \
  --language=python \
  --source-root=/path/to/python/project
```

**Database structure:**
```
my-java-db/
├── codeql-database.yml      # Database metadata
├── db-java/                  # Language-specific data
└── scallop_codeql/          # Cache directory (created by plugin)
```

## @codeql_database Foreign Attribute

### Syntax

```scl
@codeql_database(debug: bool = false)
rel get_class_definition(class_id: String, class_name: String, ...)
rel get_method_definition(method_id: String, method_name: String, ...)
rel get_local_dataflow_edge(from_node: String, to_node: String)
rel get_dataflow_node(node_id: String, node_type: String, ...)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `debug` | bool | `false` | Print debugging information |

### Available Relations

| Relation | Description | Columns |
|----------|-------------|---------|
| `get_class_definition` | Extract Java class definitions | `class_id`, `class_name`, `package`, `source_file` |
| `get_method_definition` | Extract Java method definitions | `method_id`, `method_name`, `class_id`, `return_type`, `parameters` |
| `get_local_dataflow_edge` | Extract local data flow edges | `from_node`, `to_node` |
| `get_dataflow_node` | Extract data flow nodes | `node_id`, `node_type`, `node_value` |

## Examples

### Example 1: Class Hierarchy Analysis

```scl
@codeql_database(debug=false)
rel get_class_definition(class_id: String, class_name: String, package: String, source_file: String)

// Find all classes in specific package
rel security_classes(cid, cname) =
  get_class_definition(cid, cname, pkg, _),
  pkg == "com.example.security"

query security_classes
```

**Expected output (mock when database not available):**
```
security_classes: {
  ("class_001", "AuthenticationManager"),
  ("class_002", "EncryptionService"),
  ("class_003", "AccessController")
}
```

### Example 2: Method Dependency Analysis

```scl
@codeql_database
rel get_method_definition(method_id: String, method_name: String, class_id: String, return_type: String)
rel get_class_definition(class_id: String, class_name: String, package: String, source_file: String)

// Find all methods in a specific class
rel controller_methods(mname, rtype) =
  get_class_definition(cid, "UserController", _, _),
  get_method_definition(_, mname, cid, rtype)

query controller_methods
```

**Expected output (mock):**
```
controller_methods: {
  ("getUser", "User"),
  ("createUser", "void"),
  ("updateUser", "boolean"),
  ("deleteUser", "void")
}
```

### Example 3: Data Flow Tracking

```scl
@codeql_database
rel get_dataflow_node(node_id: String, node_type: String, node_value: String)
rel get_local_dataflow_edge(from_node: String, to_node: String)

// Track data flow through specific nodes
rel sources(nid) =
  get_dataflow_node(nid, "parameter", _)

rel sinks(nid) =
  get_dataflow_node(nid, "method_call", "execute")

rel path(from, to) =
  sources(from),
  get_local_dataflow_edge(from, to)

rel path(from, to) =
  path(from, mid),
  get_local_dataflow_edge(mid, to)

rel vulnerable_paths(src, sink) =
  path(src, sink),
  sinks(sink)

query vulnerable_paths
```

**Expected output (mock):**
```
vulnerable_paths: {
  ("node_001", "node_050"),  // User input → SQL execute
  ("node_003", "node_051")   // HTTP parameter → SQL execute
}
```

### Example 4: Security Vulnerability Detection

```scl
@codeql_database(debug=true)
rel get_class_definition(class_id: String, class_name: String, package: String, source_file: String)
rel get_method_definition(method_id: String, method_name: String, class_id: String, return_type: String)

// Find potentially insecure methods
rel database_methods(cid, mname) =
  get_class_definition(cid, cname, _, _),
  cname == "DatabaseManager",
  get_method_definition(_, mname, cid, _)

rel potentially_insecure(cid, mname) =
  database_methods(cid, mname),
  (mname == "executeQuery" or mname == "executeUpdate")

query potentially_insecure
```

## Running Scallop with CodeQL

### Command-Line Usage

```bash
# With explicit paths
scli analysis.scl \
  --codeql-db ./my-java-db \
  --codeql-path /usr/local/bin/codeql

# With environment variables
export CODEQL_PATH="/usr/local/bin/codeql"
scli analysis.scl --codeql-db ./my-java-db
```

### Python API Usage

```python
import scallopy

# Create context
ctx = scallopy.ScallopContext()

# Load plugins
plugin_registry = scallopy.PluginRegistry()
plugin_registry.load_plugins_from_entry_points()

# Configure CodeQL plugin
plugin_registry.configure({
    "codeql_db": "./my-java-db",
    "codeql_path": "/usr/local/bin/codeql"
}, [])

# Load into context
plugin_registry.load_into_ctx(ctx)

# Add program
ctx.add_program("""
  @codeql_database
  rel get_class_definition(class_id: String, class_name: String, package: String, source_file: String)

  rel classes(cid, cname) = get_class_definition(cid, cname, _, _)
  query classes
""")

# Run
ctx.run()

# Get results
for (cid, cname) in ctx.relation("classes"):
    print(f"Class: {cname} (ID: {cid})")
```

## Advanced Use Cases

### Pattern 1: Finding Code Smells

```scl
@codeql_database
rel get_class_definition(cid: String, cname: String, pkg: String, file: String)
rel get_method_definition(mid: String, mname: String, cid: String, rtype: String)

// Count methods per class
rel method_count(cid, count) =
  cid = get_class_definition(cid, _, _, _),
  count = count(mid: get_method_definition(mid, _, cid, _))

// Classes with too many methods (> 20)
rel god_classes(cid, cname, count) =
  method_count(cid, count),
  count > 20,
  get_class_definition(cid, cname, _, _)

query god_classes
```

### Pattern 2: Dependency Analysis

```scl
@codeql_database
rel get_class_definition(cid: String, cname: String, pkg: String, file: String)
rel get_method_definition(mid: String, mname: String, cid: String, rtype: String)

// Methods by package
rel package_methods(pkg, mname) =
  get_class_definition(cid, _, pkg, _),
  get_method_definition(_, mname, cid, _)

// Count methods per package
rel package_sizes(pkg, size) =
  pkg = get_class_definition(_, _, pkg, _),
  size = count(m: package_methods(pkg, m))

query package_sizes
```

### Pattern 3: Security Audit

```scl
@codeql_database
rel get_class_definition(cid: String, cname: String, pkg: String, file: String)
rel get_method_definition(mid: String, mname: String, cid: String, rtype: String)

// Find authentication-related methods
rel auth_methods(cid, mname) =
  get_class_definition(cid, cname, _, _),
  (cname == "AuthService" or cname == "SecurityManager"),
  get_method_definition(_, mname, cid, _)

// Methods that might need security review
rel needs_review(cid, cname, mname) =
  auth_methods(cid, mname),
  get_class_definition(cid, cname, _, _),
  (mname == "authenticate" or mname == "authorize" or mname == "login")

query needs_review
```

## Supported Languages

| Language | Status | Support Level |
|----------|--------|---------------|
| Java | ✅ Supported | Full (classes, methods, data flow) |
| Python | 🚧 Experimental | Partial |
| JavaScript | 🚧 Experimental | Partial |
| C/C++ | ❌ Not yet | Planned |
| Go | ❌ Not yet | Planned |

**Note:** Currently, only Java is fully supported with comprehensive relation extractors.

## Troubleshooting

### CodeQL CLI Not Found

**Error:**
```
[scallop_codeql] `codeql` executable not found under CODEQL_PATH
```

**Solution:**
```bash
# Set path
export CODEQL_PATH="/path/to/codeql"

# Or use command-line flag
scli program.scl --codeql-path /path/to/codeql
```

### Database Not Finalized

**Error:**
```
Incomplete CodeQL database; it is not finalised
```

**Solution:**
Ensure database creation completed successfully:
```bash
codeql database finalize my-java-db
```

### Unsupported Language

**Error:**
```
Unsupported project language `<language>`
```

**Solution:**
Use Java for now. Other languages are experimental:
```bash
# Create Java database
codeql database create my-java-db --language=java --source-root=./src
```

### Query Execution Fails

**Error:**
```
CodeQL failed to produce analysis result bqrs
```

**Solutions:**
1. Check database is valid: `codeql database info my-java-db`
2. Enable debug mode: `@codeql_database(debug=true)`
3. Verify CodeQL version: `codeql --version`

## Performance Considerations

### Database Size

CodeQL databases can be large:
- Small project (~100 files): ~50MB
- Medium project (~1000 files): ~500MB
- Large project (~10000 files): ~5GB

### Query Performance

- **First query**: Slow (minutes) - builds indexes
- **Subsequent queries**: Fast (seconds) - uses cache
- **Cache location**: `<database>/scallop_codeql/`

### Optimization Tips

1. **Limit relation scope**: Query specific packages/classes
2. **Use caching**: Results are cached in database directory
3. **Incremental analysis**: Update database instead of recreating

## Best Practices

### Database Management

**✓ Good practices:**
- Create database per project version
- Store databases with version control tags
- Keep databases up-to-date with code changes

**✗ Avoid:**
- Sharing databases across machines (architecture-specific)
- Using outdated databases (stale analysis)

### Query Design

**✓ Efficient queries:**
```scl
// Filter early
rel specific_classes(cid) =
  get_class_definition(cid, cname, "com.example", _),
  cname == "TargetClass"
```

**✗ Inefficient queries:**
```scl
// Filter late
rel all_classes(cid, cname) = get_class_definition(cid, cname, _, _)
rel specific_classes(cid) = all_classes(cid, "TargetClass")  // Loads all first!
```

## Next Steps

- **[GPU Utilities](gpu_utilities.md)** - Device management
- **[Create Your Own Plugin](create_your_own_plugin.md)** - Plugin development
- **[References](references.md)** - Plugin quick reference

For CodeQL documentation, see [CodeQL Docs](https://codeql.github.com/docs/).
