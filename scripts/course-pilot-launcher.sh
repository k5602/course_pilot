#!/bin/bash

# Course Pilot Launcher Script
# This script checks for required system dependencies before launching the application.
# Supported: Debian/Ubuntu-based, Fedora, and Arch-based distributions.

set -e

APP_NAME="course_pilot"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$SCRIPT_DIR/bin/$APP_NAME"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_err() { echo -e "${RED}[ERROR]${NC} $1"; }

check_dependencies() {
    local missing_deps=false

    # Check for GTK4
    if ! ldconfig -p | grep -q "libgtk-4.so"; then
        log_warn "GTK4 not found. It is required for the UI."
        missing_deps=true
    fi

    # Check for libadwaita
    if ! ldconfig -p | grep -q "libadwaita-1.so"; then
        log_warn "libadwaita not found. It is required for the UI."
        missing_deps=true
    fi

    # Check for SQLite3
    if ! ldconfig -p | grep -q "libsqlite3.so"; then
        log_warn "SQLite3 not found. It is required for the database."
        missing_deps=true
    fi

    # Check for GStreamer
    if ! command -v gst-inspect-1.0 >/dev/null 2>&1; then
        log_warn "GStreamer not found. It is required for video playback."
        missing_deps=true
    fi

    if [ "$missing_deps" = true ]; then
        log_info "Attempting to identify distribution and suggest install command..."

        if [ -f /etc/os-release ]; then
            . /etc/os-release
            case $ID in
                ubuntu|debian|pop|mint|kali)
                    log_info "Detected Debian-based system."
                    if sudo -n true 2>/dev/null; then
                        log_info "Installing dependencies non-interactively..."
                        sudo -n apt update
                        sudo -n apt install -y \
                            libgtk-4-1 \
                            libadwaita-1-0 \
                            libsqlite3-0 \
                            gstreamer1.0-plugins-base \
                            gstreamer1.0-plugins-good
                    else
                        log_err "Non-interactive sudo is not available."
                        log_info "Run: sudo apt update && sudo apt install -y libgtk-4-1 libadwaita-1-0 libsqlite3-0 gstreamer1.0-plugins-base gstreamer1.0-plugins-good"
                        exit 1
                    fi
                    ;;
                arch|manjaro|endeavouros)
                    log_info "Detected Arch-based system."
                    if sudo -n true 2>/dev/null; then
                        log_info "Installing dependencies non-interactively..."
                        sudo -n pacman -S --needed --noconfirm \
                            gtk4 \
                            libadwaita \
                            sqlite \
                            gstreamer \
                            gst-plugins-base \
                            gst-plugins-good
                    else
                        log_err "Non-interactive sudo is not available."
                        log_info "Run: sudo pacman -S --needed gtk4 libadwaita sqlite gstreamer gst-plugins-base gst-plugins-good"
                        exit 1
                    fi
                    ;;
                fedora)
                    log_info "Detected Fedora."
                    if sudo -n true 2>/dev/null; then
                        log_info "Installing dependencies non-interactively..."
                        sudo -n dnf install -y \
                            gtk4 \
                            libadwaita \
                            sqlite \
                            gstreamer1-plugins-base \
                            gstreamer1-plugins-good
                    else
                        log_err "Non-interactive sudo is not available."
                        log_info "Run: sudo dnf install -y gtk4 libadwaita sqlite gstreamer1-plugins-base gstreamer1-plugins-good"
                        exit 1
                    fi
                    ;;
                *)
                    log_err "Unsupported or unknown distribution: $ID"
                    log_err "Please manually install gtk4, libadwaita, sqlite3, and gstreamer."
                    exit 1
                    ;;
            esac
        else
            log_err "Could not detect OS distribution."
            exit 1
        fi
    fi
}

# 1. Check dependencies
check_dependencies

# 2. Check if binary exists
if [ ! -f "$BIN_PATH" ]; then
    BIN_PATH="$SCRIPT_DIR/$APP_NAME"
    if [ ! -f "$BIN_PATH" ]; then
        BIN_PATH="./$APP_NAME"
        if [ ! -f "$BIN_PATH" ]; then
            log_err "Binary '$APP_NAME' not found in $SCRIPT_DIR/bin, $SCRIPT_DIR, or current directory."
            exit 1
        fi
    fi
fi

# 3. Ensure executable permissions
chmod +x "$BIN_PATH"

# 4. Launch the application
log_info "Launching Course Pilot..."
exec "$BIN_PATH" "$@"
