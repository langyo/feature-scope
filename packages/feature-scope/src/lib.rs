//! # feature-scope
//!
//! A helper library that enables workspace crates to independently control their required features without cross-package interference.

pub use _macros::*;

mod build_loader;
mod utils;
pub use build_loader::load;
