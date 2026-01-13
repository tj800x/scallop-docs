# Installing Scallop Plugins

This guide covers how to install and configure Scallop plugins for your environment.

## Prerequisites

Before installing plugins, ensure you have:

### 1. Python Environment

**Python 3.8 or higher:**
```bash
python --version  # Should show Python 3.8+
```

**Virtual environment (recommended):**
```bash
# Create virtual environment
python -m venv scallop-env

# Activate it
source scallop-env/bin/activate  # On macOS/Linux
# or
scallop-env\Scripts\activate     # On Windows
```

### 2. Scallopy Installed

Plugins require the `scallopy` Python package:

```bash
# Install from PyPI
pip install scallopy

# Or install from source
git clone https://github.com/scallop-lang/scallop.git
cd scallop/etc/scallopy
pip install -e .
```

### 3. Additional Dependencies

Some plugins have specific requirements:

- **GPU Plugins** (CLIP, SAM, Face Detection):
  - CUDA-compatible GPU (optional but recommended)
  - PyTorch with CUDA support

- **API-based Plugins** (GPT, Gemini):
  - API keys (see Configuration section)

- **CodeQL Plugin**:
  - GitHub CodeQL CLI installed separately

## Installing Plugins

### Method 1: Install All Plugins (Easiest)

From the Scallop repository root:

```bash
cd /path/to/scallop
make -C etc/scallopy-plugins develop
```

This installs all 11 plugins in development mode, allowing you to modify source code without reinstalling.

### Method 2: Install Specific Plugins

**Install individual plugin with make:**
```bash
make -C etc/scallopy-plugins develop-gpt
make -C etc/scallopy-plugins develop-clip
make -C etc/scallopy-plugins develop-gpu
```

**Or install directly with pip:**
```bash
cd /path/to/scallop/etc/scallopy-plugins/gpt
pip install -e .

cd ../clip
pip install -e .
```

### Method 3: Build and Install Wheels

For production environments, build wheel files:

```bash
# Build all plugins
make -C etc/scallopy-plugins build

# Install from wheels
make -C etc/scallopy-plugins install

# Or install specific wheel
pip install etc/scallopy-plugins/gpt/dist/scallop-gpt-*.whl
```

### Development vs Production Installation

| Installation Type | Command | Use Case | Editable |
|-------------------|---------|----------|----------|
| **Development** | `make develop` or `pip install -e .` | Active development, testing | Yes |
| **Production** | `make install` or `pip install dist/*.whl` | Deployment, distribution | No |

**Development mode** creates a symlink - changes to source code take effect immediately.
**Production mode** copies files - requires reinstall after changes.

## Configuration

### Environment Variables

Many plugins require configuration via environment variables:

#### GPT Plugin

```bash
export OPENAI_API_KEY="sk-..."
```

**Get an API key:** https://platform.openai.com/api-keys

#### Gemini Plugin

```bash
export GEMINI_API_KEY="your-api-key-here"
```

**Get an API key:** https://aistudio.google.com/app/apikey

#### CodeQL Plugin

```bash
export CODEQL_PATH="/path/to/codeql-cli"
```

**Install CodeQL:** https://github.com/github/codeql-cli-binaries

### Persistent Configuration

Add environment variables to your shell profile:

**Bash/Zsh (~/.bashrc or ~/.zshrc):**
```bash
export OPENAI_API_KEY="sk-..."
export GEMINI_API_KEY="your-key..."
export CODEQL_PATH="/usr/local/bin/codeql"
```

**Or use a .env file:**
```bash
# .env
OPENAI_API_KEY=sk-...
GEMINI_API_KEY=your-key...

# Load with
source .env
```

### Command-Line Arguments

Most plugins support configuration via CLI arguments:

```bash
# GPT configuration
scli program.scl \
  --openai-gpt-model gpt-4 \
  --openai-gpt-temperature 0.0 \
  --num-allowed-openai-request 50

# GPU configuration
scli program.scl --cuda --gpu 0

# CLIP configuration
scli program.scl --clip-model-checkpoint ViT-L/14
```

**List available arguments:**
```bash
scli --help
```

### Python Configuration

When using the Python API, configure plugins programmatically:

```python
import scallopy

# Create context
ctx = scallopy.ScallopContext(provenance="minmaxprob")

# Create plugin registry
plugin_registry = scallopy.PluginRegistry(load_stdlib=True)

# Load plugins from installed packages
plugin_registry.load_plugins_from_entry_points()

# Configure with arguments
plugin_registry.configure({
    "openai_gpt_model": "gpt-4",
    "num_allowed_openai_request": 50,
    "cuda": True,
    "gpu": 0
}, [])

# Load into context
plugin_registry.load_into_ctx(ctx)
```

## Verification

### Check Installed Plugins

**From command line:**
```bash
python -c "import scallopy; registry = scallopy.PluginRegistry(); registry.load_plugins_from_entry_points(); print(registry.loaded_plugins())"
```

**Expected output:**
```
['gpt', 'gemini', 'clip', 'sam', 'transformers', 'opencv', 'face-detection', 'plip', 'codeql', 'gpu']
```

**In Python:**
```python
import scallopy

registry = scallopy.PluginRegistry()
registry.load_plugins_from_entry_points()

print("Loaded plugins:", registry.loaded_plugins())
```

### Test a Plugin

**Test GPT plugin:**
```python
import os
import scallopy

# Set API key
os.environ["OPENAI_API_KEY"] = "sk-..."

# Create context with GPT plugin
ctx = scallopy.ScallopContext()
registry = scallopy.PluginRegistry()
registry.load_plugins_from_entry_points()
registry.configure({}, [])
registry.load_into_ctx(ctx)

# Run simple test
ctx.add_program("""
  rel question = {"What is 2+2?"}
  rel answer(q, a) = question(q), gpt(q, a)
  query answer
""")
ctx.run()

result = list(ctx.relation("answer"))
print("GPT response:", result)
```

### Test CLIP plugin:
```python
import scallopy

ctx = scallopy.ScallopContext()
registry = scallopy.PluginRegistry()
registry.load_plugins_from_entry_points()
registry.configure({}, [])
registry.load_into_ctx(ctx)

# This will trigger model download on first run
ctx.add_program("""
  @clip(labels=["cat", "dog"])
  rel classify(img: Tensor, label: String)

  rel test_img = {$load_image("test.jpg")}
  rel result(img, label) = test_img(img), classify(img, label)
  query result
""")
ctx.run()
```

## Troubleshooting

### Plugin Not Found

**Error:** `Plugin 'gpt' not found`

**Solution:** Install the plugin
```bash
make -C etc/scallopy-plugins develop-gpt
```

### Import Error

**Error:** `ModuleNotFoundError: No module named 'scallopy'`

**Solution:** Install scallopy first
```bash
pip install scallopy
```

### API Key Not Set

**Error:** `OpenAI API key not found`

**Solution:** Set environment variable
```bash
export OPENAI_API_KEY="sk-..."
```

### CUDA Out of Memory

**Error:** `RuntimeError: CUDA out of memory`

**Solution:** Use CPU or smaller batch sizes
```bash
scli program.scl  # Use CPU (default)
# or reduce model size
scli program.scl --clip-model-checkpoint ViT-B/32  # Smaller model
```

### Model Download Fails

**Error:** `Failed to download model checkpoint`

**Solution:** Check internet connection or download manually
```bash
# CLIP models are cached in ~/.cache/clip/
# SAM models in ~/.cache/torch/hub/
# HuggingFace models in ~/.cache/huggingface/
```

## Next Steps

- **Try a plugin** - Follow the [GPT Plugin Guide](openai_gpt.md) or [CLIP Guide](vision_models.md)
- **Learn the APIs** - Read about [Foreign Functions](foreign_functions.md), [Predicates](foreign_predicates.md), and [Attributes](foreign_attributes.md)
- **Create your own** - Follow the [Plugin Development Guide](create_your_own_plugin.md)

For more help, see the [Plugin Reference](references.md) page.
