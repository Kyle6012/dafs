#!/bin/bash

# DAFS Web Dashboard Setup Script
# This script configures the web dashboard for the integrated DAFS system

echo "🚀 DAFS Web Dashboard Setup"
echo "=========================="

# Check if we're in the web directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: This script must be run from the web directory"
    echo "   Please run: cd web && ./setup.sh"
    exit 1
fi

# Check if Node.js is installed
if ! command -v npm &> /dev/null; then
    echo "❌ Error: Node.js/npm is not installed"
    echo "   Please install Node.js 18+ first:"
    echo "   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt install nodejs"
    exit 1
fi

echo "📦 Installing dependencies..."
npm install

if [ $? -ne 0 ]; then
    echo "❌ Error: Failed to install dependencies"
    exit 1
fi

echo "✅ Dependencies installed successfully"

# Create .env file if it doesn't exist
if [ ! -f ".env" ]; then
    echo "🔧 Creating .env file..."
    cat > .env << EOF
# DAFS Web Dashboard Configuration
# The web dashboard is served by the Rust backend on port 3093
# API calls are made to the backend on port 6543

# Backend API URL (for API calls)
VITE_API_URL=http://localhost:6543

# Web dashboard URL (served by Rust backend)
VITE_WEB_URL=http://localhost:3093

# Development mode (set to false for production builds)
VITE_DEV_MODE=false
EOF
    echo "✅ Created .env file"
else
    echo "ℹ️  .env file already exists"
fi

echo ""
echo "🎯 Setup Complete!"
echo "=================="
echo ""
echo "📋 Next Steps:"
echo "1. Build the web dashboard: npm run build"
echo "2. Start the DAFS backend: cargo run (from parent directory)"
echo "3. Access the web dashboard: http://localhost:3093"
echo ""
echo "📚 For more information, see the main README.md"
echo "" 