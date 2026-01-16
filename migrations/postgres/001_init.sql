CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    telegram_id BIGINT UNIQUE,
    username TEXT UNIQUE,
    first_name TEXT,
    last_name TEXT,
    wins BIGINT NOT NULL DEFAULT 0,
    losses BIGINT NOT NULL DEFAULT 0,
    draws BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS games (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    white_user_id BIGINT NOT NULL REFERENCES users(id),
    black_user_id BIGINT NOT NULL REFERENCES users(id),
    current_fen TEXT NOT NULL,
    turn TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'ongoing',
    result TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    last_message_id BIGINT,
    draw_proposed_by BIGINT
);

CREATE TABLE IF NOT EXISTS moves (
    id BIGSERIAL PRIMARY KEY,
    game_id BIGINT NOT NULL REFERENCES games(id),
    move_number BIGINT NOT NULL,
    uci TEXT NOT NULL,
    san TEXT,
    played_by BIGINT NOT NULL REFERENCES users(id),
    played_at TEXT NOT NULL
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
