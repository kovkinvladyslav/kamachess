use anyhow::{anyhow, Result};
use kamachess::{api, db, handlers, AppState};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::{env, sync::Arc, time::Duration};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN")
        .map_err(|_| anyhow!("TELEGRAM_BOT_TOKEN environment variable is required"))?;
    let bot_username = env::var("TELEGRAM_BOT_USERNAME")
        .map_err(|_| anyhow!("TELEGRAM_BOT_USERNAME environment variable is required"))?
        .trim_start_matches('@')
        .to_string();
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "kamachess.db".to_string());

    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(())
    });
    let pool = Pool::new(manager)?;
    db::init_db(&pool)?;

    let state = Arc::new(AppState {
        db: pool,
        telegram: api::TelegramApi::new(bot_token),
        bot_username,
    });

    info!("Bot started. Waiting for updates...");
    
    let mut offset: Option<i64> = None;
    loop {
        match state.telegram.get_updates(offset, 30).await {
            Ok(updates) => {
                for update in updates {
                    offset = Some(update.update_id + 1);
                    
                    if let Err(err) = handlers::process_update(state.clone(), update).await {
                        error!("Failed to process update: {err:?}");
                    }
                }
            }
            Err(err) => {
                error!("Error getting updates: {err:?}");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
