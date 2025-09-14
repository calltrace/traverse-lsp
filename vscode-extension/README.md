# Traverse VSCode Extension

This is the Visual Studio Code extension for Traverse, which provides Solidity smart contract visualization with automatic call graph and sequence diagram generation. The extension uses the Traverse Rust LSP server to analyze and visualize your Solidity code.

## Install

```bash
code --install-extension traverse-lsp-0.0.1.vsix
```

Or build from source:

```bash
./build-extension.sh && code --install-extension traverse-lsp-0.0.1.vsix
```

## Usage

1. Open a Solidity project
2. Right-click any folder → Select "Traverse" commands
3. Diagrams are saved to `traverse-output/` in your workspace

**Available Commands** (Cmd+Shift+P):

- `Generate Call Graph` - Visualize function relationships
- `Generate Sequence Diagram` - Show execution flow
- `Analyze Storage` - Map storage variables
- `Generate All Diagrams` - Everything at once

## Features

### Call Graph Generation

Generates DOT format graphs showing all function calls and relationships.

### Sequence Diagrams

Creates Mermaid sequence diagrams for contract interactions.

### Storage Analysis

Maps all storage variables and their access patterns across functions.

## Troubleshooting

**Extension not activating?**

- Ensure you have `.sol` files in your workspace
- Check Output panel → "Traverse LSP" for errors

**No diagrams generated?**

- Verify Solidity syntax is valid
- Check `traverse-output/` folder in workspace

**Server crashes?**

- Run `Traverse: Restart Language Server` from command palette
