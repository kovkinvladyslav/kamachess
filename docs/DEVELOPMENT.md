# Development Guide

This guide covers local development setup, Docker workflow, and testing guidelines.

## Prerequisites

- Docker and Docker Compose
- Git
- (Optional) Rust 1.83+ for local development without Docker

## Local Development Setup

### Option 1: Docker Development (Recommended)

**No local Rust installation needed!**

```bash
# Clone repository
git clone https://github.com/yourusername/kamachess.git
cd kamachess

# Copy environment file
cp .env.example .env

# Edit .env with your values
nano .env

# Start development environment
docker-compose up -d

# View logs
docker-compose logs -f bot

# Stop services
docker-compose down
```

### Option 2: Local Rust Development

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone repository
git clone https://github.com/yourusername/kamachess.git
cd kamachess

# Install dependencies
cargo build

# Run application
cargo run --release
```

## Docker Workflow

### Building Images

```bash
# Build bot image
docker-compose build bot

# Build with no cache
docker-compose build --no-cache bot

# Build all services
docker-compose build
```

### Running Tests

```bash
# Run tests in Docker
docker-compose run --rm bot cargo test

# Run specific test
docker-compose run --rm bot cargo test test_name

# Run with output
docker-compose run --rm bot cargo test -- --nocapture
```

### Debugging

```bash
# Access bot container shell
docker-compose exec bot sh

# View logs in real-time
docker-compose logs -f bot

# Inspect container
docker-compose exec bot ps aux
docker-compose exec bot env
```

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

```bash
# Edit files
vim src/handlers/game_handler.rs

# Build to check for errors
docker-compose run --rm bot cargo build
```

### 3. Run Tests

```bash
# Run all tests
docker-compose run --rm bot cargo test

# Run specific test file
docker-compose run --rm bot cargo test --test telegram_api_tests
```

### 4. Linting

```bash
# Run Clippy
docker-compose run --rm bot cargo clippy -- -D warnings

# Format code
docker-compose run --rm bot cargo fmt
```

### 5. Commit and Push

```bash
git add .
git commit -m "Add new feature"
git push origin feature/my-feature
```

## Testing Guidelines

### Unit Tests

Located in `src/` directories with `#[cfg(test)]` modules.

Run:
```bash
cargo test
```

### Integration Tests

Located in `tests/` directory.

Run:
```bash
cargo test --test telegram_api_tests
cargo test --test webhook_server_tests
```

### Test Coverage

```bash
# Install cargo-tarpaulin (optional)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

### Writing Tests

**Unit test example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        assert_eq!(function(), expected_value);
    }
}
```

**Integration test example:**
```rust
#[tokio::test]
async fn test_feature() {
    // Test implementation
    assert!(result.is_ok());
}
```

## Code Quality

### Linting with Clippy

```bash
# Run Clippy
cargo clippy -- -D warnings

# Auto-fix issues
cargo clippy --fix
```

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Pre-commit Hooks (Optional)

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
cargo fmt -- --check && cargo clippy -- -D warnings
```

## Database Development

### Using SQLite (Default)

```bash
# Default database file: kamachess.db
DATABASE_URL=sqlite://kamachess.db?mode=rwc
```

### Using PostgreSQL

```bash
# Start PostgreSQL
docker-compose up -d postgres

# Set DATABASE_URL
DATABASE_URL=postgres://kamachess:kamachess@localhost:5432/kamachess
```

### Running Migrations

```bash
# Migrations run automatically on startup
# Or manually via SQLx CLI (if installed)
sqlx migrate run
```

### Database Inspection

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U kamachess -d kamachess

# View tables
\dt

# Query data
SELECT * FROM games LIMIT 10;
```

## Debugging

### Enable Debug Logging

```bash
# In .env
RUST_LOG=debug
```

### Print Debug Statements

```rust
use tracing::{debug, info, warn, error};

debug!("Debug message: {}", value);
info!("Info message");
warn!("Warning message");
error!("Error message");
```

### Using GDB (Advanced)

```bash
# Build debug version
docker-compose run --rm bot cargo build

# Run with GDB
docker run -it --rm \
    -v $(pwd):/app \
    rust:1.88-bookworm \
    bash -c "cd /app && gdb target/debug/kamachess"
```

## Performance Profiling

### Build with Profile

```bash
# Profile build
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" cargo build --release

# Use profiler (requires perf or valgrind)
perf record ./target/release/kamachess
```

## Common Issues

### Port Already in Use

```bash
# Find process using port
lsof -i :8080

# Kill process
kill -9 <PID>
```

### Docker Build Fails

```bash
# Clear Docker cache
docker system prune -a

# Rebuild from scratch
docker-compose build --no-cache
```

### Database Connection Issues

```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Check connection string
echo $DATABASE_URL

# Test connection
docker-compose exec postgres psql -U kamachess -d kamachess -c "SELECT 1"
```

## Development Best Practices

1. **Write Tests First (TDD):**
   - Write failing test
   - Implement feature
   - Make test pass

2. **Small Commits:**
   - Commit frequently
   - Clear commit messages
   - One feature per commit

3. **Code Reviews:**
   - Create pull requests
   - Request reviews
   - Address feedback

4. **Documentation:**
   - Document public APIs
   - Add inline comments for complex logic
   - Update README for user-facing changes

5. **Performance:**
   - Profile before optimizing
   - Use appropriate data structures
   - Cache when appropriate

## IDE Setup

### VS Code

**Extensions:**
- rust-analyzer
- CodeLLDB (for debugging)
- Better TOML

**Settings:**
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": ["postgres"]
}
```

### IntelliJ IDEA / CLion

**Plugins:**
- Rust plugin
- Docker plugin

## Next Steps

- Deploy to production: [DEPLOYMENT.md](DEPLOYMENT.md)
- Setup CI/CD: [CI_CD.md](CI_CD.md)
- Configure monitoring: [MONITORING.md](MONITORING.md)
