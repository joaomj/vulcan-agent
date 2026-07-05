use chrono::{DateTime, Utc};
use vulcan_core::event::{Event, EventKind};
use vulcan_core::session::{ApprovalMode, Message, Session, SessionMode};

#[derive(Debug, Clone, Default)]
pub struct Projection {
    pub session: Option<Session>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct SessionProjector;

impl SessionProjector {
    pub fn new() -> Self {
        Self
    }

    pub fn project(&self, events: &[Event]) -> Projection {
        let mut projection = Projection::default();
        if events.is_empty() {
            return projection;
        }

        let session_id = events[0].session_id.clone();
        let mut mode = SessionMode::Ask;
        let approval_mode = ApprovalMode::Never;
        let model = String::new();
        let project_path: Option<String> = None;
        let mut messages: Vec<Message> = Vec::new();
        let mut created_at: Option<DateTime<Utc>> = None;
        let mut updated_at: Option<DateTime<Utc>> = None;
        let mut seen_session_created = false;
        let mut previous_sequence: Option<u64> = None;

        for event in events {
            if event.session_id != session_id {
                projection.warnings.push(format!(
                    "Event sequence {} belongs to session {}, expected {}",
                    event.sequence, event.session_id, session_id
                ));
            }
            if previous_sequence.is_some_and(|previous| event.sequence <= previous) {
                projection.warnings.push(format!(
                    "Out-of-order event sequence {} after {:?}",
                    event.sequence, previous_sequence
                ));
            }
            previous_sequence = Some(event.sequence);

            if event.version > 1 {
                projection.warnings.push(format!(
                    "Unsupported event version {} at sequence {}",
                    event.version, event.sequence
                ));
            }
            updated_at = Some(event.created_at);

            match &event.kind {
                EventKind::SessionCreated { mode: m } => {
                    if seen_session_created {
                        projection.warnings.push(format!(
                            "Duplicate session creation at sequence {}",
                            event.sequence
                        ));
                    }
                    seen_session_created = true;
                    mode = m.clone();
                    if created_at.is_none() {
                        created_at = Some(event.created_at);
                    }
                }
                EventKind::MessageAppended {
                    message_id,
                    role,
                    content,
                } => {
                    if !seen_session_created {
                        projection.warnings.push(format!(
                            "Message appended before session creation at sequence {}",
                            event.sequence
                        ));
                    }
                    messages.push(Message {
                        id: message_id.clone(),
                        role: role.clone(),
                        content: content.clone(),
                        created_at: event.created_at,
                    });
                }
                EventKind::ToolCallRequested { .. }
                | EventKind::ToolResultReceived { .. }
                | EventKind::ApprovalRequested { .. }
                | EventKind::ApprovalDecided { .. }
                | EventKind::ProviderCallStarted { .. }
                | EventKind::ProviderCallFinished { .. }
                | EventKind::SessionSummaryUpdated { .. } => {
                    // Projection tracks these events exist; detailed state
                    // for approval, tool, and provider tracking can be added
                    // in later steps.
                }
                EventKind::ErrorRecorded { message } => {
                    projection.errors.push(message.clone());
                }
            }
        }

        let created = created_at.unwrap_or_else(Utc::now);
        let updated = updated_at.unwrap_or_else(Utc::now);

        projection.session = Some(Session {
            id: session_id,
            mode,
            approval_mode,
            project_path,
            model,
            messages,
            created_at: created,
            updated_at: updated,
        });

        projection
    }
}

impl Default for SessionProjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use vulcan_core::event::Event;
    use vulcan_core::id::{EventId, MessageId, ProviderCallId, SessionId};
    use vulcan_core::session::{ContentBlock, MessageRole, SessionMode};

    fn event(session_id: SessionId, sequence: u64, kind: EventKind) -> Event {
        let mut event = Event::new(session_id, sequence, kind);
        event.id = EventId(format!("event-{}", sequence));
        event.created_at = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, sequence as u32)
            .unwrap();
        event
    }

    #[test]
    fn test_empty_events() {
        let projector = SessionProjector::new();
        let result = projector.project(&[]);
        assert!(result.session.is_none());
    }

    #[test]
    fn test_simple_session_projection() {
        let session_id = SessionId("sess-1".into());
        let events = vec![
            Event::new(
                session_id.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Build,
                },
            ),
            Event::new(
                session_id.clone(),
                2,
                EventKind::MessageAppended {
                    message_id: MessageId("msg-1".into()),
                    role: MessageRole::User,
                    content: vec![ContentBlock::Text {
                        text: "hello".into(),
                    }],
                },
            ),
        ];

        let projector = SessionProjector::new();
        let result = projector.project(&events);

        let session = result.session.unwrap();
        assert!(matches!(session.mode, SessionMode::Build));
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.id, session_id);
        assert!(matches!(session.messages[0].role, MessageRole::User));
        assert_eq!(session.messages[0].content.len(), 1);
    }

    #[test]
    fn test_projection_order() {
        let session_id = SessionId("sess-2".into());
        let events = vec![
            Event::new(
                session_id.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Ask,
                },
            ),
            Event::new(
                session_id.clone(),
                2,
                EventKind::MessageAppended {
                    message_id: MessageId("msg-1".into()),
                    role: MessageRole::User,
                    content: Vec::new(),
                },
            ),
            Event::new(
                session_id.clone(),
                3,
                EventKind::ProviderCallStarted {
                    provider_call_id: ProviderCallId("pc-1".into()),
                    model: "gpt-4".into(),
                },
            ),
            Event::new(
                session_id.clone(),
                4,
                EventKind::MessageAppended {
                    message_id: MessageId("msg-2".into()),
                    role: MessageRole::Assistant,
                    content: Vec::new(),
                },
            ),
            Event::new(
                session_id.clone(),
                5,
                EventKind::ErrorRecorded {
                    message: "test error".into(),
                },
            ),
        ];

        let projector = SessionProjector::new();
        let result = projector.project(&events);

        assert_eq!(result.errors, vec!["test error"]);
        assert_eq!(result.session.as_ref().unwrap().messages.len(), 2);
    }

    #[test]
    fn test_unsupported_version_warning() {
        let session_id = SessionId("sess-3".into());
        let mut event = event(
            session_id.clone(),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Ask,
            },
        );
        event.version = 99;

        let projector = SessionProjector::new();
        let result = projector.project(&[event]);

        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("99"));
    }

    #[test]
    fn projector_sets_created_and_updated_from_events() {
        let session_id = SessionId("sess-times".into());
        let events = vec![
            event(
                session_id.clone(),
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Ask,
                },
            ),
            event(
                session_id,
                2,
                EventKind::MessageAppended {
                    message_id: MessageId("msg-time".into()),
                    role: MessageRole::User,
                    content: Vec::new(),
                },
            ),
        ];

        let result = SessionProjector::new().project(&events);
        let session = result.session.unwrap();

        assert_eq!(session.created_at, events[0].created_at);
        assert_eq!(session.updated_at, events[1].created_at);
    }

    #[test]
    fn projector_warns_on_mixed_session_events() {
        let expected_session = SessionId("sess-expected".into());
        let other_session = SessionId("sess-other".into());
        let events = vec![
            event(
                expected_session,
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Ask,
                },
            ),
            event(
                other_session,
                2,
                EventKind::ErrorRecorded {
                    message: "wrong stream".into(),
                },
            ),
        ];

        let result = SessionProjector::new().project(&events);

        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("belongs to session sess-other")));
    }

    #[test]
    fn projector_warns_on_out_of_order_events() {
        let session_id = SessionId("sess-order".into());
        let events = vec![
            event(
                session_id.clone(),
                2,
                EventKind::MessageAppended {
                    message_id: MessageId("msg-order".into()),
                    role: MessageRole::User,
                    content: Vec::new(),
                },
            ),
            event(
                session_id,
                1,
                EventKind::SessionCreated {
                    mode: SessionMode::Ask,
                },
            ),
        ];

        let result = SessionProjector::new().project(&events);

        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("Out-of-order event sequence 1")));
    }

    #[test]
    fn projector_warns_when_message_precedes_session_creation() {
        let session_id = SessionId("sess-message-first".into());
        let events = vec![event(
            session_id,
            1,
            EventKind::MessageAppended {
                message_id: MessageId("msg-first".into()),
                role: MessageRole::User,
                content: Vec::new(),
            },
        )];

        let result = SessionProjector::new().project(&events);

        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("before session creation")));
        assert_eq!(result.session.unwrap().messages.len(), 1);
    }
}
