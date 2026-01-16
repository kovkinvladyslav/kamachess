# kamachess

A Telegram chess bot written in Rust. Play chess in group chats with PNG board rendering, move validation, and player statistics.

## Features

- **Game Management**: Start games by replying to users or mentioning them (`/start @user e4`)
- **Move Input**: Supports SAN (`Nf3`, `O-O`, `exd5`) and coordinate notation (`e2e4`, `g1f3`)
- **Board Rendering**: Procedural PNG generation with piece bitmaps, no external assets
- **Draw/Resign**: Propose draws, accept them, or resign mid-game
- **Statistics**: Per-chat history, win/loss/draw tracking, head-to-head records
- **Database**: SQLite (default) or PostgreSQL support via feature flags
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

For PostgreSQL support:

```bash
cargo build --release --features postgres --no-default-features
```

## Configuration

Create a `.env` file (see `.env.example`):

```
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_BOT_USERNAME=your_bot_username
DATABASE_URL=sqlite://kamachess.db?mode=rwc
LOG_DIR=logs
RUST_LOG=info
```

For PostgreSQL:

```
DATABASE_URL=postgres://user:password@localhost:5432/kamachess
```

## Running

### Local (SQLite)

```bash
cargo run --release
```

### Local (PostgreSQL)

```bash
DATABASE_URL=postgres://user:pass@localhost/kamachess cargo run --release --features postgres --no-default-features
```

### Docker (PostgreSQL)

```bash
docker-compose up
```

The bot uses long polling. Database schema is applied automatically on first run.

## Technical Notes

**Board Rendering**: Uses the `image` crate to draw boards pixel-by-pixel. Pieces are 16x16 bitmaps stored as `[u16; 16]` arrays, scaled 3x with shadows and outlines. No fonts or external images.

**Move Parsing**: The `chess` crate handles legal move generation. This project adds a SAN parser on top that handles disambiguation (`Nbd7`, `R1e2`), castling variants (`O-O`, `0-0`), and promotions (`e8=Q`).

**Database**: Supports SQLite (default) and PostgreSQL via `sqlx`. Migrations are embedded via `include_str!` and applied idempotently at startup.

**Async**: Tokio runtime with `reqwest` for HTTP. Long polling with 30s timeout.

## Testing

```bash
cargo test
```

Tests include move parsing, castling, and full game sequences from historical matches (Opera Game, Légal's Mate).

## License

MIT
