# feature-scope

## Introduction

A helper library that enables workspace crates to independently control their required features without cross-package interference.

> Still in development, the API may change in the future.

## Quick Start

You must set the `package.metadata.feature-scope` in your `Cargo.toml` file first. It's a table like `features` but with a different purpose. Each crates in the workspace can have its own `package.metadata.feature-scope` table, and the features listed in it will be used to control the visibility of the macros in this crate.

```toml
[package.metadata.feature-scope]
default = ["a"]
a = []
b = []
c = []
```

This library depends on the `build.rs` to provide the arguments to the compiler. You needs to call `load` function inside the crate like this:

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
