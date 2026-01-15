# kamachess

A Telegram chess bot written in Rust. Play chess in group chats with PNG board rendering, move validation, and player statistics.

## Features

- **Game Management**: Start games by replying to users or mentioning them (`/start @user e4`)
- **Move Input**: Supports SAN (`Nf3`, `O-O`, `exd5`) and coordinate notation (`e2e4`, `g1f3`)
- **Board Rendering**: Procedural PNG generation with piece bitmaps, no external assets
- **Draw/Resign**: Propose draws, accept them, or resign mid-game
- **Statistics**: Per-chat history, win/loss/draw tracking, head-to-head records
- **Logging**: Daily rotating log files with structured tracing

## Commands

| Command | Description |
|---------|-------------|
| `/start [move]` | Reply to a user or mention `@username` to start a game |
| `/resign` | Resign the current game (reply to board) |
| `/draw` | Propose a draw |
| `/accept` | Accept a draw proposal |
| `/history [@user] [page]` | View stats for yourself, a user, or head-to-head |
| `/help` | Show command reference |

Moves are sent by replying to the bot's board message.

## Installation

Requires Rust 1.70+.

```bash
git clone https://github.com/youruser/kamachess
cd kamachess
cargo build --release
```

## Configuration

Create a `.env` file:

```
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_BOT_USERNAME=your_bot_username
DATABASE_PATH=kamachess.db    # optional, defaults to kamachess.db
LOG_DIR=logs                  # optional, defaults to logs/
RUST_LOG=info                 # optional, tracing filter
```

## Running

```bash
cargo run --release
```

The bot uses long polling. Database schema is applied automatically on first run.

## Project Structure

```
src/
├── api/          # Telegram API client (reqwest, multipart uploads)
├── db/           # SQLite queries, migrations via include_str!
├── game/
│   ├── chess.rs  # Move parsing (SAN + UCI), board status
│   ├── render.rs # PNG generation with image crate
│   └── glyphs.rs # 16x16 bitmap patterns for pieces
├── handlers/     # Command routing and game logic
├── parsing/      # Text extraction (moves, usernames, pagination)
└── models.rs     # Telegram + DB structs
```

## Technical Notes

**Board Rendering**: Uses the `image` crate to draw boards pixel-by-pixel. Pieces are 16x16 bitmaps stored as `[u16; 16]` arrays, scaled 3x with shadows and outlines. No fonts or external images.

**Move Parsing**: The `chess` crate handles legal move generation. This project adds a SAN parser on top that handles disambiguation (`Nbd7`, `R1e2`), castling variants (`O-O`, `0-0`), and promotions (`e8=Q`).

**Database**: SQLite with `r2d2` connection pooling. Migrations are embedded via `include_str!` and applied idempotently at startup.

**Async**: Tokio runtime with `reqwest` for HTTP. Long polling with 30s timeout.

## Testing

```bash
cargo test
```

Tests include move parsing, castling, and full game sequences from historical matches (Opera Game, Légal's Mate).

## License

MIT
