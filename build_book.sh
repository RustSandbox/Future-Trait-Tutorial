#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored messages
print_message() {
    echo -e "${BLUE}[Rust Async Book]${NC} $1"
}

# Function to print success messages
print_success() {
    echo -e "${GREEN}[Success]${NC} $1"
}

# Function to print error messages
print_error() {
    echo -e "${RED}[Error]${NC} $1"
}

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    print_error "mdBook is not installed. Installing..."
    cargo install mdbook
    if [ $? -eq 0 ]; then
        print_success "mdBook installed successfully"
    else
        print_error "Failed to install mdBook"
        exit 1
    fi
fi

# Function to build the book
build_book() {
    print_message "Building the book..."
    cd book
    mdbook build
    if [ $? -eq 0 ]; then
        print_success "Book built successfully"
    else
        print_error "Failed to build book"
        exit 1
    fi
    cd ..
}

# Function to serve the book
serve_book() {
    print_message "Starting development server..."
    cd book
    mdbook serve
    if [ $? -eq 0 ]; then
        print_success "Development server started"
    else
        print_error "Failed to start development server"
        exit 1
    fi
    cd ..
}

# Function to clean build artifacts
clean() {
    print_message "Cleaning build artifacts..."
    cd book
    rm -rf book
    if [ $? -eq 0 ]; then
        print_success "Build artifacts cleaned successfully"
    else
        print_error "Failed to clean build artifacts"
        exit 1
    fi
    cd ..
}

# Parse command line arguments
case "$1" in
    "build")
        build_book
        ;;
    "serve")
        serve_book
        ;;
    "clean")
        clean
        ;;
    *)
        echo "Usage: $0 {build|serve|clean}"
        echo "  build  - Build the book"
        echo "  serve  - Start development server"
        echo "  clean  - Clean build artifacts"
        exit 1
        ;;
esac 