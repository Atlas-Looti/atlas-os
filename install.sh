#!/bin/sh
# Atlas OS installer — downloads the latest release binary for your platform.
# Usage: curl -fsSL https://raw.githubusercontent.com/nonom/atlas-os/main/install.sh | sh

set -e

REPO="Atlas-Looti/atlas-os"
BINARY="atlas"
INSTALL_DIR="${ATLAS_INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { printf "${GREEN}▸${NC} %s\n" "$1"; }
warn()  { printf "${YELLOW}▸${NC} %s\n" "$1"; }
error() { printf "${RED}✗${NC} %s\n" "$1" >&2; exit 1; }

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux*)  OS_NAME="linux" ;;
  Darwin*) OS_NAME="macos" ;;
  *)       error "Unsupported OS: $OS. Use Linux or macOS." ;;
esac

# Detect arch
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)  ARCH_NAME="x86_64" ;;
  aarch64|arm64) ARCH_NAME="aarch64" ;;
  *)             error "Unsupported architecture: $ARCH" ;;
esac

PLATFORM="${OS_NAME}-${ARCH_NAME}"
ARCHIVE="atlas-${PLATFORM}.tar.gz"

info "Detected platform: ${PLATFORM}"

# Get latest release tag
info "Fetching latest release..."
TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
[ -z "$TAG" ] && error "Could not determine latest release. Check https://github.com/${REPO}/releases"

info "Latest release: ${TAG}"

# Download
URL="https://github.com/${REPO}/releases/download/${TAG}/${ARCHIVE}"
CHECKSUM_URL="${URL}.sha256"
TMPDIR=$(mktemp -d)

info "Downloading ${ARCHIVE}..."
curl -fsSL -o "${TMPDIR}/${ARCHIVE}" "$URL" || error "Download failed. Binary may not exist for ${PLATFORM}."
curl -fsSL -o "${TMPDIR}/${ARCHIVE}.sha256" "$CHECKSUM_URL" 2>/dev/null || warn "Checksum not available, skipping verification."

# Verify checksum
if [ -f "${TMPDIR}/${ARCHIVE}.sha256" ]; then
  info "Verifying checksum..."
  cd "$TMPDIR"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c "${ARCHIVE}.sha256" || error "Checksum verification failed!"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "${ARCHIVE}.sha256" || error "Checksum verification failed!"
  else
    warn "No sha256sum or shasum found, skipping verification."
  fi
  cd - >/dev/null
fi

# Extract and install
info "Installing to ${INSTALL_DIR}..."
tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
  sudo mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi
chmod +x "${INSTALL_DIR}/${BINARY}"

# Cleanup
rm -rf "$TMPDIR"

# Verify
if command -v atlas >/dev/null 2>&1; then
  VERSION=$(atlas --version 2>/dev/null || echo "unknown")
  info "✓ Atlas OS installed successfully! (${VERSION})"
  info "Run 'atlas --help' to get started."
else
  warn "Installed to ${INSTALL_DIR}/${BINARY} but it's not in PATH."
  warn "Add ${INSTALL_DIR} to your PATH, or run: ${INSTALL_DIR}/${BINARY} --help"
fi
