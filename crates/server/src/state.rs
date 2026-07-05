use std::sync::Arc;
use tokio::sync::RwLock;
use vulcan_core::error::CoreError;
use crate::config::ServerConfig;

pub struct AppState {
    pub config: ServerConfig,
}

impl AppState {
    pub fn new(config: ServerConfig) -> Result<Self, CoreError> {
        Ok(Self { config })
    }
}

pub type SharedState = Arc<RwLock<AppState>>;
