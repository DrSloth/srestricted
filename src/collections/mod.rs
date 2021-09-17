mod std_collections;

#[cfg(any(feature="std", feature="alloc"))]
pub use std_collections::*;
