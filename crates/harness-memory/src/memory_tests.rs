use crate::{MemoryStore, MemoryEntry};

#[test]
fn store_and_retrieve_semantic_memory() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    let entry = MemoryEntry {
        id: None,
        category: "convention".into(),
        key: "code_style".into(),
        value: "Use snake_case".into(),
        confidence: 1.0,
    };
    store.store(entry).unwrap();
    let results = store.search("code style", 10).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn store_and_retrieve_episodic_memory() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    store.store_episodic("session-1", "Fixed compilation error", &["fix".into()]).unwrap();
    let results = store.search("compilation", 10).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn search_returns_relevant_results() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    store.store(MemoryEntry { id: None, category: "convention".into(), key: "rust_style".into(), value: "Use rustfmt".into(), confidence: 1.0 }).unwrap();
    store.store(MemoryEntry { id: None, category: "convention".into(), key: "python_style".into(), value: "Use black".into(), confidence: 1.0 }).unwrap();
    let results = store.search("rustfmt", 10).unwrap();
    assert!(results.iter().any(|r| r.key == "rust_style"));
}

#[test]
fn by_category_filters() {
    let mut store = MemoryStore::new_in_memory().unwrap();
    store.store(MemoryEntry { id: None, category: "convention".into(), key: "a".into(), value: "v1".into(), confidence: 1.0 }).unwrap();
    store.store(MemoryEntry { id: None, category: "preference".into(), key: "b".into(), value: "v2".into(), confidence: 1.0 }).unwrap();
    let results = store.by_category("convention").unwrap();
    assert_eq!(results.len(), 1);
}