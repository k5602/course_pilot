#!/bin/bash

# Course Pilot Launcher Script
# This script checks for required system dependencies before launching the application.
# Supported: Debian/Ubuntu-based, Fedora, and Arch-based distributions.

set -e

APP_NAME="course_pilot"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$SCRIPT_DIR/$APP_NAME"

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

    # Check for WebKit2GTK (Common dependency for Dioxus Desktop)
    if ! ldconfig -p | grep -q "libwebkit2gtk-4.0.so\|libwebkit2gtk-4.1.so"; then
        log_warn "WebKit2GTK not found. It is required for the UI."
        missing_deps=true
    fi

    # Check for SQLite3
    if ! ldconfig -p | grep -q "libsqlite3.so"; then
        log_warn "SQLite3 not found. It is required for the database."
        missing_deps=true
    fi

    # Check for GStreamer (codec support for WebKit video playback)
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
                            libwebkit2gtk-4.1-0 \
                            libgtk-3-0 \
                            libsqlite3-0 \
                            libayatana-appindicator3-1 \
                            librsvg2-2 \
                            libxdo3 \
                            gstreamer1.0-libav \
                            gstreamer1.0-plugins-base \
                            gstreamer1.0-plugins-good
                    else
                        log_err "Non-interactive sudo is not available."
                        log_info "Run: sudo apt update && sudo apt install -y libwebkit2gtk-4.1-0 libgtk-3-0 libsqlite3-0 libayatana-appindicator3-1 librsvg2-2 libxdo3 gstreamer1.0-libav gstreamer1.0-plugins-base gstreamer1.0-plugins-good"
                        exit 1
                    fi
                    ;;
                arch|manjaro|endeavouros)
                    log_info "Detected Arch-based system."
                    if sudo -n true 2>/dev/null; then
                        log_info "Installing dependencies non-interactively..."
                        sudo -n pacman -S --needed --noconfirm \
                            webkit2gtk-4.1 \
                            gtk3 \
                            sqlite \
                            libayatana-appindicator \
                            librsvg \
                            xdotool \
                            gstreamer \
                            gst-plugins-base \
                            gst-plugins-good \
                            gst-libav
                    else
                        log_err "Non-interactive sudo is not available."
                        log_info "Run: sudo pacman -S --needed webkit2gtk-4.1 gtk3 sqlite libayatana-appindicator librsvg xdotool gstreamer gst-plugins-base gst-plugins-good gst-libav"
                        exit 1
                    fi
                    ;;
                fedora)
                    log_info "Detected Fedora."
                    if sudo -n true 2>/dev/null; then
                        log_info "Installing dependencies non-interactively..."
                        sudo -n dnf install -y \
                            webkit2gtk4.1 \
                            gtk3 \
                            sqlite \
                            libayatana-appindicator \
                            librsvg2 \
                            libxdo \
                            gstreamer1-libav \
                            gstreamer1-plugins-base \
                            gstreamer1-plugins-good
                    else
                        log_err "Non-interactive sudo is not available."
                        log_info "Run: sudo dnf install -y webkit2gtk4.1 gtk3 sqlite libayatana-appindicator librsvg2 libxdo gstreamer1-libav gstreamer1-plugins-base gstreamer1-plugins-good"
                        exit 1
                    fi
                    ;;
                *)
                    log_err "Unsupported or unknown distribution: $ID"
                    log_err "Please manually install webkit2gtk (4.0 or 4.1), gtk3, and sqlite3."
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
    # Fallback to looking in the current directory if script is moved
    BIN_PATH="./$APP_NAME"
    if [ ! -f "$BIN_PATH" ]; then
        log_err "Binary '$APP_NAME' not found in $SCRIPT_DIR or current directory."
        exit 1
    fi
fi

# 3. Ensure executable permissions
chmod +x "$BIN_PATH"

# 4. Launch the application
log_info "Launching Course Pilot..."
exec "$BIN_PATH" "$@"
