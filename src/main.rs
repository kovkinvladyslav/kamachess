use anyhow::{anyhow, Result};
use kamachess::{api, db, server, AppState};
use sqlx::any::AnyPoolOptions;
use std::{env, sync::Arc};
use tracing::info;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let log_dir = env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    std::fs::create_dir_all(&log_dir)?;
    let file_appender = tracing_appender::rolling::daily(&log_dir, "kamachess.log");
    let (non_blocking, _log_guard) = tracing_appender::non_blocking(file_appender);
    let env_filter = tracing_subscriber::EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN")
        .map_err(|_| anyhow!("TELEGRAM_BOT_TOKEN environment variable is required"))?;
    let bot_username = env::var("TELEGRAM_BOT_USERNAME")
        .map_err(|_| anyhow!("TELEGRAM_BOT_USERNAME environment variable is required"))?
        .trim_start_matches('@')
        .to_string();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://kamachess.db?mode=rwc".to_string());
    
    let no_trash = !env::args().any(|arg| arg == "--keep-messages");

    sqlx::any::install_default_drivers();

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    db::run_migrations(&pool, &database_url).await?;

    let state = Arc::new(AppState {
        db: pool,
        telegram: api::TelegramApi::new(bot_token),
        bot_username,
        no_trash,
    });
    
    if !no_trash {
        info!("Keep-messages mode: previous board messages will be kept during gameplay");
    }

    let webhook_url = env::var("WEBHOOK_URL")
        .map_err(|_| anyhow!("WEBHOOK_URL environment variable is required"))?;
    let webhook_port = env::var("WEBHOOK_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .map_err(|_| anyhow!("WEBHOOK_PORT must be a valid port number"))?;
    let webhook_path = env::var("WEBHOOK_PATH")
        .unwrap_or_else(|_| "/webhook".to_string());
    let webhook_secret_token = env::var("WEBHOOK_SECRET_TOKEN").ok();

    info!("Starting webhook server...");
    info!(webhook_url = %webhook_url, webhook_port = webhook_port, webhook_path = %webhook_path, "Webhook configuration");

    server::start_webhook_server(
        state,
        webhook_url,
        webhook_port,
        webhook_path,
        webhook_secret_token,
    )
    .await
}
