use async_trait::async_trait;
use vulcan_llm::error::LlmError;
use vulcan_llm::provider::LlmProvider;
use vulcan_llm::types::{ProviderRequest, ProviderResponse};

pub struct OpenAiProvider;

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, _request: ProviderRequest) -> Result<ProviderResponse, LlmError> {
        Err(LlmError::Provider("Not implemented".into()))
    }
}
