# PLIP Plugin

The PLIP (Protein-Ligand Interaction Prediction) plugin integrates a specialized vision-language model for analyzing protein and molecular structures. Built on CLIP architecture and fine-tuned for biomedical imaging, PLIP enables zero-shot classification of protein-ligand interactions and molecular properties.

## Overview

PLIP provides a **single foreign attribute** for molecular image analysis:

- **`@plip`** - Zero-shot protein-ligand classification

### Key Features

- **Domain-specific**: Fine-tuned for protein and ligand structures
- **Zero-shot learning**: No training data required for new classes
- **Probabilistic outputs**: Confidence scores for scientific analysis
- **GPU acceleration**: Automatic CUDA detection
- **Flexible prompts**: Customizable classification templates

### Use Cases

- **Protein classification**: Identify protein types from structures
- **Ligand binding prediction**: Classify binding modes
- **Molecular property prediction**: Predict properties from visualizations
- **Structure-activity relationships**: Analyze molecular interactions
- **Drug discovery**: Screen compounds based on structure

## Installation

```bash
# Install PLIP plugin
cd /path/to/scallop
make -C etc/scallopy-plugins develop-plip

# Or with pip
cd etc/scallopy-plugins/plip
pip install -e .
```

### Dependencies

- `transformers` - HuggingFace library
- `torch` - PyTorch for inference
- `PIL` - Image processing
- `scallop-gpu` - GPU management (optional)

**Install dependencies:**
```bash
pip install transformers torch pillow
```

### Model Download

The PLIP model (vinid/plip) is automatically downloaded on first use:

- **Size**: ~600MB
- **Cache**: `~/.cache/huggingface/hub/`
- **Architecture**: CLIP-based, fine-tuned on biomedical data

## @plip Foreign Attribute

### Syntax

```scl
@plip(
  labels: List[String] = None,
  prompt: String = None,
  score_threshold: float = 0.0,
  unknown_class: String = "?",
  debug: bool = false
)
rel relation_name(img: Tensor, label: String)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `labels` | List[String] | `None` | Classification labels (required) |
| `prompt` | String | `None` | Prompt template with `{{}}` placeholder |
| `score_threshold` | float | `0.0` | Minimum confidence score |
| `unknown_class` | String | `"?"` | Label for low-confidence predictions |
| `debug` | bool | `false` | Print debugging information |

## Examples

### Example 1: Protein Type Classification

```scl
@plip(
  labels=["kinase", "protease", "transcription factor", "receptor"],
  score_threshold=0.3
)
rel classify_protein(img: Tensor, protein_type: String)

rel protein_images = {
  $load_image("protein_1.png"),
  $load_image("protein_2.png"),
  $load_image("protein_3.png")
}

rel classifications(img, type) =
  protein_images(img),
  classify_protein(img, type)

query classifications
```

**Expected output (mock when model not available):**
```
classifications: {
  0.8::(image_tensor("protein_1.png"), "kinase"),
  0.75::(image_tensor("protein_2.png"), "protease"),
  0.65::(image_tensor("protein_3.png"), "receptor")
}
```

### Example 2: Binding Mode Classification

```scl
@plip(
  labels=["competitive", "non-competitive", "allosteric", "uncompetitive"],
  prompt="binding mode: {{}}",
  score_threshold=0.4
)
rel classify_binding(img: Tensor, mode: String)

rel ligand_complex_images = {
  $load_image("complex_A.png"),
  $load_image("complex_B.png")
}

rel binding_modes(img, mode) =
  ligand_complex_images(img),
  classify_binding(img, mode)

query binding_modes
```

**Expected output (mock):**
```
binding_modes: {
  0.7::(image_tensor("complex_A.png"), "competitive"),
  0.6::(image_tensor("complex_B.png"), "allosteric")
}
```

### Example 3: Molecular Property Prediction

```scl
@plip(
  labels=["hydrophobic", "hydrophilic", "amphipathic", "charged"],
  score_threshold=0.5
)
rel predict_property(img: Tensor, property: String)

rel molecule_structures = {
  $load_image("molecule_1.png"),
  $load_image("molecule_2.png"),
  $load_image("molecule_3.png"),
  $load_image("molecule_4.png")
}

rel properties(img, prop) =
  molecule_structures(img),
  predict_property(img, prop)

// Count molecules with each property
rel property_counts(prop, count) =
  prop = ["hydrophobic", "hydrophilic", "amphipathic", "charged"],
  count = count(img: properties(img, prop))

query property_counts
```

**Expected output (mock):**
```
property_counts: {
  ("hydrophobic", 2),
  ("hydrophilic", 1),
  ("amphipathic", 1),
  ("charged", 0)
}
```

### Example 4: Dynamic Labels

```scl
@plip(score_threshold=0.3)
rel classify_dynamic(img: Tensor, labels: String, class: String)

rel protein_image = {$load_image("protein.png")}
rel label_sets = {
  "alpha helix;beta sheet;random coil",
  "active;inactive;intermediate"
}

rel all_classifications(labels, class) =
  protein_image(img),
  label_sets(labels),
  classify_dynamic(img, labels, class)

query all_classifications
```

**Expected output (mock):**
```
all_classifications: {
  0.7::("alpha helix;beta sheet;random coil", "alpha helix"),
  0.5::("alpha helix;beta sheet;random coil", "beta sheet"),
  0.8::("active;inactive;intermediate", "active")
}
```

## Prompt Engineering for PLIP

### Basic Usage

Without prompt, use label directly:

```scl
@plip(labels=["enzyme", "antibody", "hormone"])
rel classify(img: Tensor, type: String)
```

Generated queries to model:
- "enzyme"
- "antibody"
- "hormone"

### With Prompt Template

Use `{{}}` placeholder for labels:

```scl
@plip(
  labels=["enzyme", "antibody", "hormone"],
  prompt="a protein structure showing: {{}}"
)
rel classify(img: Tensor, type: String)
```

Generated queries:
- "a protein structure showing: enzyme"
- "a protein structure showing: antibody"
- "a protein structure showing: hormone"

### Domain-Specific Prompts

```scl
@plip(
  labels=["ATP", "GTP", "NAD", "FAD"],
  prompt="this molecule is a cofactor of type: {{}}"
)
rel identify_cofactor(img: Tensor, cofactor: String)
```

## Scientific Applications

### Drug Discovery Pipeline

```scl
// Step 1: Classify protein target
@plip(labels=["GPCR", "ion channel", "enzyme", "nuclear receptor"])
rel classify_target(img: Tensor, target_type: String)

// Step 2: Predict binding site
@plip(labels=["orthosteric", "allosteric", "none"])
rel predict_binding_site(img: Tensor, site: String)

// Step 3: Assess druggability
@plip(labels=["druggable", "challenging", "undruggable"])
rel assess_druggability(img: Tensor, assessment: String)

rel target_images = {$load_image("target_protein.png")}

rel analysis(type, site, drug) =
  target_images(img),
  classify_target(img, type),
  predict_binding_site(img, site),
  assess_druggability(img, drug)

query analysis
```

### Structure-Activity Relationship (SAR)

```scl
@plip(
  labels=["high_activity", "moderate_activity", "low_activity", "inactive"],
  score_threshold=0.4
)
rel predict_activity(img: Tensor, activity: String)

rel compound_structures = {
  $load_image("compound_A.png"),
  $load_image("compound_B.png"),
  $load_image("compound_C.png")
}

rel activities(img, act) =
  compound_structures(img),
  predict_activity(img, act)

// Identify promising candidates
rel promising_compounds(img) =
  activities(img, "high_activity")

query promising_compounds
```

### Protein Family Classification

```scl
@plip(
  labels=["serine protease", "cysteine protease", "aspartic protease", "metalloprotease"],
  prompt="protease family: {{}}"
)
rel classify_protease_family(img: Tensor, family: String)

rel protease_structures = {
  $load_image("protease_1.png"),
  $load_image("protease_2.png"),
  $load_image("protease_3.png")
}

rel families(img, fam) =
  protease_structures(img),
  classify_protease_family(img, fam)

// Group by family
rel family_count(fam, count) =
  fam = ["serine protease", "cysteine protease", "aspartic protease", "metalloprotease"],
  count = count(img: families(img, fam))

query family_count
```

## PLIP vs CLIP

| Feature | PLIP | CLIP |
|---------|------|------|
| **Training Data** | Biomedical images | General images |
| **Domain** | Proteins, ligands, molecules | General objects, scenes |
| **Best For** | Scientific/medical analysis | General image classification |
| **Model Checkpoint** | `vinid/plip` | `openai/clip-*` |
| **Typical Labels** | "kinase", "ligand binding" | "cat", "car", "tree" |

**Use PLIP for:**
- Protein structure analysis
- Drug discovery
- Molecular property prediction
- Biomedical imaging

**Use CLIP for:**
- General image classification
- Scene understanding
- Object detection

## GPU Configuration

Enable GPU for faster inference:

```bash
# Use GPU
scli program.scl --cuda --gpu 0

# CPU only (slower)
scli program.scl
```

**Performance:**
- GPU: ~50ms per image
- CPU: ~500ms per image

## Troubleshooting

### Model Download Fails

**Error:**
```
Failed to download model checkpoint
```

**Solution:**
```bash
# Manually download
python -c "from transformers import CLIPModel; CLIPModel.from_pretrained('vinid/plip')"
```

### Out of Memory

**Error:**
```
RuntimeError: CUDA out of memory
```

**Solutions:**
- Use CPU: `scli program.scl` (no `--cuda`)
- Process fewer images at once
- Free GPU memory: `torch.cuda.empty_cache()`

### Low Confidence Scores

If all scores are below threshold:

1. **Lower threshold**: Try `score_threshold=0.1`
2. **Refine labels**: Use domain-specific terms
3. **Improve image quality**: Higher resolution, better contrast
4. **Use prompt templates**: Guide the model with context

### Wrong Classifications

**Debugging steps:**
1. Enable debug mode: `debug=true`
2. Review confidence scores
3. Try alternative label phrasings
4. Ensure images show target structures clearly

## Best Practices

### Label Selection

**✓ Good labels:**
- Domain-specific: "beta-lactamase", "kinase inhibitor"
- Clear categories: "active", "inactive"
- Standard nomenclature: "GPCR", "ion channel"

**✗ Poor labels:**
- Too general: "thing", "structure"
- Ambiguous: "maybe", "unknown"
- Non-visual: "expensive", "patented"

### Image Preparation

**Optimal images:**
- Clear protein/ligand structures
- Standard molecular visualizations
- High resolution (512x512 or higher)
- Good contrast and lighting

**Avoid:**
- Text overlays
- Low resolution images
- Multiple unrelated structures in one image

### Score Thresholds

| Threshold | Use Case |
|-----------|----------|
| 0.0 | Exploratory analysis, see all predictions |
| 0.2-0.3 | Moderate confidence filtering |
| 0.5+ | High confidence results only |
| 0.7+ | Very conservative filtering |

## Next Steps

- **[Transformers Plugin](transformers.md)** - General vision and language models
- **[CLIP Plugin](vision_models.md)** - General image classification
- **[GPU Utilities](gpu_utilities.md)** - Device management

For more on PLIP research, see the [paper](https://arxiv.org/abs/2202.13138) and [model card](https://huggingface.co/vinid/plip).
