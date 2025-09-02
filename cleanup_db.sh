#!/bin/bash

echo "🧹 Cleaning up Gillean blockchain database..."

# Kill any running gillean processes
echo "Stopping any running gillean processes..."
pkill -f gillean 2>/dev/null || true

# Wait a moment for processes to stop
sleep 2

# Remove data directory
echo "Removing data directory..."
rm -rf ./data 2>/dev/null || true

# Remove any lock files
echo "Removing any lock files..."
find . -name "*.lock" -delete 2>/dev/null || true
find . -name "LOCK" -delete 2>/dev/null || true

echo "✅ Database cleanup complete!"
echo "💡 You can now start the blockchain fresh with: cargo run -- demo"
