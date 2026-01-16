CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    telegram_id INTEGER UNIQUE,
    username TEXT UNIQUE,
    first_name TEXT,
    last_name TEXT,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    draws INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY,
    chat_id INTEGER NOT NULL,
    white_user_id INTEGER NOT NULL,
    black_user_id INTEGER NOT NULL,
    current_fen TEXT NOT NULL,
    turn TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'ongoing',
    result TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    last_message_id INTEGER,
    draw_proposed_by INTEGER,
    FOREIGN KEY(white_user_id) REFERENCES users(id),
    FOREIGN KEY(black_user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS moves (
    id INTEGER PRIMARY KEY,
    game_id INTEGER NOT NULL,
    move_number INTEGER NOT NULL,
    uci TEXT NOT NULL,
    san TEXT,
    played_by INTEGER NOT NULL,
    played_at TEXT NOT NULL,
    FOREIGN KEY(game_id) REFERENCES games(id),
    FOREIGN KEY(played_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_games_chat_message
    ON games(chat_id, last_message_id);

CREATE INDEX IF NOT EXISTS idx_games_chat_status
    ON games(chat_id, status);

CREATE INDEX IF NOT EXISTS idx_games_chat_players
    ON games(chat_id, white_user_id, black_user_id);

CREATE INDEX IF NOT EXISTS idx_moves_game
    ON moves(game_id);

CREATE INDEX IF NOT EXISTS idx_moves_game_number
    ON moves(game_id, move_number);
