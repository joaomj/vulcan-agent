use crate::error::StorageError;
use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    // Migration 0: schema version tracking
    "CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY,
        applied_at TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    // Migration 1: sessions table
    "CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )",
    // Migration 2: events table
    "CREATE TABLE IF NOT EXISTS events (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        sequence INTEGER NOT NULL,
        event_type TEXT NOT NULL,
        payload TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (session_id) REFERENCES sessions(id)
    )",
    // Migration 3: event order index
    "CREATE INDEX IF NOT EXISTS idx_events_session_sequence
        ON events(session_id, sequence)",
    // Migration 4: messages table
    "CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        role TEXT NOT NULL,
        content TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (session_id) REFERENCES sessions(id)
    )",
    // Migration 5: messages session index
    "CREATE INDEX IF NOT EXISTS idx_messages_session
        ON messages(session_id, created_at)",
    // Migration 6: add version column to events
    "ALTER TABLE events ADD COLUMN version INTEGER NOT NULL DEFAULT 1",
    // Migration 7: tool_calls queryable table
    "CREATE TABLE IF NOT EXISTS tool_calls (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        name TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        created_at TEXT NOT NULL,
        FOREIGN KEY (session_id) REFERENCES sessions(id)
    )",
    "CREATE INDEX IF NOT EXISTS idx_tool_calls_session
        ON tool_calls(session_id, created_at)",
    // Migration 8: provider_calls queryable table
    "CREATE TABLE IF NOT EXISTS provider_calls (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        model TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'started',
        created_at TEXT NOT NULL,
        FOREIGN KEY (session_id) REFERENCES sessions(id)
    )",
    "CREATE INDEX IF NOT EXISTS idx_provider_calls_session
        ON provider_calls(session_id, created_at)",
    // Migration 9: approvals queryable table
    "CREATE TABLE IF NOT EXISTS approvals (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        tool_call_id TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        decision TEXT,
        created_at TEXT NOT NULL,
        FOREIGN KEY (session_id) REFERENCES sessions(id)
    )",
    "CREATE INDEX IF NOT EXISTS idx_approvals_session
        ON approvals(session_id, created_at)",
    // Migration 10: snapshots table
    "CREATE TABLE IF NOT EXISTS snapshots (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        worktree_path TEXT NOT NULL,
        head_revision TEXT NOT NULL,
        dirty_summary TEXT,
        created_at TEXT NOT NULL,
        FOREIGN KEY (session_id) REFERENCES sessions(id)
    )",
    "CREATE INDEX IF NOT EXISTS idx_snapshots_session
        ON snapshots(session_id, created_at)",
    // Migration 11: recall FTS5 table
    "CREATE VIRTUAL TABLE IF NOT EXISTS recall_fts USING fts5(
        session_id UNINDEXED,
        content,
        tokenize='porter unicode61'
    )",
];

pub fn run(conn: &Connection) -> Result<(), StorageError> {
    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i64;
        if version <= current_version {
            continue;
        }
        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [version],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        run(&conn).unwrap();
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, MIGRATIONS.len() as i64);
    }
}
