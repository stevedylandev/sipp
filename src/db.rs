use rand::RngExt;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Connection>>;

pub struct Snippet {
    #[allow(dead_code)]
    pub id: i64,
    pub short_id: String,
    pub content: String,
    pub name: String,
}

const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn generate_short_id() -> String {
    let mut rng = rand::rng();
    (0..10)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

pub fn init_db() -> Db {
    let conn = Connection::open("sipp.sqlite").expect("Failed to open database");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS snippets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            short_id TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL,
            name TEXT NOT NULL
        )",
        [],
    )
    .expect("Failed to create table");
    Arc::new(Mutex::new(conn))
}

pub fn create_snippet(db: &Db, name: &str, content: &str) -> Snippet {
    let conn = db.lock().unwrap();
    let short_id = generate_short_id();
    conn.execute(
        "INSERT INTO snippets (short_id, content, name) VALUES (?1, ?2, ?3)",
        params![short_id, content, name],
    )
    .expect("Failed to insert snippet");
    let id = conn.last_insert_rowid();
    Snippet {
        id,
        short_id,
        content: content.to_string(),
        name: name.to_string(),
    }
}

pub fn get_snippet_by_short_id(db: &Db, short_id: &str) -> Option<Snippet> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, short_id, content, name FROM snippets WHERE short_id = ?1",
        params![short_id],
        |row| {
            Ok(Snippet {
                id: row.get(0)?,
                short_id: row.get(1)?,
                content: row.get(2)?,
                name: row.get(3)?,
            })
        },
    )
    .ok()
}

pub fn get_all_snippets(db: &Db) -> Vec<Snippet> {
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, short_id, content, name FROM snippets ORDER BY id DESC")
        .expect("Failed to prepare statement");
    stmt.query_map([], |row| {
        Ok(Snippet {
            id: row.get(0)?,
            short_id: row.get(1)?,
            content: row.get(2)?,
            name: row.get(3)?,
        })
    })
    .expect("Failed to query snippets")
    .filter_map(|r| r.ok())
    .collect()
}
