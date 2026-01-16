use crate::models::{DbUser, GameRow, HistoryRow, User};
use anyhow::Result;
use chrono::Utc;
use sqlx::{Any, Pool, Row};
use std::collections::HashMap;

pub async fn run_migrations(pool: &Pool<Any>, database_url: &str) -> Result<()> {
    if database_url.starts_with("postgres") {
        sqlx::raw_sql(include_str!("../../migrations/postgres/001_init.sql"))
            .execute(pool)
            .await?;
        let _ = sqlx::raw_sql(include_str!(
            "../../migrations/postgres/002_add_draw_proposed_by.sql"
        ))
        .execute(pool)
        .await;
    } else {
        sqlx::raw_sql(include_str!("../../migrations/sqlite/001_init.sql"))
            .execute(pool)
            .await?;
        let _ = sqlx::raw_sql(include_str!(
            "../../migrations/sqlite/002_add_draw_proposed_by.sql"
        ))
        .execute(pool)
        .await;
    }
    Ok(())
}

pub async fn upsert_user(pool: &Pool<Any>, user: &User) -> Result<DbUser> {
    sqlx::query(
        "INSERT INTO users (telegram_id, username, first_name, last_name)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT(telegram_id) DO UPDATE SET
            username = excluded.username,
            first_name = excluded.first_name,
            last_name = excluded.last_name",
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .execute(pool)
    .await?;

    if let Some(username) = user.username.as_deref() {
        sqlx::query(
            "UPDATE users
             SET telegram_id = $1, first_name = $2, last_name = $3
             WHERE username = $4 AND (telegram_id IS NULL OR telegram_id = $1)",
        )
        .bind(user.id)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(username)
        .execute(pool)
        .await?;
    }

    get_user_by_telegram_id(pool, user.id).await
}

pub async fn upsert_user_by_username(pool: &Pool<Any>, username: &str) -> Result<DbUser> {
    sqlx::query(
        "INSERT INTO users (username) VALUES ($1)
         ON CONFLICT(username) DO NOTHING",
    )
    .bind(username)
    .execute(pool)
    .await?;
    get_user_by_username(pool, username).await
}

pub async fn get_user_by_telegram_id(pool: &Pool<Any>, telegram_id: i64) -> Result<DbUser> {
    let row = sqlx::query(
        "SELECT id, telegram_id, username, first_name, last_name, wins, losses, draws
         FROM users WHERE telegram_id = $1",
    )
    .bind(telegram_id)
    .fetch_one(pool)
    .await?;

    Ok(DbUser {
        id: row.get("id"),
        telegram_id: row.get("telegram_id"),
        username: row.get("username"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        wins: row.get("wins"),
        losses: row.get("losses"),
        draws: row.get("draws"),
    })
}

pub async fn get_user_by_username(pool: &Pool<Any>, username: &str) -> Result<DbUser> {
    let row = sqlx::query(
        "SELECT id, telegram_id, username, first_name, last_name, wins, losses, draws
         FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_one(pool)
    .await?;

    Ok(DbUser {
        id: row.get("id"),
        telegram_id: row.get("telegram_id"),
        username: row.get("username"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        wins: row.get("wins"),
        losses: row.get("losses"),
        draws: row.get("draws"),
    })
}

pub async fn get_user_by_id(pool: &Pool<Any>, id: i64) -> Result<DbUser> {
    let row = sqlx::query(
        "SELECT id, telegram_id, username, first_name, last_name, wins, losses, draws
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(DbUser {
        id: row.get("id"),
        telegram_id: row.get("telegram_id"),
        username: row.get("username"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        wins: row.get("wins"),
        losses: row.get("losses"),
        draws: row.get("draws"),
    })
}

pub async fn create_game(
    pool: &Pool<Any>,
    chat_id: i64,
    white_user_id: i64,
    black_user_id: i64,
    fen: &str,
    turn: &str,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let row = sqlx::query(
        "INSERT INTO games (chat_id, white_user_id, black_user_id, current_fen, turn, started_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
    )
    .bind(chat_id)
    .bind(white_user_id)
    .bind(black_user_id)
    .bind(fen)
    .bind(turn)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

pub async fn update_game_message(pool: &Pool<Any>, game_id: i64, message_id: i64) -> Result<()> {
    sqlx::query("UPDATE games SET last_message_id = $1 WHERE id = $2")
        .bind(message_id)
        .bind(game_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_game_fen(pool: &Pool<Any>, game_id: i64, fen: &str, turn: &str) -> Result<()> {
    sqlx::query("UPDATE games SET current_fen = $1, turn = $2 WHERE id = $3")
        .bind(fen)
        .bind(turn)
        .bind(game_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_game_result(
    pool: &Pool<Any>,
    game_id: i64,
    result: &Option<String>,
    status: &str,
) -> Result<()> {
    let ended = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE games SET result = $1, status = $2, ended_at = $3, draw_proposed_by = NULL WHERE id = $4",
    )
    .bind(result)
    .bind(status)
    .bind(ended)
    .bind(game_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn propose_draw(pool: &Pool<Any>, game_id: i64, player_id: i64) -> Result<()> {
    sqlx::query("UPDATE games SET draw_proposed_by = $1 WHERE id = $2")
        .bind(player_id)
        .bind(game_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn clear_draw_proposal(pool: &Pool<Any>, game_id: i64) -> Result<()> {
    sqlx::query("UPDATE games SET draw_proposed_by = NULL WHERE id = $1")
        .bind(game_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_player_stats(
    pool: &Pool<Any>,
    white_id: i64,
    black_id: i64,
    result: &str,
) -> Result<()> {
    match result {
        "1-0" => {
            sqlx::query("UPDATE users SET wins = wins + 1 WHERE id = $1")
                .bind(white_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE users SET losses = losses + 1 WHERE id = $1")
                .bind(black_id)
                .execute(pool)
                .await?;
        }
        "0-1" => {
            sqlx::query("UPDATE users SET wins = wins + 1 WHERE id = $1")
                .bind(black_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE users SET losses = losses + 1 WHERE id = $1")
                .bind(white_id)
                .execute(pool)
                .await?;
        }
        "1/2-1/2" => {
            sqlx::query("UPDATE users SET draws = draws + 1 WHERE id = $1")
                .bind(white_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE users SET draws = draws + 1 WHERE id = $1")
                .bind(black_id)
                .execute(pool)
                .await?;
        }
        _ => {}
    }
    Ok(())
}

pub async fn insert_move(
    pool: &Pool<Any>,
    game_id: i64,
    player_id: i64,
    move_number: i64,
    uci: &str,
    san: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO moves (game_id, move_number, uci, san, played_by, played_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(game_id)
    .bind(move_number)
    .bind(uci)
    .bind(san)
    .bind(player_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn next_move_number(pool: &Pool<Any>, game_id: i64) -> Result<i64> {
    let row = sqlx::query("SELECT COALESCE(MAX(move_number), 0) + 1 as next FROM moves WHERE game_id = $1")
        .bind(game_id)
        .fetch_one(pool)
        .await?;
    Ok(row.get("next"))
}

async fn get_games_san_moves(pool: &Pool<Any>, game_ids: &[i64]) -> HashMap<i64, Vec<String>> {
    if game_ids.is_empty() {
        return HashMap::new();
    }

    let placeholders: String = game_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT game_id, san, uci FROM moves WHERE game_id IN ({}) ORDER BY game_id, move_number ASC",
        placeholders
    );

    let mut query = sqlx::query(&sql);
    for id in game_ids {
        query = query.bind(id);
    }

    let rows = match query.fetch_all(pool).await {
        Ok(r) => r,
        Err(_) => return HashMap::new(),
    };

    let mut result: HashMap<i64, Vec<String>> = HashMap::new();
    for row in rows {
        let game_id: i64 = row.get("game_id");
        let san: Option<String> = row.get("san");
        let uci: String = row.get("uci");
        result.entry(game_id).or_default().push(san.unwrap_or(uci));
    }
    result
}

fn build_lichess_url_from_moves(moves: &[String]) -> String {
    if moves.is_empty() {
        return "https://lichess.org/analysis".to_string();
    }

    let mut pgn = String::new();
    for (i, mv) in moves.iter().enumerate() {
        if i % 2 == 0 {
            if !pgn.is_empty() {
                pgn.push(' ');
            }
            pgn.push_str(&format!("{}.", i / 2 + 1));
        }
        pgn.push(' ');
        pgn.push_str(mv);
    }

    let encoded: String = pgn
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '.' => ".".to_string(),
            '#' => "%23".to_string(),
            '+' => "%2B".to_string(),
            '=' => "%3D".to_string(),
            _ if c.is_ascii_alphanumeric() || c == '-' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect();

    format!("https://lichess.org/analysis/pgn/{}", encoded)
}

pub async fn find_ongoing_game(
    pool: &Pool<Any>,
    chat_id: i64,
    white_id: i64,
    black_id: i64,
) -> Result<Option<GameRow>> {
    let row = sqlx::query(
        "SELECT id, chat_id, white_user_id, black_user_id, current_fen, turn, status, result, last_message_id, draw_proposed_by
         FROM games
         WHERE chat_id = $1 AND status = 'ongoing'
           AND ((white_user_id = $2 AND black_user_id = $3)
             OR (white_user_id = $3 AND black_user_id = $2))
         LIMIT 1",
    )
    .bind(chat_id)
    .bind(white_id)
    .bind(black_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| GameRow {
        id: r.get("id"),
        chat_id: r.get("chat_id"),
        white_user_id: r.get("white_user_id"),
        black_user_id: r.get("black_user_id"),
        current_fen: r.get("current_fen"),
        turn: r.get("turn"),
        status: r.get("status"),
        result: r.get("result"),
        last_message_id: r.get("last_message_id"),
        draw_proposed_by: r.get("draw_proposed_by"),
    }))
}

pub async fn find_game_by_message(
    pool: &Pool<Any>,
    chat_id: i64,
    message_id: i64,
) -> Result<Option<GameRow>> {
    let row = sqlx::query(
        "SELECT id, chat_id, white_user_id, black_user_id, current_fen, turn, status, result, last_message_id, draw_proposed_by
         FROM games
         WHERE chat_id = $1 AND last_message_id = $2
         LIMIT 1",
    )
    .bind(chat_id)
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| GameRow {
        id: r.get("id"),
        chat_id: r.get("chat_id"),
        white_user_id: r.get("white_user_id"),
        black_user_id: r.get("black_user_id"),
        current_fen: r.get("current_fen"),
        turn: r.get("turn"),
        status: r.get("status"),
        result: r.get("result"),
        last_message_id: r.get("last_message_id"),
        draw_proposed_by: r.get("draw_proposed_by"),
    }))
}

pub async fn format_user_history(
    pool: &Pool<Any>,
    user: &DbUser,
    chat_id: i64,
    page: u32,
) -> Result<String> {
    let stats_row = sqlx::query(
        "SELECT
            SUM(CASE
                WHEN result = '1-0' AND white_user_id = $1 THEN 1
                WHEN result = '0-1' AND black_user_id = $1 THEN 1
                ELSE 0
            END) AS wins,
            SUM(CASE
                WHEN result = '0-1' AND white_user_id = $1 THEN 1
                WHEN result = '1-0' AND black_user_id = $1 THEN 1
                ELSE 0
            END) AS losses,
            SUM(CASE
                WHEN result = '1/2-1/2' THEN 1
                ELSE 0
            END) AS draws
         FROM games
         WHERE chat_id = $2
           AND (white_user_id = $1 OR black_user_id = $1)",
    )
    .bind(user.id)
    .bind(chat_id)
    .fetch_one(pool)
    .await?;

    let wins: i64 = stats_row.try_get::<i64, _>("wins").unwrap_or(0);
    let losses: i64 = stats_row.try_get::<i64, _>("losses").unwrap_or(0);
    let draws: i64 = stats_row.try_get::<i64, _>("draws").unwrap_or(0);

    let total = wins + losses + draws;
    let win_pct = if total == 0 {
        0.0
    } else {
        (wins as f64) * 100.0 / (total as f64)
    };

    let limit: i64 = 10;
    let offset = ((page - 1) as i64) * limit;
    let history_rows: Vec<HistoryRow> = sqlx::query_as(
        "WITH numbered AS (
            SELECT g.id, g.started_at, g.result, u1.username AS white_username, u2.username AS black_username,
                   ROW_NUMBER() OVER (ORDER BY g.started_at ASC) AS local_num
            FROM games g
            JOIN users u1 ON g.white_user_id = u1.id
            JOIN users u2 ON g.black_user_id = u2.id
            WHERE g.chat_id = $1
              AND (g.white_user_id = $2 OR g.black_user_id = $2)
        )
        SELECT id, local_num, started_at, result, white_username, black_username
        FROM numbered
        ORDER BY started_at DESC
        LIMIT $3 OFFSET $4",
    )
    .bind(chat_id)
    .bind(user.id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let game_ids: Vec<i64> = history_rows.iter().map(|r| r.id).collect();
    let all_moves = get_games_san_moves(pool, &game_ids).await;

    let mut lines = Vec::new();
    for row in &history_rows {
        let result = row.result.clone().unwrap_or_else(|| "ongoing".to_string());
        let white_name = crate::utils::format_username(&row.white_username);
        let black_name = crate::utils::format_username(&row.black_username);
        let moves = all_moves.get(&row.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let lichess_url = build_lichess_url_from_moves(moves);
        lines.push(format!(
            "#{}: {} vs {} ({}) - <a href=\"{}\">analysis</a>",
            row.local_num, white_name, black_name, result, lichess_url
        ));
    }

    let mut output = format!(
        "History for {} in this chat.\nWins: {}, Losses: {}, Draws: {}, Win%: {:.1}\n\n",
        crate::utils::escape_html(&user.display_name()),
        wins,
        losses,
        draws,
        win_pct
    );
    output.push_str(&lines.join("\n"));
    if lines.is_empty() {
        output.push_str("No games yet.");
    }
    output.push_str("\nUse /history &lt;page&gt; for more.");
    Ok(output)
}

pub async fn format_head_to_head(
    pool: &Pool<Any>,
    user_a: &DbUser,
    user_b: &DbUser,
    chat_id: i64,
    page: u32,
) -> Result<String> {
    let count_row = sqlx::query(
        "SELECT COUNT(*) as total FROM games
         WHERE chat_id = $3
           AND ((white_user_id = $1 AND black_user_id = $2)
             OR (white_user_id = $2 AND black_user_id = $1))",
    )
    .bind(user_a.id)
    .bind(user_b.id)
    .bind(chat_id)
    .fetch_one(pool)
    .await?;
    let total: i64 = count_row.get("total");

    let limit: i64 = 10;
    let offset = ((page - 1) as i64) * limit;
    let history_rows: Vec<HistoryRow> = sqlx::query_as(
        "WITH numbered AS (
            SELECT g.id, g.started_at, g.result, u1.username AS white_username, u2.username AS black_username,
                   ROW_NUMBER() OVER (ORDER BY g.started_at ASC) AS local_num
            FROM games g
            JOIN users u1 ON g.white_user_id = u1.id
            JOIN users u2 ON g.black_user_id = u2.id
            WHERE g.chat_id = $3
              AND ((g.white_user_id = $1 AND g.black_user_id = $2)
                OR (g.white_user_id = $2 AND g.black_user_id = $1))
        )
        SELECT id, local_num, started_at, result, white_username, black_username
        FROM numbered
        ORDER BY started_at DESC
        LIMIT $4 OFFSET $5",
    )
    .bind(user_a.id)
    .bind(user_b.id)
    .bind(chat_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let game_ids: Vec<i64> = history_rows.iter().map(|r| r.id).collect();
    let all_moves = get_games_san_moves(pool, &game_ids).await;

    let mut lines = Vec::new();
    for row in &history_rows {
        let result = row.result.clone().unwrap_or_else(|| "ongoing".to_string());
        let white_name = crate::utils::format_username(&row.white_username);
        let black_name = crate::utils::format_username(&row.black_username);
        let moves = all_moves.get(&row.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let lichess_url = build_lichess_url_from_moves(moves);
        lines.push(format!(
            "#{}: {} vs {} ({}) - <a href=\"{}\">analysis</a>",
            row.local_num, white_name, black_name, result, lichess_url
        ));
    }

    let mut output = format!(
        "Head-to-head {} vs {} in this chat. Total games: {}\n\n",
        crate::utils::escape_html(&user_a.display_name()),
        crate::utils::escape_html(&user_b.display_name()),
        total
    );
    output.push_str(&lines.join("\n"));
    if lines.is_empty() {
        output.push_str("No games yet.");
    }
    output.push_str("\nUse /history &lt;page&gt; for more.");
    Ok(output)
}
