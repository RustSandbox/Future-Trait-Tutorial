# The Complete Future Trait Guide - EPUB Edition

This directory contains the EPUB version of "The Complete Future Trait Guide: Mastering Async Programming in Rust".

## Building the EPUB

### Prerequisites

1. **mdBook**: Install the mdBook tool
   ```bash
   cargo install mdbook
   ```

2. **mdbook-epub**: Install the EPUB plugin for mdBook
   ```bash
   cargo install mdbook-epub
   ```

3. **ImageMagick** (optional): For cover image conversion
   ```bash
   # macOS
   brew install imagemagick
   
   # Ubuntu/Debian
   sudo apt install imagemagick
   
   # Windows
   # Download from https://imagemagick.org/script/download.php
   ```

### Building

From the project root directory:

```bash
# Use the automated build script
./build_epub.sh

# Or build manually
cd epub-book
mdbook build
```

The EPUB file will be generated in the `book/` directory.

## EPUB Features

### Optimized for E-readers

- **Typography**: Serif fonts for body text, sans-serif for headings
- **Responsive**: Adapts to different screen sizes and orientations
- **Dark mode support**: Automatically adjusts for dark theme preferences
- **Page breaks**: Proper chapter breaks and pagination
- **Code formatting**: Monospace fonts with syntax highlighting

### Metadata

The EPUB includes comprehensive metadata:

- **Title**: The Complete Future Trait Guide
- **Author**: Future Trait Tutorial
- **Language**: English
- **Subject**: Programming, Rust, Async Programming, Future Trait
- **Description**: A comprehensive guide to mastering async programming in Rust through the Future trait

### Navigation

- **Table of Contents**: Full hierarchical navigation
- **Chapter links**: Easy navigation between chapters
- **Cross-references**: Links to related sections and appendices

## File Structure

```
epub-book/
├── book.toml              # mdBook configuration for EPUB
├── src/                   # Book source files
│   ├── SUMMARY.md         # Table of contents
│   ├── introduction.md    # Introduction chapter
│   ├── chapter-*.md       # Main chapters
│   └── appendix-*.md      # Appendices
├── theme/                 # EPUB-specific styling
│   ├── epub.css          # EPUB stylesheet
│   ├── cover.svg         # Cover image (SVG)
│   └── cover.png         # Cover image (PNG, generated)
└── README.md             # This file
```

## Compatibility

The generated EPUB is compatible with:

### E-readers
- **Kindle** (via conversion with Calibre)
- **Kobo**
- **Nook**
- **Sony Reader**
- **PocketBook**

### Software
- **Apple Books** (macOS/iOS)
- **Calibre** (Windows/macOS/Linux)
- **Adobe Digital Editions**
- **Google Play Books**
- **Microsoft Edge** (built-in EPUB support)

### Mobile Apps
- **Apple Books** (iOS)
- **Google Play Books** (Android/iOS)
- **Kindle** (via conversion)
- **Moon+ Reader** (Android)
- **FBReader** (Android/iOS)

## Customization

### Styling

Edit `theme/epub.css` to customize:
- Typography and fonts
- Colors and themes
- Layout and spacing
- Code block styling

### Cover Image

Replace `theme/cover.svg` with your own design. The build script will automatically convert it to PNG if ImageMagick is available.

### Metadata

Edit `book.toml` to update:
- Title and description
- Author information
- Publication details
- Language and subjects

## Troubleshooting

### Common Issues

1. **mdbook-epub not found**
   ```bash
   cargo install mdbook-epub
   ```

2. **Cover image not displaying**
   - Ensure `theme/cover.png` exists
   - Check that the path in `book.toml` is correct

3. **Formatting issues**
   - Validate your Markdown syntax
   - Check for unsupported HTML elements
   - Ensure proper heading hierarchy

4. **Large file size**
   - Optimize images before including
   - Consider splitting very long chapters
   - Remove unnecessary styling

### Getting Help

- **mdBook Documentation**: https://rust-lang.github.io/mdBook/
- **mdbook-epub Plugin**: https://github.com/Michael-F-Bryan/mdbook-epub
- **EPUB Specification**: https://www.w3.org/publishing/epub32/

## License

This educational content is provided as part of the Future Trait Tutorial project. The EPUB format and styling are optimized for educational use and can be freely shared for learning purposes. 