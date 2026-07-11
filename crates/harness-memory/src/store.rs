use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Option<i64>,
    pub category: String,
    pub key: String,
    pub value: String,
    pub confidence: f64,
}

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn new_in_memory() -> crate::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn new(path: &str) -> crate::Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> crate::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS semantic_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                confidence REAL DEFAULT 1.0,
                last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(category, key)
            );
            CREATE TABLE IF NOT EXISTS episodic_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                summary TEXT NOT NULL,
                tags TEXT
            );"
        )?;
        Ok(())
    }

    pub fn store(&mut self, entry: MemoryEntry) -> crate::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO semantic_memory (category, key, value, confidence) VALUES (?1, ?2, ?3, ?4)",
            params![entry.category, entry.key, entry.value, entry.confidence],
        )?;
        Ok(())
    }

    pub fn store_episodic(&mut self, session_id: &str, summary: &str, tags: &[String]) -> crate::Result<()> {
        let tags_json = serde_json::to_string(tags).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO episodic_memory (session_id, summary, tags) VALUES (?1, ?2, ?3)",
            params![session_id, summary, tags_json],
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> crate::Result<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category, key, value, confidence FROM semantic_memory
             WHERE key LIKE ?1 OR value LIKE ?1 OR category LIKE ?1
             ORDER BY confidence DESC
             LIMIT ?2"
        )?;
        let pattern = format!("%{}%", query);
        let entries = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok(MemoryEntry {
                id: Some(row.get(0)?),
                category: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                confidence: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(entries)
    }

    pub fn by_category(&self, category: &str) -> crate::Result<Vec<MemoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category, key, value, confidence FROM semantic_memory WHERE category = ?1"
        )?;
        let entries = stmt.query_map(params![category], |row| {
            Ok(MemoryEntry {
                id: Some(row.get(0)?),
                category: row.get(1)?,
                key: row.get(2)?,
                value: row.get(3)?,
                confidence: row.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(entries)
    }
}