use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::id::{MessageId, SessionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub mode: SessionMode,
    pub approval_mode: ApprovalMode,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SessionMode {
    #[default]
    Ask,
    Plan,
    Build,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ApprovalMode {
    #[default]
    Never,
    OnToolCall,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentBlock {
    Text { text: String },
    ToolCall { id: String, name: String, arguments: serde_json::Value },
    ToolResult { id: String, name: String, content: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_session_serde_roundtrip() {
        let ts = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let session = Session {
            id: SessionId("sess-1".into()),
            mode: SessionMode::Build,
            approval_mode: ApprovalMode::OnToolCall,
            messages: vec![Message {
                id: MessageId("msg-1".into()),
                role: MessageRole::User,
                content: vec![ContentBlock::Text { text: "hello".into() }],
                created_at: ts,
            }],
            created_at: ts,
            updated_at: ts,
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.messages.len(), 1);
        assert!(matches!(deserialized.messages[0].role, MessageRole::User));
    }

    #[test]
    fn test_message_role_serde() {
        let json = serde_json::to_string(&MessageRole::Assistant).unwrap();
        assert_eq!(json, r#""Assistant""#);
        let role: MessageRole = serde_json::from_str(r#""User""#).unwrap();
        assert!(matches!(role, MessageRole::User));
    }

    #[test]
    fn test_content_block_serde() {
        let block = ContentBlock::Text { text: "hi".into() };
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, ContentBlock::Text { .. }));
    }
}
