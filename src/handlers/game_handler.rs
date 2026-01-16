use crate::models::{Message, User, UserRef};
use crate::{db, game, parsing, AppState};
use anyhow::{anyhow, Result};
use chess::Board;
use chess::Color;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle_start_game(
    state: Arc<AppState>,
    message: &Message,
    from: &User,
    text: &str,
) -> Result<()> {
    let chat_id = message.chat.id;

    let opponent_ref = match determine_opponent(message, text) {
        Ok(opponent) => opponent,
        Err(_) => {
            state
                .telegram
                .send_message(
                    chat_id,
                    message.message_id,
                    "Reply to a user's message or use /start @username [move].",
                )
                .await?;
            return Ok(());
        }
    };

    let white = db::upsert_user(&state.db, from).await?;
    let black = match opponent_ref {
        UserRef::Telegram(user) => db::upsert_user(&state.db, &user).await?,
        UserRef::Username(username) => db::upsert_user_by_username(&state.db, &username).await?,
    };

    if white.id == black.id {
        state
            .telegram
            .send_message(
                chat_id,
                message.message_id,
                "You cannot play against yourself.",
            )
            .await?;
        return Ok(());
    }

    if db::find_ongoing_game(&state.db, chat_id, white.id, black.id)
        .await?
        .is_some()
    {
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

    if let Some(candidate) = parsing::extract_move(text) {
        let before_fen = board.to_string();
        let mv = game::parse_move(&board, &candidate)?;
        board = board.make_move_new(mv);
        initial_move = Some(mv);
        let uci = game::uci_string(mv);
        let after_fen = board.to_string();
        info!(
            chat_id = chat_id,
            player_id = white.id,
            move_text = candidate.as_str(),
            uci = uci.as_str(),
            from = %mv.get_source(),
            to = %mv.get_dest(),
            fen_before = %before_fen,
            fen_after = %after_fen,
            "Initial move applied"
        );
    }

    let game_id = db::create_game(
        &state.db,
        chat_id,
        white.id,
        black.id,
        &board.to_string(),
        game::color_to_turn(board.side_to_move()),
    )
    .await?;

    if let Some(mv) = initial_move {
        let san = game::move_to_san(&Board::default(), mv);
        db::insert_move(
            &state.db,
            game_id,
            white.id,
            1,
            &game::uci_string(mv),
            Some(&san),
        )
        .await?;
    }

    let message_id = send_board_update(
        state.clone(),
        chat_id,
        None,
        "Game started",
        &board,
        &white,
        &black,
        None,
    )
    .await?;

    db::update_game_message(&state.db, game_id, message_id).await?;

    Ok(())
}

pub async fn handle_move(
    state: Arc<AppState>,
    message: &Message,
    from: &User,
    text: &str,
) -> Result<()> {
    let chat_id = message.chat.id;

    let reply_id = message
        .reply_to_message
        .as_ref()
        .map(|msg| msg.message_id)
        .ok_or_else(|| anyhow!("Move must be a reply to the bot's board message"))?;

    let Some(mut game) = db::find_game_by_message(&state.db, chat_id, reply_id).await? else {
        return Ok(());
    };

    if game.status != "ongoing" {
        return Ok(());
    }

    let player = db::upsert_user(&state.db, from).await?;
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
                "Please send a move like e4, e2e4, or Nf6.",
            )
            .await?;
        return Ok(());
    };

    let before_fen = board.to_string();
    let mv = match game::parse_move(&board, &candidate) {
        Ok(mv) => mv,
        Err(err) => {
            warn!(
                chat_id = chat_id,
                game_id = game.id,
                player_id = player.id,
                move_text = candidate.as_str(),
                fen = before_fen.as_str(),
                "Move parse failed: {err:?}"
            );
            state
                .telegram
                .send_message(chat_id, message.message_id, &format!("Invalid move: {err}"))
                .await?;
            return Ok(());
        }
    };
    let next_board = board.make_move_new(mv);
    let uci = game::uci_string(mv);
    let after_fen = next_board.to_string();
    let from_sq = mv.get_source();
    let to_sq = mv.get_dest();
    info!(
        chat_id = chat_id,
        game_id = game.id,
        player_id = player.id,
        move_text = candidate.as_str(),
        uci = uci.as_str(),
        from = %from_sq,
        to = %to_sq,
        fen_before = %before_fen,
        fen_after = %after_fen,
        "Move applied"
    );

    if game.draw_proposed_by.is_some() {
        db::clear_draw_proposal(&state.db, game.id).await?;
    }

    let san = game::move_to_san(&board, mv);
    let move_number = db::next_move_number(&state.db, game.id).await?;
    db::insert_move(
        &state.db,
        game.id,
        player.id,
        move_number,
        &game::uci_string(mv),
        Some(&san),
    )
    .await?;

    game.current_fen = next_board.to_string();
    game.turn = game::color_to_turn(next_board.side_to_move()).to_string();

    let white = db::get_user_by_id(&state.db, game.white_user_id).await?;
    let black = db::get_user_by_id(&state.db, game.black_user_id).await?;

    let status = next_board.status();
    let mut result_line = None;

    if status != chess::BoardStatus::Ongoing {
        let (status_text, result) = determine_game_result(&status, side_to_move, &white, &black);
        result_line = Some(status_text);
        game.status = "finished".to_string();
        game.result = Some(result.to_string());
        db::update_game_result(&state.db, game.id, &game.result, &game.status).await?;
        db::update_player_stats(&state.db, game.white_user_id, game.black_user_id, result).await?;
    }

    db::update_game_fen(&state.db, game.id, &game.current_fen, &game.turn).await?;

    let message_id = send_board_update(
        state.clone(),
        chat_id,
        Some(message.message_id),
        "Move played",
        &next_board,
        &white,
        &black,
        result_line,
    )
    .await?;

    db::update_game_message(&state.db, game.id, message_id).await?;

    Ok(())
}

fn determine_opponent(message: &Message, text: &str) -> Result<UserRef> {
    if let Some(reply) = &message.reply_to_message {
        if let Some(opponent) = reply.from.clone() {
            if !opponent.is_bot {
                return Ok(UserRef::Telegram(opponent));
            }
        }
    }

    if let Some(username) = parsing::extract_usernames(text).into_iter().next() {
        return Ok(UserRef::Username(username));
    }

    Err(anyhow!(
        "Reply to a user's message or use /start @username [move] to begin a game."
    ))
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
                if side_to_move == Color::White {
                    "0-1"
                } else {
                    "1-0"
                },
            )
        }
        chess::BoardStatus::Stalemate => ("Draw by stalemate.".to_string(), "1/2-1/2"),
        chess::BoardStatus::Ongoing => ("".to_string(), ""),
    }
}

pub async fn handle_resign(state: Arc<AppState>, message: &Message, from: &User) -> Result<()> {
    let chat_id = message.chat.id;

    let reply_id = message
        .reply_to_message
        .as_ref()
        .map(|msg| msg.message_id)
        .ok_or_else(|| anyhow!("Resign must be a reply to the bot's board message"))?;

    let Some(game) = db::find_game_by_message(&state.db, chat_id, reply_id).await? else {
        return Ok(());
    };

    if game.status != "ongoing" {
        return Ok(());
    }

    let player = db::upsert_user(&state.db, from).await?;
    if player.id != game.white_user_id && player.id != game.black_user_id {
        return Ok(());
    }

    let white = db::get_user_by_id(&state.db, game.white_user_id).await?;
    let black = db::get_user_by_id(&state.db, game.black_user_id).await?;

    let board = Board::from_str(&game.current_fen).map_err(|e| anyhow!("Invalid FEN: {}", e))?;

    let (winner, loser, result) = if player.id == game.white_user_id {
        (&black, &white, "0-1")
    } else {
        (&white, &black, "1-0")
    };

    db::update_game_result(&state.db, game.id, &Some(result.to_string()), "finished").await?;
    db::update_player_stats(&state.db, game.white_user_id, game.black_user_id, result).await?;

    let result_line = format!(
        "{} resigned. {} wins.",
        loser.mention_html(),
        winner.mention_html()
    );

    let message_id = send_board_update(
        state.clone(),
        chat_id,
        Some(message.message_id),
        "Game ended",
        &board,
        &white,
        &black,
        Some(result_line),
    )
    .await?;

    db::update_game_message(&state.db, game.id, message_id).await?;

    Ok(())
}

pub async fn handle_draw_proposal(
    state: Arc<AppState>,
    message: &Message,
    from: &User,
) -> Result<()> {
    let chat_id = message.chat.id;

    let reply_id = message
        .reply_to_message
        .as_ref()
        .map(|msg| msg.message_id)
        .ok_or_else(|| anyhow!("Draw proposal must be a reply to the bot's board message"))?;

    let Some(game) = db::find_game_by_message(&state.db, chat_id, reply_id).await? else {
        return Ok(());
    };

    if game.status != "ongoing" {
        return Ok(());
    }

    let player = db::upsert_user(&state.db, from).await?;
    if player.id != game.white_user_id && player.id != game.black_user_id {
        return Ok(());
    }

    db::propose_draw(&state.db, game.id, player.id).await?;

    let white = db::get_user_by_id(&state.db, game.white_user_id).await?;
    let black = db::get_user_by_id(&state.db, game.black_user_id).await?;
    let opponent = if player.id == game.white_user_id {
        &black
    } else {
        &white
    };

    state
        .telegram
        .send_message(
            chat_id,
            message.message_id,
            &format!(
                "{} proposed a draw. {} can accept with /accept or continue playing.",
                player.mention_html(),
                opponent.mention_html()
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_accept_draw(
    state: Arc<AppState>,
    message: &Message,
    from: &User,
) -> Result<()> {
    let chat_id = message.chat.id;

    let reply_id = message
        .reply_to_message
        .as_ref()
        .map(|msg| msg.message_id)
        .ok_or_else(|| anyhow!("Accept must be a reply to the bot's board message"))?;

    let Some(game) = db::find_game_by_message(&state.db, chat_id, reply_id).await? else {
        return Ok(());
    };

    if game.status != "ongoing" {
        return Ok(());
    }

    let player = db::upsert_user(&state.db, from).await?;
    if player.id != game.white_user_id && player.id != game.black_user_id {
        return Ok(());
    }

    let Some(proposer_id) = game.draw_proposed_by else {
        state
            .telegram
            .send_message(chat_id, message.message_id, "No draw proposal is pending.")
            .await?;
        return Ok(());
    };

    if proposer_id == player.id {
        state
            .telegram
            .send_message(
                chat_id,
                message.message_id,
                "You cannot accept your own draw proposal.",
            )
            .await?;
        return Ok(());
    }

    let board = Board::from_str(&game.current_fen).map_err(|e| anyhow!("Invalid FEN: {}", e))?;
    let white = db::get_user_by_id(&state.db, game.white_user_id).await?;
    let black = db::get_user_by_id(&state.db, game.black_user_id).await?;

    db::update_game_result(&state.db, game.id, &Some("1/2-1/2".to_string()), "finished").await?;
    db::update_player_stats(&state.db, game.white_user_id, game.black_user_id, "1/2-1/2").await?;

    let result_line = format!("Draw accepted by {}.", player.mention_html());

    let message_id = send_board_update(
        state.clone(),
        chat_id,
        Some(message.message_id),
        "Game ended",
        &board,
        &white,
        &black,
        Some(result_line),
    )
    .await?;

    db::update_game_message(&state.db, game.id, message_id).await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    let flip_board = board.side_to_move() == Color::Black;
    let image = game::render_board_png(board, flip_board)?;
    state
        .telegram
        .send_photo(chat_id, reply_to, &caption, image)
        .await
}
