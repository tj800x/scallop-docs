# Text Embeddings

Text embeddings are vector representations of text that capture semantic meaning. Scallop integrates with various text embedding models to combine neural language understanding with symbolic reasoning.

## Overview

Text embeddings enable Scallop to:
- Match natural language descriptions to structured data
- Perform semantic similarity comparisons
- Bridge neural text understanding with logical reasoning
- Handle multi-modal tasks (text + vision, text + video)

## Integration Pattern

Text embeddings are typically provided as **input relations** to Scallop programs:

```python
import scallopy
from transformers import AutoTokenizer, AutoModel

# Create embedding model
tokenizer = AutoTokenizer.from_pretrained("distilbert-base-uncased")
model = AutoModel.from_pretrained("distilbert-base-uncased")

# Create Scallop context
ctx = scallopy.ScallopContext()

# Define input relation for text embeddings
ctx.add_relation("text_embedding", (int, str, list))

# Process text and add embeddings
text = "example description"
embedding = get_embedding(text)  # Get embedding vector
ctx.add_facts("text_embedding", [(0, text, embedding)])

# Add reasoning rules
ctx.add_rule("match(id) = text_embedding(id, text, emb), similarity(emb, target) > 0.8")
```

## Example: Video-Text Matching (Mugen Dataset)

This example demonstrates using text embeddings with video action recognition to match natural language descriptions to video content.

### Neural Components

- **Text Embedding**: DistilBERT for text description encoding
- **Vision Embedding**: S3D for video frame encoding
- **MLP**: 2-layer network (hidden size 256) for feature fusion

### Scallop Program

```scallop
// Input from neural networks
type action(usize, String)        // Video actions detected
type expr(usize, String)          // Text expressions from description
type expr_start(usize)            // Start of text expression
type expr_end(usize)              // End of text expression
type action_start(usize)          // Start of video action
type action_end(usize)            // End of video action

type match_single(usize, usize, usize)      // Single action-expression match
type match_sub(usize, usize, usize, usize)  // Subsequence match

// Check whether a text expression matches a video action
rel match_single(tid, vid, vid + 1) =
    expr(tid, a),
    action(vid, a)

// Match a single text expression to video subsequence
rel match_sub(tid, tid, vid_start, vid_end) =
    match_single(tid, vid_start, vid_end)

rel match_sub(tid, tid, vid_start, vid_end) =
    match_sub(tid, tid, vid_start, vid_mid),
    match_single(tid, vid_mid, vid_end)

// Match a sequence of text expressions to video subsequence
rel match_sub(tid_start, tid_end, vid_start, vid_end) =
    match_sub(tid_start, tid_end - 1, vid_start, vid_mid),
    match_single(tid_end, vid_mid, vid_end)

// Check whether the whole text specification matches the video
rel match() =
    expr_start(tid_start),
    expr_end(tid_end),
    action_start(vid_start),
    action_end(vid_end),
    match_sub(tid_start, tid_end, vid_start, vid_end)

// Integrity constraint: detect too many consecutive identical expressions
rel too_many_consecutive_expr() =
    expr(tid, a),
    expr(tid + 1, a),
    expr(tid + 2, a),
    expr(tid + 3, a)
```

### Training Configuration

- **Dataset**: 1K Mugen video-text pairs (training), 1K (testing)
- **Training**: 1000 epochs, learning rate 0.0001, batch size 3
- **Loss**: BCE-loss for end-to-end training
- **Neural-Symbolic Integration**: Embeddings flow into Scallop's logical reasoning

### Key Insights

1. **Structured Matching**: Logical rules enforce alignment between text sequence and video sequence
2. **Compositional Reasoning**: Text expressions can match video action subsequences
3. **Constraint Enforcement**: Integrity constraints detect anomalies (repeated expressions)
4. **Differentiable**: Entire pipeline is trainable end-to-end

## Common Text Embedding Models

### Transformer-based
- **BERT** (`bert-base-uncased`) - General-purpose text understanding
- **DistilBERT** (`distilbert-base-uncased`) - Faster, lighter BERT variant
- **RoBERTa** (`roberta-base`) - Robustly optimized BERT
- **T5** (`t5-base`) - Text-to-text transformer

### Sentence Embeddings
- **Sentence-BERT** (`sentence-transformers`) - Optimized for sentence similarity
- **MPNet** - Strong general-purpose sentence embeddings
- **Universal Sentence Encoder** - Google's multilingual embeddings

### Domain-Specific
- **BioBERT** - Biomedical text
- **SciBERT** - Scientific literature
- **CodeBERT** - Source code

## Integration with Scallop Plugins

The `scallop-ext` plugin system provides built-in support for text embeddings:

```python
import scallopy

# Use OpenAI embeddings
ctx = scallopy.ScallopContext()
ctx.import_plugin("openai_gpt")

# Text similarity using embeddings
ctx.add_rule("""
  rel similar_docs(d1, d2) =
    document(d1, text1),
    document(d2, text2),
    $openai_text_similarity(text1, text2) > 0.85
""")
```

## Best Practices

1. **Normalize embeddings** - Use L2 normalization for cosine similarity
2. **Cache embeddings** - Compute once, reuse for multiple queries
3. **Batch processing** - Embed multiple texts together for efficiency
4. **Threshold tuning** - Adjust similarity thresholds for your domain
5. **Hybrid approaches** - Combine embeddings with symbolic rules for robustness

## Example Use Cases

- **Document retrieval** - Semantic search over document collections
- **Text classification** - Combine neural embeddings with logical rules
- **Named entity resolution** - Match entities using semantic similarity
- **Multi-modal reasoning** - Align text with images/video using embeddings
- **Question answering** - Match questions to answers semantically

## References

- **Scallop Paper**: [Scallop: A Language for Neurosymbolic Programming](https://arxiv.org/abs/2304.04812)
- **Mugen Dataset**: [Hayes et al. 2022] Video-text alignment benchmark
- **Transformers**: [Hugging Face Transformers](https://huggingface.co/transformers/)
- **Sentence-BERT**: [Sentence-Transformers](https://www.sbert.net/)

---

*For more examples of using embeddings with Scallop, see the [OpenAI GPT](openai_gpt.md) and [Transformers](transformers.md) integration guides.*
