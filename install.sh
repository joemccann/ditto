#!/bin/bash
set -e

cargo build --release

BINARY="$(cd "$(dirname "$0")" && pwd)/target/release/ditto"

if [ ! -f "$BINARY" ]; then
  echo "ERROR: build failed, binary not found at $BINARY"
  exit 1
fi

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

ln -sf "$BINARY" "$INSTALL_DIR/ditto"

echo "✓ ditto installed → $(which ditto)"
ditto --help | head -3
