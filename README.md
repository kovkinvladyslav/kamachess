# kamachess

A Telegram chess bot written in Rust for playing chess in group chats.

![Game Screenshot](screenshots/game.png)

## Features

**Game Management** — Start a game by replying to any user's message with `/start` or mention them directly with `/start @username`. Include an opening move like `/start e4` or `/start @user Nf3` to play immediately.

**Move Notation** — Supports standard algebraic notation (SAN) including pawn moves (`e4`, `exd5`), piece moves (`Nf3`, `Bb5`), disambiguation (`Nbd7`, `R1e2`), castling (`O-O`, `O-O-O`, `0-0`), and promotions (`e8=Q`). Coordinate notation also works (`e2e4`, `g1f3`).

**Board Rendering** — Generates PNG board images procedurally using 16x16 pixel piece bitmaps scaled 3x with shadows and outlines. No external fonts or image assets required.

**Draw and Resign** — Propose a draw with `/draw`, accept with `/accept` or `/acceptdraw`. Resign with `/resign`. All game commands work with `@botname` suffix for group clarity (`/draw@kamachessbot`).

**Statistics and History** — Track wins, losses, draws, and win percentage per chat. View your stats with `/history`, another player's with `/history @user`, or head-to-head records with `/history @user1 @user2`. Pagination supported with `/history 2`.

![History Screenshot](screenshots/history.png)

**Lichess Analysis Links** — Each game in history includes a link to analyze the full game on Lichess.

**Self-Play Prevention** — Users cannot start games against themselves or have multiple simultaneous games against the same opponent in one chat.

**Database Support** — SQLite by default for simple deployments, PostgreSQL via feature flag for production. Schema migrations run automatically on startup.

**Docker Support** — Includes Dockerfile and docker-compose.yml for containerized PostgreSQL deployments.

**Logging** — Daily rotating log files with structured tracing via `tracing-subscriber`.

## Commands

| Command | Description |
|---------|-------------|
| `/start [@user] [move]` | Start a game by replying to a user or mentioning them |
| `/move <move>` | Play a move (alternative to sending move directly) |
| `/resign` | Resign the current game |
| `/draw` | Propose a draw |
| `/accept` | Accept a draw proposal |
| `/history [@user] [page]` | View game history and statistics |
| `/help` | Show command reference |

Reply to the bot's board message to play moves or use game commands.

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

Create a `.env` file:

```
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_BOT_USERNAME=your_bot_username
DATABASE_URL=sqlite://kamachess.db?mode=rwc
LOG_DIR=logs
RUST_LOG=info
```

For PostgreSQL use `DATABASE_URL=postgres://user:password@localhost:5432/kamachess`.

## Running

Run locally with SQLite using `cargo run --release`. For PostgreSQL, set the `DATABASE_URL` and add `--features postgres --no-default-features`. For Docker, run `docker-compose up` which starts both PostgreSQL and the bot.

The bot uses long polling with a 30-second timeout. Database schema applies automatically on first run.

## Testing

```bash
cargo test
```

Tests cover move parsing, castling variants, coordinate notation, database operations, and full game sequences from historical matches including the Opera Game and Légal's Mate.

## Technical Details

Board rendering uses the `image` crate with piece bitmaps stored as `[u16; 16]` arrays. Move parsing builds on the `chess` crate's legal move generation with custom SAN parsing for disambiguation and special moves. Database operations use `sqlx` with the Any driver for runtime database selection. The bot runs on Tokio with `reqwest` for Telegram API calls.

## License

MIT
