use serde::{Deserialize, Serialize};
use vulcan_core::id::{ProviderCallId, ToolCallId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub messages: Vec<ProviderMessage>,
    pub tools: Vec<ToolSpec>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentBlock {
    Text { text: String },
    ToolCall {
        id: ToolCallId,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        id: ToolCallId,
        name: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub message: ProviderMessage,
    pub usage: Option<Usage>,
    pub provider_call_id: ProviderCallId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl ProviderRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            messages: Vec::new(),
            tools: Vec::new(),
            model: model.into(),
        }
    }
}
