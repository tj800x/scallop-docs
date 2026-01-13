# Plugin Quick Reference

This page provides quick reference tables for all documented Scallop plugins, common configuration patterns, and troubleshooting guidance.

## Plugin Overview

| Plugin | Purpose | API Key Required | Models Run Locally | GPU Support |
|--------|---------|------------------|-------------------|-------------|
| **GPT** | LLM text processing | ✅ OPENAI_API_KEY | ❌ Cloud API | N/A |
| **Gemini** | LLM text processing | ✅ GEMINI_API_KEY | ❌ Cloud API | N/A |
| **Transformers** | Vision & language models | ❌ No | ✅ Local | ✅ CUDA |
| **PLIP** | Protein-ligand analysis | ❌ No | ✅ Local | ✅ CUDA |
| **CodeQL** | Static code analysis | ❌ No | ✅ Local | ❌ No |
| **GPU** | Device management | ❌ No | N/A | ✅ CUDA |

## Foreign Constructs by Plugin

### GPT Plugin

| Type | Name | Signature | Description |
|------|------|-----------|-------------|
| Function | `$gpt` | `String → String` | Simple text generation |
| Predicate | `gpt` | `(String, String)` | Fact generation with memoization |
| Attribute | `@gpt` | Various | Few-shot classification/extraction |
| Attribute | `@gpt_extract_info` | Various | Structured JSON extraction |
| Attribute | `@gpt_encoder` | `String → Tensor` | Text embeddings |

**Example:**
```scl
// Foreign function
rel answer = {$gpt("What is 2+2?")}

// Foreign attribute
@gpt(header="Classify:", prompts=[...])
rel classify(text: String, label: String)
```

### Gemini Plugin

| Type | Name | Signature | Description |
|------|------|-----------|-------------|
| Function | `$gemini` | `String → String` | Simple text generation |
| Predicate | `gemini` | `(String, String)` | Fact generation with memoization |
| Attribute | `@gemini` | Various | Few-shot classification/extraction |
| Attribute | `@gemini_extract_info` | Various | Structured JSON extraction |

**Example:**
```scl
// Foreign function
rel answer = {$gemini("Translate to French: Hello")}

// Foreign attribute
@gemini(header="Extract:", prompts=[...])
rel extract(text: String, entity: String)
```

### Transformers Plugin

| Type | Name | Signature | Description |
|------|------|-----------|-------------|
| Attribute | `@vilt` | `(Tensor, String, String)` | Visual question answering |
| Attribute | `@owl_vit` | `(Tensor, String, ...)` | Open-vocabulary object detection |
| Attribute | `@roberta_encoder` | `String → Tensor` | Text embeddings |

**Example:**
```scl
// ViLT for VQA
@vilt(question="What is in the image?", top=5)
rel answer(img: Tensor, ans: String)

// OWL-ViT for detection
@owl_vit(object_queries=["cat", "dog"], output_fields=["class", "bbox-x", "bbox-y"])
rel detect(img: Tensor, class: String, x: u32, y: u32)

// RoBERTa for embeddings
@roberta_encoder
type encode(text: String) -> Tensor
```

### PLIP Plugin

| Type | Name | Signature | Description |
|------|------|-----------|-------------|
| Attribute | `@plip` | `(Tensor, String)` | Protein-ligand classification |

**Example:**
```scl
@plip(labels=["active", "inactive"], score_threshold=0.5)
rel classify(img: Tensor, label: String)
```

### CodeQL Plugin

| Type | Name | Signature | Description |
|------|------|-----------|-------------|
| Attribute | `@codeql_database` | Various | Extract code analysis relations |

**Available Relations:**
- `get_class_definition(class_id, class_name, package, source_file)`
- `get_method_definition(method_id, method_name, class_id, return_type)`
- `get_local_dataflow_edge(from_node, to_node)`
- `get_dataflow_node(node_id, node_type, node_value)`

**Example:**
```scl
@codeql_database(debug=false)
rel get_class_definition(class_id: String, class_name: String, package: String, file: String)
```

### GPU Plugin

**No foreign constructs** - provides device management via configuration only.

## Configuration Reference

### Command-Line Arguments

| Plugin | Flag | Type | Default | Description |
|--------|------|------|---------|-------------|
| GPT | `--openai-gpt-model` | string | `gpt-3.5-turbo` | OpenAI model name |
| GPT | `--openai-gpt-temperature` | float | `0.0` | Sampling temperature |
| GPT | `--num-allowed-openai-request` | int | `100` | Request limit |
| Gemini | `--gemini-model` | string | `gemini-2.0-flash` | Gemini model name |
| Gemini | `--gemini-temperature` | float | `0.0` | Sampling temperature |
| Gemini | `--num-allowed-gemini-request` | int | `100` | Request limit |
| CodeQL | `--codeql-db` | string | - | Path to CodeQL database |
| CodeQL | `--codeql-path` | string | - | Path to CodeQL CLI |
| GPU | `--cuda` | flag | `false` | Enable CUDA |
| GPU | `--gpu` | int | `0` | GPU device ID |

### Environment Variables

| Plugin | Variable | Required | Description |
|--------|----------|----------|-------------|
| GPT | `OPENAI_API_KEY` | ✅ Yes | OpenAI API key from platform.openai.com |
| Gemini | `GEMINI_API_KEY` | ✅ Yes | Google Gemini key from aistudio.google.com |
| CodeQL | `CODEQL_PATH` | ⚠️ Optional | Path to CodeQL CLI (if not in PATH) |
| Weather (example) | `WEATHER_API_KEY` | ⚠️ Optional | For custom weather plugin |

### Python API Configuration

```python
import scallopy

ctx = scallopy.ScallopContext()
plugin_registry = scallopy.PluginRegistry()
plugin_registry.load_plugins_from_entry_points()

# Configure plugins
plugin_registry.configure({
    # GPT configuration
    "openai_gpt_model": "gpt-4",
    "openai_gpt_temperature": 0.0,
    "num_allowed_openai_request": 50,

    # Gemini configuration
    "gemini_model": "gemini-1.5-pro",
    "gemini_temperature": 0.0,

    # CodeQL configuration
    "codeql_db": "./my-java-db",
    "codeql_path": "/usr/local/bin/codeql",

    # GPU configuration
    "cuda": True,
    "gpu": 0,
}, [])

plugin_registry.load_into_ctx(ctx)
```

## Installation Quick Reference

### Install All Plugins

```bash
cd /path/to/scallop
make -C etc/scallopy-plugins develop
```

### Install Specific Plugin

```bash
# Using make
make -C etc/scallopy-plugins develop-gpt
make -C etc/scallopy-plugins develop-gemini
make -C etc/scallopy-plugins develop-transformers
make -C etc/scallopy-plugins develop-plip
make -C etc/scallopy-plugins develop-codeql
make -C etc/scallopy-plugins develop-gpu

# Using pip
cd etc/scallopy-plugins/gpt
pip install -e .
```

### Install from Wheels

```bash
# Build wheels
cd etc/scallopy-plugins
make wheel-<plugin_name>

# Install wheel
pip install dist/scallop_<plugin>-*.whl
```

## Common Patterns

### Pattern 1: Few-Shot Classification

**GPT/Gemini:**
```scl
@gpt(
  header="Classify the sentiment:",
  prompts=[
    {text: "Great!", sentiment: "positive"},
    {text: "Terrible", sentiment: "negative"},
    {text: "Okay", sentiment: "neutral"}
  ]
)
rel classify(text: String, sentiment: String)

rel reviews = {"Amazing product", "Waste of money"}
rel results(r, s) = reviews(r), classify(r, s)
query results
```

### Pattern 2: Information Extraction

**GPT with extract_info:**
```scl
@gpt_extract_info(
  header="Extract entities:",
  prompts=["Extract all people", "Extract all companies"],
  examples=[
    (
      ["Alice works at Google."],
      [[("Alice",)], [("Google",)]]
    )
  ]
)
rel person(text: String, name: String)
rel company(text: String, org: String)

rel text = {"Bob joined Microsoft."}
rel people(n) = text(t), person(t, n)
query people
```

### Pattern 3: Visual Question Answering

**ViLT:**
```scl
@vilt(question="What color is the car?", top=3)
rel answer_question(img: Tensor, answer: String)

rel image = {$load_image("photo.jpg")}
rel answers(a) = image(img), answer_question(img, a)
query answers
```

### Pattern 4: Object Detection

**OWL-ViT:**
```scl
@owl_vit(
  object_queries=["person", "car"],
  output_fields=["class", "bbox-x", "bbox-y", "bbox-w", "bbox-h"],
  score_threshold=0.3
)
rel detect(img: Tensor, cls: String, x: u32, y: u32, w: u32, h: u32)

rel image = {$load_image("street.jpg")}
rel detections(c, x, y, w, h) = image(img), detect(img, c, x, y, w, h)
query detections
```

### Pattern 5: Code Analysis

**CodeQL:**
```scl
@codeql_database
rel get_class_definition(cid: String, cname: String, pkg: String, file: String)
rel get_method_definition(mid: String, mname: String, cid: String, rtype: String)

// Find classes in specific package
rel security_classes(cid, cname) =
  get_class_definition(cid, cname, "com.example.security", _)

// Count methods per class
rel method_count(cid, count) =
  cid = get_class_definition(cid, _, _, _),
  count = count(mid: get_method_definition(mid, _, cid, _))

query security_classes
query method_count
```

### Pattern 6: GPU Acceleration

**Any vision/language plugin:**
```bash
# Use GPU for faster inference
scli program.scl --cuda --gpu 0

# Or in Python
plugin_registry.configure({"cuda": True, "gpu": 0}, [])
```

## Troubleshooting Guide

### API Key Issues

#### Error: "API key not found"

**Symptoms:**
```
[scallop_gpt] `OPENAI_API_KEY` not found in environment variable
```

**Solutions:**
```bash
# Set environment variable
export OPENAI_API_KEY="sk-..."
export GEMINI_API_KEY="your-key"

# Or use command-line flag (if supported)
scli program.scl --openai-api-key "sk-..."

# Verify
echo $OPENAI_API_KEY
```

#### Error: "Invalid API key"

**Solutions:**
1. Check key is correct (no extra spaces)
2. Verify key is active on provider website
3. Check account has credits/quota
4. Try regenerating key

### Model Loading Issues

#### Error: "Failed to download model"

**Symptoms:**
```
Failed to download model checkpoint from HuggingFace
```

**Solutions:**
```bash
# Check internet connection
ping huggingface.co

# Manually download
python -c "from transformers import ViltForQuestionAnswering; ViltForQuestionAnswering.from_pretrained('dandelin/vilt-b32-finetuned-vqa')"

# Check HuggingFace cache
ls ~/.cache/huggingface/hub/
```

#### Error: "Out of memory"

**Symptoms:**
```
RuntimeError: CUDA out of memory
```

**Solutions:**
1. Use CPU instead: Remove `--cuda` flag
2. Use smaller model checkpoints
3. Reduce batch size / top-k / limit parameters
4. Free GPU memory: `torch.cuda.empty_cache()`
5. Use different GPU: `--cuda --gpu 1`

### CodeQL Issues

#### Error: "codeql executable not found"

**Solutions:**
```bash
# Install CodeQL CLI
curl -L https://github.com/github/codeql-cli-binaries/releases/latest/download/codeql-osx64.zip -o codeql.zip
unzip codeql.zip
mv codeql /usr/local/bin/

# Set path
export CODEQL_PATH="/usr/local/bin/codeql"

# Or use flag
scli program.scl --codeql-path /usr/local/bin/codeql
```

#### Error: "Database not finalized"

**Solutions:**
```bash
# Finalize database
codeql database finalize my-java-db

# Verify
codeql database info my-java-db
```

### Rate Limiting

#### Error: "Exceeding allowed number of requests"

**Solutions:**
```bash
# Increase limit
scli program.scl --num-allowed-openai-request 200

# Or in Python
plugin_registry.configure({
    "num_allowed_openai_request": 200
}, [])
```

### GPU Issues

#### Error: "CUDA not available"

**Solutions:**
```bash
# Check CUDA availability
python -c "import torch; print(torch.cuda.is_available())"

# If False:
# 1. Install CUDA toolkit from nvidia.com
# 2. Install PyTorch with CUDA support
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118

# 3. Verify NVIDIA drivers
nvidia-smi
```

#### Error: "Wrong GPU selected"

**Solutions:**
```bash
# Use specific GPU
scli program.scl --cuda --gpu 1

# Or in Python
plugin_registry.configure({"cuda": True, "gpu": 1}, [])
```

### Plugin Not Found

#### Error: "Plugin not loaded"

**Solutions:**
```bash
# Verify plugin installed
pip list | grep scallop

# Reinstall plugin
cd etc/scallopy-plugins/gpt
pip install -e .

# Check entry points
python -c "import scallopy; print(scallopy.PluginRegistry().available_plugins())"
```

## Performance Tips

### Optimization Strategies

| Scenario | Recommendation | Improvement |
|----------|----------------|-------------|
| Vision models (CLIP, ViLT, PLIP) | Use GPU (`--cuda`) | ~10x faster |
| Multiple API calls | Rely on memoization | Automatic caching |
| Large batch processing | Increase request limits | Avoid premature stops |
| Slow model loading | Use lazy loading pattern | Faster startup |
| Repeated queries | Cache results externally | Reduce API costs |
| High memory usage | Use smaller models | Lower memory footprint |

### Model Size Comparison

| Model | Size | Speed (CPU) | Speed (GPU) | Use Case |
|-------|------|-------------|-------------|----------|
| ViLT | ~450MB | Medium | Fast | Visual QA |
| OWL-ViT | ~500MB | Slow | Medium | Object detection |
| RoBERTa-base | ~500MB | Fast | Very Fast | Text encoding |
| PLIP | ~600MB | Medium | Fast | Protein analysis |
| GPT-3.5 | API | N/A | N/A | General LLM tasks |
| Gemini Flash | API | N/A | N/A | Fast LLM tasks |

## Type Reference

### Scallop ↔ Python Type Mapping

| Scallop Type | Python Type | Example |
|--------------|-------------|---------|
| `i8`, `i16`, `i32`, `i64` | `int` | `42` |
| `u8`, `u16`, `u32`, `u64` | `int` | `100` |
| `f32`, `f64` | `float` | `3.14` |
| `bool` | `bool` | `True` |
| `String` | `str` | `"hello"` |
| `Tensor` | `torch.Tensor` | Image or embedding |
| `(T1, T2, ...)` | `tuple` | `(1, "a", 3.14)` |

### Common Output Type Patterns

```python
# Single output
@foreign_function(name="func")
def func(x: int) -> float:
    return x * 1.5

# Multiple outputs (use predicate)
@foreign_predicate(
    name="pred",
    input_arg_types=[int],
    output_arg_types=[float, str]
)
def pred(x: int) -> Facts[float, Tuple[float, str]]:
    yield (1.0, (x * 1.5, "result"))
```

## Next Steps

- **[Scallop Plugin System](scallop_plugins.md)** - Architecture overview
- **[Installation Guide](installation.md)** - Setup instructions
- **[Create Your Own Plugin](create_your_own_plugin.md)** - Plugin development tutorial
- **[Foreign Functions](foreign_functions.md)** - Function API reference
- **[Foreign Predicates](foreign_predicates.md)** - Predicate API reference
- **[Foreign Attributes](foreign_attributes.md)** - Attribute API reference

For more examples, see the `/examples/plugins/` directory.
