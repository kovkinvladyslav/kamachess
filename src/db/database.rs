use crate::models::{DbUser, GameRow, HistoryRow, User};
use anyhow::{anyhow, Result};
use chrono::Utc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};

const INIT_SQL: &str = include_str!("../../migrations/001_init.sql");
const MIGRATION_002: &str = include_str!("../../migrations/002_add_draw_proposed_by.sql");

pub fn init_db(pool: &Pool<SqliteConnectionManager>) -> Result<()> {
    let conn = pool.get()?;
    conn.execute_batch(INIT_SQL)?;
    let _ = conn.execute_batch(MIGRATION_002);
    Ok(())
}

pub fn upsert_user(conn: &rusqlite::Connection, user: &User) -> Result<DbUser> {
    conn.execute(
        "INSERT INTO users (telegram_id, username, first_name, last_name)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(telegram_id) DO UPDATE SET
            username = excluded.username,
            first_name = excluded.first_name,
            last_name = excluded.last_name",
        params![user.id, user.username, user.first_name, user.last_name],
    )?;

    if let Some(username) = user.username.as_deref() {
        conn.execute(
            "UPDATE users
             SET telegram_id = ?1, first_name = ?2, last_name = ?3
             WHERE username = ?4 AND (telegram_id IS NULL OR telegram_id = ?1)",
            params![user.id, user.first_name, user.last_name, username],
        )?;
    }

    get_user_by_telegram_id(conn, user.id)
}

pub fn upsert_user_by_username(conn: &rusqlite::Connection, username: &str) -> Result<DbUser> {
    conn.execute(
        "INSERT INTO users (username) VALUES (?1)
         ON CONFLICT(username) DO NOTHING",
        params![username],
    )?;
    get_user_by_username(conn, username)
}

pub fn get_user_by_telegram_id(conn: &rusqlite::Connection, telegram_id: i64) -> Result<DbUser> {
    conn.query_row(
        "SELECT id, telegram_id, username, first_name, last_name, wins, losses, draws
         FROM users WHERE telegram_id = ?1",
        params![telegram_id],
        DbUser::from_row,
    )
    .map_err(|err| anyhow!(err))
}

pub fn get_user_by_username(conn: &rusqlite::Connection, username: &str) -> Result<DbUser> {
    conn.query_row(
        "SELECT id, telegram_id, username, first_name, last_name, wins, losses, draws
         FROM users WHERE username = ?1",
        params![username],
        DbUser::from_row,
    )
    .map_err(|err| anyhow!(err))
}

pub fn get_user_by_id(conn: &rusqlite::Connection, id: i64) -> Result<DbUser> {
    conn.query_row(
        "SELECT id, telegram_id, username, first_name, last_name, wins, losses, draws
         FROM users WHERE id = ?1",
        params![id],
        DbUser::from_row,
    )
    .map_err(|err| anyhow!(err))
}

pub fn create_game(
    conn: &rusqlite::Connection,
    chat_id: i64,
    white_user_id: i64,
    black_user_id: i64,
    fen: &str,
    turn: &str,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO games (chat_id, white_user_id, black_user_id, current_fen, turn, started_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![chat_id, white_user_id, black_user_id, fen, turn, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_game_message(
    conn: &rusqlite::Connection,
    game_id: i64,
    message_id: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE games SET last_message_id = ?1 WHERE id = ?2",
        params![message_id, game_id],
    )?;
    Ok(())
}

pub fn update_game_fen(
    conn: &rusqlite::Connection,
    game_id: i64,
    fen: &str,
    turn: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE games SET current_fen = ?1, turn = ?2 WHERE id = ?3",
        params![fen, turn, game_id],
    )?;
    Ok(())
}

pub fn update_game_result(
    conn: &rusqlite::Connection,
    game_id: i64,
    result: &Option<String>,
    status: &str,
) -> Result<()> {
    let ended = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE games SET result = ?1, status = ?2, ended_at = ?3, draw_proposed_by = NULL WHERE id = ?4",
        params![result, status, ended, game_id],
    )?;
    Ok(())
}

pub fn propose_draw(conn: &rusqlite::Connection, game_id: i64, player_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE games SET draw_proposed_by = ?1 WHERE id = ?2",
        params![player_id, game_id],
    )?;
    Ok(())
}

pub fn clear_draw_proposal(conn: &rusqlite::Connection, game_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE games SET draw_proposed_by = NULL WHERE id = ?1",
        params![game_id],
    )?;
    Ok(())
}

pub fn update_player_stats(
    conn: &rusqlite::Connection,
    white_id: i64,
    black_id: i64,
    result: &str,
) -> Result<()> {
    match result {
        "1-0" => {
            conn.execute(
                "UPDATE users SET wins = wins + 1 WHERE id = ?1",
                params![white_id],
            )?;
            conn.execute(
                "UPDATE users SET losses = losses + 1 WHERE id = ?1",
                params![black_id],
            )?;
        }
        "0-1" => {
            conn.execute(
                "UPDATE users SET wins = wins + 1 WHERE id = ?1",
                params![black_id],
            )?;
            conn.execute(
                "UPDATE users SET losses = losses + 1 WHERE id = ?1",
                params![white_id],
            )?;
        }
        "1/2-1/2" => {
            conn.execute(
                "UPDATE users SET draws = draws + 1 WHERE id = ?1",
                params![white_id],
            )?;
            conn.execute(
                "UPDATE users SET draws = draws + 1 WHERE id = ?1",
                params![black_id],
            )?;
        }
        _ => {}
    }
    Ok(())
}

pub fn insert_move(
    conn: &rusqlite::Connection,
    game_id: i64,
    player_id: i64,
    move_number: i64,
    uci: &str,
    san: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO moves (game_id, move_number, uci, san, played_by, played_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![game_id, move_number, uci, san, player_id, now],
    )?;
    Ok(())
}

pub fn next_move_number(conn: &rusqlite::Connection, game_id: i64) -> Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM moves WHERE game_id = ?1",
        params![game_id],
        |row| {
            let count: i64 = row.get(0)?;
            Ok(count + 1)
        },
    )
    .map_err(|err| anyhow!(err))
}

fn get_game_san_moves(conn: &rusqlite::Connection, game_id: i64) -> Vec<String> {
    let mut stmt = match conn
        .prepare("SELECT san, uci FROM moves WHERE game_id = ?1 ORDER BY move_number ASC")
    {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map(params![game_id], |row| {
        let san: Option<String> = row.get(0)?;
        let uci: String = row.get(1)?;
        Ok(san.unwrap_or(uci))
    })
    .ok()
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

fn build_lichess_url(conn: &rusqlite::Connection, game_id: i64) -> String {
    let moves = get_game_san_moves(conn, game_id);
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

pub fn find_ongoing_game(
    conn: &rusqlite::Connection,
    chat_id: i64,
    white_id: i64,
    black_id: i64,
) -> Result<Option<GameRow>> {
    conn.query_row(
        "SELECT id, chat_id, white_user_id, black_user_id, current_fen, turn, status, result, last_message_id, draw_proposed_by
         FROM games
         WHERE chat_id = ?1 AND status = 'ongoing'
           AND ((white_user_id = ?2 AND black_user_id = ?3)
             OR (white_user_id = ?3 AND black_user_id = ?2))
         LIMIT 1",
        params![chat_id, white_id, black_id],
        GameRow::from_row,
    )
    .optional()
    .map_err(|err| anyhow!(err))
}

pub fn find_game_by_message(
    conn: &rusqlite::Connection,
    chat_id: i64,
    message_id: i64,
) -> Result<Option<GameRow>> {
    conn.query_row(
        "SELECT id, chat_id, white_user_id, black_user_id, current_fen, turn, status, result, last_message_id, draw_proposed_by
         FROM games
         WHERE chat_id = ?1 AND last_message_id = ?2
         LIMIT 1",
        params![chat_id, message_id],
        GameRow::from_row,
    )
    .optional()
    .map_err(|err| anyhow!(err))
}

pub fn format_user_history(
    conn: &rusqlite::Connection,
    user: &DbUser,
    chat_id: i64,
    page: u32,
) -> Result<String> {
    let (wins, losses, draws) = conn.query_row(
        "SELECT
            SUM(CASE
                WHEN result = '1-0' AND white_user_id = ?1 THEN 1
                WHEN result = '0-1' AND black_user_id = ?1 THEN 1
                ELSE 0
            END) AS wins,
            SUM(CASE
                WHEN result = '0-1' AND white_user_id = ?1 THEN 1
                WHEN result = '1-0' AND black_user_id = ?1 THEN 1
                ELSE 0
            END) AS losses,
            SUM(CASE
                WHEN result = '1/2-1/2' THEN 1
                ELSE 0
            END) AS draws
         FROM games
         WHERE chat_id = ?2
           AND (white_user_id = ?1 OR black_user_id = ?1)",
        params![user.id, chat_id],
        |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<i64>>(2)?.unwrap_or(0),
            ))
        },
    )?;

    let total = wins + losses + draws;
    let win_pct = if total == 0 {
        0.0
    } else {
        (wins as f64) * 100.0 / (total as f64)
    };

    let limit = 10;
    let offset = ((page - 1) * limit) as i64;
    let mut stmt = conn.prepare(
        "WITH numbered AS (
            SELECT g.id, g.started_at, g.result, u1.username AS white_username, u2.username AS black_username,
                   ROW_NUMBER() OVER (ORDER BY g.started_at ASC) AS local_num
            FROM games g
            JOIN users u1 ON g.white_user_id = u1.id
            JOIN users u2 ON g.black_user_id = u2.id
            WHERE g.chat_id = ?1
              AND (g.white_user_id = ?2 OR g.black_user_id = ?2)
        )
        SELECT id, local_num, started_at, result, white_username, black_username
        FROM numbered
        ORDER BY started_at DESC
        LIMIT ?3 OFFSET ?4",
    )?;
    let rows = stmt.query_map(params![chat_id, user.id, limit, offset], |row| {
        Ok(HistoryRow {
            id: row.get(0)?,
            local_num: row.get(1)?,
            started_at: row.get(2)?,
            result: row.get::<_, Option<String>>(3)?,
            white_username: row.get(4)?,
            black_username: row.get(5)?,
        })
    })?;

    let mut lines = Vec::new();
    for row in rows {
        let row = row?;
        let result = row.result.unwrap_or_else(|| "ongoing".to_string());
        let white_name = crate::utils::format_username(&row.white_username);
        let black_name = crate::utils::format_username(&row.black_username);
        let lichess_url = build_lichess_url(conn, row.id);
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

pub fn format_head_to_head(
    conn: &rusqlite::Connection,
    user_a: &DbUser,
    user_b: &DbUser,
    chat_id: i64,
    page: u32,
) -> Result<String> {
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) FROM games
         WHERE chat_id = ?3
           AND ((white_user_id = ?1 AND black_user_id = ?2)
             OR (white_user_id = ?2 AND black_user_id = ?1))",
    )?;
    let total: i64 = stmt.query_row(params![user_a.id, user_b.id, chat_id], |row| row.get(0))?;

    let limit = 10;
    let offset = ((page - 1) * limit) as i64;
    let mut stmt = conn.prepare(
        "WITH numbered AS (
            SELECT g.id, g.started_at, g.result, u1.username AS white_username, u2.username AS black_username,
                   ROW_NUMBER() OVER (ORDER BY g.started_at ASC) AS local_num
            FROM games g
            JOIN users u1 ON g.white_user_id = u1.id
            JOIN users u2 ON g.black_user_id = u2.id
            WHERE g.chat_id = ?3
              AND ((g.white_user_id = ?1 AND g.black_user_id = ?2)
                OR (g.white_user_id = ?2 AND g.black_user_id = ?1))
        )
        SELECT id, local_num, started_at, result, white_username, black_username
        FROM numbered
        ORDER BY started_at DESC
        LIMIT ?4 OFFSET ?5",
    )?;
    let rows = stmt.query_map(
        params![user_a.id, user_b.id, chat_id, limit, offset],
        |row| {
            Ok(HistoryRow {
                id: row.get(0)?,
                local_num: row.get(1)?,
                started_at: row.get(2)?,
                result: row.get::<_, Option<String>>(3)?,
                white_username: row.get(4)?,
                black_username: row.get(5)?,
            })
        },
    )?;

    let mut lines = Vec::new();
    for row in rows {
        let row = row?;
        let result = row.result.unwrap_or_else(|| "ongoing".to_string());
        let white_name = crate::utils::format_username(&row.white_username);
        let black_name = crate::utils::format_username(&row.black_username);
        let lichess_url = build_lichess_url(conn, row.id);
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
