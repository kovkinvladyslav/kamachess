use anyhow::Result;
use std::sync::Arc;
use crate::models::Message;
use crate::AppState;

pub async fn handle_help(
    state: Arc<AppState>,
    message: &Message,
) -> Result<()> {
    let chat_id = message.chat.id;
    
    let help_text = r#"<b>Chess Bot Commands:</b>

<b>/start [@user] [move]</b>
Reply to a user's message or mention a user to start a game.
Examples: /start e4, /start @user Nf3

<b>/history [@user] [@user2] [page]</b>
View game history or head-to-head stats.
Examples:
• /history - Your stats
• /history @username - User's stats
• /history @user1 @user2 - Head-to-head
• /history 2 - Page 2

<b>Making Moves:</b>
Reply to the bot's board message with your move.
Supports: e4, e2e4, Nf6, O-O, etc.

<b>/resign</b>
Reply to the bot's board message to resign.

<b>/draw</b>
Reply to the bot's board message to propose a draw.

<b>/accept</b>
Reply to the bot's board message to accept a draw proposal.

Use /help to show this message."#;

    state
        .telegram
        .send_message(chat_id, message.message_id, help_text)
        .await?;
    
    Ok(())
}
