use crate::id::{ApprovalId, EventId, MessageId, ProviderCallId, SessionId, ToolCallId};
use crate::session::{ContentBlock, MessageRole, SessionMode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub session_id: SessionId,
    pub sequence: u64,
    pub version: u64,
    pub kind: EventKind,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventKind {
    SessionCreated {
        mode: SessionMode,
    },
    MessageAppended {
        message_id: MessageId,
        role: MessageRole,
        content: Vec<ContentBlock>,
    },
    ToolCallRequested {
        tool_call_id: ToolCallId,
        name: String,
    },
    ToolResultReceived {
        tool_call_id: ToolCallId,
        status: String,
    },
    ApprovalRequested {
        approval_id: ApprovalId,
        tool_call_id: ToolCallId,
    },
    ApprovalDecided {
        approval_id: ApprovalId,
        decision: String,
    },
    ProviderCallStarted {
        provider_call_id: ProviderCallId,
        model: String,
    },
    ProviderCallFinished {
        provider_call_id: ProviderCallId,
        usage: Option<UsageData>,
    },
    ErrorRecorded {
        message: String,
    },
    SessionSummaryUpdated {
        summary: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl Event {
    pub fn new(session_id: SessionId, sequence: u64, kind: EventKind) -> Self {
        Self {
            id: EventId(uuid::Uuid::new_v4().to_string()),
            session_id,
            sequence,
            version: 1,
            kind,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let session_id = SessionId("sess-1".into());
        let event = Event::new(
            session_id.clone(),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Build,
            },
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, session_id);
        assert_eq!(deserialized.sequence, 1);
        assert_eq!(deserialized.version, 1);
    }

    #[test]
    fn test_event_kind_serde() {
        let kind = EventKind::SessionCreated {
            mode: SessionMode::Ask,
        };
        let json = serde_json::to_string(&kind).unwrap();
        assert!(json.contains(r#""type":"SessionCreated""#));
        assert!(json.contains(r#""Ask""#));
        let deserialized: EventKind = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, EventKind::SessionCreated { .. }));
    }

    #[test]
    fn test_event_version_default() {
        let event = Event::new(
            SessionId("sess-1".into()),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Ask,
            },
        );
        assert_eq!(event.version, 1);
    }

    #[test]
    fn test_event_ordering_by_sequence() {
        let sid = SessionId("sess-1".into());
        let e1 = Event::new(
            sid.clone(),
            1,
            EventKind::SessionCreated {
                mode: SessionMode::Ask,
            },
        );
        let e2 = Event::new(
            sid.clone(),
            2,
            EventKind::MessageAppended {
                message_id: MessageId("msg-1".into()),
                role: MessageRole::User,
                content: vec![ContentBlock::Text { text: "hi".into() }],
            },
        );
        assert!(e1.sequence < e2.sequence);
    }
}
