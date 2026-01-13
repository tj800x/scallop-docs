# Google Gemini Plugin

The Google Gemini plugin integrates Google's Gemini language models into Scallop, providing an alternative to OpenAI GPT with similar capabilities for text processing, classification, and information extraction.

## Overview

The Gemini plugin provides the **same four foreign constructs** as the GPT plugin:

1. **`$gemini(prompt)`** - Foreign function for text generation
2. **`gemini(input, output)`** - Foreign predicate for fact generation
3. **`@gemini`** - Foreign attribute for few-shot learning
4. **`@gemini_extract_info`** - Foreign attribute for structured extraction

### Use Cases

- **Text classification**: Sentiment, intent, categories
- **Information extraction**: Entities, relations, structured data
- **Text generation**: Q&A, translation, summarization
- **Few-shot learning**: Example-driven classification
- **Multimodal reasoning**: Text and image processing (future)

### Model Support

| Model | Description | Use Case |
|-------|-------------|----------|
| `gemini-2.0-flash` | Fast, efficient (default) | General tasks, high throughput |
| `gemini-1.5-pro` | Most capable | Complex reasoning, long context |
| `gemini-1.5-flash` | Balanced | Cost-effective with good quality |

## Setup and Configuration

### Installation

```bash
# Install Gemini plugin
cd /path/to/scallop
make -C etc/scallopy-plugins develop-gemini

# Or with pip
cd etc/scallopy-plugins/gemini
pip install -e .
```

### API Key Configuration

**Set environment variable:**
```bash
export GEMINI_API_KEY="your-api-key-here"
```

**Get an API key:**
1. Visit https://aistudio.google.com/app/apikey
2. Sign in with Google account
3. Create a new API key
4. Copy and save it securely

**Verify configuration:**
```bash
echo $GEMINI_API_KEY
# Should print your key
```

### Command-Line Options

```bash
scli program.scl \
  --gemini-model gemini-1.5-pro \
  --gemini-temperature 0.0 \
  --num-allowed-gemini-request 50
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--gemini-model` | string | `gemini-2.0-flash` | Gemini model to use |
| `--gemini-temperature` | float | `0.0` | Sampling temperature (0.0 = deterministic) |
| `--num-allowed-gemini-request` | int | `100` | Maximum API calls per run |

### Python API Configuration

```python
import scallopy

ctx = scallopy.ScallopContext()
plugin_registry = scallopy.PluginRegistry()
plugin_registry.load_plugins_from_entry_points()

# Configure Gemini plugin
plugin_registry.configure({
    "gemini_model": "gemini-1.5-pro",
    "gemini_temperature": 0.0,
    "num_allowed_gemini_request": 50
}, [])

plugin_registry.load_into_ctx(ctx)
```

## Foreign Function: $gemini

Simple text-to-text generation:

### Usage

```scl
rel questions = {
  "What is the capital of Japan?",
  "Translate to French: Hello world",
  "What is 7 * 8?"
}

rel answers = {$gemini(q) | questions(q)}
query answers
```

**Expected output (mock when API key not set):**
```
answers: {
  ("Tokyo"),
  ("Bonjour le monde"),
  ("56")
}
```

### Features

- **Automatic memoization**: Identical prompts cached
- **Rate limiting**: Prevents runaway costs
- **Error handling**: Graceful fallback when API unavailable

## Foreign Predicate: gemini(...)

Flexible fact generation with bound/free patterns:

### Usage

**Generation (bf pattern):**
```scl
rel questions = {
  "What is Rust?",
  "What is Docker?"
}

rel qa(q, a) = questions(q), gemini(q, a)
query qa
```

**Verification (bb pattern):**
```scl
rel candidates = {
  ("What is 5+5?", "10"),
  ("What is 5+5?", "55")
}

rel verified(q, a) = candidates(q, a), gemini(q, a)
// Result: {("What is 5+5?", "10")}
```

## Foreign Attribute: @gemini

Few-shot classification and extraction:

### Example: Sentiment Analysis

```scl
@gemini(
  header="Classify sentiment:",
  prompts=[
    {text: "I love it!", sentiment: "positive"},
    {text: "Terrible experience", sentiment: "negative"},
    {text: "It's fine", sentiment: "neutral"}
  ],
  model="gemini-2.0-flash"
)
rel classify_sentiment(text: String, sentiment: String)

rel reviews = {
  "Best product ever!",
  "Complete waste of money",
  "Decent for the price"
}

rel results(review, sent) = reviews(review), classify_sentiment(review, sent)
query results
```

**Expected output (mock):**
```
results: {
  ("Best product ever!", "positive"),
  ("Complete waste of money", "negative"),
  ("Decent for the price", "neutral")
}
```

### Example: Intent Detection

```scl
@gemini(
  header="Determine the user's intent:",
  prompts=[
    {query: "What's the weather?", intent: "weather"},
    {query: "Book a flight to Paris", intent: "travel"},
    {query: "Order pizza", intent: "food"}
  ]
)
rel detect_intent(query: String, intent: String)

rel user_queries = {
  "Show me today's forecast",
  "Reserve a hotel in Rome",
  "Find restaurants nearby"
}

rel intents(q, i) = user_queries(q), detect_intent(q, i)
query intents
```

**Expected output (mock):**
```
intents: {
  ("Show me today's forecast", "weather"),
  ("Reserve a hotel in Rome", "travel"),
  ("Find restaurants nearby", "food")
}
```

## Foreign Attribute: @gemini_extract_info

Structured information extraction with JSON output:

### Example: Named Entity Recognition

```scl
@gemini_extract_info(
  header="Extract entities from the text:",
  prompts=[
    "Extract all people",
    "Extract all companies",
    "Extract all cities"
  ],
  examples=[
    (
      ["Alice works at Google in New York."],
      [
        [("Alice",)],         // people
        [("Google",)],        // companies
        [("New York",)]       // cities
      ]
    )
  ]
)
rel person(text: String, name: String)
rel company(text: String, org: String)
rel city(text: String, place: String)

rel texts = {
  "Bob joined Microsoft in Seattle.",
  "Carol founded Anthropic in San Francisco."
}

rel all_people(t, p) = texts(t), person(t, p)
rel all_companies(t, c) = texts(t), company(t, c)
rel all_cities(t, ci) = texts(t), city(t, ci)

query all_people
query all_companies
query all_cities
```

**Expected output (mock):**
```
all_people: {
  ("Bob joined Microsoft in Seattle.", "Bob"),
  ("Carol founded Anthropic in San Francisco.", "Carol")
}
all_companies: {
  ("Bob joined Microsoft in Seattle.", "Microsoft"),
  ("Carol founded Anthropic in San Francisco.", "Anthropic")
}
all_cities: {
  ("Bob joined Microsoft in Seattle.", "Seattle"),
  ("Carol founded Anthropic in San Francisco.", "San Francisco")
}
```

## Gemini vs GPT: Key Differences

### API and Configuration

| Feature | Gemini | GPT |
|---------|--------|-----|
| **API Key Env Var** | `GEMINI_API_KEY` | `OPENAI_API_KEY` |
| **Default Model** | `gemini-2.0-flash` | `gpt-3.5-turbo` |
| **CLI Prefix** | `--gemini-*` | `--openai-gpt-*` |
| **Request Limit Flag** | `--num-allowed-gemini-request` | `--num-allowed-openai-request` |

### Construct Names

All constructs have the same API but different names:

| Construct Type | Gemini | GPT |
|----------------|--------|-----|
| Foreign Function | `$gemini(...)` | `$gpt(...)` |
| Foreign Predicate | `gemini(...)` | `gpt(...)` |
| Foreign Attribute | `@gemini(...)` | `@gpt(...)` |
| Extract Info | `@gemini_extract_info(...)` | `@gpt_extract_info(...)` |

### Model Capabilities

**Gemini advantages:**
- Longer context windows (Gemini 1.5 Pro: up to 1M tokens)
- Multimodal support (text + images in future versions)
- Often faster inference
- Competitive pricing

**GPT advantages:**
- More mature ecosystem
- Better documented
- GPT-4 has strong reasoning capabilities

## Migration Guide

### From GPT to Gemini

To migrate from GPT to Gemini:

1. **Change environment variable:**
   ```bash
   # Old
   export OPENAI_API_KEY="sk-..."

   # New
   export GEMINI_API_KEY="your-gemini-key"
   ```

2. **Update construct names:**
   ```scl
   // Old
   @gpt(header="...", prompts=[...])
   rel classify(text: String, label: String)

   // New
   @gemini(header="...", prompts=[...])
   rel classify(text: String, label: String)
   ```

3. **Update CLI flags:**
   ```bash
   # Old
   scli program.scl --openai-gpt-model gpt-4

   # New
   scli program.scl --gemini-model gemini-1.5-pro
   ```

4. **No code changes needed!** The syntax and behavior are identical.

### Mixed Usage

You can use both plugins simultaneously:

```scl
// Use GPT for one task
@gpt(header="Complex reasoning:", prompts=[...])
rel gpt_classify(text: String, category: String)

// Use Gemini for another
@gemini(header="Quick classification:", prompts=[...])
rel gemini_classify(text: String, label: String)

// Compare results
rel agreement(text, cat) =
  texts(text),
  gpt_classify(text, cat),
  gemini_classify(text, cat)
```

## Best Practices

### Temperature Settings

Same as GPT:
- `0.0` for classification/extraction
- `0.3-0.5` for varied generation
- `0.7-1.0` for creative tasks

### Prompt Engineering

Gemini responds well to:
- **Clear instructions**: Be explicit about format
- **Few-shot examples**: 2-5 examples work best
- **Structured output**: Request JSON for extraction
- **Task decomposition**: Break complex tasks into steps

### Cost Optimization

- Use `gemini-2.0-flash` (fastest, cheapest)
- Set request limits: `--num-allowed-gemini-request 50`
- Leverage memoization (automatic)
- Cache results at application level

## Troubleshooting

### API Key Not Found

**Error:**
```
[scallop_gemini] `GEMINI_API_KEY` not found, consider setting it in the environment variable
```

**Solution:**
```bash
export GEMINI_API_KEY="your-key"
echo $GEMINI_API_KEY  # Verify
```

### Rate Limit Exceeded

**Error:**
```
Exceeding allowed number of requests
```

**Solution:**
```bash
scli program.scl --num-allowed-gemini-request 200
```

### Model Not Available

If model unavailable, fall back:

```bash
# Default model
scli program.scl --gemini-model gemini-2.0-flash

# Or older version
scli program.scl --gemini-model gemini-1.5-flash
```

### Response Format Issues

Same debugging approach as GPT:
1. Check prompt clarity
2. Use `temperature=0.0`
3. Add more examples
4. Use `debug=true` flag

## Next Steps

- **[GPT Plugin](openai_gpt.md)** - Similar API with OpenAI models
- **[Foreign Attributes](foreign_attributes.md)** - Learn attribute mechanics
- **[Transformers Plugin](transformers.md)** - Local models (no API required)

For API details and pricing, see [Google AI Studio](https://aistudio.google.com).
