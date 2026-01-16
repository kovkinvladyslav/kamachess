CREATE TABLE IF NOT EXISTS game_messages (
    id BIGSERIAL PRIMARY KEY,
    game_id BIGINT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    message_id BIGINT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_game_messages_game_id
    ON game_messages(game_id);
