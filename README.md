# Scallop Documentation

Comprehensive documentation for [Scallop](https://github.com/scallop-lang/scallop) - a neurosymbolic programming language that combines logic programming with probabilistic reasoning and differentiable computation.

## 📖 View Documentation

**Live Documentation**: [https://[username].github.io/scallop-docs/](https://[username].github.io/scallop-docs/) *(update after deployment)*

## 📚 What's Inside

This repository contains:

### Language Documentation
- **Language Reference Guide** - Complete guide to Scallop syntax and semantics
  - Relations and Facts
  - Writing Rules
  - Values and Types
  - Queries and Recursion
  - Negations and Aggregations
  - Algebraic Data Types (ADTs)
  - Loading from CSV
  - Foreign Functions and Predicates
- **Provenance and Probabilistic Programming** - Probabilistic reasoning features
  - Provenance types and semantics
  - Facts with probabilities
  - Logic and probability integration
  - Sampling and aggregation

### API Documentation

#### Rust API (Complete)
- **Getting Started** - IntegrateContext basics
- **IntegrateContext API** - Core API patterns and usage
- **Foreign Functions** - Implementing custom functions (ForeignFunction trait)
- **Foreign Predicates** - Implementing custom predicates (ForeignPredicate trait)
- **Provenance Types** - Unit, TopKProofs, Incremental, and more
- **Working Examples** - 6 verified, working Rust examples

#### Python API (scallopy)
- **Getting Started** - Scallop Context basics
- **Branching Executions**
- **Configuring Provenance**
- **Creating Modules**
- **Input/Output Relations**
- **Foreign Functions and Predicates**
- **Saving and Loading**
- **Debugging Proofs**

### Plugin System
- **Foundation Models Integration** - OpenAI GPT, Gemini, Transformers
- **Create Your Own Plugin** - Complete plugin development guide
- **Foreign Functions, Predicates, and Attributes**
- **GPU Utilities and CodeQL integration**

### Working Examples
Six verified Rust examples demonstrating core functionality:
- **basic** - IntegrateContext fundamentals
- **foreign_functions** - Custom function implementation
- **foreign_predicates** - CSV-style data generation with bf/ff patterns
- **incremental_evaluation** - Dynamic fact addition
- **complex_reasoning** - TopKProofs provenance and proof tracking
- **test_stdlib_ff** - Using built-in foreign functions

All examples compile and run successfully (verified 2026-01-13).

## 🚀 Building Locally

### Prerequisites
- [mdBook](https://github.com/rust-lang/mdBook) - Install with: `cargo install mdbook`

### Build and Serve
```bash
# Clone the repository
git clone https://github.com/[username]/scallop-docs.git
cd scallop-docs

# Build the book
mdbook build

# Serve locally (opens at http://localhost:3000)
mdbook serve
```

The built documentation will be in the `book/` directory.

## 📝 Documentation Structure

```
scallop-docs/
├── src/                    # Documentation source (Markdown)
│   ├── SUMMARY.md         # Table of contents
│   ├── introduction.md
│   ├── language/          # Language reference
│   ├── probabilistic/     # Provenance & probability
│   ├── rust_api/          # Rust API docs
│   ├── scallopy/          # Python API docs
│   ├── foundation_models/ # Plugin system docs
│   └── toolchain/         # CLI tools
├── examples/              # Working code examples
│   └── rust/             # Rust examples
├── book.toml             # mdBook configuration
└── README.md             # This file
```

## 🔗 Related Resources

- **Main Scallop Repository**: [scallop-lang/scallop](https://github.com/scallop-lang/scallop)
- **Scallop Website**: https://scallop-lang.github.io/
- **Academic Papers**: See the main repository for publications

## ⚖️ License

This documentation repository is licensed under the MIT License, the same as the main Scallop project.

**Copyright (c) 2023 Ziyang Li**

This repository includes documentation and examples derived from the [Scallop project](https://github.com/scallop-lang/scallop), which is licensed under the MIT License. See the [LICENSE](LICENSE) file for full details.

## 🤝 Contributing

This documentation repository is maintained separately from the main Scallop project. For documentation improvements:

1. Fork this repository
2. Make your changes
3. Test locally with `mdbook serve`
4. Submit a pull request

For Scallop language/compiler contributions, see the [main repository](https://github.com/scallop-lang/scallop).

---

**Note**: This is an independent documentation repository created to provide comprehensive, accessible documentation for Scallop users. It includes verified working examples and detailed API documentation based on extensive testing and analysis.
