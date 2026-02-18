#!/bin/bash

# claWasm Quick Start Script
# Usage: ./start.sh

set -e

echo "🦀 claWasm Starting..."
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install it first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if wasm32 target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Check if wasm-bindgen is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "📦 Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Build WASM
echo "🔨 Building WASM module..."
cargo build --target wasm32-unknown-unknown --release

# Generate JS bindings
echo "🔗 Generating JavaScript bindings..."
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/clawasm.wasm

# Build proxy (if features available)
echo "🚀 Building proxy server..."
cargo build --bin proxy --features proxy --release 2>/dev/null || echo "⚠️  Proxy build skipped (optional)"

echo ""
echo "✅ Build complete!"
echo ""

# Kill any existing proxy on port 3000
if lsof -ti :3000 &>/dev/null; then
    echo "🔄 Stopping existing proxy on port 3000..."
    kill $(lsof -ti :3000) 2>/dev/null
    sleep 1
fi

# Start proxy in background
if [ -f "./target/release/proxy" ]; then
    echo "🔄 Starting proxy server on http://localhost:3000..."
    ./target/release/proxy &
    PROXY_PID=$!
    sleep 1
fi

# Start web server
echo "🌐 Starting web server on http://localhost:5001..."
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  claWasm is running!"
echo ""
echo "  📱 Open: http://localhost:5001"
echo "  🔧 Proxy: http://localhost:3000"
echo ""
echo "  Press Ctrl+C to stop"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Trap exit to kill proxy
trap 'if [ ! -z "$PROXY_PID" ]; then kill $PROXY_PID 2>/dev/null; fi' EXIT

# Start Python HTTP server
cd web
python3 -m http.server 5001
