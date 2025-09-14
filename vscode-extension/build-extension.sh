#!/bin/bash

# Build script for Traverse LSP VSCode Extension with ARM64 macOS binary
# This script builds the extension with bundled LSP server binary

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
EXTENSION_DIR="$(pwd)"
LSP_DIR="../traverse-lsp"
BINARY_NAME="traverse-lsp-darwin-aarch64"
VERSION=$(grep '"version"' package.json | cut -d '"' -f 4)

echo -e "${GREEN}=== Traverse LSP Extension Build Script ===${NC}"
echo "Version: $VERSION"
echo "Platform: macOS ARM64"
echo ""

# Step 1: Pre-flight checks
echo -e "${YELLOW}Step 1: Pre-flight checks${NC}"

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "src" ]; then
    echo -e "${RED}Error: Not in vscode-extension directory${NC}"
    exit 1
fi

# Check required tools
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Error: cargo not found${NC}"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo -e "${RED}Error: npm not found${NC}"; exit 1; }
command -v node >/dev/null 2>&1 || { echo -e "${RED}Error: node not found${NC}"; exit 1; }

# Check platform
if [[ "$(uname)" != "Darwin" ]]; then
    echo -e "${YELLOW}Warning: Not on macOS, build may not work correctly${NC}"
fi

if [[ "$(uname -m)" != "arm64" ]]; then
    echo -e "${YELLOW}Warning: Not on ARM64, building for ARM64 anyway${NC}"
fi

echo -e "${GREEN}✓ Pre-flight checks passed${NC}"
echo ""

# Step 2: Build Rust LSP Server
echo -e "${YELLOW}Step 2: Building Rust LSP server${NC}"

cd "$LSP_DIR"
cargo build --release

if [ ! -f "target/release/traverse-lsp" ]; then
    echo -e "${RED}Error: Failed to build LSP server${NC}"
    exit 1
fi

# Strip debug symbols to reduce size
echo "Stripping debug symbols..."
strip target/release/traverse-lsp

# Get binary size
BINARY_SIZE=$(ls -lh target/release/traverse-lsp | awk '{print $5}')
echo -e "${GREEN}✓ LSP server built successfully (${BINARY_SIZE})${NC}"
echo ""

# Step 3: Prepare extension directory
echo -e "${YELLOW}Step 3: Preparing extension directory${NC}"

cd "$EXTENSION_DIR"

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf out/ server/ *.vsix

# Create server binary directory
mkdir -p server/bin

# Copy and rename binary
cp "$LSP_DIR/target/release/traverse-lsp" "server/bin/$BINARY_NAME"
chmod +x "server/bin/$BINARY_NAME"

echo -e "${GREEN}✓ Binary copied to extension${NC}"
echo ""

# Step 4: Update package.json to include server files
echo -e "${YELLOW}Step 4: Updating package configuration${NC}"

# Create or update .vscodeignore to NOT exclude server directory
cat > .vscodeignore << 'EOF'
.vscode/**
.vscode-test/**
src/**
.gitignore
.yarnrc
vsc-extension-quickstart.md
**/tsconfig.json
**/.eslintrc.json
**/*.map
**/*.ts
node_modules/**
!server/bin/**
EOF

echo -e "${GREEN}✓ Package configuration updated${NC}"
echo ""

# Step 5: Build extension
echo -e "${YELLOW}Step 5: Building extension${NC}"

# Install dependencies
echo "Installing dependencies..."
npm install

# Compile TypeScript
echo "Compiling TypeScript..."
npm run compile

if [ ! -d "out" ]; then
    echo -e "${RED}Error: TypeScript compilation failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Extension built successfully${NC}"
echo ""

# Step 6: Package extension
echo -e "${YELLOW}Step 6: Packaging extension${NC}"

# Check if vsce is installed
if ! command -v vsce &> /dev/null; then
    echo "Installing vsce..."
    npm install -g @vscode/vsce
fi

# Package the extension
echo "Creating .vsix package..."
vsce package --no-dependencies

# Find the generated .vsix file
VSIX_FILE=$(ls *.vsix 2>/dev/null | head -n 1)

if [ -z "$VSIX_FILE" ]; then
    echo -e "${RED}Error: Failed to create .vsix package${NC}"
    exit 1
fi

# Get package size
PACKAGE_SIZE=$(ls -lh "$VSIX_FILE" | awk '{print $5}')

echo -e "${GREEN}✓ Extension packaged successfully${NC}"
echo ""

# Step 7: Summary
echo -e "${GREEN}=== Build Complete ===${NC}"
echo "Package: $VSIX_FILE"
echo "Size: $PACKAGE_SIZE"
echo "Binary: $BINARY_NAME (${BINARY_SIZE})"
echo ""
echo "To install the extension, run:"
echo -e "${YELLOW}  code --install-extension $VSIX_FILE${NC}"
echo ""
echo "To test in a new VSCode window:"
echo -e "${YELLOW}  code --new-window --install-extension $VSIX_FILE${NC}"

# Optional: Generate build info
cat > build-info.txt << EOF
Traverse LSP Extension Build Info
==================================
Date: $(date)
Version: $VERSION
Platform: macOS ARM64
Binary: $BINARY_NAME
Binary Size: $BINARY_SIZE
Package: $VSIX_FILE
Package Size: $PACKAGE_SIZE
Git Commit: $(cd "$LSP_DIR" && git rev-parse --short HEAD 2>/dev/null || echo "unknown")
EOF

echo ""
echo "Build info saved to build-info.txt"