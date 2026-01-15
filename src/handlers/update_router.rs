use anyhow::Result;
use std::sync::Arc;
use crate::models::Update;
use crate::AppState;
use super::{game_handler, history_handler};

pub async fn process_update(state: Arc<AppState>, update: Update) -> Result<()> {
    let Some(message) = update.message else {
        return Ok(());
    };
    let Some(text) = &message.text else {
        return Ok(());
    };
    let Some(from) = &message.from else {
        return Ok(());
    };
    
    if from.is_bot {
        return Ok(());
    }

    if text.starts_with("/history") {
        history_handler::handle_history(state, &message, from, text).await?;
        return Ok(());
    }

    if text.starts_with("/start") {
        let replied_to_user = message
            .reply_to_message
            .as_ref()
            .and_then(|msg| msg.from.as_ref())
            .map(|user| !user.is_bot)
            .unwrap_or(false);

        if replied_to_user {
            game_handler::handle_start_game(state, &message, from, text).await?;
            return Ok(());
        }
    }

    let replied_to_bot = message
        .reply_to_message
        .as_ref()
        .and_then(|msg| msg.from.as_ref())
        .map(|user| user.is_bot)
        .unwrap_or(false);

    if replied_to_bot {
        game_handler::handle_move(state, &message, from, text).await?;
        return Ok(());
    }

    Ok(())
}
