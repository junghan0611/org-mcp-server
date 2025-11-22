#!/usr/bin/env bash
# org-mcp-server NixOS Build Script
# Usage: ./build-nixos.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULT_BIN_SERVER="$SCRIPT_DIR/result/bin/org-mcp-server"
RESULT_BIN_CLI="$SCRIPT_DIR/result/bin/org-cli"
TARGET_DIR="$HOME/.local/bin"
TARGET_LINK_SERVER="$TARGET_DIR/org-mcp-server"
TARGET_LINK_CLI="$TARGET_DIR/org-cli"

echo "🔨 Building org-mcp-server for NixOS..."
cd "$SCRIPT_DIR"

# Run nix build
if nix build .#default -L; then
    echo "✅ Build successful!"

    # Verify binaries
    if [ -f "$RESULT_BIN_SERVER" ] && [ -f "$RESULT_BIN_CLI" ]; then
        VERSION=$("$RESULT_BIN_SERVER" --version)
        echo "📦 Version: $VERSION"

        # Create target directory
        mkdir -p "$TARGET_DIR"

        # Create symlinks
        ln -sf "$RESULT_BIN_SERVER" "$TARGET_LINK_SERVER"
        ln -sf "$RESULT_BIN_CLI" "$TARGET_LINK_CLI"
        echo "🔗 Symlinks created:"
        echo "   $TARGET_LINK_SERVER -> $RESULT_BIN_SERVER"
        echo "   $TARGET_LINK_CLI -> $RESULT_BIN_CLI"

        # Verify installation
        if command -v org-mcp-server &> /dev/null; then
            ACTIVE_VERSION=$(org-mcp-server --version)
            echo "✨ Active version: $ACTIVE_VERSION"
        else
            echo "⚠️  Warning: 'org-mcp-server' not in PATH"
            echo "   Add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
    else
        echo "❌ Error: Binaries not found at:"
        echo "   - $RESULT_BIN_SERVER"
        echo "   - $RESULT_BIN_CLI"
        exit 1
    fi
else
    echo "❌ Build failed!"
    exit 1
fi
