#!/usr/bin/env sh
# alf installer — downloads the right binary from GitHub Releases.
# Usage: curl -sSfL https://raw.githubusercontent.com/sancara/alf/main/install.sh | sh

set -e

REPO="sancara/alf"
BIN_NAME="alf"
INSTALL_DIR="${ALF_INSTALL_DIR:-/usr/local/bin}"

# ---------- detect platform ----------

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  ARTIFACT="alf-linux-x86_64" ;;
      *)
        echo "Unsupported Linux architecture: $ARCH"
        echo "Please build from source: cargo install --git https://github.com/$REPO"
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      arm64)   ARTIFACT="alf-macos-arm64" ;;
      x86_64)  ARTIFACT="alf-macos-x86_64" ;;
      *)
        echo "Unsupported macOS architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "On Windows, download alf-windows-x86_64.exe from:"
    echo "  https://github.com/$REPO/releases/latest"
    exit 1
    ;;
esac

# ---------- resolve latest version ----------

LATEST_URL="https://api.github.com/repos/$REPO/releases/latest"
TAG=$(curl -sSfL "$LATEST_URL" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$TAG" ]; then
  echo "Could not determine the latest release tag."
  echo "Check https://github.com/$REPO/releases for the latest version."
  exit 1
fi

echo "Installing alf $TAG..."

# ---------- download ----------

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$TAG/$ARTIFACT"
TMP="$(mktemp)"
curl -sSfL "$DOWNLOAD_URL" -o "$TMP"
chmod +x "$TMP"

# ---------- install ----------

# Try the install dir; fall back to ~/.local/bin if we don't have write access.
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP" "$INSTALL_DIR/$BIN_NAME"
  echo "Installed to $INSTALL_DIR/$BIN_NAME"
else
  FALLBACK="$HOME/.local/bin"
  mkdir -p "$FALLBACK"
  mv "$TMP" "$FALLBACK/$BIN_NAME"
  echo "Installed to $FALLBACK/$BIN_NAME"
  echo ""
  echo "Make sure $FALLBACK is in your PATH:"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
alf --version 2>/dev/null || "$INSTALL_DIR/$BIN_NAME" --version 2>/dev/null || true
echo ""
echo "All done. Run 'alf --help' to get started."
