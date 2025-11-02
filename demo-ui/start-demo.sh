#!/bin/bash

# Medical Authorization Portal - Demo Startup Script

set -e

echo "=========================================="
echo "Medical Authorization Portal - Demo Setup"
echo "=========================================="
echo ""

# Check if node is installed
if ! command -v node &> /dev/null; then
    echo "❌ Error: Node.js is not installed"
    echo "   Please install Node.js v18 or later"
    exit 1
fi

echo "✓ Node.js version: $(node --version)"

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "❌ Error: npm is not installed"
    exit 1
fi

echo "✓ npm version: $(npm --version)"

# Check if cargo is installed (needed for backend)
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo (Rust) is not installed"
    echo "   Please install Rust from https://rustup.rs"
    exit 1
fi

echo "✓ Cargo version: $(cargo --version)"
echo ""

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing npm dependencies..."
    npm install
    echo "✓ Dependencies installed"
    echo ""
else
    echo "✓ npm dependencies already installed"
    echo ""
fi

# Check if zk-agent is built
echo "🔍 Checking ZK backend..."
if [ ! -f "../target/release/authz" ]; then
    echo "⚙️  Building ZK backend (this may take a few minutes)..."
    cd ..
    cargo build --release --package zk-agent
    cd demo-ui
    echo "✓ ZK backend built successfully"
    echo ""
else
    echo "✓ ZK backend already built"
    echo ""
fi

# Set environment variables
export SSZKP_BLOCKED_IFFT=1
echo "✓ Environment configured (streaming mode enabled)"
echo ""

echo "=========================================="
echo "🚀 Starting Medical Authorization Portal"
echo "=========================================="
echo ""
echo "The demo will open at: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Start the dev server
npm run dev

