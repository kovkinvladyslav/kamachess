# Kamachess

A Telegram chess bot written in Rust.

## Setup

1.  **Clone and Build**:
    ```bash
    git clone https://github.com/youruser/kamachess
    cd kamachess
    cargo build --release
    ```

2.  **Configuration**:
    Create `.env`:
    ```ini
    TELEGRAM_BOT_TOKEN=your_token
    TELEGRAM_BOT_USERNAME=your_bot_username
    DATABASE_URL=sqlite://kamachess.db?mode=rwc
    LOG_DIR=logs
    RUST_LOG=info
    ```

3.  **Run**:
    ```bash
    cargo run --release
    ```

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

## Features

*   **Move Validation**: Enforces legal moves, castling, promotion, and en passant.
*   **Board Support**: Generates board images locally (cached).
*   **Stats**: Tracks wins/losses/draws per chat.
*   **Database**: SQLite (default) or PostgreSQL.

## Technical Details

*   **Language**: Rust
*   **Crates**: `chess`, `sqlx`, `image`, `tokio`, `teloxide` (or `reqwest` for API).
*   **Rendering**: Procedural generation using bitmap assets.

## License

MIT
