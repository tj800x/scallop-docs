# Save and Load

Scallop modules integrate seamlessly with PyTorch's serialization system, allowing you to save trained models and load them later for inference or continued training.

## Why Save and Load?

**Training is expensive**: After training a neurosymbolic model, you want to save the learned neural parameters without retraining.

**Deployment**: Load trained models in production environments for inference.

**Checkpointing**: Save intermediate models during long training runs to resume if interrupted.

**Model sharing**: Distribute trained models to others.

---

## Basic Saving and Loading

Scallop modules are `torch.nn.Module` subclasses, so they use standard PyTorch serialization.

### Saving a Module

Use `torch.save()` to save the entire module:

```python
import torch
import scallopy

# Create and train your model
model = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="rel sum_2(a + b) = digit_a(a) and digit_b(b)",
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)

# ... training code ...

# Save the entire module
torch.save(model, "my_model.pt")
```

### Loading a Module

Use `torch.load()` to load the saved module:

```python
import torch

# Load the entire module
loaded_model = torch.load("my_model.pt")

# Use immediately for inference
input_data = torch.randn(16, 10)
result = loaded_model(digit_a=input_data, digit_b=input_data)
```

**Important**: The Scallop program, rules, and mappings are all preserved when saving/loading.

---

## Saving State Dictionaries

For more flexibility, save only the model parameters (state dict) instead of the entire module.

### Saving State Dict

```python
# Save only the parameters
torch.save(model.state_dict(), "model_weights.pth")
```

### Loading State Dict

```python
# First, recreate the model architecture
model = scallopy.Module(
  provenance="difftopkproofs",
  k=3,
  program="rel sum_2(a + b) = digit_a(a) and digit_b(b)",
  input_mappings={"digit_a": range(10), "digit_b": range(10)},
  output_mapping=("sum_2", range(19))
)

# Then load the saved parameters
model.load_state_dict(torch.load("model_weights.pth"))

# Set to evaluation mode
model.eval()
```

**Advantage**: State dicts are more portable across PyTorch versions and modifications to the module structure.

**Requirement**: You must recreate the exact same module architecture before loading the state dict.

---

## Complete Example

Here's a full workflow showing training, saving, and loading:

```python
import torch
import torch.nn as nn
import scallopy

# Define a neural network with Scallop reasoning
class DigitAdder(nn.Module):
  def __init__(self):
    super().__init__()

    # Neural perception layers
    self.encoder = nn.Sequential(
      nn.Linear(784, 128),
      nn.ReLU(),
      nn.Linear(128, 10),
    )

    # Symbolic reasoning layer
    self.scallop_adder = scallopy.Module(
      provenance="difftopkproofs",
      k=3,
      program="rel sum(a + b) = digit_a(a) and digit_b(b)",
      input_mappings={"digit_a": range(10), "digit_b": range(10)},
      output_mapping=("sum", range(19))
    )

  def forward(self, img_a, img_b):
    logits_a = self.encoder(img_a)
    logits_b = self.encoder(img_b)

    probs_a = torch.softmax(logits_a, dim=1)
    probs_b = torch.softmax(logits_b, dim=1)

    sum_probs = self.scallop_adder(digit_a=probs_a, digit_b=probs_b)
    return sum_probs

# Training
model = DigitAdder()
optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)
loss_fn = nn.CrossEntropyLoss()

for epoch in range(num_epochs):
  for img_a, img_b, target_sum in train_loader:
    optimizer.zero_grad()

    sum_probs = model(img_a, img_b)
    loss = loss_fn(sum_probs, target_sum)

    loss.backward()
    optimizer.step()

  # Save checkpoint after each epoch
  torch.save({
    'epoch': epoch,
    'model_state_dict': model.state_dict(),
    'optimizer_state_dict': optimizer.state_dict(),
    'loss': loss.item(),
  }, f"checkpoint_epoch_{epoch}.pth")

# Save final model
torch.save(model, "digit_adder_final.pt")

# Later: Load for inference
loaded_model = torch.load("digit_adder_final.pt")
loaded_model.eval()

with torch.no_grad():
  result = loaded_model(test_img_a, test_img_b)
  predicted_sum = torch.argmax(result, dim=1)
```

---

## Checkpointing

For long training runs, save checkpoints with full training state:

### Saving Checkpoints

```python
# Save everything needed to resume training
checkpoint = {
  'epoch': epoch,
  'model_state_dict': model.state_dict(),
  'optimizer_state_dict': optimizer.state_dict(),
  'loss': loss.item(),
  'train_accuracy': train_acc,
  'val_accuracy': val_acc,
}

torch.save(checkpoint, f"checkpoint_epoch_{epoch}.pth")
```

### Resuming from Checkpoint

```python
# Recreate model and optimizer
model = DigitAdder()
optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)

# Load checkpoint
checkpoint = torch.load("checkpoint_epoch_42.pth")

# Restore state
model.load_state_dict(checkpoint['model_state_dict'])
optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
start_epoch = checkpoint['epoch'] + 1
last_loss = checkpoint['loss']

# Resume training
model.train()
for epoch in range(start_epoch, num_epochs):
  # Continue training...
  pass
```

---

## GPU/CPU Compatibility

Handle device mismatches when loading models:

### Saving on GPU, Loading on CPU

```python
# Model was trained on GPU
# ...

# Save
torch.save(model, "model_gpu.pt")

# Load on CPU
model = torch.load("model_gpu.pt", map_location=torch.device('cpu'))
```

### Saving on CPU, Loading on GPU

```python
# Load and move to GPU
model = torch.load("model_cpu.pt")
model = model.to('cuda')

# Or in one step
model = torch.load("model_cpu.pt", map_location='cuda:0')
```

### Flexible Loading

```python
# Load to current device
device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
model = torch.load("model.pt", map_location=device)
```

---

## Best Practices

### 1. Save State Dicts for Production

```python
# Recommended: Save state dict
torch.save(model.state_dict(), "model_weights.pth")

# Less recommended: Save entire module
torch.save(model, "model_full.pt")
```

**Why?** State dicts are more robust to code changes and PyTorch version updates.

### 2. Include Metadata

```python
torch.save({
  'model_state_dict': model.state_dict(),
  'optimizer_state_dict': optimizer.state_dict(),
  'epoch': epoch,
  'loss': loss.item(),
  'config': {
    'k': 3,
    'provenance': 'difftopkproofs',
    'learning_rate': 1e-3,
  }
}, "checkpoint.pth")
```

### 3. Version Your Models

```python
torch.save({
  'version': '1.0.0',
  'model_state_dict': model.state_dict(),
  # ...
}, f"model_v1.0.0_{timestamp}.pth")
```

### 4. Validate After Loading

```python
# Load model
model = torch.load("model.pt")

# Sanity check on validation data
model.eval()
with torch.no_grad():
  val_loss = compute_validation_loss(model, val_loader)
  print(f"Validation loss after loading: {val_loss:.4f}")
```

### 5. Use Eval Mode for Inference

```python
# Always set to eval mode after loading for inference
model = torch.load("model.pt")
model.eval()  # Disables dropout, batch norm, etc.

with torch.no_grad():  # Disable gradient computation
  predictions = model(input_data)
```

---

## Troubleshooting

### Error: "Can't find module"

**Problem**: Loading a saved module but Scallopy is not imported.

**Solution**: Import scallopy before loading:
```python
import scallopy
import torch

model = torch.load("model.pt")
```

### Error: "State dict keys don't match"

**Problem**: Module architecture changed between saving and loading.

**Solution**: Ensure the module architecture is identical:
```python
# Must recreate exact same architecture
model = scallopy.Module(
  # ... exact same parameters as when saved ...
)
model.load_state_dict(torch.load("weights.pth"))
```

### Model Behavior Differs After Loading

**Problem**: Model gives different results after loading.

**Checklist**:
1. Set model to eval mode: `model.eval()`
2. Disable gradients: `with torch.no_grad():`
3. Check device (CPU vs GPU)
4. Verify input preprocessing is identical

---

## Summary

- **Standard PyTorch**: Use `torch.save()` and `torch.load()`
- **Two approaches**: Save entire module or just state dict
- **State dict recommended**: More portable and robust
- **Checkpointing**: Save epoch, model, optimizer state for resuming
- **Device handling**: Use `map_location` for GPU/CPU compatibility
- **Best practices**: Eval mode, validation, versioning

For more details:
- [Creating Modules](module.md) - Building Scallop modules
- [PyTorch Serialization Docs](https://pytorch.org/docs/stable/notes/serialization.html) - Official PyTorch guide
