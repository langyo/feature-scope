//! # feature-scope Macros
//!
//! Procedural macros for the `feature-scope` library that enables workspace crates to
//! independently control their required features without cross-package interference.
//!
//! ## Overview
//!
//! This crate provides the `#[feature_scope]` and `#[feature_scope_default]` attribute macros
//! that allow you to conditionally compile code based on feature flags defined in your `Cargo.toml`.
//!
//! ## Configuration
//!
//! This library uses a two-step configuration approach:
//!
//! 1. **Declare features** in library crates using `package.metadata.feature-scope-decl`:
//!
//! ```toml
//! # In your library crate's Cargo.toml
//! [package.metadata.feature-scope-decl]
//! default = ["a"]
//! a = []
//! b = []
//! c = []
//! ```
//!
//! 2. **Configure feature usage** in consumer crates using `package.metadata.feature-scope`:
//!
//! ```toml
//! # In your binary/consumer crate's Cargo.toml
//! [[package.metadata.feature-scope]]
//! package = "your-library-name"
//! features = ["b"]
//! default-features = false
//! ```
//!
//! ## Usage
//!
//! Use the macros in your library code:
//!
//! ```rust
//! use feature_scope::{feature_scope, feature_scope_default};
//!
//! #[feature_scope_default(a)]
//! pub fn feature_a_function() {
//!     println!("This compiles when feature 'a' is enabled or by default");
//! }
//!
//! #[feature_scope(b)]
//! pub fn feature_b_function() {
//!     println!("This only compiles when feature 'b' is enabled");
//! }
//!
//! #[feature_scope_default]
//! pub fn default_function() {
//!     println!("This compiles by default");
//! }
//! ```
//!
//! ## Build Commands
//!
//! Use `cargo feature-scope` commands instead of regular `cargo` commands to build your project:
//!
//! ```bash
//! cargo feature-scope build
//! cargo feature-scope run
//! cargo feature-scope test
//! ```

mod parser;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn feature_scope(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let attr = parse_macro_input!(attr as parser::FeatureScope);

    let parser::FeatureScope { ident } = attr;
    quote! {
        #[allow(unexpected_cfgs)]
        #[cfg(#ident)]
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn feature_scope_default(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let attr = parse_macro_input!(_attr as parser::FeatureScopeDefault);

    if let Some(ident) = attr.ident {
        quote! {
            #[allow(unexpected_cfgs)]
            #[cfg(any(__scope_default, #ident))]
            #input
        }
        .into()
    } else {
        quote! {
            #[allow(unexpected_cfgs)]
            #[cfg(__scope_default)]
            #input
        }
        .into()
    }
}
