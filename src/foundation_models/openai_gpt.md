# OpenAI GPT Plugin

The OpenAI GPT plugin integrates GPT-3.5 and GPT-4 language models into Scallop, enabling LLM-powered text processing, classification, information extraction, and generation within logical programs.

## Overview

The GPT plugin provides **four foreign constructs** for different use cases:

1. **`$gpt(prompt)`** - Foreign function for simple text generation
2. **`gpt(input, output)`** - Foreign predicate for flexible fact generation
3. **`@gpt`** - Foreign attribute for few-shot classification/extraction
4. **`@gpt_extract_info`** - Foreign attribute for structured information extraction

### Use Cases

- **Text classification**: Sentiment analysis, intent detection, category labeling
- **Information extraction**: Named entity recognition, relation extraction
- **Text generation**: Question answering, translation, summarization
- **Few-shot learning**: Provide examples, get consistent results
- **Probabilistic reasoning**: Combine LLM outputs with logical rules

### Model Support

| Model | Description | Use Case |
|-------|-------------|----------|
| `gpt-3.5-turbo` | Fast, cost-effective | General classification, extraction |
| `gpt-4` | Most capable | Complex reasoning, nuanced understanding |
| `gpt-4-turbo` | Latest GPT-4 | Balanced cost and capability |

## Setup and Configuration

### Installation

```bash
# Install GPT plugin
cd /path/to/scallop
make -C etc/scallopy-plugins develop-gpt

# Or install with pip
cd etc/scallopy-plugins/gpt
pip install -e .
```

### API Key Configuration

The plugin requires an OpenAI API key:

**Set environment variable:**
```bash
export OPENAI_API_KEY="sk-..."
```

**Get an API key:**
1. Visit https://platform.openai.com/api-keys
2. Sign up or log in
3. Create a new secret key
4. Copy and save it securely

**Verify configuration:**
```bash
echo $OPENAI_API_KEY
# Should print: sk-...
```

### Command-Line Options

Configure the plugin when running Scallop programs:

```bash
scli program.scl \
  --openai-gpt-model gpt-4 \
  --openai-gpt-temperature 0.0 \
  --num-allowed-openai-request 50
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--openai-gpt-model` | string | `gpt-3.5-turbo` | GPT model to use |
| `--openai-gpt-temperature` | float | `0.0` | Sampling temperature (0.0 = deterministic) |
| `--num-allowed-openai-request` | int | `100` | Maximum API calls per run |

### Python API Configuration

```python
import scallopy

# Create context
ctx = scallopy.ScallopContext()

# Load plugin registry
plugin_registry = scallopy.PluginRegistry()
plugin_registry.load_plugins_from_entry_points()

# Configure GPT plugin
plugin_registry.configure({
    "openai_gpt_model": "gpt-4",
    "openai_gpt_temperature": 0.0,
    "num_allowed_openai_request": 50
}, [])

# Load into context
plugin_registry.load_into_ctx(ctx)

# Now use GPT constructs in your program
```

### Rate Limiting

The plugin automatically limits API calls to prevent runaway costs:

- Default: 100 requests per run
- Customize with `--num-allowed-openai-request`
- Exceeding limit raises exception
- Memoization reduces redundant calls

## Foreign Function: $gpt

The `$gpt` foreign function provides **simple text-to-text generation**:

### Signature

```
$gpt(prompt: String) -> String
```

### Usage

**Direct invocation:**
```scl
rel questions = {
  "What is the capital of France?",
  "Translate to Spanish: Good morning",
  "What is 15 * 24?"
}

rel answers(q) = questions(q), a = $gpt(q)
query answers
```

**Expected output (mock when API key not set):**
```
answers: {
  ("Paris"),
  ("Buenos días"),
  ("360")
}
```

**In expressions:**
```scl
rel prompts = {"Explain quantum computing in one sentence"}
rel response = {$gpt(p) | prompts(p)}
query response
```

### Memoization

The function **automatically memoizes** results:

```scl
rel repeated = {
  "What is 2+2?",
  "What is 2+2?",  // Same prompt
  "What is 2+2?"   // Same prompt
}

rel answers = {$gpt(q) | repeated(q)}
// Only makes 1 API call!
```

### Error Handling

If API key is not set or rate limit exceeded:

```
[scallop_openai] `OPENAI_API_KEY` not found, consider setting it in the environment variable
```

## Foreign Predicate: gpt(...)

The `gpt` foreign predicate provides **flexible fact generation** with multiple calling patterns:

### Signature

```
gpt(input: String, output: String)
```

### Calling Patterns

**Bound-Free (bf) - Generation:**
```scl
rel questions = {"What is the capital of Spain?", "What is 10 + 5?"}
rel qa(q, a) = questions(q), gpt(q, a)
query qa

// Result: {
//   ("What is the capital of Spain?", "Madrid"),
//   ("What is 10 + 5?", "15")
// }
```

**Bound-Bound (bb) - Verification:**
```scl
rel candidate_answers = {
  ("What is 2+2?", "4"),
  ("What is 2+2?", "5"),
  ("What is 2+2?", "22")
}

rel verified(q, a) = candidate_answers(q, a), gpt(q, a)
query verified

// Result: {("What is 2+2?", "4")}
// Only the correct answer passes verification
```

### Multiple Outputs

GPT can return multiple completions (requires API configuration):

```scl
// With n=3 in API call
rel question = {"What are some programming languages?"}
rel languages(q, lang) = question(q), gpt(q, lang)

// Result (multiple facts from one input):
// languages: {
//   ("What are some programming languages?", "Python, Java, C++"),
//   ("What are some programming languages?", "JavaScript, Ruby, Go"),
//   ("What are some programming languages?", "Rust, Swift, Kotlin")
// }
```

## Foreign Attribute: @gpt

The `@gpt` attribute provides **few-shot classification and extraction** with prompt engineering:

### Syntax

```scl
@gpt(
  header: String,
  prompts: List[{key: value, ...}],
  model: String = "gpt-3.5-turbo",
  debug: bool = false
)
rel relation_name(input1: String, ..., inputN: String, output1: String, ..., outputM: String)
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `header` | String | Instruction/context for the task |
| `prompts` | List[{...}] | Few-shot examples (input → output) |
| `model` | String | GPT model to use (default: `gpt-3.5-turbo`) |
| `debug` | bool | Print prompts and responses (default: `false`) |

### Example: Sentiment Classification

```scl
@gpt(
  header="Classify the sentiment of the following text:",
  prompts=[
    {text: "I love this product!", sentiment: "positive"},
    {text: "This is terrible.", sentiment: "negative"},
    {text: "It's okay, nothing special.", sentiment: "neutral"}
  ],
  model="gpt-3.5-turbo"
)
rel classify_sentiment(text: String, sentiment: String)

rel reviews = {
  "Amazing quality and fast shipping!",
  "Worst purchase ever.",
  "Not bad, could be better.",
  "Absolutely fantastic!"
}

rel results(review, sent) = reviews(review), classify_sentiment(review, sent)
query results
```

**Expected output (mock when API key not set):**
```
results: {
  ("Amazing quality and fast shipping!", "positive"),
  ("Worst purchase ever.", "negative"),
  ("Not bad, could be better.", "neutral"),
  ("Absolutely fantastic!", "positive")
}
```

### Example: Named Entity Extraction

```scl
@gpt(
  header="Extract the person's name from the text:",
  prompts=[
    {text: "John went to the store", name: "John"},
    {text: "Mary likes apples", name: "Mary"},
    {text: "Dr. Smith gave a lecture", name: "Dr. Smith"}
  ]
)
rel extract_name(text: String, name: String)

rel sentences = {
  "Alice bought a new car",
  "Bob and Charlie went fishing",
  "Professor Johnson teaches physics"
}

rel people(sentence, person) = sentences(sentence), extract_name(sentence, person)
query people
```

**Expected output (mock when API key not set):**
```
people: {
  ("Alice bought a new car", "Alice"),
  ("Bob and Charlie went fishing", "Bob, Charlie"),
  ("Professor Johnson teaches physics", "Professor Johnson")
}
```

### Example: Multi-Input Pattern

```scl
@gpt(
  header="Determine the relationship between two people:",
  prompts=[
    {person1: "Alice", person2: "Bob", relation: "parent"},
    {person1: "Carol", person2: "Dave", relation: "sibling"}
  ]
)
rel infer_relation(person1: String, person2: String, relation: String)

rel pairs = {
  ("John", "Mary"),
  ("Sarah", "Tom"),
  ("Emily", "Emma")
}

rel relationships(p1, p2, rel) = pairs(p1, p2), infer_relation(p1, p2, rel)
query relationships
```

### How It Works

1. **Pattern detection**: Analyzes relation signature to determine bound/free variables
2. **Prompt construction**: Builds prompt with header + examples + user input
3. **API call**: Sends to OpenAI with configured model and temperature
4. **Response parsing**: Extracts answer and yields as Scallop fact
5. **Memoization**: Caches results to avoid redundant API calls

## Foreign Attribute: @gpt_extract_info

The `@gpt_extract_info` attribute provides **structured information extraction** with JSON output:

### Syntax

```scl
@gpt_extract_info(
  header: String,
  prompts: List[String],
  examples: List[(List[String], List[List[Tuple[...]]])],
  model: String = "gpt-3.5-turbo",
  cot: List[bool] = None,
  debug: bool = false
)
rel relation1(input1: String, ..., output1: String, ...)
rel relation2(input1: String, ..., output2: String, ...)
...
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `header` | String | Task instruction |
| `prompts` | List[String] | One prompt per relation |
| `examples` | List[(inputs, outputs)] | Few-shot examples with expected JSON |
| `model` | String | GPT model (default: `gpt-3.5-turbo`) |
| `cot` | List[bool] | Chain-of-thought per relation (default: `None`) |
| `debug` | bool | Print debugging info (default: `false`) |

### Example: Multi-Relation Extraction

```scl
@gpt_extract_info(
  header="Extract entities and their properties from the text:",
  prompts=[
    "Extract all people mentioned",
    "Extract all locations mentioned",
    "Extract all organizations mentioned"
  ],
  examples=[
    // (inputs, [people_output, location_output, org_output])
    (
      ["Alice works at Google in Mountain View."],
      [
        [("Alice",)],              // people
        [("Mountain View",)],      // locations
        [("Google",)]              // organizations
      ]
    ),
    (
      ["Bob visited Microsoft headquarters in Redmond."],
      [
        [("Bob",)],
        [("Redmond",)],
        [("Microsoft",)]
      ]
    )
  ]
)
rel person(text: String, name: String)
rel location(text: String, place: String)
rel organization(text: String, org: String)

rel texts = {
  "Sarah joined Apple in Cupertino last year.",
  "The meeting with Amazon was held in Seattle."
}

rel all_people(t, p) = texts(t), person(t, p)
rel all_locations(t, l) = texts(t), location(t, l)
rel all_orgs(t, o) = texts(t), organization(t, o)

query all_people
query all_locations
query all_orgs
```

**Expected output (mock when API key not set):**
```
all_people: {
  ("Sarah joined Apple in Cupertino last year.", "Sarah")
}
all_locations: {
  ("Sarah joined Apple in Cupertino last year.", "Cupertino"),
  ("The meeting with Amazon was held in Seattle.", "Seattle")
}
all_orgs: {
  ("Sarah joined Apple in Cupertino last year.", "Apple"),
  ("The meeting with Amazon was held in Seattle.", "Amazon")
}
```

### JSON Output Format

GPT returns JSON like:
```json
{
  "person": [{"name": "Sarah"}],
  "location": [{"place": "Cupertino"}],
  "organization": [{"org": "Apple"}]
}
```

The plugin automatically:
- Parses JSON response
- Maps to declared relations
- Yields facts with correct types

## Best Practices

### Temperature Settings

| Temperature | Behavior | Use Case |
|-------------|----------|----------|
| `0.0` | Deterministic | Classification, extraction |
| `0.3-0.5` | Slightly varied | Creative text generation |
| `0.7-1.0` | Very creative | Brainstorming, diverse outputs |

**Recommendation:** Use `0.0` for classification/extraction to ensure consistent results.

### Prompt Engineering

**✓ Good prompts:**
- Clear, specific instructions
- 2-5 few-shot examples
- Consistent formatting
- Edge cases covered

**✗ Bad prompts:**
- Vague instructions
- No examples
- Inconsistent formats
- Missing edge cases

### Cost Management

**Minimize API calls:**
- Use memoization (automatic)
- Set `--num-allowed-openai-request` limit
- Use `gpt-3.5-turbo` for simple tasks
- Batch similar queries

**Example:**
```bash
# Limit to 20 requests for testing
scli program.scl --num-allowed-openai-request 20
```

### Debugging

Enable debug mode to see prompts and responses:

```scl
@gpt(
  header="Classify:",
  prompts=[...],
  debug=true  // Enable debugging
)
rel classify(text: String, label: String)
```

Output:
```
Prompt: Classify:
Example: {text: "I love this", label: "positive"}
Now classify: {text: "This is great"}
Responses: ["positive"]
```

## Troubleshooting

### API Key Not Found

**Error:**
```
[scallop_openai] `OPENAI_API_KEY` not found, consider setting it in the environment variable
```

**Solution:**
```bash
export OPENAI_API_KEY="sk-..."
# Verify
echo $OPENAI_API_KEY
```

### Rate Limit Exceeded

**Error:**
```
Exceeding allowed number of requests
```

**Solution:**
```bash
# Increase limit
scli program.scl --num-allowed-openai-request 200
```

### Incorrect Response Format

If GPT returns unexpected format:

1. **Check prompts**: Ensure examples are clear and consistent
2. **Lower temperature**: Use `0.0` for deterministic output
3. **Add more examples**: 3-5 examples usually work best
4. **Use `debug=true`**: See actual prompts and responses

### Model Not Available

If model not accessible:

```bash
# Fall back to gpt-3.5-turbo
scli program.scl --openai-gpt-model gpt-3.5-turbo
```

## Next Steps

- **[Gemini Plugin](gemini.md)** - Alternative LLM with similar API
- **[Foreign Attributes](foreign_attributes.md)** - Learn more about attributes
- **[Create Your Own Plugin](create_your_own_plugin.md)** - Build custom LLM integrations

For API details and pricing, see the [OpenAI API Documentation](https://platform.openai.com/docs).
