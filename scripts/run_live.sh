#!/bin/bash

# Hyperliquid Trading Bot - Live Trading Script
# This script runs the bot in live trading mode
# WARNING: This will execute real trades!

set -e

echo "⚠️  Hyperliquid Trading Bot - LIVE TRADING MODE"
echo "=============================================="
echo "WARNING: This will execute REAL TRADES with REAL MONEY!"
echo ""

# Confirmation prompt
read -p "Are you sure you want to run in LIVE mode? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "❌ Live trading cancelled"
    exit 1
fi

# Check if .env file exists
if [ ! -f .env ]; then
    echo "❌ .env file not found. Please run setup.sh first."
    exit 1
fi

# Check if production config exists
CONFIG_FILE=${1:-"config/production.toml"}
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Production configuration file not found: $CONFIG_FILE"
    echo "Please create config/production.toml with your live trading settings."
    exit 1
fi

echo "📋 Using configuration: $CONFIG_FILE"
echo "💰 Running in LIVE TRADING mode"
echo ""

# Final confirmation
read -p "Final confirmation - proceed with live trading? (yes/no): " final_confirm
if [ "$final_confirm" != "yes" ]; then
    echo "❌ Live trading cancelled"
    exit 1
fi

echo "🚀 Starting live trading bot..."
echo ""

# Run the bot in live mode
cargo run --release -- --config "$CONFIG_FILE"

echo ""
echo "✅ Live trading session ended!"
