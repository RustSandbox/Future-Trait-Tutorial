#!/bin/bash

# Future Trait Tutorial - EPUB Build Script

echo "📚 Building EPUB version of The Complete Future Trait Guide"
echo "=========================================================="

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

# Check if mdbook-epub is installed
if ! mdbook --help | grep -q "epub"; then
    echo "📦 mdbook-epub plugin not found. Installing..."
    cargo install mdbook-epub
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install mdbook-epub. Please install manually:"
        echo "   cargo install mdbook-epub"
        exit 1
    fi
    echo "✅ mdbook-epub installed successfully!"
fi

# Prepare cover image for EPUB
echo "🖼️  Preparing cover image..."
cp epub-book/theme/cover.png epub-book/src/cover.png
mkdir -p epub-book/src/theme
cp epub-book/theme/epub.css epub-book/src/theme/
echo "✅ Cover and theme files prepared!"

# Create output directory
mkdir -p epub-output

# Build the EPUB
echo "🔨 Building EPUB..."
cd epub-book

# Build the book
mdbook build

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ EPUB build completed successfully!"
    
    # Copy the EPUB to output directory
    if [ -f "book/The Complete Future Trait Guide.epub" ]; then
        cp "book/The Complete Future Trait Guide.epub" "../epub-output/"
        echo "📖 EPUB saved to: epub-output/The Complete Future Trait Guide.epub"
    elif [ -f "book/epub/The Complete Future Trait Guide.epub" ]; then
        cp "book/epub/The Complete Future Trait Guide.epub" "../epub-output/"
        echo "📖 EPUB saved to: epub-output/The Complete Future Trait Guide.epub"
    else
        echo "⚠️  EPUB file not found in expected location. Checking book directory..."
        find book -name "*.epub" -exec cp {} "../epub-output/" \;
    fi
    
    # Show file info
    if [ -f "../epub-output/The Complete Future Trait Guide.epub" ]; then
        echo ""
        echo "📊 EPUB Information:"
        echo "   File: The Complete Future Trait Guide.epub"
        echo "   Size: $(du -h "../epub-output/The Complete Future Trait Guide.epub" | cut -f1)"
        echo "   Location: $(pwd)/../epub-output/"
        echo ""
        echo "🎉 Your EPUB is ready! You can now:"
        echo "   • Open it in any EPUB reader (Apple Books, Calibre, etc.)"
        echo "   • Transfer it to your e-reader device"
        echo "   • Share it with others"
    fi
else
    echo "❌ EPUB build failed. Please check the error messages above."
    exit 1
fi

cd ..

echo ""
echo "🚀 Build complete! The EPUB version of 'The Complete Future Trait Guide' is ready."
echo "   Find it in: epub-output/The Complete Future Trait Guide.epub" 