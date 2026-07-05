use crate::error::StorageError;
use rusqlite::{params, Connection, OptionalExtension};
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

        let inserted = tx.execute(
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
        if inserted == 0 {
            let existing_event_id: Option<String> = tx
                .query_row(
                    "SELECT id FROM events WHERE session_id = ?1 AND sequence = ?2",
                    params![event.session_id.to_string(), event.sequence],
                    |row| row.get(0),
                )
                .optional()?;
            if existing_event_id.as_deref() != Some(event.id.0.as_str()) {
                return Err(StorageError::Conflict(format!(
                    "Event sequence {} already exists for session {}",
                    event.sequence, event.session_id
                )));
            }
        }

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

    pub fn delete_session(&mut self, session_id: &SessionId) -> Result<(), StorageError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM events WHERE session_id = ?1",
            params![session_id.to_string()],
        )?;
        tx.execute(
            "DELETE FROM sessions WHERE id = ?1",
            params![session_id.to_string()],
        )?;
        tx.commit()?;
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
    use chrono::{TimeZone, Utc};
    use rusqlite::Connection;
    use vulcan_core::event::{Event, EventKind};
    use vulcan_core::id::{EventId, MessageId, SessionId};
    use vulcan_core::session::{ContentBlock, MessageRole, SessionMode};

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migration::run(&mut conn).unwrap();
        conn
    }

    fn setup_store(conn: &mut Connection) -> EventStore<'_> {
        EventStore::new(conn)
    }

    fn event(session_id: SessionId, sequence: u64, kind: EventKind) -> Event {
        let mut event = Event::new(session_id, sequence, kind);
        event.id = EventId(format!("event-{}-{}", event.session_id, sequence));
        event.created_at = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, sequence as u32)
            .unwrap();
        event
    }

    #[test]
    fn test_append_and_replay() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-1".into());
        let event = event(
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
    fn append_is_idempotent_for_same_event_id() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-3".into());
        let event = event(
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
    fn append_rejects_duplicate_sequence_for_different_event() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-conflict".into());
        let first = event(
            session_id.clone(),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Build,
            },
        );
        let mut second = event(
            session_id.clone(),
            1,
            EventKind::MessageAppended {
                message_id: MessageId("msg-conflict".into()),
                role: MessageRole::User,
                content: vec![ContentBlock::Text {
                    text: "conflicting sequence".into(),
                }],
            },
        );
        second.id = EventId("different-event-id".into());

        store.append(&first).unwrap();
        let err = store.append(&second).unwrap_err();
        assert!(matches!(err, StorageError::Conflict(_)));

        let events = store.replay(&session_id).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].kind, EventKind::SessionCreated { .. }));
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
            .append(&event(
                s1.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Ask,
                },
            ))
            .unwrap();
        store
            .append(&event(
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
    fn delete_session_preserves_other_sessions() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-del".into());
        let other_session_id = SessionId("sess-keep".into());
        store
            .append(&event(
                session_id.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Build,
                },
            ))
            .unwrap();
        store
            .append(&event(
                other_session_id.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Ask,
                },
            ))
            .unwrap();

        store.delete_session(&session_id).unwrap();
        let events = store.replay(&session_id).unwrap();
        assert!(events.is_empty());

        let other_events = store.replay(&other_session_id).unwrap();
        assert_eq!(other_events.len(), 1);

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions, vec![other_session_id]);
    }

    #[test]
    fn replay_roundtrips_full_event_payload() {
        let mut conn = setup_db();
        let mut store = setup_store(&mut conn);

        let session_id = SessionId("sess-roundtrip".into());
        let expected = event(
            session_id.clone(),
            1,
            EventKind::MessageAppended {
                message_id: MessageId("msg-roundtrip".into()),
                role: MessageRole::User,
                content: vec![ContentBlock::Text {
                    text: "full payload".into(),
                }],
            },
        );

        store.append(&expected).unwrap();
        let events = store.replay(&session_id).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, expected.id);
        assert_eq!(events[0].session_id, expected.session_id);
        assert_eq!(events[0].sequence, expected.sequence);
        assert_eq!(events[0].version, expected.version);
        assert_eq!(events[0].created_at, expected.created_at);
        match &events[0].kind {
            EventKind::MessageAppended {
                message_id,
                role,
                content,
            } => {
                assert_eq!(message_id, &MessageId("msg-roundtrip".into()));
                assert!(matches!(role, MessageRole::User));
                assert!(
                    matches!(content.as_slice(), [ContentBlock::Text { text }] if text == "full payload")
                );
            }
            other => panic!("unexpected event kind: {other:?}"),
        }
    }
}
