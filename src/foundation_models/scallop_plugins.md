# Scallop Plugins

Scallop plugins extend the language with external capabilities like large language models, vision processing, code analysis, and more. This guide explains how the plugin system works and what plugins are available.

## What are Scallop Plugins?

**Scallop plugins** are Python packages that extend Scallop's capabilities by registering **foreign functions**, **foreign predicates**, and **foreign attributes** into the Scallop runtime. They allow you to:

- **Integrate external APIs** - Connect to OpenAI GPT, Google Gemini, etc.
- **Process vision and images** - Use CLIP, SAM, face detection models
- **Analyze code** - Integrate GitHub CodeQL for static analysis
- **Configure execution** - Manage GPU/CPU device selection
- **Create domain-specific constructs** - Build specialized reasoning tools

Plugins work seamlessly with Scallop's probabilistic reasoning, provenance tracking, and logical inference capabilities.

### Three Extension Mechanisms

Plugins extend Scallop through three primary mechanisms:

1. **Foreign Functions** - Pure computations called with `$function_name(args)`
   ```scl
   rel result = {$gpt("Translate to French: Hello")}
   ```

2. **Foreign Predicates** - Generate facts with bound/free variable patterns
   ```scl
   rel answer(q, a) = question(q), gpt(q, a)
   ```

3. **Foreign Attributes** - Metaprogramming decorators like `@gpt`, `@clip`
   ```scl
   @gpt(header="Classify sentiment:", prompts=[...])
   rel classify_sentiment(text: String, sentiment: String)
   ```

## Plugin Architecture

### The Three-Hook Lifecycle

Every Scallop plugin implements three lifecycle hooks:

**1. `setup_argparse(parser)` - Declare Command-Line Arguments**
```python
def setup_argparse(self, parser):
    parser.add_argument("--gpt-model", type=str, default="gpt-3.5-turbo")
    parser.add_argument("--num-allowed-openai-request", type=int, default=100)
```

**2. `configure(args, unknown_args)` - Initialize Plugin State**
```python
def configure(self, args, unknown_args):
    import os
    self.api_key = os.getenv("OPENAI_API_KEY")
    self.model = args["gpt_model"]
```

**3. `load_into_ctx(ctx)` - Register Extensions**
```python
def load_into_ctx(self, ctx):
    ctx.register_foreign_function(my_function)
    ctx.register_foreign_predicate(my_predicate)
    ctx.register_foreign_attribute(my_attribute)
```

### Plugin Discovery

Plugins are discovered automatically via **Python entry points**:

```toml
# pyproject.toml
[project.entry-points."scallop.plugin"]
gpt = "scallop_gpt:ScallopGPTPlugin"
```

When you run a Scallop program, the plugin registry:
1. Discovers all installed plugins via entry points
2. Calls `setup_argparse()` to gather CLI arguments
3. Parses command-line arguments
4. Calls `configure()` to initialize plugins
5. Calls `load_into_ctx()` to register extensions
6. Runs your Scallop program with all extensions available

### Using Plugins in Python

```python
import scallopy

# Create context and plugin registry
ctx = scallopy.ScallopContext(provenance="minmaxprob")
plugin_registry = scallopy.PluginRegistry(load_stdlib=True)

# Load plugins from installed packages
plugin_registry.load_plugins_from_entry_points()

# Configure plugins with arguments
plugin_registry.configure({"gpt_model": "gpt-4"}, [])

# Load plugins into context
plugin_registry.load_into_ctx(ctx)

# Now use plugins in your program
ctx.add_program("""
  rel question = {"What is the capital of France?"}
  rel answer(q, a) = question(q), gpt(q, a)
  query answer
""")
ctx.run()
```

### Using Plugins with CLI

```bash
# Plugins are automatically loaded when using scli
scli program.scl --gpt-model gpt-4 --num-allowed-openai-request 10

# Set API keys via environment variables
export OPENAI_API_KEY="sk-..."
scli program.scl
```

## Available Plugins

Scallop provides 11 plugins across 4 categories:

### Language Models (API-based)

| Plugin | Description | API Key Required |
|--------|-------------|------------------|
| **GPT** | OpenAI GPT-3.5/4 integration for text generation, extraction, classification | Yes (`OPENAI_API_KEY`) |
| **Gemini** | Google Gemini 2.0 integration with similar capabilities to GPT | Yes (`GEMINI_API_KEY`) |

**Use cases:** Sentiment analysis, information extraction, text classification, question answering

### Vision Models (Local)

| Plugin | Description | Model Download |
|--------|-------------|----------------|
| **CLIP** | OpenAI CLIP for zero-shot image classification | Auto-download |
| **SAM** | Meta's Segment Anything Model for image segmentation | Auto-download (~2.5GB) |
| **Face Detection** | DSFD-based face localization and cropping | Auto-download |
| **OWL-ViT** | Open-vocabulary object detection via text queries | Auto-download |

**Use cases:** Image classification, object detection, segmentation, face recognition

### Utilities

| Plugin | Description | Purpose |
|--------|-------------|---------|
| **GPU** | Device management for CUDA/CPU selection | Configure execution device globally |
| **OpenCV** | Image I/O and manipulation (load, save, crop, transform) | Image processing pipelines |

**Use cases:** Load/save images, crop regions, GPU acceleration

### Specialized

| Plugin | Description | Domain |
|--------|-------------|--------|
| **Transformers** | HuggingFace models: ViLT (VQA), RoBERTa (text encoding) | Multi-modal AI |
| **PLIP** | Protein-ligand interaction prediction (fine-tuned CLIP) | Scientific computing |
| **CodeQL** | GitHub CodeQL integration for static code analysis | Software engineering |

**Use cases:** Visual question answering, protein analysis, vulnerability detection

## Getting Started

### Quick Example: Using GPT Plugin

**1. Install the plugin**
```bash
cd /path/to/scallop
make -C etc/scallopy-plugins develop-gpt
```

**2. Set API key**
```bash
export OPENAI_API_KEY="sk-..."
```

**3. Create a Scallop program**
```scl
// sentiment.scl
@gpt(
  header="Classify the sentiment:",
  prompts=[
    {text: "I love this!", sentiment: "positive"},
    {text: "This is terrible.", sentiment: "negative"}
  ]
)
rel classify_sentiment(text: String, sentiment: String)

rel reviews = {
  "Amazing product!",
  "Worst purchase ever.",
  "It's okay."
}

rel result(review, sent) = reviews(review), classify_sentiment(review, sent)
query result
```

**4. Run it**
```bash
scli sentiment.scl
```

**Expected output:**
```
result: {
  ("Amazing product!", "positive"),
  ("Worst purchase ever.", "negative"),
  ("It's okay.", "neutral")
}
```

### Quick Example: Vision with CLIP

**1. Install plugin**
```bash
make -C etc/scallopy-plugins develop-clip
```

**2. Create program**
```scl
// classify_images.scl
@clip(labels=["cat", "dog", "car", "person"])
rel classify(img: Tensor, label: String)

rel images = {$load_image("photo1.jpg"), $load_image("photo2.jpg")}
rel result(img, label) = images(img), classify(img, label)
query result
```

**3. Run it**
```bash
scli classify_images.scl --cuda  # Use GPU if available
```

## Common Workflows

### Workflow 1: LLM-Powered Classification

```scl
// 1. Define input data
rel documents = {
  "This product exceeded expectations.",
  "Delivery was slow and frustrating.",
  "Average quality for the price."
}

// 2. Use @gpt attribute for classification
@gpt(
  header="Classify as positive/negative/neutral:",
  prompts=[{text: "Great!", label: "positive"}]
)
rel classify(text: String, label: String)

// 3. Apply classification
rel classification(doc, label) = documents(doc), classify(doc, label)

// 4. Aggregate results
rel positive_count(n) = n = count(doc: classification(doc, "positive"))
rel negative_count(n) = n = count(doc: classification(doc, "negative"))

query positive_count
query negative_count
```

### Workflow 2: Vision Pipeline

```scl
// 1. Load images
rel image_paths = {"img1.jpg", "img2.jpg", "img3.jpg"}
rel loaded(path, img) = image_paths(path), img = $load_image(path)

// 2. Classify with CLIP
@clip(labels=["indoor", "outdoor"], score_threshold=0.7)
rel classify_scene(img: Tensor, scene: String)

rel scenes(path, scene) = loaded(path, img), classify_scene(img, scene)

// 3. Filter and analyze
rel outdoor_images(path) = scenes(path, "outdoor")

query outdoor_images
```

### Workflow 3: Multi-Plugin Integration

```scl
// Combine GPT + Transformers
rel questions = {"What color is the sky?", "How many people?"}

// Use ViLT for visual QA
@vilt(top=3)
rel visual_answer(img: Tensor, q: String, a: String)

// Use GPT to refine answers
@gpt(header="Summarize answer:")
rel refine(raw_answer: String, summary: String)

rel image = {$load_image("scene.jpg")}
rel raw_answers(q, a) = questions(q), image(img), visual_answer(img, q, a)
rel final_answers(q, s) = raw_answers(q, a), refine(a, s)

query final_answers
```

## Documentation Roadmap

- **[Installation](installation.md)** - How to install and configure plugins
- **[Foreign Functions](foreign_functions.md)** - Using and creating foreign functions
- **[Foreign Predicates](foreign_predicates.md)** - Using and creating foreign predicates
- **[Foreign Attributes](foreign_attributes.md)** - Using and creating foreign attributes
- **[GPT Plugin](openai_gpt.md)** - OpenAI GPT integration guide
- **[Gemini Plugin](gemini.md)** - Google Gemini integration guide
- **[Transformers Plugin](transformers.md)** - HuggingFace models (ViLT, OWL-ViT, RoBERTa)
- **[PLIP Plugin](plip.md)** - Protein-ligand analysis
- **[CodeQL Plugin](codeql.md)** - Code analysis integration
- **[GPU Utilities](gpu_utilities.md)** - Device management
- **[Creating Your Own Plugin](create_your_own_plugin.md)** - Plugin development guide
- **[Plugin Reference](references.md)** - Quick reference and troubleshooting

## Next Steps

1. **Install a plugin** - Start with [Installation Guide](installation.md)
2. **Try an example** - Pick a plugin from the list above and follow its guide
3. **Combine plugins** - Use multiple plugins together for complex reasoning
4. **Create your own** - Follow the [Plugin Development Guide](create_your_own_plugin.md)

For questions or issues, see the [References](references.md) page for troubleshooting tips.
