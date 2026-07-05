use crate::error::StorageError;
use rusqlite::{params, Connection};
use vulcan_core::event::{Event, EventKind};
use vulcan_core::id::SessionId;

pub struct EventStore<'a> {
    conn: &'a mut Connection,
}

impl<'a> EventStore<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    pub fn append(&mut self, event: &Event) -> Result<(), StorageError> {
        let payload = serde_json::to_value(&event.kind)?;
        let event_type = event_type_name(&event.kind);
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT OR IGNORE INTO sessions (id, created_at, updated_at)
             VALUES (?1, ?2, ?2)",
            params![event.session_id.to_string(), event.created_at.to_rfc3339(),],
        )?;

        tx.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![event.created_at.to_rfc3339(), event.session_id.to_string(),],
        )?;

        tx.execute(
            "INSERT OR IGNORE INTO events (id, session_id, sequence, version, event_type, payload, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.id.to_string(),
                event.session_id.to_string(),
                event.sequence,
                event.version,
                event_type,
                payload.to_string(),
                event.created_at.to_rfc3339(),
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn replay(&self, session_id: &SessionId) -> Result<Vec<Event>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, sequence, version, event_type, payload, created_at
             FROM events
             WHERE session_id = ?1
             ORDER BY sequence ASC",
        )?;

        let rows = stmt.query_map(params![session_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let sid: String = row.get(1)?;
            let sequence: u64 = row.get(2)?;
            let version: u64 = row.get(3)?;
            let payload: String = row.get(5)?;
            let created_at_str: String = row.get(6)?;

            let kind: EventKind = serde_json::from_str(&payload)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                .with_timezone(&chrono::Utc);

            Ok(Event {
                id: vulcan_core::id::EventId(id),
                session_id: vulcan_core::id::SessionId(sid),
                sequence,
                version,
                kind,
                created_at,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionId>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM sessions ORDER BY updated_at DESC")?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(SessionId(id))
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn delete_session(&self, session_id: &SessionId) -> Result<(), StorageError> {
        self.conn.execute(
            "DELETE FROM events WHERE session_id = ?1",
            params![session_id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM sessions WHERE id = ?1",
            params![session_id.to_string()],
        )?;
        Ok(())
    }
}

fn event_type_name(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::SessionCreated { .. } => "session_created",
        EventKind::MessageAppended { .. } => "message_appended",
        EventKind::ToolCallRequested { .. } => "tool_call_requested",
        EventKind::ToolResultReceived { .. } => "tool_result_received",
        EventKind::ApprovalRequested { .. } => "approval_requested",
        EventKind::ApprovalDecided { .. } => "approval_decided",
        EventKind::ProviderCallStarted { .. } => "provider_call_started",
        EventKind::ProviderCallFinished { .. } => "provider_call_finished",
        EventKind::ErrorRecorded { .. } => "error_recorded",
        EventKind::SessionSummaryUpdated { .. } => "session_summary_updated",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration;
    use rusqlite::Connection;
    use vulcan_core::event::EventKind;
    use vulcan_core::id::SessionId;
    use vulcan_core::session::{MessageRole, SessionMode};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migration::run(&conn).unwrap();
        conn
    }

    fn setup_store(conn: &mut Connection) -> EventStore<'_> {
        EventStore::new(conn)
    }

    #[test]
    fn test_append_and_replay() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-1".into());
        let event = Event::new(
            session_id.clone(),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Build,
            },
        );

        store.append(&event).unwrap();

        let events = store.replay(&session_id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 1);
        assert_eq!(events[0].version, 1);
        assert!(matches!(events[0].kind, EventKind::SessionCreated { .. }));
    }

    #[test]
    fn test_replay_order() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-2".into());

        let e1 = Event::new(
            session_id.clone(),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Build,
            },
        );
        let e2 = Event::new(
            session_id.clone(),
            2,
            EventKind::MessageAppended {
                message_id: vulcan_core::id::MessageId("msg-1".into()),
                role: MessageRole::User,
                content: Vec::new(),
            },
        );
        let e3 = Event::new(
            session_id.clone(),
            3,
            EventKind::ProviderCallStarted {
                provider_call_id: vulcan_core::id::ProviderCallId("pc-1".into()),
                model: "gpt-4".into(),
            },
        );

        store.append(&e1).unwrap();
        store.append(&e3).unwrap();
        store.append(&e2).unwrap();

        let events = store.replay(&session_id).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].sequence, 1);
        assert_eq!(events[1].sequence, 2);
        assert_eq!(events[2].sequence, 3);
    }

    #[test]
    fn test_duplicate_append() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-3".into());
        let event = Event::new(
            session_id.clone(),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Build,
            },
        );

        store.append(&event).unwrap();
        store.append(&event).unwrap();

        let events = store.replay(&session_id).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_missing_session_replay() {
        let mut conn = setup_db();
        let store = setup_store(&mut conn);

        let events = store.replay(&SessionId("nonexistent".into())).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_list_sessions() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let s1 = SessionId("sess-a".into());
        let s2 = SessionId("sess-b".into());

        store
            .append(&Event::new(
                s1.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Ask,
                },
            ))
            .unwrap();
        store
            .append(&Event::new(
                s2.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Build,
                },
            ))
            .unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_delete_session() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-del".into());
        store
            .append(&Event::new(
                session_id.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Build,
                },
            ))
            .unwrap();

        store.delete_session(&session_id).unwrap();
        let events = store.replay(&session_id).unwrap();
        assert!(events.is_empty());

        let sessions = store.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }
}
