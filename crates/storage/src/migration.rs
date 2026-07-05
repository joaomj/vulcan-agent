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
    // Migration 12: unique event sequence per session
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_events_session_sequence_unique
        ON events(session_id, sequence)",
];

pub fn run(conn: &mut Connection) -> Result<(), StorageError> {
    run_migrations(conn, MIGRATIONS)
}

fn run_migrations(conn: &mut Connection, migrations: &[&str]) -> Result<(), StorageError> {
    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for (i, sql) in migrations.iter().enumerate() {
        let version = (i + 1) as i64;
        if version <= current_version {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [version],
        )?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        run(&mut conn).unwrap();
        run(&mut conn).unwrap();
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, MIGRATIONS.len() as i64);
    }

    #[test]
    fn migrations_create_expected_events_schema() {
        let mut conn = Connection::open_in_memory().unwrap();
        run(&mut conn).unwrap();

        let version_column_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('events') WHERE name = 'version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version_column_count, 1);

        let recall_fts_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'recall_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(recall_fts_count, 1);
    }

    #[test]
    fn migrations_create_unique_event_sequence_index() {
        let mut conn = Connection::open_in_memory().unwrap();
        run(&mut conn).unwrap();

        let is_unique: i64 = conn
            .query_row(
                "SELECT [unique] FROM pragma_index_list('events') WHERE name = 'idx_events_session_sequence_unique'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(is_unique, 1);

        let indexed_columns: String = conn
            .query_row(
                "SELECT group_concat(name, ',') FROM pragma_index_info('idx_events_session_sequence_unique') ORDER BY seqno",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(indexed_columns, "session_id,sequence");
    }

    #[test]
    fn failed_migration_rolls_back_schema_and_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = [
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            "CREATE TABLE stable (id INTEGER PRIMARY KEY)",
            "CREATE TABLE partial (id INTEGER PRIMARY KEY);
             SELECT * FROM table_that_does_not_exist",
        ];

        let err = run_migrations(&mut conn, &migrations).unwrap_err();
        assert!(err.to_string().contains("table_that_does_not_exist"));

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 2);

        let partial_table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'partial'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(partial_table_count, 0);
    }
}
