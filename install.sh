#!/bin/bash
set -e

cargo build --release

BINARY="$(cd "$(dirname "$0")" && pwd)/target/release/ditto"

if [ ! -f "$BINARY" ]; then
  echo "ERROR: build failed, binary not found at $BINARY"
  exit 1
fi

INSTALL_DIR="/usr/local/bin"

if [ -w "$INSTALL_DIR" ]; then
  ln -sf "$BINARY" "$INSTALL_DIR/ditto"
else
  sudo ln -sf "$BINARY" "$INSTALL_DIR/ditto"
fi

echo "✓ ditto installed → $(which ditto)"
ditto --help | head -3
