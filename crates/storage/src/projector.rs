use chrono::{DateTime, Utc};
use vulcan_core::event::{Event, EventKind};
use vulcan_core::session::{ApprovalMode, Message, MessageRole, Session, SessionMode};

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
        let mut messages: Vec<Message> = Vec::new();
        let mut created_at: Option<DateTime<Utc>> = None;
        let mut updated_at: Option<DateTime<Utc>> = None;

        for event in events {
            updated_at = Some(event.created_at);

            match &event.kind {
                EventKind::SessionCreated { mode: m } => {
                    mode = match m.as_str() {
                        "ask" => SessionMode::Ask,
                        "plan" => SessionMode::Plan,
                        "build" => SessionMode::Build,
                        "debug" => SessionMode::Debug,
                        other => {
                            projection.warnings.push(format!(
                                "Unknown session mode '{}' at event {}",
                                other, event.sequence
                            ));
                            SessionMode::Ask
                        }
                    };
                    if created_at.is_none() {
                        created_at = Some(event.created_at);
                    }
                }
                EventKind::MessageAppended { message_id, role } => {
                    let msg_role = match role.as_str() {
                        "user" => MessageRole::User,
                        "assistant" => MessageRole::Assistant,
                        "system" => MessageRole::System,
                        "tool" => MessageRole::Tool,
                        other => {
                            projection.warnings.push(format!(
                                "Unknown message role '{}' at event {}",
                                other, event.sequence
                            ));
                            MessageRole::User
                        }
                    };
                    messages.push(Message {
                        id: message_id.clone(),
                        role: msg_role,
                        content: Vec::new(),
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
    use vulcan_core::event::Event;
    use vulcan_core::id::{MessageId, ProviderCallId, SessionId};

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
            Event::new(session_id.clone(), 1, EventKind::SessionCreated { mode: "build".into() }),
            Event::new(session_id.clone(), 2, EventKind::MessageAppended {
                message_id: MessageId("msg-1".into()),
                role: "user".into(),
            }),
        ];

        let projector = SessionProjector::new();
        let result = projector.project(&events);

        let session = result.session.unwrap();
        assert!(matches!(session.mode, SessionMode::Build));
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.id, session_id);
    }

    #[test]
    fn test_projection_order() {
        let session_id = SessionId("sess-2".into());
        let events = vec![
            Event::new(session_id.clone(), 1, EventKind::SessionCreated { mode: "ask".into() }),
            Event::new(session_id.clone(), 2, EventKind::MessageAppended {
                message_id: MessageId("msg-1".into()),
                role: "user".into(),
            }),
            Event::new(session_id.clone(), 3, EventKind::ProviderCallStarted {
                provider_call_id: ProviderCallId("pc-1".into()),
                model: "gpt-4".into(),
            }),
            Event::new(session_id.clone(), 4, EventKind::MessageAppended {
                message_id: MessageId("msg-2".into()),
                role: "assistant".into(),
            }),
            Event::new(session_id.clone(), 5, EventKind::ErrorRecorded {
                message: "test error".into(),
            }),
        ];

        let projector = SessionProjector::new();
        let result = projector.project(&events);

        assert_eq!(result.errors, vec!["test error"]);
        assert_eq!(result.session.as_ref().unwrap().messages.len(), 2);
    }

    #[test]
    fn test_unknown_mode_warning() {
        let session_id = SessionId("sess-3".into());
        let events = vec![
            Event::new(session_id.clone(), 1, EventKind::SessionCreated { mode: "unknown_mode".into() }),
        ];

        let projector = SessionProjector::new();
        let result = projector.project(&events);

        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("unknown_mode"));
    }
}
