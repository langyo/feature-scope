# feature-scope

![Crates.io License](https://img.shields.io/crates/l/feature-scope)
[![Crates.io Version](https://img.shields.io/crates/v/feature-scope)](https://docs.rs/feature-scope)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/langyo/feature-scope/test.yml)

> **English | [中文](README_zh.md)**

## Introduction

A helper library that enables workspace crates to independently control their required features without cross-package interference.

> Still in development, the API may change in the future.

## How It Works

This library solves the **feature unification problem** in Rust workspaces by implementing a custom feature scoping mechanism based on Rust's `-cfg` compilation parameters.

### The Problem

In traditional Cargo workspaces, when multiple crates depend on the same library with different feature requirements, Cargo unifies all features together. This can lead to:

- **Feature conflicts**: Different crates requiring mutually exclusive features of the same dependency
- **Unintended compilation**: Code being compiled with features that weren't explicitly requested
- **Diamond dependency issues**: Transitive dependencies causing unexpected feature combinations

### The Solution

`feature-scope` bypasses Cargo's feature unification by:

1. **Custom cfg flags**: Instead of using Cargo features, it generates custom `--cfg __scope_<feature>` flags
2. **CLI wrapper**: The `cargo feature-scope` command intercepts build commands and injects the appropriate cfg flags
3. **Procedural macros**: `#[feature_scope]` and `#[feature_scope_default]` macros translate your feature declarations into cfg-based conditional compilation
4. **Independent control**: Each crate in the workspace can specify exactly which features it wants from each dependency

This approach allows different crates in the same workspace to use completely different feature sets from the same dependency without interference, solving the feature unification problem at the compilation level.

## Installation

To use this library effectively, you need to install the `cargo-feature-scope` CLI tool:

```bash
cargo install cargo-feature-scope
```

Or install from source:

```bash
git clone https://github.com/langyo/feature-scope.git
cd feature-scope
cargo install --path packages/cli
```

## Quick Start

This library uses a two-step configuration approach:

1. **Declare features** in library crates using `package.metadata.feature-scope-decl`:

```toml
# In your library crate's Cargo.toml
[package.metadata.feature-scope-decl]
default = ["a"]
a = []
b = []
c = []
```

1. **Configure feature usage** in consumer crates using `package.metadata.feature-scope`:

```toml
# In your binary/consumer crate's Cargo.toml
[[package.metadata.feature-scope]]
package = "your-library-name"
features = ["b"]
default-features = false
```

This library depends on the `cargo-feature-scope` CLI tool to provide the correct compiler arguments. You need to use `cargo feature-scope` instead of regular `cargo` commands when building or running your project:

```bash
# Build your project
cargo feature-scope build

# Run your project
cargo feature-scope run

# Run a specific package in workspace
cargo feature-scope run -p your-package-name

# Run tests
cargo feature-scope test
```

Then, you can use the `feature_scope` macro in your code:

```rust
// This function will be available when feature 'a' is enabled or by default
#[feature_scope_default(a)]
pub fn feature_a_function() {
    println!("feature_scope_a");
}

// This function only compiles when feature 'b' is enabled
#[feature_scope(b)]
pub fn feature_b_function() {
    println!("feature_scope_b");
}

// This function is available by default
#[feature_scope_default]
pub fn default_function() {
    println!("default function");
}

// Call the appropriate functions based on the feature flags set in Cargo.toml
feature_a_function();
feature_b_function();
default_function();
```

## Examples

The repository includes working examples in the `examples/` directory. To run the basic workspace example:

```bash
# Navigate to the example directory
cd examples/basic_workspace

# Build all packages in the workspace
cargo feature-scope build

# Run the default entry point (uses default features)
cargo feature-scope run -p entry_default
# Output:
# a type
# default type

# Run the custom entry point (uses feature 'b')
cargo feature-scope run -p entry_custom
# Output:
# b type
# b type
```

This demonstrates how different packages in the same workspace can have different feature configurations:

- `entry_default`: Uses the default features defined in `types` package (feature `a`)
- `entry_custom`: Uses a custom feature configuration (feature `b`)
- `types`: The shared library that provides different implementations based on enabled features

## Development

### Running Tests

The project includes comprehensive CI workflows that test both the main codebase and examples:

1. **Unit Tests**: Run with `cargo test --all-targets --all-features --workspace`
2. **Examples Tests**: Automated testing of the `basic_workspace` example to ensure correct output
3. **Code Quality**: Clippy linting and formatting checks

To run the example tests locally:

```bash
# Using cargo-make (recommended)
cargo install cargo-make
cargo make test-examples

# Or run all tests including examples
cargo make test-all

# Or manually
cd examples/basic_workspace
cargo feature-scope build
cargo feature-scope run -p entry_default
cargo feature-scope run -p entry_custom
```

---

**Note**: Starting from version 0.2.0, all source code is implemented through human-AI collaboration using Copilot. Only architecture design, result review, and commit records involve manual intervention.
