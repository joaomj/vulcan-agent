use serde::{Deserialize, Serialize};
use vulcan_core::id::ToolCallId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub approval_class: ApprovalClass,
    pub isolation_level: IsolationLevel,
    pub cancellation_support: CancellationSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalClass {
    Never,
    OnMode,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    None,
    Worktree,
    ChildWorktree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CancellationSupport {
    None,
    Request,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: ToolCallId,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: ToolCallId,
    pub status: ToolStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub metadata: Option<ToolMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStatus {
    Success,
    Error,
    Cancelled,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub duration_ms: u64,
    pub redacted_count: u32,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentPolicy {
    pub max_parallel: u32,
    pub allowed_tools: Vec<String>,
    pub blocked_tools: Vec<String>,
}
