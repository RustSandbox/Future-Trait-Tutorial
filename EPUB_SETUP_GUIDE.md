# EPUB Setup Guide

This guide explains how to generate and use the EPUB version of "The Complete Future Trait Guide".

## 📚 What's Included

The EPUB version provides the complete Future trait tutorial in a format optimized for e-readers and mobile devices:

### Content Structure
- **Introduction** - Welcome and overview
- **Part I: Fundamentals** (Chapters 1-3)
  - Understanding Async Programming
  - The Future Trait
  - Basic async/await
- **Part II: Implementation** (Chapter 4)
  - Custom Future Implementation
- **Part III: Composition and Patterns** (Chapter 8)
  - Error Handling
- **Part IV: Advanced Topics** (Chapter 10)
  - Autonomous Agent Example
- **Appendices**
  - Code Examples Reference

### Features
✅ **E-reader optimized** typography and layout  
✅ **Dark mode support** for comfortable reading  
✅ **Proper navigation** with hierarchical table of contents  
✅ **Code syntax highlighting** optimized for e-ink displays  
✅ **Cross-references** between chapters and sections  
✅ **Responsive design** that adapts to different screen sizes  

## 🛠️ Building the EPUB

### Quick Start

```bash
# One-command build
./build_epub.sh
```

This script will:
1. Check for required dependencies
2. Install mdbook and mdbook-epub if needed
3. Prepare cover image and theme files
4. Build the EPUB
5. Save it to `epub-output/`

### Manual Build

If you prefer to build manually:

```bash
# Install dependencies
cargo install mdbook mdbook-epub

# Build the EPUB
cd epub-book
mdbook build

# Find the generated EPUB
find book -name "*.epub"
```

### Prerequisites

1. **Rust and Cargo** - For installing mdbook tools
2. **mdbook** - The book generation tool
   ```bash
   cargo install mdbook
   ```
3. **mdbook-epub** - EPUB plugin for mdbook
   ```bash
   cargo install mdbook-epub
   ```
4. **Cover Image** - The build script automatically handles cover image setup

## 📱 Using the EPUB

### Compatible Devices and Apps

**E-readers:**
- Kobo (all models)
- Nook (all models)
- Sony Reader
- PocketBook
- Most Android-based e-readers

**Mobile Apps:**
- Apple Books (iOS/macOS)
- Google Play Books (Android/iOS)
- Moon+ Reader (Android)
- FBReader (Android/iOS)
- Kindle (via conversion)

**Desktop Software:**
- Calibre (Windows/macOS/Linux)
- Adobe Digital Editions
- Microsoft Edge (built-in EPUB support)

### For Kindle Users

Kindle doesn't natively support EPUB, but you can convert it:

1. **Using Calibre:**
   ```bash
   # Install Calibre, then:
   ebook-convert "The Complete Future Trait Guide.epub" "The Complete Future Trait Guide.mobi"
   ```

2. **Using Amazon's Send to Kindle:**
   - Email the EPUB to your Kindle email address
   - Amazon will automatically convert it

3. **Using Kindle Previewer:**
   - Download from Amazon's developer tools
   - Convert EPUB to KPF format

## 🎨 Customization

### Styling

The EPUB uses custom CSS optimized for e-readers. To customize:

1. Edit `epub-book/theme/epub.css`
2. Modify typography, colors, or layout
3. Rebuild the EPUB

Key customizable elements:
- **Fonts**: Body text uses serif, headings use sans-serif
- **Colors**: Optimized for both light and dark modes
- **Code blocks**: Monospace with syntax highlighting
- **Page breaks**: Automatic chapter breaks

### Cover Image

To use a custom cover:

1. Replace `epub-book/theme/cover.png` with your design
2. Ensure dimensions are appropriate for book covers (e.g., 1024x1536 for 2:3 aspect ratio)
3. The build script will automatically copy it to the correct location

### Metadata

Edit `epub-book/book.toml` to customize:
- Title and description
- Author information
- Publication date
- Subject categories
- Language settings

## 📊 File Structure

```
epub-book/
├── book.toml                    # EPUB configuration
├── src/                         # Book content
│   ├── SUMMARY.md              # Table of contents
│   ├── introduction.md         # Introduction
│   ├── chapter-*.md            # Main chapters
│   └── appendix-*.md           # Appendices
├── theme/                      # EPUB styling
│   ├── epub.css               # Stylesheet
│   └── cover.png              # Cover image
└── book/                       # Generated output
    └── *.epub                  # Final EPUB file
```

## 🔧 Troubleshooting

### Common Issues

**1. mdbook-epub not found**
```bash
cargo install mdbook-epub
```

**2. Build fails with "renderer not found"**
```bash
# Ensure mdbook-epub is properly installed
mdbook --help | grep epub
```

**3. Cover image not showing**
- Check that `theme/cover.png` exists
- Verify the path in `book.toml` is correct
- Ensure image dimensions are reasonable (600x800 recommended)

**4. Code blocks not formatted properly**
- Ensure proper Markdown syntax with language tags
- Check that code blocks are properly indented
- Verify CSS is loading correctly

**5. Large file size**
- Optimize images before including
- Remove unnecessary styling
- Consider splitting very long chapters

### Getting Help

- **mdBook Documentation**: https://rust-lang.github.io/mdBook/
- **mdbook-epub Plugin**: https://github.com/Michael-F-Bryan/mdbook-epub
- **EPUB Specification**: https://www.w3.org/publishing/epub32/

## 🚀 Distribution

### Sharing the EPUB

The generated EPUB can be:
- **Shared directly** - Send the .epub file to others
- **Uploaded to cloud storage** - Google Drive, Dropbox, etc.
- **Added to library management** - Calibre, Plex, etc.
- **Converted to other formats** - PDF, MOBI, etc.

### File Size

The typical EPUB size is:
- **Text content**: ~500KB
- **With images**: ~1-2MB
- **Compressed**: Very efficient for distribution

## 📝 License

This educational content is provided as part of the Future Trait Tutorial project. The EPUB format and styling are optimized for educational use and can be freely shared for learning purposes.

---

## Quick Commands Reference

```bash
# Build EPUB (automated)
./build_epub.sh

# Build EPUB (manual)
cd epub-book && mdbook build

# Install dependencies
cargo install mdbook mdbook-epub

# Prepare cover and theme files (done automatically by build script)

# Find generated EPUB
find epub-book/book -name "*.epub"
```

**Happy reading! 📖** 