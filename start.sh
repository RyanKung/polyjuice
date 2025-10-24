#!/bin/bash

# Polyjuice - Farcaster FID Search Engine
# Simple startup script

echo "🧪 Starting Polyjuice - Farcaster FID Search Engine"
echo "=================================================="

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ Trunk is not installed. Installing..."
    cargo install trunk
fi

# Build and serve the application
echo "🚀 Building and serving Polyjuice..."
echo "📱 Open your browser to: http://localhost:8080"
echo "🔍 Try searching for FIDs like: 1, 2, 3, 100, 1000"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Try to start trunk serve
trunk serve --port 8080 --open || {
    echo "⚠️  Trunk serve failed, trying alternative method..."
    echo "🔧 Building with cargo..."
    cargo build --target wasm32-unknown-unknown --release
    echo "✅ Build completed. You can manually serve the files from the dist/ directory"
}
