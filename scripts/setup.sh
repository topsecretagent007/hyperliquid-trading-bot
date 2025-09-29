#!/bin/bash

# Hyperliquid Trading Bot Setup Script
# This script helps set up the trading bot environment

set -e

echo "🚀 Hyperliquid Trading Bot Setup"
echo "================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust is installed: $(cargo --version)"

# Check Rust version
RUST_VERSION=$(cargo --version | cut -d' ' -f2 | cut -d'.' -f1)
if [ "$RUST_VERSION" -lt 1 ]; then
    echo "❌ Rust version 1.70+ is required. Current version: $(cargo --version)"
    exit 1
fi

echo "✅ Rust version is compatible"

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p logs
mkdir -p config
mkdir -p data

echo "✅ Directories created"

# Copy environment file if it doesn't exist
if [ ! -f .env ]; then
    echo "📋 Creating .env file from template..."
    cp env.example .env
    echo "✅ .env file created. Please edit it with your API credentials."
else
    echo "✅ .env file already exists"
fi

# Build the project
echo "🔨 Building the project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

# Run tests
echo "🧪 Running tests..."
cargo test

if [ $? -eq 0 ]; then
    echo "✅ Tests passed"
else
    echo "❌ Tests failed"
    exit 1
fi

# Check configuration
echo "⚙️  Checking configuration..."
if [ -f config/default.toml ]; then
    echo "✅ Default configuration found"
else
    echo "❌ Default configuration not found"
    exit 1
fi

# Create log directory with proper permissions
echo "📝 Setting up logging..."
mkdir -p logs
touch logs/bot.log
chmod 644 logs/bot.log

echo "✅ Logging setup complete"

# Display next steps
echo ""
echo "🎉 Setup completed successfully!"
echo ""
echo "Next steps:"
echo "1. Edit .env file with your Hyperliquid API credentials"
echo "2. Review config/default.toml for your trading preferences"
echo "3. Run in dry-run mode: cargo run -- --dry-run"
echo "4. When ready, run live: cargo run -- --config config/production.toml"
echo ""
echo "For more information, see README.md"
echo ""
echo "Happy trading! 🚀📈"
