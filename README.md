# Kamachess

A Telegram chess bot written in Rust.

![Game Screenshot](screenshots/game.png)

## Features

Kamachess allows you to play chess directly in your Telegram group chats. It supports standard chess rules, move validation, and procedurally generated board images using high-quality bitmap assets.

*   **Move Validation**: Enforces legal moves, castling, promotion, and en passant.
*   **Board Rendering**: Procedurally generates board images using bitmap assets (cached for performance).
*   **Stats**: Tracks wins/losses/draws per chat.

    ![History Screenshot](screenshots/history.png)
*   **Database**: SQLite (default) or PostgreSQL.

## Usage

**Note**: The bot is designed for **group chats**. Add the bot to a group to play.

| Command | Description |
| :--- | :--- |
| `/start [@user] [move]` | Start a game by replying to a user or mentioning them. |
| `/resign` | Resign the current game. |
| `/draw` | Propose a draw. |
| `/accept` | Accept a draw proposal. |
| `/history` | View game statistics. |
| `/help` | Show available commands. |

**Playing**: Reply to the bot's board message with your move in algebraic notation (e.g., `e4`, `Nf3`, `O-O`).

## Deployment

### Local Development
1.  Setup `.env` (see below).
2.  `cargo run --release`

### Production with Docker
For a robust production environment using PostgreSQL:

1.  **Configure Environment**:
    Create a `.env` file in the project root:
    ```ini
    TELEGRAM_BOT_TOKEN=ur_token
    TELEGRAM_BOT_USERNAME=ur_bot_name
    # DATABASE_URL is handled automatically by docker-compose
    ```

2.  **Start Services**:
    ```bash
    docker-compose up -d
    ```
    This starts a PostgreSQL container and the bot container. The database schema is applied automatically on startup.

3.  **View Logs**:
    ```bash
    docker-compose logs -f
    ```

## License

MIT
