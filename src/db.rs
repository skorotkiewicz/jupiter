use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                display_name TEXT NOT NULL DEFAULT '',
                bio TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL REFERENCES users(id),
                role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_conversations_user ON conversations(user_id, created_at);

            CREATE TABLE IF NOT EXISTS agent_profiles (
                user_id TEXT PRIMARY KEY REFERENCES users(id),
                personality_summary TEXT NOT NULL DEFAULT '',
                interests TEXT NOT NULL DEFAULT '',
                core_values TEXT NOT NULL DEFAULT '',
                communication_style TEXT NOT NULL DEFAULT '',
                looking_for TEXT NOT NULL DEFAULT '',
                deal_breakers TEXT NOT NULL DEFAULT '',
                raw_notes TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS agent_peer_notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_user_id TEXT NOT NULL REFERENCES users(id),
                about_user_id TEXT NOT NULL REFERENCES users(id),
                compatibility_score REAL NOT NULL DEFAULT 0.0,
                notes TEXT NOT NULL DEFAULT '',
                recommends_match INTEGER NOT NULL DEFAULT 0,
                conversation_count INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(agent_user_id, about_user_id)
            );

            CREATE TABLE IF NOT EXISTS matches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_a_id TEXT NOT NULL REFERENCES users(id),
                user_b_id TEXT NOT NULL REFERENCES users(id),
                agent_a_approves INTEGER NOT NULL DEFAULT 0,
                agent_b_approves INTEGER NOT NULL DEFAULT 0,
                is_matched INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(user_a_id, user_b_id)
            );

            CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL REFERENCES users(id),
                notification_type TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                related_user_id TEXT REFERENCES users(id),
                is_read INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, is_read, created_at);

            CREATE TABLE IF NOT EXISTS direct_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id INTEGER NOT NULL REFERENCES matches(id),
                sender_id TEXT NOT NULL REFERENCES users(id),
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_dm_match ON direct_messages(match_id, created_at);
            ",
        )?;
        Ok(())
    }
}
