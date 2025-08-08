# feature-scope

![Crates.io License](https://img.shields.io/crates/l/feature-scope)
[![Crates.io Version](https://img.shields.io/crates/v/feature-scope)](https://docs.rs/feature-scope)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/langyo/feature-scope/test.yml)

## Introduction

A helper library that enables workspace crates to independently control their required features without cross-package interference.

> Still in development, the API may change in the future.

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

You must set the `package.metadata.feature-scope` in your `Cargo.toml` file first. It's a table like `features` but with a different purpose. Each crates in the workspace can have its own `package.metadata.feature-scope` table, and the features listed in it will be used to control the visibility of the macros in this crate.

```toml
[package.metadata.feature-scope]
default = ["a"]
a = []
b = []
c = []
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

For the macro configuration, you need to call `load` function inside the build script like this:

```rust
// build.rs

fn main() {
    // Load the feature scope from the Cargo.toml file.
    feature_scope::load().unwrap();
}
```

Then, you can use the `feature_scope` macro in your code:

```rust
// The default feature is `a`, so this function will be available when the `a` feature is enabled.
#[feature_scope(a)]
pub fn basic_expand() {
    println!("feature_scope_a");
}

#[feature_scope(any(b, c))]
pub fn basic_expand() {
    println!("feature_scope_b_or_c");
}

#[feature_scope(only(a, b, c))]
compile_error! {
    // This will cause a compile error if all features are enabled.
    "This should not be compiled!"
}


// prints "feature_scope_a" or "feature_scope_b_c" depending on the feature flag set in Cargo.toml
basic_expand();
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
