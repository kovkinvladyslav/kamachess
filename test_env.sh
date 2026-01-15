#!/bin/bash
cd /home/jb/rust/kamachess
echo "Testing environment variable loading..."
echo ""
echo "Testing with dotenv (should work):"
RUST_BACKTRACE=1 cargo run 2>&1 | head -n 10
echo ""
echo "Checking if .env file exists:"
test -f .env && echo "✓ .env file exists" || echo "✗ .env file not found"
echo ""
echo "First 3 lines of .env (checking format):"
head -n 3 .env
