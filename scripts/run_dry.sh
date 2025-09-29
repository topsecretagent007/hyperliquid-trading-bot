#!/bin/bash

# Hyperliquid Trading Bot - Dry Run Script
# This script runs the bot in dry-run mode for testing

set -e

echo "🔍 Hyperliquid Trading Bot - Dry Run Mode"
echo "========================================="

# Check if .env file exists
if [ ! -f .env ]; then
    echo "❌ .env file not found. Please run setup.sh first."
    exit 1
fi

# Check if config file exists
CONFIG_FILE=${1:-"config/default.toml"}
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Configuration file not found: $CONFIG_FILE"
    exit 1
fi

echo "📋 Using configuration: $CONFIG_FILE"
echo "🔍 Running in DRY RUN mode (no actual trades will be executed)"
echo ""

# Run the bot in dry-run mode
cargo run -- --config "$CONFIG_FILE" --dry-run --debug

echo ""
echo "✅ Dry run completed!"
