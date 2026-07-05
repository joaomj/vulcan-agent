use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::config::ServerConfig;
use crate::state::AppState;

pub type SharedState = Arc<RwLock<AppState>>;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

pub async fn serve(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(RwLock::new(AppState::new(config.clone())?));
    let router = build_router(state);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let config = ServerConfig::default();
        let state = Arc::new(RwLock::new(AppState::new(config).unwrap()));
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
