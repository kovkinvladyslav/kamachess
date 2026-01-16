use kamachess::{
    api,
    models::{Chat, Message, Update, User},
    server::{create_router_for_test, WebhookConfig},
    AppState,
};
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use sqlx::any::AnyPoolOptions;
use std::sync::Arc;
use tower::util::ServiceExt;

async fn create_test_state() -> Arc<AppState> {
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    Arc::new(AppState {
        db: pool,
        telegram: api::TelegramApi::new("test-token".to_string()),
        bot_username: "testbot".to_string(),
        no_trash: true,
    })
}

fn create_test_update() -> Update {
    Update {
        update_id: 1,
        message: Some(Message {
            message_id: 1,
            chat: Chat { id: 123 },
            text: Some("/help".to_string()),
            from: Some(User {
                id: 456,
                is_bot: false,
                username: Some("testuser".to_string()),
                first_name: Some("Test".to_string()),
                last_name: None,
            }),
            reply_to_message: None,
        }),
    }
}

#[tokio::test]
async fn test_webhook_handler_returns_ok() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: None,
        }),
        "/webhook".to_string(),
    );

    let update = create_test_update();
    let body = serde_json::to_string(&update).unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/webhook")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_check() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: None,
        }),
        "/webhook".to_string(),
    );

    let request = Request::builder()
        .method("POST")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_secret_token_middleware_without_token() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: None,
        }),
        "/webhook".to_string(),
    );

    let update = create_test_update();
    let body = serde_json::to_string(&update).unwrap();

    // Request without secret token header should succeed when no token is configured
    let request = Request::builder()
        .method("POST")
        .uri("/webhook")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_secret_token_middleware_with_valid_token() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: Some("test-secret".to_string()),
        }),
        "/webhook".to_string(),
    );

    let update = create_test_update();
    let body = serde_json::to_string(&update).unwrap();

    // Request with valid secret token should succeed
    let request = Request::builder()
        .method("POST")
        .uri("/webhook")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Telegram-Bot-Api-Secret-Token", "test-secret")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_secret_token_middleware_with_invalid_token() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: Some("test-secret".to_string()),
        }),
        "/webhook".to_string(),
    );

    let update = create_test_update();
    let body = serde_json::to_string(&update).unwrap();

    // Request with invalid secret token should fail
    let request = Request::builder()
        .method("POST")
        .uri("/webhook")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Telegram-Bot-Api-Secret-Token", "wrong-secret")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_secret_token_middleware_missing_header() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: Some("test-secret".to_string()),
        }),
        "/webhook".to_string(),
    );

    let update = create_test_update();
    let body = serde_json::to_string(&update).unwrap();

    // Request without secret token header should fail when token is required
    let request = Request::builder()
        .method("POST")
        .uri("/webhook")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_custom_webhook_path() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: None,
        }),
        "/custom/webhook".to_string(),
    );

    let update = create_test_update();
    let body = serde_json::to_string(&update).unwrap();

    // Request to custom path should work
    let request = Request::builder()
        .method("POST")
        .uri("/custom/webhook")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_webhook_handler_with_invalid_json() {
    let state = create_test_state().await;
    let app = create_router_for_test(
        state.clone(),
        Arc::new(WebhookConfig {
            secret_token: None,
        }),
        "/webhook".to_string(),
    );

    let request = Request::builder()
        .method("POST")
        .uri("/webhook")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 400 Bad Request for invalid JSON
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
