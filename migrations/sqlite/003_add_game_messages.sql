CREATE TABLE IF NOT EXISTS game_messages (
    id INTEGER PRIMARY KEY,
    game_id INTEGER NOT NULL,
    message_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_game_messages_game_id
    ON game_messages(game_id);
