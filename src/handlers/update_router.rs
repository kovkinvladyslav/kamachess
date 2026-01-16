use super::{game_handler, help_handler, history_handler};
use crate::models::Update;
use crate::AppState;
use anyhow::Result;
use std::sync::Arc;

fn strip_bot_suffix<'a>(text: &'a str, bot_username: &str) -> &'a str {
    let trimmed = text.trim();
    if let Some(at_pos) = trimmed.find('@') {
        let suffix = &trimmed[at_pos + 1..];
        if suffix.eq_ignore_ascii_case(bot_username) {
            return &trimmed[..at_pos];
        }
    }
    trimmed
}

fn command_matches(text: &str, command: &str, bot_username: &str) -> bool {
    let stripped = strip_bot_suffix(text, bot_username);
    stripped.eq_ignore_ascii_case(command)
}

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

    if text.starts_with("/help") {
        help_handler::handle_help(state, &message).await?;
        return Ok(());
    }

    if text.starts_with("/history") {
        history_handler::handle_history(state, &message, from, text).await?;
        return Ok(());
    }

    let replied_to_bot = message
        .reply_to_message
        .as_ref()
        .and_then(|msg| msg.from.as_ref())
        .map(|user| user.is_bot)
        .unwrap_or(false);

    if text.starts_with("/start") {
        game_handler::handle_start_game(state, &message, from, text).await?;
        return Ok(());
    }

    if replied_to_bot {
        if command_matches(text, "/resign", &state.bot_username) {
            game_handler::handle_resign(state, &message, from).await?;
            return Ok(());
        }

        if command_matches(text, "/draw", &state.bot_username) {
            game_handler::handle_draw_proposal(state, &message, from).await?;
            return Ok(());
        }

        if command_matches(text, "/accept", &state.bot_username)
            || command_matches(text, "/acceptdraw", &state.bot_username)
        {
            game_handler::handle_accept_draw(state, &message, from).await?;
            return Ok(());
        }

        if text.starts_with("/move") {
            let move_text = strip_bot_suffix(text, &state.bot_username);
            let move_part = move_text.strip_prefix("/move").unwrap_or("").trim();
            if !move_part.is_empty() {
                game_handler::handle_move(state, &message, from, move_part).await?;
            } else {
                game_handler::handle_move(state, &message, from, text).await?;
            }
            return Ok(());
        }

        game_handler::handle_move(state, &message, from, text).await?;
        return Ok(());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_bot_suffix() {
        assert_eq!(strip_bot_suffix("/resign@testbot", "testbot"), "/resign");
        assert_eq!(strip_bot_suffix("/resign@TESTBOT", "testbot"), "/resign");
        assert_eq!(strip_bot_suffix("/resign", "testbot"), "/resign");
        assert_eq!(strip_bot_suffix("/resign@otherbot", "testbot"), "/resign@otherbot");
    }

    #[test]
    fn test_command_matches() {
        assert!(command_matches("/resign", "/resign", "testbot"));
        assert!(command_matches("/resign@testbot", "/resign", "testbot"));
        assert!(command_matches("/RESIGN@TestBot", "/resign", "testbot"));
        assert!(command_matches("  /resign@testbot  ", "/resign", "testbot"));
        assert!(!command_matches("/resign@otherbot", "/resign", "testbot"));
        assert!(!command_matches("/draw", "/resign", "testbot"));
    }

    #[test]
    fn test_command_matches_accept_variants() {
        assert!(command_matches("/accept", "/accept", "mybot"));
        assert!(command_matches("/accept@mybot", "/accept", "mybot"));
        assert!(command_matches("/acceptdraw", "/acceptdraw", "mybot"));
        assert!(command_matches("/acceptdraw@mybot", "/acceptdraw", "mybot"));
    }

    #[test]
    fn test_command_matches_draw() {
        assert!(command_matches("/draw", "/draw", "chessbot"));
        assert!(command_matches("/draw@chessbot", "/draw", "chessbot"));
        assert!(command_matches("/DRAW@ChessBot", "/draw", "chessbot"));
    }
}
