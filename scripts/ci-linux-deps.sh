#!/usr/bin/env bash
# Tauri Linux build dependencies for GitHub Actions.
# Usage: ci-linux-deps.sh [webkit-pkg] [appindicator-pkg]
# Defaults match Ubuntu 24.04 (ubuntu-latest).
set -euo pipefail

WEBKIT_PKG="${1:-libwebkit2gtk-4.1-dev}"
INDICATOR_PKG="${2:-libayatana-appindicator3-dev}"

sudo apt-get update
sudo apt-get install -y \
  "$WEBKIT_PKG" \
  "$INDICATOR_PKG" \
  build-essential \
  libssl-dev \
  libxdo-dev \
  librsvg2-dev \
  libglib2.0-dev \
  libgtk-3-dev \
  patchelf \
  pkg-config \
  xdg-utils

pkg-config --modversion glib-2.0
