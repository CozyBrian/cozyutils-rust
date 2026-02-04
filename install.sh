#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Building cozyutils in release mode...${NC}"
RUSTFLAGS="-C link-arg=-s" cargo build --release

# Define installation directory
INSTALL_DIR="$HOME/.cozyutils/bin"
BINARY_SOURCE="target/release/cozyutils"
BINARY_DEST="$INSTALL_DIR/cozyutils"

# Create directory if it doesn't exist
echo -e "${BLUE}Creating installation directory: $INSTALL_DIR${NC}"
mkdir -p "$INSTALL_DIR"

# Move the binary
if [ -f "$BINARY_SOURCE" ]; then
  echo -e "${BLUE}Installing binary to $BINARY_DEST${NC}"
  cp "$BINARY_SOURCE" "$BINARY_DEST"
  chmod +x "$BINARY_DEST"
else
  echo -e "\033[0;31mError: Build failed, binary not found at $BINARY_SOURCE\033[0m"
  exit 1
fi

# Add to PATH if not already present
PATH_ENTRY="export PATH=\"\$HOME/.cozyutils:\$PATH\""
UPDATED=false

# Check if INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  SHELL_CONFIGS=("$HOME/.zshrc" "$HOME/.bashrc")

  for CONFIG in "${SHELL_CONFIGS[@]}"; do
    if [ -f "$CONFIG" ]; then
      if ! grep -q ".cozyutils" "$CONFIG"; then
        echo -e "${BLUE}Adding $INSTALL_DIR to $CONFIG${NC}"
        echo "" >>"$CONFIG"
        echo "# cozyutils path" >>"$CONFIG"
        echo "$PATH_ENTRY" >>"$CONFIG"
        UPDATED=true
      fi
    fi
  done
fi

echo -e "${GREEN}Installation complete!${NC}"

if [ "$UPDATED" = true ]; then
  echo -e "Please restart your terminal or run: ${BLUE}source ~/.zshrc${NC} (or ~/.bashrc)"
else
  echo -e "Binary updated at $BINARY_DEST"
fi

echo -e "Try running: ${GREEN}cozyutils -h${NC}"
