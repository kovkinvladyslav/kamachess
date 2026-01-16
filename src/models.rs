use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub message_id: i64,
    pub chat: Chat,
    pub text: Option<String>,
    pub from: Option<User>,
    pub reply_to_message: Option<ReplyMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyMessage {
    pub message_id: i64,
    pub from: Option<User>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Chat {
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct DbUser {
    pub id: i64,
    pub telegram_id: Option<i64>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    #[allow(dead_code)]
    pub last_name: Option<String>,
    pub wins: i64,
    pub losses: i64,
    pub draws: i64,
}

impl DbUser {
    pub fn display_name(&self) -> String {
        if let Some(username) = &self.username {
            format!("@{}", username)
        } else if let Some(first) = &self.first_name {
            first.clone()
        } else if let Some(id) = self.telegram_id {
            format!("user{}", id)
        } else {
            "player".to_string()
        }
    }

    pub fn mention_html(&self) -> String {
        if let Some(id) = self.telegram_id {
            let name = self
                .first_name
                .as_deref()
                .or(self.username.as_deref())
                .unwrap_or("player");
            format!(
                "<a href=\"tg://user?id={}\">{}</a>",
                id,
                crate::utils::escape_html(name)
            )
        } else if let Some(username) = &self.username {
            format!("@{}", crate::utils::escape_html(username))
        } else {
            "player".to_string()
        }
    }
}

#[derive(Debug, FromRow)]
pub struct GameRow {
    pub id: i64,
    #[allow(dead_code)]
    pub chat_id: i64,
    pub white_user_id: i64,
    pub black_user_id: i64,
    pub current_fen: String,
    pub turn: String,
    pub status: String,
    pub result: Option<String>,
    #[allow(dead_code)]
    pub last_message_id: Option<i64>,
    pub draw_proposed_by: Option<i64>,
    pub draw_proposal_message_id: Option<i64>,
}

#[derive(Debug, FromRow)]
pub struct HistoryRow {
    pub id: i64,
    pub local_num: i64,
    #[allow(dead_code)]
    pub started_at: String,
    pub result: Option<String>,
    pub white_username: Option<String>,
    pub black_username: Option<String>,
}

#[derive(Debug)]
pub enum UserRef {
    Telegram(User),
    #[allow(dead_code)]
    Username(String),
}

#[derive(Serialize)]
pub struct SendMessageRequest {
    pub chat_id: i64,
    pub text: String,
    pub reply_to_message_id: Option<i64>,
    pub parse_mode: Option<String>,
}

#[derive(Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    #[allow(dead_code)]
    pub error_code: Option<i32>,
    pub description: Option<String>,
}
