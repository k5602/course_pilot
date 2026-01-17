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

case $OS in
    ubuntu|debian|pop|mint|kali)
        echo -e "Installing dependencies via ${GREEN}apt${NC}..."
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            pkg-config \
            libssl-dev \
            libgtk-3-dev \
            libwebkit2gtk-4.1-dev \
            libsqlite3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            libxdo-dev \
            cmake
        ;;
    arch|manjaro|endeavouros)
        echo -e "Installing dependencies via ${GREEN}pacman${NC}..."
        sudo pacman -S --needed --noconfirm \
            base-devel \
            pkgconf \
            openssl \
            gtk3 \
            webkit2gtk-4.1 \
            sqlite \
            libayatana-appindicator \
            librsvg \
            xdotool \
            cmake
        ;;
    fedora)
        echo -e "Installing dependencies via ${GREEN}dnf${NC}..."
        sudo dnf groupinstall -y "Development Tools" "C Development Tools and Libraries"
        sudo dnf install -y \
            pkgconf-pkg-config \
            openssl-devel \
            gtk3-devel \
            webkit2gtk4.1-devel \
            sqlite-devel \
            libayatana-appindicator-devel \
            librsvg2-devel \
            libxdo-devel \
            cmake
        ;;
    *)
        echo -e "Unsupported distribution: ${BLUE}$OS${NC}"
        echo "Please manually install the following development headers:"
        echo "- GTK3"
        echo "- WebKit2GTK (4.1 preferred)"
        echo "- SQLite3"
        echo "- OpenSSL"
        echo "- libayatana-appindicator"
        exit 1
        ;;
esac

echo -e "${GREEN}✅ System dependencies installed successfully!${NC}"
echo -e "Next steps:"
echo "1. Ensure you have Rust installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "2. Install Dioxus CLI: cargo install dioxus-cli"
echo "3. Run the app: dx serve"
