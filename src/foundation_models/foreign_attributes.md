# Foreign Attributes

Foreign attributes are **metaprogramming decorators** that transform Scallop declarations at load time. They allow plugins to provide high-level abstractions that automatically generate foreign functions, foreign predicates, or complex rule patterns based on declaration annotations.

## What are Foreign Attributes?

### Definition

A foreign attribute is a **Python function** that:
- Processes Scallop declarations (relations, functions, types)
- Receives attribute parameters from user code
- Generates and registers foreign predicates/functions dynamically
- Can validate types and argument patterns

### Syntax

Foreign attributes are applied with `@` syntax:

```scl
@attribute_name(param1, param2, key=value)
rel relation_name(arg1: Type1, arg2: Type2)

@attribute_name(params)
type function_name(args) -> ReturnType
```

### Foreign Attributes vs Functions/Predicates

| Feature | Foreign Function | Foreign Predicate | Foreign Attribute |
|---------|------------------|-------------------|-------------------|
| **Applied to** | Called in expressions | Called as relations | Decorates declarations |
| **When runs** | Query execution | Query execution | Program load time |
| **Purpose** | Compute values | Generate facts | Transform declarations |
| **Output** | Single value | Facts | Function/predicate/nothing |
| **Example** | `$load_image(path)` | `gpt(input, output)` | `@gpt(prompts=[...])` |

### Key Characteristics

**Metaprogramming:**
- Runs at program load time, not query execution
- Can inspect and validate declaration structure
- Generates code dynamically

**High-level abstraction:**
- Wraps complex patterns with simple syntax
- Provides domain-specific language extensions
- Reduces boilerplate for common operations

**Type-aware:**
- Can check argument types and patterns
- Validates adornment (bound/free patterns)
- Ensures correct usage at load time

## How Attributes Work

### Attribute Lifecycle

1. **User writes Scallop program:**
   ```scl
   @gpt(header="Classify:", prompts=[...])
   rel classify(text: String, label: String)
   ```

2. **Scallop parser creates AST** with attribute attached to declaration

3. **Plugin's attribute processor is called:**
   ```python
   @scallopy.foreign_attribute
   def gpt(item, header, prompts):
       # Receives the declaration and parameters
       # Returns a foreign predicate or function
   ```

4. **Generated predicate/function is registered** in the context

5. **User code can call the generated construct:**
   ```scl
   rel result(t, l) = texts(t), classify(t, l)
   ```

### Attribute Parameters

Foreign attributes receive:

**Positional parameters:**
```scl
@clip(["cat", "dog", "bird"])  // labels list
```

**Keyword parameters:**
```scl
@gpt(header="Question:", model="gpt-4", temperature=0.0)
```

**In Python:**
```python
@scallopy.foreign_attribute
def my_attr(
    item,                    # The AST item being decorated
    pos_param,               # Positional parameter
    *,                       # Force keyword-only arguments
    key_param="default",     # Keyword parameter with default
    optional_param=None      # Optional parameter
):
    # Process item and parameters
    pass
```

### Inspecting Declarations

The `item` parameter provides access to the declaration structure:

**Check declaration type:**
```python
item.is_relation_decl()  # Is it a relation declaration?
item.is_function_decl()  # Is it a function declaration?
item.is_type_decl()      # Is it a type declaration?
```

**Access relation details:**
```python
relation_decl = item.relation_decl(0)
name = relation_decl.name.name  # Relation name
args = relation_decl.arg_bindings  # Argument list

for arg in args:
    arg_name = arg.name.name    # Argument name
    arg_type = arg.ty            # Type (String, Tensor, etc.)
    arg_adornment = arg.adornment  # Bound/free annotation
```

**Check argument adornment (bound/free pattern):**
```python
pattern = "".join([
    "b" if ab.adornment and ab.adornment.is_bound() else "f"
    for ab in relation_decl.arg_bindings
])
# Example patterns: "bf", "bbf", "bff"
```

### Returning Constructs

Attributes return what should replace the declaration:

**Return a foreign predicate:**
```python
@scallopy.foreign_attribute
def my_attr(item, ...):
    @scallopy.foreign_predicate(name=relation_name)
    def generated_predicate(...):
        # Implementation
        yield (tag, tuple)

    return generated_predicate
```

**Return a foreign function:**
```python
@scallopy.foreign_attribute
def my_attr(item, ...):
    @scallopy.foreign_function(name=function_name)
    def generated_function(...):
        return result

    return generated_function
```

**Return None (remove declaration):**
```python
@scallopy.foreign_attribute
def my_attr(item, ...):
    # Attribute has side effects but doesn't create a construct
    do_something_with(item)
    return None  # Declaration is removed from program
```

## Examples from Plugins

### GPT Plugin: @gpt Attribute

The `@gpt` attribute provides LLM-powered predicates with few-shot learning:

**Usage:**
```scl
@gpt(
  header="Classify the sentiment:",
  prompts=[
    {text: "I love this!", sentiment: "positive"},
    {text: "This is terrible.", sentiment: "negative"},
    {text: "It's okay.", sentiment: "neutral"}
  ],
  model="gpt-3.5-turbo",
  temperature=0.0
)
rel classify_sentiment(text: String, sentiment: String)

rel reviews = {
  "Amazing quality!",
  "Worst purchase ever.",
  "Not bad, could be better."
}

rel results(review, sent) = reviews(review), classify_sentiment(review, sent)
query results
```

**Expected output (mock when API key not set):**
```
results: {
  ("Amazing quality!", "positive"),
  ("Worst purchase ever.", "negative"),
  ("Not bad, could be better.", "neutral")
}
```

**Implementation details:**
```python
@scallopy.foreign_attribute
def gpt(
    item,
    prompt: str,
    *,
    header: str = "",
    examples: List[List[str]] = [],
    model: Optional[str] = None,
    debug: bool = False,
):
    # Validate: must be relation declaration
    assert item.is_relation_decl()

    # Extract relation info
    relation_decl = item.relation_decl(0)
    arg_names = [ab.name.name for ab in relation_decl.arg_bindings]
    arg_types = [ab.ty for ab in relation_decl.arg_bindings]

    # Check pattern: must be "b+f+" (one or more bound, one or more free)
    pattern = get_pattern(relation_decl.arg_bindings)
    assert re.match("^(b*)(f+)$", pattern), "Pattern must be bound* followed by free+"

    # Build prompt from header, examples, and user inputs
    # ...

    # Generate foreign predicate
    @scallopy.foreign_predicate(name=relation_decl.name.name)
    def invoke_gpt(*args):
        # Call OpenAI API with filled prompt
        # Parse response
        # Yield facts
        pass

    return invoke_gpt
```

### CLIP Plugin: @clip Attribute

The `@clip` attribute provides zero-shot image classification:

**Usage:**
```scl
@clip(
  labels=["cat", "dog", "bird", "car"],
  score_threshold=0.3
)
rel classify_image(img: Tensor, label: String)

rel images = {
  $load_image("photo1.jpg"),
  $load_image("photo2.jpg")
}

rel classifications(img, label) = images(img), classify_image(img, label)
query classifications
```

**With dynamic labels:**
```scl
@clip(score_threshold=0.5)
rel classify_dynamic(img: Tensor, labels: String, label: String)

rel image = {$load_image("photo.jpg")}
rel labels_str = {"cat;dog;bird;fish"}  // Semicolon-separated
rel result(img, label) = image(img), labels_str(ls), classify_dynamic(img, ls, label)
```

**Implementation highlights:**
```python
@scallopy.foreign_attribute
def clip(
    item,
    labels: Optional[List[str]] = None,
    *,
    score_threshold: float = 0,
    unknown_class: str = "?",
    debug: bool = False,
):
    relation_decl = item.relation_decl(0)
    args = relation_decl.arg_bindings

    # Static labels: (img: Tensor, label: String)
    if labels is not None:
        assert len(args) == 2
        assert args[0].ty.is_tensor() and args[0].adornment.is_bound()
        assert args[1].ty.is_string() and args[1].adornment.is_free()

        @scallopy.foreign_predicate(name=relation_decl.name.name)
        def clip_classify(img: scallopy.Tensor):
            # Run CLIP model
            # Yield (probability, (label,)) for each class
            pass

        return clip_classify

    # Dynamic labels: (img: Tensor, labels: String, label: String)
    else:
        assert len(args) == 3
        # Similar but parse labels from input string
        pass
```

### Stdlib: @cmd_arg Attribute

The `@cmd_arg` attribute binds command-line arguments to relations:

**Usage:**
```scl
@cmd_arg("-n", long="--num-iterations", default=10)
rel num_iterations(n: i32)

// Run: scli program.scl --num-iterations 20
// num_iterations: {(20,)}
```

**Implementation:**
```python
@foreign_attribute
def cmd_arg(item, short: str, *, long: Optional[str] = None, default: Optional[Any] = None):
    relation_type_decl = item.relation_decl(0)
    name = relation_type_decl.name.name

    # Must be arity-1
    assert len(relation_type_decl.arg_bindings) == 1
    arg_type = relation_type_decl.arg_bindings[0].ty

    # Create argument parser
    parser = ArgumentParser()
    if long is not None:
        parser.add_argument(short, long, default=default, type=arg_type.to_python_type())
    else:
        parser.add_argument(short, default=default, type=arg_type.to_python_type())

    @foreign_predicate(name=name, output_arg_types=[arg_type])
    def get_arg():
        args, _ = parser.parse_known_args(unknown_args)
        if len(args.__dict__) > 0:
            value = list(args.__dict__.values())[0]
            if value is not None:
                yield (value,)

    return get_arg
```

### Stdlib: @py_eval Attribute

The `@py_eval` attribute evaluates Python expressions:

**Usage:**
```scl
@py_eval
type eval_python(expr: String) -> i32

rel expressions = {"2 + 2", "10 * 5", "3 ** 4"}
rel results(expr, val) = expressions(expr), val = $eval_python(expr)
query results

// Result: {("2 + 2", 4), ("10 * 5", 50), ("3 ** 4", 81)}
```

**Implementation:**
```python
@foreign_attribute
def py_eval(item, *, suppress_warning=True):
    assert item.is_function_decl()

    name = item.function_decl_name()
    arg_types = item.function_decl_arg_types()
    ret_type = item.function_decl_ret_type()

    assert len(arg_types) == 1 and arg_types[0].is_string()

    @foreign_function(name=name, ret_type=ret_type)
    def python_evaluate(text: str):
        return eval(text, None, None)

    return python_evaluate
```

## Advanced Usage

### Pattern Validation

Ensure correct adornment patterns:

```python
@scallopy.foreign_attribute
def my_attr(item, ...):
    relation_decl = item.relation_decl(0)

    # Build pattern string
    pattern = "".join([
        "b" if ab.adornment and ab.adornment.is_bound() else "f"
        for ab in relation_decl.arg_bindings
    ])

    # Validate pattern
    if pattern == "bf":
        # Input-output pattern: good
        pass
    elif pattern == "bbf":
        # Two inputs, one output: good
        pass
    elif pattern == "ff":
        # No inputs: error
        raise ValueError("Attribute requires at least one bound argument")
    else:
        raise ValueError(f"Unsupported pattern: {pattern}")
```

### Type Checking

Validate argument types:

```python
@scallopy.foreign_attribute
def my_attr(item, ...):
    relation_decl = item.relation_decl(0)
    args = relation_decl.arg_bindings

    # Check first arg is Tensor
    assert args[0].ty.is_tensor(), "First argument must be Tensor"

    # Check all output args are String
    for arg in args[1:]:
        if not arg.adornment or arg.adornment.is_free():
            assert arg.ty.is_string(), "Output arguments must be String"
```

### Prompt Engineering

Build prompts from attribute parameters:

```python
def build_prompt(header, examples, user_inputs, arg_names):
    prompt = header + "\n\n"

    # Add few-shot examples
    for example in examples:
        example_str = ", ".join([
            f"{name}: {example[name]}"
            for name in arg_names
        ])
        prompt += f"Example: {example_str}\n"

    # Add user inputs
    input_str = ", ".join([
        f"{name}: {value}"
        for name, value in zip(arg_names, user_inputs)
    ])
    prompt += f"\nNow classify: {input_str}\n"
    prompt += "Answer:"

    return prompt
```

### Memoization at Attribute Level

Cache results across invocations:

```python
@scallopy.foreign_attribute
def cached_attr(item, ...):
    CACHE = {}  # Shared across all calls to generated predicate

    @scallopy.foreign_predicate(...)
    def cached_predicate(*args):
        key = tuple(args)
        if key not in CACHE:
            CACHE[key] = expensive_operation(*args)

        for result in CACHE[key]:
            yield result

    return cached_predicate
```

### Error Messages

Provide clear error messages with attribute name:

```python
ERR_HEAD = "[@my_attr]"

@scallopy.foreign_attribute
def my_attr(item, ...):
    assert item.is_relation_decl(), \
        f"{ERR_HEAD} must be applied to a relation declaration"

    assert len(item.relation_decls()) == 1, \
        f"{ERR_HEAD} cannot annotate multiple relations"

    relation_decl = item.relation_decl(0)
    args = relation_decl.arg_bindings

    assert len(args) >= 2, \
        f"{ERR_HEAD} requires at least 2 arguments, got {len(args)}"
```

## Best Practices

### Use Attributes for High-Level Patterns

**✓ Good - Complex pattern wrapped in attribute:**
```scl
@gpt(header="Extract name:", prompts=[...])
rel extract_name(text: String, name: String)
```

**✗ Bad - Manual implementation every time:**
```scl
rel extract_name(text, name) = text(text), gpt_raw(complex_prompt, name)
// User has to build prompt manually each time
```

### Validate Early

Fail fast at load time, not query time:

```python
@scallopy.foreign_attribute
def my_attr(item, param):
    # ✓ Check at load time
    assert item.is_relation_decl(), "Must be relation"
    assert param > 0, "Param must be positive"

    @scallopy.foreign_predicate(...)
    def pred(*args):
        # Don't check here - too late!
        pass
```

### Document Patterns

Clearly document supported patterns:

```python
@scallopy.foreign_attribute
def my_attr(item, ...):
    """
    Attribute for custom processing.

    Supported patterns:
    - (bound Tensor, free String) → bf pattern
    - (bound String, bound String, free String) → bbf pattern

    Example:
        @my_attr(param=value)
        rel classify(img: Tensor, label: String)
    """
    pass
```

### Keep Attribute Logic Simple

Attributes should orchestrate, not implement:

```python
# ✓ Good - delegate to helper functions
@scallopy.foreign_attribute
def my_attr(item, ...):
    validate_declaration(item)
    config = build_config(item, params)
    predicate = create_predicate(config)
    return predicate

# ✗ Bad - too much logic in attribute
@scallopy.foreign_attribute
def my_attr(item, ...):
    # 100 lines of complex logic here...
    pass
```

## Next Steps

- **[Foreign Functions](foreign_functions.md)** - Pure computational functions
- **[Foreign Predicates](foreign_predicates.md)** - Fact generators
- **[GPT Plugin](openai_gpt.md)** - Complete LLM integration with attributes
- **[CLIP Plugin](vision_models.md)** - Vision model attributes
- **[Create Your Own Plugin](create_your_own_plugin.md)** - Build custom plugins

For implementation details, see the [Plugin Development Guide](create_your_own_plugin.md).
