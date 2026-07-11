pub mod store;
#[cfg(test)] mod memory_tests;

pub use store::{MemoryStore, MemoryEntry};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;