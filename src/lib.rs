pub mod api;
pub mod db;
pub mod game;
pub mod handlers;
pub mod models;
pub mod parsing;
pub mod server;
pub mod utils;

use sqlx::{Any, Pool};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Any>,
    pub telegram: api::TelegramApi,
    pub bot_username: String,
    pub no_trash: bool,
}
