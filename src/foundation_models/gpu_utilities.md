# GPU Utilities

The GPU utilities plugin provides **device management** for Scallop plugins that use PyTorch models. It enables GPU acceleration for vision and language models without requiring code changes in individual plugins.

## Overview

The GPU plugin acts as a **centralized device manager** that:
- Configures CUDA/CPU execution globally
- Provides device selection API for other plugins
- Enables GPU acceleration with simple flags
- Falls back to CPU when CUDA unavailable

### Supported Plugins

All vision and language model plugins use GPU utilities:
- **CLIP** - Image classification
- **SAM** - Image segmentation
- **ViLT** - Visual question answering
- **OWL-ViT** - Object detection
- **Face Detection** - Face localization
- **PLIP** - Protein-ligand analysis
- **Transformers** - RoBERTa text encoding

## Installation

```bash
# Install GPU plugin
cd /path/to/scallop
make -C etc/scallopy-plugins develop-gpu

# Or with pip
cd etc/scallopy-plugins/gpu
pip install -e .
```

**Dependencies:**
- `torch` - PyTorch with CUDA support (for GPU)
- NVIDIA GPU with CUDA drivers (optional)

## Configuration

### Command-Line Flags

```bash
# Use default GPU (cuda:0)
scli program.scl --cuda

# Use specific GPU
scli program.scl --cuda --gpu 1

# CPU only (default)
scli program.scl
```

### Python API

```python
import scallopy

ctx = scallopy.ScallopContext()
plugin_registry = scallopy.PluginRegistry()
plugin_registry.load_plugins_from_entry_points()

# Configure GPU
plugin_registry.configure({
    "cuda": True,
    "gpu": 0  # GPU ID (optional)
}, [])

plugin_registry.load_into_ctx(ctx)
```

## Usage Examples

### Example 1: CLIP with GPU

```bash
# CPU (slow)
scli vision.scl

# GPU (fast)
scli vision.scl --cuda
```

**Scallop program (`vision.scl`):**
```scl
@clip(labels=["cat", "dog", "bird"])
rel classify(img: Tensor, label: String)

rel images = {$load_image("photo.jpg")}
rel result(img, label) = images(img), classify(img, label)
query result
```

### Example 2: Multi-GPU System

```bash
# Use GPU 0
scli program1.scl --cuda --gpu 0 &

# Use GPU 1 (parallel execution)
scli program2.scl --cuda --gpu 1 &
```

### Example 3: Python Script

```python
import scallopy

# GPU configuration
ctx = scallopy.ScallopContext()
registry = scallopy.PluginRegistry()
registry.load_plugins_from_entry_points()
registry.configure({"cuda": True, "gpu": 0}, [])
registry.load_into_ctx(ctx)

# Run CLIP classification
ctx.add_program("""
  @clip(labels=["cat", "dog"])
  rel classify(img: Tensor, label: String)

  rel image = {$load_image("photo.jpg")}
  rel result(img, label) = image(img), classify(img, label)
  query result
""")

ctx.run()
print(list(ctx.relation("result")))
```

## Device Selection

### Available Devices

| Device String | Description |
|---------------|-------------|
| `cpu` | CPU execution (default, no GPU required) |
| `cuda` | Default CUDA device (usually cuda:0) |
| `cuda:0` | First GPU |
| `cuda:1` | Second GPU |
| `cuda:N` | N-th GPU |

### How It Works

1. **Plugin configuration**: GPU plugin sets global device
2. **Model loading**: Other plugins call `get_device()`
3. **Model placement**: Models moved to selected device
4. **Inference**: Computations run on configured device

**Code example (internal):**
```python
from scallop_gpu import get_device

# In CLIP plugin
device = get_device()  # Returns "cuda:0" or "cpu"
model.to(device)  # Move model to device
```

## Performance Comparison

### CLIP Classification

| Device | Time per Image | Speedup |
|--------|----------------|---------|
| CPU | ~500ms | 1x |
| CUDA (GPU) | ~50ms | **10x** |

### ViLT Visual QA

| Device | Time per Question | Speedup |
|--------|-------------------|---------|
| CPU | ~800ms | 1x |
| CUDA (GPU) | ~80ms | **10x** |

### Batch Processing

**10 images with CLIP:**
- CPU: ~5 seconds
- GPU: ~0.5 seconds

**Performance varies by:**
- Model size
- Image resolution
- GPU model (RTX 3090 > RTX 2060 > GTX 1080)

## Checking CUDA Availability

### Python Check

```python
import torch

print("CUDA available:", torch.cuda.is_available())
print("CUDA version:", torch.version.cuda)
print("GPU count:", torch.cuda.device_count())

if torch.cuda.is_available():
    for i in range(torch.cuda.device_count()):
        print(f"GPU {i}: {torch.cuda.get_device_name(i)}")
```

### Expected Output (with GPU):
```
CUDA available: True
CUDA version: 11.7
GPU count: 2
GPU 0: NVIDIA GeForce RTX 3090
GPU 1: NVIDIA GeForce RTX 2060
```

### Expected Output (CPU only):
```
CUDA available: False
CUDA version: None
GPU count: 0
```

## Troubleshooting

### CUDA Not Available

**Symptom:**
Program runs but uses CPU (slow)

**Check:**
```bash
python -c "import torch; print(torch.cuda.is_available())"
```

**If False:**
1. Install CUDA toolkit: https://developer.nvidia.com/cuda-downloads
2. Install PyTorch with CUDA:
   ```bash
   pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118
   ```
3. Verify NVIDIA drivers: `nvidia-smi`

### Out of Memory

**Error:**
```
RuntimeError: CUDA out of memory
```

**Solutions:**
1. **Use smaller models**: e.g., CLIP ViT-B/32 instead of ViT-L/14
2. **Process fewer images**: Reduce batch size
3. **Use different GPU**: `--cuda --gpu 1`
4. **Clear cache**: `torch.cuda.empty_cache()`
5. **Fall back to CPU**: Remove `--cuda` flag

### Wrong GPU Selected

**Symptom:**
Using GPU 0 but want GPU 1

**Solution:**
```bash
# Explicit GPU selection
scli program.scl --cuda --gpu 1

# Or in Python
registry.configure({"cuda": True, "gpu": 1}, [])
```

### Multiple Processes

**Issue:**
Two processes trying to use same GPU

**Solution:**
```bash
# Process 1 on GPU 0
scli program1.scl --cuda --gpu 0 &

# Process 2 on GPU 1
scli program2.scl --cuda --gpu 1 &
```

## Best Practices

### Development Workflow

```bash
# Development (fast iteration, CPU)
scli test.scl

# Production (fast execution, GPU)
scli test.scl --cuda
```

### Resource Management

**✓ Good:**
- Use GPU for production workloads
- Use CPU for quick tests
- Select specific GPU in multi-GPU systems

**✗ Avoid:**
- Loading large models on CPU in production
- Forgetting `--cuda` flag for performance-critical tasks
- Running multiple heavy models on same GPU

### Memory Management

**Monitor GPU memory:**
```bash
nvidia-smi  # Shows GPU usage
watch -n 1 nvidia-smi  # Real-time monitoring
```

**Free memory in Python:**
```python
import torch
torch.cuda.empty_cache()
```

## Integration with Other Plugins

All vision/language plugins automatically use GPU utilities:

```bash
# All these benefit from --cuda flag
scli clip_classification.scl --cuda
scli vilt_vqa.scl --cuda
scli plip_analysis.scl --cuda
scli transformers_detection.scl --cuda
```

**No code changes needed!** Just add the `--cuda` flag.

## Next Steps

- **[CLIP Plugin](vision_models.md)** - Image classification with GPU
- **[Transformers Plugin](transformers.md)** - Vision-language models
- **[Create Your Own Plugin](create_your_own_plugin.md)** - Use GPU utilities in custom plugins

For PyTorch CUDA documentation, see [PyTorch Docs](https://pytorch.org/docs/stable/cuda.html).
