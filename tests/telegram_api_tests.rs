use kamachess::api::TelegramApi;
use serde_json::json;
use wiremock::{
    matchers::{body_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_set_webhook_success() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    let expected_body = json!({
        "url": "https://example.com/webhook"
    });

    Mock::given(method("POST"))
        .and(path("/bot123/setWebhook"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": true,
            "description": "Webhook was set"
        })))
        .mount(&mock_server)
        .await;

    let result = api
        .set_webhook("https://example.com/webhook", None)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_webhook_with_secret_token() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    let expected_body = json!({
        "url": "https://example.com/webhook",
        "secret_token": "my-secret-token"
    });

    Mock::given(method("POST"))
        .and(path("/bot123/setWebhook"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": true,
            "description": "Webhook was set"
        })))
        .mount(&mock_server)
        .await;

    let result = api
        .set_webhook("https://example.com/webhook", Some("my-secret-token"))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_webhook_error() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    Mock::given(method("POST"))
        .and(path("/bot123/setWebhook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": false,
            "error_code": 400,
            "description": "Bad Request: webhook URL must start with 'https://'"
        })))
        .mount(&mock_server)
        .await;

    let result = api.set_webhook("http://example.com/webhook", None).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("webhook URL must start with 'https://'"));
}

#[tokio::test]
async fn test_delete_webhook_success() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    Mock::given(method("POST"))
        .and(path("/bot123/deleteWebhook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": true,
            "description": "Webhook was deleted"
        })))
        .mount(&mock_server)
        .await;

    let result = api.delete_webhook().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_webhook_error() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    Mock::given(method("POST"))
        .and(path("/bot123/deleteWebhook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": false,
            "error_code": 401,
            "description": "Unauthorized"
        })))
        .mount(&mock_server)
        .await;

    let result = api.delete_webhook().await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unauthorized"));
}

#[tokio::test]
async fn test_get_webhook_info_success() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    let expected_info = json!({
        "url": "https://example.com/webhook",
        "has_custom_certificate": false,
        "pending_update_count": 0
    });

    Mock::given(method("GET"))
        .and(path("/bot123/getWebhookInfo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": expected_info
        })))
        .mount(&mock_server)
        .await;

    let result = api.get_webhook_info().await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info["url"], "https://example.com/webhook");
    assert_eq!(info["pending_update_count"], 0);
}

#[tokio::test]
async fn test_get_webhook_info_no_webhook() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    Mock::given(method("GET"))
        .and(path("/bot123/getWebhookInfo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": {
                "url": "",
                "has_custom_certificate": false,
                "pending_update_count": 0
            }
        })))
        .mount(&mock_server)
        .await;

    let result = api.get_webhook_info().await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info["url"], "");
}

#[tokio::test]
async fn test_get_webhook_info_error() {
    let mock_server = MockServer::start().await;
    let api = TelegramApi::new_with_base_url(format!("http://{}/bot123", mock_server.address()));

    Mock::given(method("GET"))
        .and(path("/bot123/getWebhookInfo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": false,
            "error_code": 401,
            "description": "Unauthorized"
        })))
        .mount(&mock_server)
        .await;

    let result = api.get_webhook_info().await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unauthorized"));
}
