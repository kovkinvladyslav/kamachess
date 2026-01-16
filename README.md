# Kamachess

A feature-rich Telegram chess bot built with Rust, enabling users to play chess games directly in Telegram chats with real-time board visualization and comprehensive game tracking.

![Game Example](screenshots/game.png)

## Features

- **Interactive Chess Games**: Play chess with other users in any Telegram chat
- **Real-time Board Rendering**: Custom PNG board generation with piece visualization
- **Intelligent Move Parsing**: Supports multiple notation formats (SAN, UCI, algebraic)
- **Game Management**: Start, resign, propose draws, and accept draw offers
- **Player Statistics**: Track wins, losses, draws, and head-to-head records
- **Game History**: View past games with pagination and analysis links
- **Image Caching**: Optimized board rendering with FEN-based caching
- **Database Support**: SQLite (default) or PostgreSQL backends
- **Docker Deployment**: Production-ready containerized setup

## Tech Stack

- **Language**: Rust 2021 Edition
- **Chess Engine**: [`chess`](https://crates.io/crates/chess) crate for move validation and board state
- **Image Processing**: Custom PNG rendering with the `image` crate
- **Database**: SQLx with support for SQLite and PostgreSQL
- **HTTP Client**: Reqwest with rustls-tls for Telegram Bot API
- **Async Runtime**: Tokio
- **Logging**: Tracing with file rotation

## Prerequisites

- Rust 1.83+ (for local development)
- Docker and Docker Compose (for containerized deployment)
- A Telegram Bot Token (obtain from [@BotFather](https://t.me/botfather))

## Setup

### 1. Clone the Repository

```bash
git clone https://github.com/kovkinvladyslav/kamachess.git
cd kamachess
```

### 2. Configure Environment Variables

Copy the example environment file and configure it:

```bash
cp .env.example .env
```

Edit `.env` with your credentials:

```env
TELEGRAM_BOT_TOKEN=your_bot_token_here
TELEGRAM_BOT_USERNAME=your_bot_username
DATABASE_URL=sqlite://kamachess.db?mode=rwc
LOG_DIR=logs
RUST_LOG=info
```

### 3. Local Development

#### Using SQLite (Default)

```bash
# Build and run
cargo build --release
cargo run --release
```

#### Using PostgreSQL

```bash
# Build with PostgreSQL feature
cargo build --release --features postgres --no-default-features

# Set DATABASE_URL in .env
# DATABASE_URL=postgres://user:password@localhost:5432/kamachess

cargo run --release --features postgres --no-default-features
```

### 4. Docker Deployment

The project includes a complete Docker Compose setup with PostgreSQL:

```bash
# Set your bot credentials in .env
export TELEGRAM_BOT_TOKEN=your_token
export TELEGRAM_BOT_USERNAME=your_username

# Start the services
docker-compose up -d

# View logs
docker-compose logs -f bot

# Stop the services
docker-compose down
```

## Usage

### Starting a Game

Reply to another user's message or mention them:

```
/start @username
/start @username e4
```

### Making Moves

Reply to the bot's board message with your move in any supported format:

```
e4
e2e4
Nf3
O-O
```

### Game Commands

- `/resign` - Resign the current game (reply to board)
- `/draw` - Propose a draw (reply to board)
- `/accept` - Accept a draw proposal (reply to board)

### Viewing Statistics

```
/history                    # Your stats in this chat
/history @username          # Another user's stats
/history @user1 @user2      # Head-to-head record
/history 2                  # Page 2 of your history
```

![History Example](screenshots/history.png)

### Help

```
/help
```

Commands work with bot username suffix in group chats:

```
/start@your_bot_username
/draw@your_bot_username
```

## Project Structure

```
kamachess/
├── src/
│   ├── api/              # Telegram Bot API client
│   ├── db/               # Database operations and migrations
│   ├── game/             # Chess logic, rendering, and notation
│   ├── handlers/         # Command and update handlers
│   ├── parsing/          # Move and command parsing
│   ├── models.rs         # Data structures
│   ├── utils.rs          # Utility functions
│   ├── lib.rs            # Library root
│   └── main.rs           # Application entry point
├── migrations/           # Database schema migrations
│   ├── sqlite/
│   └── postgres/
├── tests/                # Integration tests
├── images_cache/         # Cached board images
├── Dockerfile            # Multi-stage production build
└── docker-compose.yml    # PostgreSQL + bot setup
```

## Architecture

### Game Flow

1. User initiates game with `/start @opponent [optional_move]`
2. Bot creates game record and sends initial board image
3. Players reply to board message with moves
4. Bot validates moves, updates board state, and sends new image
5. Game ends on checkmate, stalemate, resignation, or draw acceptance

### Board Rendering

- Custom pixel-perfect PNG generation
- FEN-based caching for performance
- Automatic board flipping for black's perspective
- Embedded coordinate labels and piece glyphs
- Shadow effects for visual depth

### Database Schema

- **users**: Player profiles with Telegram metadata
- **games**: Game state, FEN positions, and results
- **moves**: Complete move history with UCI and SAN notation
- **stats**: Aggregated win/loss/draw statistics

## Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test chess_tests
cargo test image_cache_tests
```

## Logging

Logs are written to both stdout and rotating daily files in the `logs/` directory:

```bash
tail -f logs/kamachess.log
```

Configure log level via `RUST_LOG` environment variable:

```env
RUST_LOG=debug  # trace, debug, info, warn, error
```

## License

This project is open source. See the repository for license details.

## Contributing

Contributions are welcome! Please ensure all tests pass before submitting pull requests.

## Acknowledgments

- Chess logic powered by the [`chess`](https://crates.io/crates/chess) crate
- Board analysis links via [lichess.org](https://lichess.org)
