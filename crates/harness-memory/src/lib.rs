pub mod store;

pub use store::{MemoryStore, MemoryEntry};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod memory_tests;