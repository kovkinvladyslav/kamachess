use anyhow::{anyhow, Result};
use chess::Board;
use chess::Color;
use std::str::FromStr;
use std::sync::Arc;
use crate::models::{Message, User, UserRef};
use crate::{AppState, db, game, parsing};

pub async fn handle_start_game(
    state: Arc<AppState>,
    message: &Message,
    from: &User,
    text: &str,
) -> Result<()> {
    let conn = state.db.get()?;
    let chat_id = message.chat.id;

    let opponent_ref = determine_opponent_from_reply(message)?;

    let white = db::upsert_user(&conn, from)?;
    let black = match opponent_ref {
        UserRef::Telegram(user) => db::upsert_user(&conn, &user)?,
        UserRef::Username(_) => unreachable!(),
    };

    if db::find_ongoing_game(&conn, chat_id, white.id, black.id)?.is_some() {
        state
            .telegram
            .send_message(
                chat_id,
                message.message_id,
                "There is already an ongoing game between these players in this chat.",
            )
            .await?;
        return Ok(());
    }

    let mut board = Board::default();
    let mut initial_move: Option<chess::ChessMove> = None;
    let mut move_text: Option<String> = None;
    
    if let Some(candidate) = parsing::extract_move(text) {
        let mv = game::parse_move(&board, &candidate)?;
        board = board.make_move_new(mv);
        initial_move = Some(mv);
        move_text = Some(candidate);
    }

    let game_id = db::create_game(
        &conn,
        chat_id,
        white.id,
        black.id,
        &board.to_string(),
        game::color_to_turn(board.side_to_move()),
    )?;

    if let Some(mv) = initial_move {
        db::insert_move(
            &conn,
            game_id,
            white.id,
            1,
            &game::uci_string(mv),
            move_text.as_deref(),
        )?;
    }

    let message_id = send_board_update(state.clone(), chat_id, None, "Game started", &board, &white, &black, None).await?;
    
    db::update_game_message(&conn, game_id, message_id)?;

    Ok(())
}

pub async fn handle_move(
    state: Arc<AppState>,
    message: &Message,
    from: &User,
    text: &str,
) -> Result<()> {
    let conn = state.db.get()?;
    let chat_id = message.chat.id;
    
    let reply_id = message
        .reply_to_message
        .as_ref()
        .map(|msg| msg.message_id)
        .ok_or_else(|| anyhow!("Move must be a reply to the bot's board message"))?;

    let Some(mut game) = db::find_game_by_message(&conn, chat_id, reply_id)? else {
        return Ok(());
    };

    if game.status != "ongoing" {
        return Ok(());
    }

    let player = db::upsert_user(&conn, from)?;
    if player.id != game.white_user_id && player.id != game.black_user_id {
        state
            .telegram
            .send_message(
                chat_id,
                message.message_id,
                "This game belongs to other players.",
            )
            .await?;
        return Ok(());
    }

    let board = Board::from_str(&game.current_fen).map_err(|e| anyhow!("Invalid FEN: {}", e))?;
    let side_to_move = board.side_to_move();
    let expected_id = if side_to_move == Color::White {
        game.white_user_id
    } else {
        game.black_user_id
    };
    
    if player.id != expected_id {
        state
            .telegram
            .send_message(chat_id, message.message_id, "It is not your turn.")
            .await?;
        return Ok(());
    }

    let Some(candidate) = parsing::extract_move(text) else {
        state
            .telegram
            .send_message(
                chat_id,
                message.message_id,
                "Please send a move like e4 or e2e4.",
            )
            .await?;
        return Ok(());
    };

    let mv = game::parse_move(&board, &candidate)?;
    let next_board = board.make_move_new(mv);
    
    let move_number = db::next_move_number(&conn, game.id)?;
    db::insert_move(
        &conn,
        game.id,
        player.id,
        move_number,
        &game::uci_string(mv),
        Some(&candidate),
    )?;

    game.current_fen = next_board.to_string();
    game.turn = game::color_to_turn(next_board.side_to_move()).to_string();

    let white = db::get_user_by_id(&conn, game.white_user_id)?;
    let black = db::get_user_by_id(&conn, game.black_user_id)?;

    let status = next_board.status();
    let mut result_line = None;
    
    if status != chess::BoardStatus::Ongoing {
        let (status_text, result) = determine_game_result(&status, side_to_move, &white, &black);
        result_line = Some(status_text);
        game.status = "finished".to_string();
        game.result = Some(result.to_string());
        db::update_game_result(&conn, game.id, &game.result, &game.status)?;
        db::update_player_stats(&conn, game.white_user_id, game.black_user_id, result)?;
    }

    db::update_game_fen(&conn, game.id, &game.current_fen, &game.turn)?;

    let message_id = send_board_update(
        state.clone(),
        chat_id,
        Some(message.message_id),
        "Move played",
        &next_board,
        &white,
        &black,
        result_line,
    ).await?;

    db::update_game_message(&conn, game.id, message_id)?;

    Ok(())
}

fn determine_opponent_from_reply(message: &Message) -> Result<UserRef> {
    if let Some(reply) = &message.reply_to_message {
        if let Some(opponent) = reply.from.clone() {
            return Ok(UserRef::Telegram(opponent));
        }
    }
    Err(anyhow!("Reply to a user's message with /start <move> to begin a game."))
}

fn determine_game_result(
    status: &chess::BoardStatus,
    side_to_move: Color,
    white: &crate::models::DbUser,
    black: &crate::models::DbUser,
) -> (String, &'static str) {
    match status {
        chess::BoardStatus::Checkmate => {
            let winner = if side_to_move == Color::White {
                black.mention_html()
            } else {
                white.mention_html()
            };
            (
                format!("Checkmate. {} wins.", winner),
                if side_to_move == Color::White { "0-1" } else { "1-0" },
            )
        }
        chess::BoardStatus::Stalemate => ("Draw by stalemate.".to_string(), "1/2-1/2"),
        chess::BoardStatus::Ongoing => ("".to_string(), ""),
    }
}

async fn send_board_update(
    state: Arc<AppState>,
    chat_id: i64,
    reply_to: Option<i64>,
    header: &str,
    board: &Board,
    white: &crate::models::DbUser,
    black: &crate::models::DbUser,
    result_line: Option<String>,
) -> Result<i64> {
    let caption = game::build_caption(
        header,
        board,
        white,
        black,
        board.side_to_move(),
        result_line,
    );
    let image = game::render_board_png(board)?;
    state
        .telegram
        .send_photo(chat_id, reply_to, &caption, image)
        .await
}
