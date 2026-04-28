#!/bin/bash

# Course Pilot Developer Setup Script (Linux)
# This script installs the system dependencies required to build and run Course Pilot.

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Course Pilot | Linux Dependency Setup${NC}"

if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo "Error: Could not detect OS distribution via /etc/os-release."
    exit 1
fi

echo -e "Detected OS: ${GREEN}$OS${NC}"

if ! sudo -n true 2>/dev/null; then
    echo "Non-interactive sudo is required to install dependencies."
    echo "Configure passwordless sudo for this user or run the install commands manually."
    exit 1
fi

case $OS in
    ubuntu|debian|pop|mint|kali)
        echo -e "Installing dependencies via ${GREEN}apt${NC}..."
        sudo -n apt-get update
        sudo -n apt-get install -y \
            build-essential \
            pkg-config \
            libssl-dev \
            libgtk-4-dev \
            libadwaita-1-dev \
            libgstreamer1.0-dev \
            libgstreamer-plugins-base1.0-dev \
            libgraphene-1.0-dev \
            libsqlite3-dev \
            gstreamer1.0-plugins-good \
            cmake
        ;;
    arch|manjaro|endeavouros)
        echo -e "Installing dependencies via ${GREEN}pacman${NC}..."
        sudo -n pacman -S --needed --noconfirm \
            base-devel \
            pkgconf \
            openssl \
            gtk4 \
            libadwaita \
            gstreamer \
            gst-plugins-base \
            gst-plugins-good \
            graphene \
            sqlite \
            cmake
        ;;
    fedora)
        echo -e "Installing dependencies via ${GREEN}dnf${NC}..."
        sudo -n dnf groupinstall -y "Development Tools" "C Development Tools and Libraries"
        sudo -n dnf install -y \
            pkgconf-pkg-config \
            openssl-devel \
            gtk4-devel \
            libadwaita-devel \
            gstreamer1-devel \
            gstreamer1-plugins-base-devel \
            graphene-devel \
            sqlite-devel \
            cmake
        ;;
    *)
        echo -e "Unsupported distribution: ${BLUE}$OS${NC}"
        echo "Please manually install the following development headers:"
        echo "- GTK4"
        echo "- libadwaita"
        echo "- GStreamer (base + plugins)"
        echo "- libgraphene"
        echo "- SQLite3"
        echo "- OpenSSL"
        exit 1
        ;;
esac

echo -e "${GREEN}System dependencies installed successfully!${NC}"
echo -e "Next steps:"
echo "1. Ensure you have Rust installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "2. Run the app: cargo run --release"
