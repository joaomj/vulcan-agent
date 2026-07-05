use crate::error::LlmError;
use crate::types::{ProviderRequest, ProviderResponse};
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: ProviderRequest) -> Result<ProviderResponse, LlmError>;

    async fn cancel(&self) -> Result<(), LlmError> {
        let _ = self;
        Ok(())
    }
}
