# Create Your Own Plugin

This guide walks through creating a custom Scallop plugin from scratch. You'll learn the plugin development workflow, implement foreign constructs, and package your plugin for distribution.

## Overview

Creating a Scallop plugin involves:
1. **Project setup** - Directory structure and configuration
2. **Plugin class** - Implement three lifecycle hooks
3. **Foreign constructs** - Add functions, predicates, or attributes
4. **Testing** - Verify functionality locally
5. **Distribution** - Package and share your plugin

### Prerequisites

- Python 3.8+
- `scallopy` installed (`pip install scallopy`)
- Basic understanding of Scallop syntax
- Familiarity with Python decorators

## Complete Tutorial: Weather Plugin

We'll build a plugin that fetches weather data from an API and makes it available in Scallop programs.

### Step 1: Project Structure

Create the following directory structure:

```
scallop-weather/
├── pyproject.toml
├── README.md
└── src/
    └── scallop_weather/
        ├── __init__.py
        └── plugin.py
```

**Create the project directory:**
```bash
mkdir -p scallop-weather/src/scallop_weather
cd scallop-weather
```

### Step 2: Configuration File

**File: `pyproject.toml`**

```toml
[project]
name = "scallop-weather"
version = "0.1.0"
description = "Weather data integration for Scallop"
authors = [{name = "Your Name", email = "you@example.com"}]
readme = "README.md"
requires-python = ">=3.8"
dependencies = [
    "scallopy>=0.1.0",
    "requests>=2.28.0",
]

[project.entry-points."scallop.plugin"]
weather = "scallop_weather:ScallopWeatherPlugin"

[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.build_meta"

[tool.setuptools.packages.find]
where = ["src"]
```

**Key elements:**
- `dependencies` - Required packages (scallopy, requests for API calls)
- `project.entry-points."scallop.plugin"` - Registers plugin for auto-discovery
- Entry point format: `plugin_name = "module:PluginClass"`

### Step 3: Plugin Implementation

**File: `src/scallop_weather/__init__.py`**

```python
from .plugin import ScallopWeatherPlugin

__all__ = ["ScallopWeatherPlugin"]
```

**File: `src/scallop_weather/plugin.py`**

```python
import os
import requests
import scallopy
from scallopy import foreign_function, foreign_predicate, Facts
from typing import Tuple, Optional

class ScallopWeatherPlugin(scallopy.Plugin):
    """Plugin for fetching weather data in Scallop programs."""

    def __init__(self):
        super().__init__("weather")
        self._api_key: Optional[str] = None
        self._base_url = "https://api.openweathermap.org/data/2.5/weather"
        self._cache = {}  # Memoization cache

    def setup_argparse(self, parser):
        """Hook 1: Declare command-line arguments."""
        parser.add_argument(
            "--weather-api-key",
            type=str,
            help="OpenWeatherMap API key"
        )
        parser.add_argument(
            "--weather-units",
            type=str,
            default="metric",
            choices=["metric", "imperial"],
            help="Temperature units (metric=Celsius, imperial=Fahrenheit)"
        )

    def configure(self, args, unknown_args):
        """Hook 2: Initialize plugin state from arguments."""
        # Get API key from args or environment
        self._api_key = args.get("weather_api_key") or os.getenv("WEATHER_API_KEY")
        self._units = args.get("weather_units", "metric")

        if not self._api_key:
            print("[scallop-weather] Warning: No API key provided.")
            print("  Set WEATHER_API_KEY environment variable or use --weather-api-key")
            print("  Using mock data for demonstrations.")

    def load_into_ctx(self, ctx):
        """Hook 3: Register foreign constructs with Scallop context."""

        # Foreign function: Simple temperature lookup
        @foreign_function(name="get_temperature")
        def get_temperature(city: str) -> float:
            """Get current temperature for a city."""
            if not self._api_key:
                # Mock data when no API key
                mock_temps = {"London": 15.5, "Paris": 18.2, "Tokyo": 22.0}
                return mock_temps.get(city, 20.0)

            # Check cache first
            cache_key = f"temp_{city}"
            if cache_key in self._cache:
                return self._cache[cache_key]

            try:
                response = requests.get(
                    self._base_url,
                    params={
                        "q": city,
                        "appid": self._api_key,
                        "units": self._units
                    },
                    timeout=5
                )
                response.raise_for_status()
                data = response.json()
                temp = data["main"]["temp"]

                # Cache result
                self._cache[cache_key] = temp
                return temp

            except Exception as e:
                print(f"[scallop-weather] Error fetching temperature for {city}: {e}")
                return 0.0

        # Foreign predicate: Full weather data with multiple results
        @foreign_predicate(
            name="weather",
            input_arg_types=[str],
            output_arg_types=[str, float, int]
        )
        def weather_data(city: str) -> Facts[float, Tuple[str, float, int]]:
            """Get weather condition, temperature, and humidity."""
            if not self._api_key:
                # Mock data when no API key
                mock_data = {
                    "London": [("cloudy", 15.5, 72)],
                    "Paris": [("sunny", 18.2, 45), ("partly cloudy", 18.0, 50)],
                    "Tokyo": [("rainy", 22.0, 85)]
                }
                results = mock_data.get(city, [("clear", 20.0, 50)])
                for condition, temp, humidity in results:
                    yield (1.0, (condition, temp, humidity))
                return

            # Check cache
            cache_key = f"weather_{city}"
            if cache_key in self._cache:
                condition, temp, humidity = self._cache[cache_key]
                yield (1.0, (condition, temp, humidity))
                return

            try:
                response = requests.get(
                    self._base_url,
                    params={
                        "q": city,
                        "appid": self._api_key,
                        "units": self._units
                    },
                    timeout=5
                )
                response.raise_for_status()
                data = response.json()

                condition = data["weather"][0]["description"]
                temp = data["main"]["temp"]
                humidity = data["main"]["humidity"]

                # Cache result
                self._cache[cache_key] = (condition, temp, humidity)

                # Yield with probability 1.0 (certain fact)
                yield (1.0, (condition, temp, humidity))

            except Exception as e:
                print(f"[scallop-weather] Error fetching weather for {city}: {e}")

        # Register both constructs with context
        ctx.register_foreign_function(get_temperature)
        ctx.register_foreign_predicate(weather_data)
```

### Step 4: Testing Locally

**Install in development mode:**
```bash
# From scallop-weather/ directory
pip install -e .
```

**Create test Scallop program (`test_weather.scl`):**
```scl
// Test foreign function
rel cities = {"London", "Paris", "Tokyo"}
rel temperatures(city, temp) = cities(city), temp = $get_temperature(city)

// Test foreign predicate
rel forecast(city, condition, temp, humidity) =
  cities(city),
  weather(city, condition, temp, humidity)

query temperatures
query forecast
```

**Run the test:**
```bash
# Without API key (uses mock data)
scli test_weather.scl

# With API key
export WEATHER_API_KEY="your-api-key"
scli test_weather.scl --weather-units metric
```

**Expected output (mock data):**
```
temperatures: {
  ("London", 15.5),
  ("Paris", 18.2),
  ("Tokyo", 22.0)
}

forecast: {
  ("London", "cloudy", 15.5, 72),
  ("Paris", "sunny", 18.2, 45),
  ("Paris", "partly cloudy", 18.0, 50),
  ("Tokyo", "rainy", 22.0, 85)
}
```

### Step 5: Advanced Features

#### Foreign Attribute Example

Add a foreign attribute for automatic weather monitoring:

```python
# In plugin.py, add to load_into_ctx():

from scallopy import foreign_attribute

@foreign_attribute
def monitor_weather(
    item,
    cities: list,
    update_interval: int = 3600
):
    """
    Foreign attribute that periodically fetches weather.

    Usage:
      @monitor_weather(cities=["London", "Paris"], update_interval=3600)
      rel current_weather(city: String, condition: String, temp: f32)
    """
    # Validate the relation has correct arity
    if not item.is_relation():
        raise Exception("@monitor_weather can only be applied to relations")

    # Generate a foreign predicate
    pred_name = f"_monitor_{item.relation_name()}"

    @foreign_predicate(name=pred_name, output_arg_types=[str, str, float])
    def monitor_impl() -> Facts[float, Tuple[str, str, float]]:
        for city in cities:
            # Call the weather API
            try:
                response = requests.get(
                    self._base_url,
                    params={"q": city, "appid": self._api_key, "units": self._units},
                    timeout=5
                )
                data = response.json()
                condition = data["weather"][0]["description"]
                temp = data["main"]["temp"]
                yield (1.0, (city, condition, temp))
            except:
                pass

    ctx.register_foreign_predicate(monitor_impl)

    # Add rule to populate the relation
    ctx.add_rule(f"{item.relation_name()}(city, condition, temp) :- {pred_name}(city, condition, temp)")

ctx.register_foreign_attribute(monitor_weather)
```

**Usage in Scallop:**
```scl
@monitor_weather(cities=["London", "Paris"])
rel current_weather(city: String, condition: String, temp: f32)

query current_weather
```

## Best Practices

### 1. Error Handling

**Always provide graceful fallbacks:**
```python
try:
    result = expensive_operation()
    return result
except Exception as e:
    print(f"[plugin-name] Error: {e}")
    return default_value
```

### 2. Memoization

**Cache expensive operations:**
```python
def __init__(self):
    super().__init__("plugin_name")
    self._cache = {}

def expensive_function(self, key):
    if key in self._cache:
        return self._cache[key]

    result = compute_result(key)
    self._cache[key] = result
    return result
```

### 3. Lazy Loading

**Load heavy dependencies only when needed:**
```python
class MyPlugin(scallopy.Plugin):
    def __init__(self):
        super().__init__("my_plugin")
        self._model = None  # Don't load yet

    def _load_model(self):
        if self._model is None:
            import heavy_library
            self._model = heavy_library.load_model()
        return self._model

    def load_into_ctx(self, ctx):
        @foreign_function(name="predict")
        def predict(input_data):
            model = self._load_model()  # Load on first use
            return model.predict(input_data)

        ctx.register_foreign_function(predict)
```

### 4. GPU Support

**Integrate with GPU utilities plugin:**
```python
def load_into_ctx(self, ctx):
    try:
        from scallop_gpu import get_device
        device = get_device()
    except ImportError:
        device = "cpu"

    # Use device for PyTorch models
    model = load_model().to(device)
```

### 5. API Key Management

**Support multiple configuration methods:**
```python
def configure(self, args, unknown_args):
    # Priority: CLI arg > environment > config file
    self._api_key = (
        args.get("my_api_key") or
        os.getenv("MY_API_KEY") or
        self._load_from_config()
    )

    if not self._api_key:
        print("[plugin] Warning: No API key provided")
```

### 6. Type Annotations

**Always specify types for foreign constructs:**
```python
# Foreign function with types
@foreign_function(name="func", return_type=float)
def func(x: int, y: str) -> float:
    ...

# Foreign predicate with input/output types
@foreign_predicate(
    name="pred",
    input_arg_types=[str, int],
    output_arg_types=[float, bool]
)
def pred(s: str, n: int) -> Facts[float, Tuple[float, bool]]:
    ...
```

### 7. Mock Data Pattern

**Always provide mock data for API-based plugins:**
```python
def api_call(self, param):
    if not self._has_credentials():
        # Return mock data with comment
        mock_result = {"data": "example"}
        print("[plugin] Using mock data (no API key)")
        return mock_result

    # Real API call
    return requests.get(self._url, params={"q": param}).json()
```

## Distribution

### Building Wheels

**Create distributable package:**
```bash
# Install build tools
pip install build

# Build wheel and source distribution
python -m build

# Output in dist/
# dist/scallop_weather-0.1.0-py3-none-any.whl
# dist/scallop_weather-0.1.0.tar.gz
```

### Publishing to PyPI

**Upload to PyPI:**
```bash
pip install twine

# Test on TestPyPI first
twine upload --repository testpypi dist/*

# Production upload
twine upload dist/*
```

**Users can then install:**
```bash
pip install scallop-weather
```

### Local Installation Methods

**For development:**
```bash
# Editable install (changes reflected immediately)
pip install -e .

# Or use in Scallop's plugin directory
cd /path/to/scallop/etc/scallopy-plugins
ln -s /path/to/scallop-weather ./weather
make develop-weather
```

## Common Patterns

### Pattern 1: Database Integration

```python
@foreign_predicate(name="sql_query", output_arg_types=[str, int])
def sql_query(query: str) -> Facts[float, Tuple[str, int]]:
    """Execute SQL query and return results."""
    import sqlite3
    conn = sqlite3.connect(self._db_path)
    cursor = conn.execute(query)
    for row in cursor:
        yield (1.0, tuple(row))
```

### Pattern 2: File Processing

```python
@foreign_function(name="read_csv")
def read_csv(filepath: str) -> list:
    """Read CSV file and return as list of tuples."""
    import csv
    with open(filepath) as f:
        reader = csv.reader(f)
        return [tuple(row) for row in reader]
```

### Pattern 3: External Tool Integration

```python
@foreign_predicate(name="lint_code", output_arg_types=[str, int, str])
def lint_code(filepath: str) -> Facts[float, Tuple[str, int, str]]:
    """Run linter and yield warnings."""
    import subprocess
    result = subprocess.run(
        ["pylint", filepath],
        capture_output=True,
        text=True
    )
    # Parse output and yield issues
    for line in result.stdout.split('\n'):
        if match := parse_warning(line):
            yield (1.0, match)
```

## Debugging Tips

**Enable debug mode:**
```python
def __init__(self):
    super().__init__("plugin_name")
    self._debug = False

def setup_argparse(self, parser):
    parser.add_argument("--plugin-debug", action="store_true")

def configure(self, args, unknown_args):
    self._debug = args.get("plugin_debug", False)
    if self._debug:
        print("[plugin] Debug mode enabled")

def load_into_ctx(self, ctx):
    @foreign_function(name="func")
    def func(x):
        if self._debug:
            print(f"[plugin] func called with x={x}")
        return compute(x)
```

**Run with debug flag:**
```bash
scli program.scl --plugin-debug
```

## Testing Your Plugin

**Create test suite (`tests/test_plugin.py`):**
```python
import scallopy
from scallop_weather import ScallopWeatherPlugin

def test_plugin_loads():
    ctx = scallopy.ScallopContext()
    plugin = ScallopWeatherPlugin()
    plugin.configure({}, [])
    plugin.load_into_ctx(ctx)
    assert "get_temperature" in ctx.list_foreign_functions()

def test_foreign_function():
    ctx = scallopy.ScallopContext()
    plugin = ScallopWeatherPlugin()
    plugin.configure({}, [])
    plugin.load_into_ctx(ctx)

    ctx.add_program("""
        rel result = {$get_temperature("London")}
        query result
    """)
    ctx.run()
    results = list(ctx.relation("result"))
    assert len(results) == 1
    assert results[0][0] > 0  # Temperature is positive

def test_foreign_predicate():
    ctx = scallopy.ScallopContext()
    plugin = ScallopWeatherPlugin()
    plugin.configure({}, [])
    plugin.load_into_ctx(ctx)

    ctx.add_program("""
        rel forecast(c, cond, t, h) = weather("Paris", cond, t, h)
        query forecast
    """)
    ctx.run()
    results = list(ctx.relation("forecast"))
    assert len(results) > 0
```

**Run tests:**
```bash
pip install pytest
pytest tests/
```

## Next Steps

Now that you've created a plugin, explore these advanced topics:

- **[GPU Utilities](gpu_utilities.md)** - Add GPU acceleration to your plugin
- **[OpenAI GPT Plugin](openai_gpt.md)** - Example of API integration with memoization
- **[Transformers Plugin](transformers.md)** - Example of model loading and foreign attributes
- **[References](references.md)** - Quick API reference for plugin development

For questions or contributions, see the [Scallop GitHub repository](https://github.com/scallop-lang/scallop).
