#!/bin/bash

# Future Trait Tutorial - Book Server Startup Script

echo "🦀 Future Trait Tutorial - Starting Book Server"
echo "=============================================="

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo "📦 mdBook not found. Installing..."
    cargo install mdbook
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install mdBook. Please install manually:"
        echo "   cargo install mdbook"
        exit 1
    fi
    echo "✅ mdBook installed successfully!"
fi

echo "🚀 Starting book server..."
echo "📖 The book will be available at: http://localhost:3000"
echo "🔄 The server will auto-reload when you make changes"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Start the book server
mdbook serve --open

echo "👋 Book server stopped. Thanks for learning!" 