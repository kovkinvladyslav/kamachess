use crate::models::{Message, User};
use crate::{db, parsing, AppState};
use anyhow::Result;
use std::sync::Arc;

pub async fn handle_history(
    state: Arc<AppState>,
    message: &Message,
    from: &User,
    text: &str,
) -> Result<()> {
    let conn = state.db.get()?;
    let chat_id = message.chat.id;

    let usernames: Vec<String> = parsing::extract_usernames(text)
        .into_iter()
        .filter(|name| !name.eq_ignore_ascii_case(&state.bot_username))
        .collect();

    let mut page = parsing::extract_page(text).unwrap_or(1);
    if page == 0 {
        page = 1;
    }

    let user_a = if let Some(username) = usernames.first() {
        db::upsert_user_by_username(&conn, username)?
    } else {
        db::upsert_user(&conn, from)?
    };

    let response = if let Some(username_b) = usernames.get(1) {
        let user_b = db::upsert_user_by_username(&conn, username_b)?;
        db::format_head_to_head(&conn, &user_a, &user_b, chat_id, page)?
    } else {
        db::format_user_history(&conn, &user_a, chat_id, page)?
    };

    state
        .telegram
        .send_message(chat_id, message.message_id, &response)
        .await?;

    Ok(())
}
