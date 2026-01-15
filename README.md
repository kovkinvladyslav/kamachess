# KamaChess

A Telegram chess bot that allows users to play chess in group chats. Players can start games, make moves, and view their game history and statistics.

## Features

- Start chess games by replying to a user's message with `/start <move>`
- Make moves by replying to board messages
- View game history and statistics with `/history` command
- Head-to-head statistics between two players
- Automatic game state tracking and result recording
- Board visualization with material advantage display
- Uses long polling (no webhook setup required)

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- A Telegram bot token from [@BotFather](https://t.me/BotFather)

## Quick Start

### 1. Get Bot Token

1. Open Telegram and search for `@BotFather`
2. Send `/newbot` and follow the prompts
3. Save your bot token and username

### 2. Configure Environment

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your bot credentials:

```bash
TELEGRAM_BOT_TOKEN=your_actual_token_here
TELEGRAM_BOT_USERNAME=your_bot_username
```

### 3. Build and Run

```bash
cargo build --release
cargo run
```

The bot uses long polling to receive updates. Simply run `cargo run` and it will automatically start polling Telegram for updates. No webhook setup needed!

For development with debug logging:

```bash
RUST_LOG=debug cargo run
```

## Usage

### Starting a Game

Reply to any user's message with `/start` followed by your first move:

```
/start e4
/start e2e4
/start Nf3
```

This will start a new game with the user whose message you replied to.

### Making Moves

Reply to the bot's board message with your move:

```
e5
Nf3
e2e4
```

Supports both short notation (e4) and coordinate notation (e2e4).

### Viewing History

```
@kamachessbot /history
@kamachessbot @username /history
@kamachessbot @user1 @user2 /history
@kamachessbot /history 2
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TELEGRAM_BOT_TOKEN` | Yes | Bot token from BotFather |
| `TELEGRAM_BOT_USERNAME` | Yes | Bot username (without @) |
| `DATABASE_PATH` | No | SQLite database path (default: `kamachess.db`) |
| `RUST_LOG` | No | Logging level (default: `info`) |

## Project Structure

```
kamachess/
├── migrations/           # SQL schema files
│   └── 001_init.sql    # Initial database schema
├── src/
│   ├── api/            # Telegram API client
│   ├── db/             # Database operations
│   ├── game/           # Chess logic and rendering
│   ├── handlers/       # Message handlers
│   ├── models/         # Data structures
│   ├── parsing/        # Text parsing utilities
│   └── utils/          # Utility functions
├── .env                # Environment variables (create from .env.example)
└── Cargo.toml
```

## Development

Run tests:

```bash
cargo test
```

Check for issues:

```bash
cargo clippy
cargo fmt
```

## License

MIT
