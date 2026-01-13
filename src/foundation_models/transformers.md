# Transformers Plugin

The Transformers plugin integrates HuggingFace Transformers models into Scallop, providing **local** vision and language models without requiring API keys. This plugin includes three models: ViLT for visual question answering, OWL-ViT for object detection, and RoBERTa for text encoding.

## Overview

The Transformers plugin provides **three foreign attributes** for different AI tasks:

1. **`@vilt`** - Visual Question Answering (VQA)
2. **`@owl_vit`** - Open-vocabulary object detection
3. **`@roberta_encoder`** - Text embedding generation

### Key Features

- **No API keys required**: Models run locally
- **Automatic model download**: First run downloads models from HuggingFace
- **GPU acceleration**: Automatic CUDA detection
- **Probabilistic outputs**: Confidence scores for reasoning
- **Flexible configuration**: Custom checkpoints and parameters

### Use Cases

- **Visual QA**: Answer questions about images
- **Object detection**: Find objects using text queries
- **Text similarity**: Semantic search and clustering
- **Multi-modal reasoning**: Combine vision and language

## Installation

```bash
# Install Transformers plugin
cd /path/to/scallop
make -C etc/scallopy-plugins develop-transformers

# Or with pip
cd etc/scallopy-plugins/transformers
pip install -e .
```

### Dependencies

The plugin requires:
- `transformers` - HuggingFace Transformers library
- `torch` - PyTorch for model inference
- `PIL` - Image processing
- `scallop-gpu` - GPU device management (optional)

**Install dependencies:**
```bash
pip install transformers torch pillow
```

### Model Downloads

Models are automatically downloaded on first use:

| Model | Size | Cache Location |
|-------|------|----------------|
| ViLT | ~450MB | `~/.cache/huggingface/hub/` |
| OWL-ViT | ~500MB | `~/.cache/huggingface/hub/` |
| RoBERTa | ~500MB | `~/.cache/huggingface/hub/` |

## @vilt: Visual Question Answering

ViLT (Vision-and-Language Transformer) answers questions about images.

### Syntax

```scl
@vilt(
  question: String = None,
  top: int = 5,
  score_threshold: float = 0.1,
  checkpoint: str = "dandelin/vilt-b32-finetuned-vqa",
  debug: bool = false
)
rel relation_name(img: Tensor, question: String, answer: String)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `question` | String | `None` | Fixed question (if provided, relation is arity-2) |
| `top` | int | `5` | Number of top answers to return |
| `score_threshold` | float | `0.1` | Minimum confidence score |
| `checkpoint` | String | `"dandelin/vilt-b32-finetuned-vqa"` | Model checkpoint |
| `debug` | bool | `false` | Print debug information |

### Example: Fixed Question

```scl
@vilt(question="What is in the image?", top=3)
rel answer_question(img: Tensor, answer: String)

rel images = {
  $load_image("cat.jpg"),
  $load_image("car.jpg"),
  $load_image("tree.jpg")
}

rel answers(img, ans) = images(img), answer_question(img, ans)
query answers
```

**Expected output (mock when model not available):**
```
answers: {
  0.9::(image_tensor("cat.jpg"), "a cat"),
  0.7::(image_tensor("cat.jpg"), "an animal"),
  0.5::(image_tensor("cat.jpg"), "a pet"),
  0.85::(image_tensor("car.jpg"), "a car"),
  0.75::(image_tensor("car.jpg"), "a vehicle"),
  0.6::(image_tensor("car.jpg"), "an automobile")
}
```

### Example: Dynamic Questions

```scl
@vilt(top=5, score_threshold=0.3)
rel vqa(img: Tensor, question: String, answer: String)

rel image = {$load_image("scene.jpg")}
rel questions = {
  "What color is the sky?",
  "How many people are there?",
  "What is the weather like?"
}

rel qa(q, a) = image(img), questions(q), vqa(img, q, a)
query qa
```

**Expected output (mock):**
```
qa: {
  0.9::("What color is the sky?", "blue"),
  0.8::("How many people are there?", "3"),
  0.7::("What is the weather like?", "sunny"),
  0.5::("What is the weather like?", "clear")
}
```

### Supported Questions

ViLT handles various question types:

- **What**: "What is this?", "What color?"
- **How many**: "How many people?", "How many cars?"
- **Where**: "Where is the cat?", "Where are they?"
- **Yes/No**: "Is there a dog?", "Is it raining?"
- **Who**: "Who is in the picture?"

### Performance Tips

**Reduce top-k for faster inference:**
```scl
@vilt(top=1, score_threshold=0.5)  // Only best answer
rel quick_answer(img: Tensor, question: String, answer: String)
```

**Use GPU acceleration:**
```bash
scli program.scl --cuda --gpu 0
```

## @owl_vit: Open-Vocabulary Object Detection

OWL-ViT detects objects using text queries instead of predefined classes.

### Syntax

```scl
@owl_vit(
  object_queries: List[String] = None,
  output_fields: List[String] = ["class"],
  score_threshold: float = 0.1,
  limit: int = None,
  checkpoint: str = "google/owlvit-base-patch32",
  debug: bool = false
)
rel relation_name(img: Tensor, queries: String, ...output_fields...)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `object_queries` | List[String] | `None` | Fixed object queries |
| `output_fields` | List[String] | `["class"]` | Fields to return (see below) |
| `score_threshold` | float | `0.1` | Minimum detection confidence |
| `limit` | int | `None` | Maximum detections per image |
| `checkpoint` | String | `"google/owlvit-base-patch32"` | Model checkpoint |

### Output Fields

| Field | Type | Description |
|-------|------|-------------|
| `class` | String | Detected object class |
| `bbox-x`, `bbox-y` | u32 | Bounding box top-left corner |
| `bbox-w`, `bbox-h` | u32 | Bounding box width and height |
| `bbox-center-x`, `bbox-center-y` | u32 | Bounding box center |
| `area` | u32 | Bounding box area (pixels) |
| `cropped-image` | Tensor | Cropped region tensor |

### Example: Fixed Queries

```scl
@owl_vit(
  object_queries=["person", "car", "tree"],
  output_fields=["class", "bbox-x", "bbox-y", "bbox-w", "bbox-h"],
  score_threshold=0.3
)
rel detect(img: Tensor, class: String, x: u32, y: u32, w: u32, h: u32)

rel image = {$load_image("street.jpg")}
rel detections(c, x, y, w, h) = image(img), detect(img, c, x, y, w, h)
query detections
```

**Expected output (mock):**
```
detections: {
  0.9::("person", 120, 80, 50, 150),
  0.85::("person", 300, 100, 60, 140),
  0.75::("car", 450, 200, 120, 80),
  0.7::("tree", 50, 10, 80, 200)
}
```

### Example: Dynamic Queries

```scl
@owl_vit(
  output_fields=["class"],
  score_threshold=0.5
)
rel detect_objects(img: Tensor, queries: String, class: String)

rel image = {$load_image("photo.jpg")}
rel query_list = {"dog;cat;bird"}  // Semicolon-separated

rel found(obj) = image(img), query_list(q), detect_objects(img, q, obj)
query found
```

### Example: Extract Cropped Regions

```scl
@owl_vit(
  object_queries=["face"],
  output_fields=["cropped-image"],
  score_threshold=0.6
)
rel extract_faces(img: Tensor, face: Tensor)

rel original = {$load_image("group_photo.jpg")}
rel faces(f) = original(img), extract_faces(img, f)

// Now process each face separately
@clip(labels=["happy", "sad", "neutral"])
rel classify_emotion(face: Tensor, emotion: String)

rel emotions(e) = faces(f), classify_emotion(f, e)
query emotions
```

### Use Cases

- **Custom object detection**: Any object describable by text
- **Zero-shot detection**: No training data required
- **Region extraction**: Get bounding boxes or cropped images
- **Counting**: Count objects of specific types

## @roberta_encoder: Text Embeddings

RoBERTa generates text embeddings for semantic similarity and search.

### Syntax

```scl
@roberta_encoder(checkpoint: str = "roberta-base")
type function_name(text: String) -> Tensor
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `checkpoint` | String | `"roberta-base"` | Model checkpoint |

**Available checkpoints:**
- `"roberta-base"` - 768-dimensional embeddings
- `"roberta-large"` - 1024-dimensional embeddings
- `"distilroberta-base"` - Smaller, faster

### Example: Text Similarity

```scl
@roberta_encoder(checkpoint="roberta-base")
type encode(text: String) -> Tensor

rel texts = {
  "The cat sat on the mat",
  "A feline rested on the rug",
  "Dogs are loyal animals",
  "Cars are fast vehicles"
}

// Encode all texts
rel embeddings(t, e) = texts(t), e = $encode(t)

// Compute similarity (using cosine similarity foreign function)
rel similarity(t1, t2, sim) =
  embeddings(t1, e1),
  embeddings(t2, e2),
  t1 != t2,
  sim = $cosine_similarity(e1, e2)

query similarity
```

**Expected output (mock):**
```
similarity: {
  ("The cat sat on the mat", "A feline rested on the rug", 0.92),
  ("Dogs are loyal animals", "Cars are fast vehicles", 0.15),
  // ... (high similarity for semantically similar texts)
}
```

### Example: Semantic Search

```scl
@roberta_encoder
type encode(text: String) -> Tensor

rel documents = {
  "Machine learning is a subset of AI",
  "Neural networks power deep learning",
  "Python is a programming language",
  "Data science uses statistical methods"
}

rel query_text = {"artificial intelligence and neural networks"}

rel doc_embeddings(doc, emb) = documents(doc), emb = $encode(doc)
rel query_embedding(emb) = query_text(q), emb = $encode(q)

rel ranked_docs(doc, score) =
  doc_embeddings(doc, d_emb),
  query_embedding(q_emb),
  score = $cosine_similarity(d_emb, q_emb)

// Order by descending score (most similar first)
query ranked_docs
```

### Use Cases

- **Semantic search**: Find relevant documents
- **Clustering**: Group similar texts
- **Duplicate detection**: Find near-duplicates
- **Recommendation**: Suggest similar items

## Model Comparison

| Feature | ViLT | OWL-ViT | RoBERTa |
|---------|------|---------|---------|
| **Input** | Image + Text | Image + Text | Text only |
| **Output** | Text answers | Bounding boxes | Embeddings (Tensor) |
| **Task** | Question answering | Object detection | Text encoding |
| **Requires GPU** | Recommended | Recommended | Optional |
| **Model Size** | ~450MB | ~500MB | ~500MB |
| **Inference Speed** | Medium | Slow | Fast |

## GPU Configuration

All models support GPU acceleration:

```bash
# Use GPU
scli program.scl --cuda --gpu 0

# Use specific GPU
scli program.scl --cuda --gpu 1

# CPU only (slower)
scli program.scl
```

**In Python:**
```python
plugin_registry.configure({"cuda": True, "gpu": 0}, [])
```

## Troubleshooting

### Model Download Fails

**Error:**
```
Failed to download model checkpoint
```

**Solution:**
1. Check internet connection
2. Verify HuggingFace is accessible
3. Manually download:
   ```bash
   python -c "from transformers import ViltForQuestionAnswering; ViltForQuestionAnswering.from_pretrained('dandelin/vilt-b32-finetuned-vqa')"
   ```

### Out of Memory

**Error:**
```
RuntimeError: CUDA out of memory
```

**Solutions:**
- Use CPU: `scli program.scl` (no `--cuda`)
- Reduce `top` parameter for ViLT
- Reduce `limit` parameter for OWL-ViT
- Use smaller model checkpoints

### Slow Inference

**Optimizations:**
- Enable GPU: `--cuda --gpu 0`
- Use smaller checkpoints (e.g., `distilroberta-base`)
- Reduce top-k for ViLT
- Batch similar queries

### Wrong Answers

**Tips:**
- Lower `score_threshold` to see more candidates
- Use `debug=true` to see scores
- Try different model checkpoints
- Rephrase questions for ViLT

## Best Practices

### ViLT Questions

**✓ Good questions:**
- "What color is the car?"
- "How many people are in the image?"
- "Is there a dog?"

**✗ Ambiguous questions:**
- "What is it?" (too vague)
- "Tell me everything" (too broad)

### OWL-ViT Queries

**✓ Good queries:**
- Specific objects: "red car", "person wearing hat"
- Simple categories: "dog", "tree", "building"

**✗ Bad queries:**
- Abstract concepts: "happiness", "beauty"
- Complex descriptions: "person doing something unusual"

### RoBERTa Encoding

**Best for:**
- Short texts (< 512 tokens)
- English text (base model)
- Sentence-level similarity

**Not ideal for:**
- Very long documents (use summarization first)
- Non-English (use multilingual models)

## Next Steps

- **[PLIP Plugin](plip.md)** - Protein-ligand analysis (specialized CLIP)
- **[GPT Plugin](openai_gpt.md)** - LLM-based question answering
- **[GPU Utilities](gpu_utilities.md)** - Device management details

For model documentation, see [HuggingFace Model Hub](https://huggingface.co/models).
