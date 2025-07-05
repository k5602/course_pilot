#!/bin/bash

# Course Pilot Run Script
# Simple script to build and run the Course Pilot desktop application

set -e  # Exit on any error

echo "🚀 Starting Course Pilot..."
echo "========================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo not found. Please install Rust first:"
    echo "   https://rustup.rs/"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the course_pilot directory"
    exit 1
fi

# Build the application
echo "🔨 Building Course Pilot..."
cargo build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build completed successfully!"
    echo ""
    echo "🎯 Launching Course Pilot..."
    echo "   Close the application window to stop."
    echo ""

    # Run the application
    cargo run --release
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi
