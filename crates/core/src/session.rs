use crate::id::{MessageId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub mode: SessionMode,
    pub approval_mode: ApprovalMode,
    pub project_path: Option<String>,
    pub model: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeMetadata {
    pub source_project_path: String,
    pub worktree_path: String,
    pub head_revision: String,
    pub dirty_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SessionMode {
    #[default]
    Ask,
    Plan,
    Build,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ApprovalMode {
    #[default]
    Never,
    OnMode,
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
    Text {
        text: String,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        id: String,
        name: String,
        content: String,
    },
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
            approval_mode: ApprovalMode::OnMode,
            project_path: Some("/tmp/project".into()),
            model: "gpt-4o-mini".into(),
            messages: vec![Message {
                id: MessageId("msg-1".into()),
                role: MessageRole::User,
                content: vec![ContentBlock::Text {
                    text: "hello".into(),
                }],
                created_at: ts,
            }],
            created_at: ts,
            updated_at: ts,
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.project_path, Some("/tmp/project".into()));
        assert_eq!(deserialized.model, "gpt-4o-mini");
        assert_eq!(deserialized.messages.len(), 1);
        assert!(matches!(deserialized.messages[0].role, MessageRole::User));
    }

    #[test]
    fn test_worktree_metadata_serde() {
        let meta = WorktreeMetadata {
            source_project_path: "/project".into(),
            worktree_path: "/data/worktrees/project/sess-1".into(),
            head_revision: "abc123".into(),
            dirty_summary: Some("1 file modified".into()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: WorktreeMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source_project_path, "/project");
        assert_eq!(deserialized.head_revision, "abc123");
    }

    #[test]
    fn test_message_role_serde() {
        let json = serde_json::to_string(&MessageRole::Assistant).unwrap();
        assert_eq!(json, r#""Assistant""#);
        let role: MessageRole = serde_json::from_str(r#""User""#).unwrap();
        assert!(matches!(role, MessageRole::User));
    }

    #[test]
    fn test_prd_session_modes_serde() {
        let modes = [SessionMode::Ask, SessionMode::Plan, SessionMode::Build];
        let json = serde_json::to_string(&modes).unwrap();
        assert_eq!(json, r#"["Ask","Plan","Build"]"#);
    }

    #[test]
    fn test_prd_approval_modes_serde() {
        let modes = [
            ApprovalMode::Never,
            ApprovalMode::OnMode,
            ApprovalMode::Always,
        ];
        let json = serde_json::to_string(&modes).unwrap();
        assert_eq!(json, r#"["Never","OnMode","Always"]"#);
    }

    #[test]
    fn test_content_block_serde() {
        let block = ContentBlock::Text { text: "hi".into() };
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, ContentBlock::Text { .. }));
    }
}
