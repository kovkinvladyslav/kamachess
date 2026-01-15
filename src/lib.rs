pub mod api;
pub mod db;
pub mod game;
pub mod handlers;
pub mod models;
pub mod parsing;
pub mod utils;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<SqliteConnectionManager>,
    pub telegram: api::TelegramApi,
    pub bot_username: String,
}
