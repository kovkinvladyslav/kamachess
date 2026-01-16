use crate::{handlers, AppState};
use anyhow::{anyhow, Result};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    routing::post,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

pub struct WebhookConfig {
    pub secret_token: Option<String>,
}

pub async fn start_webhook_server(
    state: Arc<AppState>,
    webhook_url: String,
    webhook_port: u16,
    webhook_path: String,
    secret_token: Option<String>,
) -> Result<()> {
    info!(webhook_url = %webhook_url, "Setting webhook URL");
    if let Err(err) = state
        .telegram
        .set_webhook(&webhook_url, secret_token.as_deref())
        .await
    {
        error!("Failed to set webhook: {err:?}");
        return Err(anyhow!("Failed to set webhook: {}", err));
    }
    info!("Webhook set successfully");

    let webhook_config = Arc::new(WebhookConfig { secret_token });
    let app = create_router(state.clone(), webhook_config.clone(), webhook_path);
    let addr = SocketAddr::from(([0, 0, 0, 0], webhook_port));
    info!(port = webhook_port, "Starting webhook server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(state));

    if let Err(err) = server.await {
        error!("Server error: {err:?}");
        return Err(anyhow!("Server error: {}", err));
    }

    Ok(())
}

pub fn create_router_for_test(
    state: Arc<AppState>,
    webhook_config: Arc<WebhookConfig>,
    webhook_path: String,
) -> Router {
    create_router(state, webhook_config, webhook_path)
}

fn create_router(
    state: Arc<AppState>,
    webhook_config: Arc<WebhookConfig>,
    webhook_path: String,
) -> Router {
    Router::new()
        .route(&webhook_path, post(webhook_handler))
        .route("/health", post(health_check))
        .layer(axum::middleware::from_fn_with_state(
            webhook_config,
            verify_secret_token_middleware,
        ))
        .with_state(state)
}

async fn verify_secret_token_middleware(
    State(config): State<Arc<WebhookConfig>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(expected_token) = &config.secret_token {
        let header_value = request
            .headers()
            .get("X-Telegram-Bot-Api-Secret-Token")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        if header_value != expected_token {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(request).await)
}

async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    axum::Json(update): axum::Json<crate::models::Update>,
) -> StatusCode {
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(err) = handlers::process_update(state_clone, update).await {
            error!("Failed to process update: {err:?}");
        }
    });

    StatusCode::OK
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn shutdown_signal(state: Arc<AppState>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, deleting webhook...");
    if let Err(err) = state.telegram.delete_webhook().await {
        warn!("Failed to delete webhook during shutdown: {err:?}");
    } else {
        info!("Webhook deleted successfully");
    }
}