//! Implementation of [`LinearSizedCollection`] for various types

mod alloc_collections;

#[cfg(feature = "alloc")]
pub use alloc_collections::*;
