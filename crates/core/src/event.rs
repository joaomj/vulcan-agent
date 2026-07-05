use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::id::{ApprovalId, EventId, MessageId, ProviderCallId, SessionId, ToolCallId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub session_id: SessionId,
    pub sequence: u64,
    pub kind: EventKind,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventKind {
    SessionCreated { mode: String },
    MessageAppended { message_id: MessageId, role: String },
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
            EventKind::SessionCreated { mode: "build".into() },
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, session_id);
        assert_eq!(deserialized.sequence, 1);
    }

    #[test]
    fn test_event_kind_serde() {
        let kind = EventKind::SessionCreated { mode: "ask".into() };
        let json = serde_json::to_string(&kind).unwrap();
        assert!(json.contains(r#""type":"SessionCreated""#));
        let deserialized: EventKind = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, EventKind::SessionCreated { .. }));
    }
}
