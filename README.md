# Traverse LSP Server

Language Server Protocol implementation for Solidity that generates call graphs and sequence diagrams at workspace level.

## Build

```bash
cargo build --release
```

The binary will be at `target/release/traverse-lsp`

## Architecture

The LSP server communicates via stdio and operates exclusively at workspace level, analyzing entire Solidity projects rather than individual files. It uses a background worker thread for diagram generation to keep the main LSP message loop responsive.

## LSP Capabilities

### Workspace Commands

The server implements the following workspace-level commands via `workspace/executeCommand`:

- `traverse.generateCallGraph.workspace` - Generate call graph for all contracts
- `traverse.generateSequenceDiagram.workspace` - Create sequence diagrams  
- `traverse.generateAll.workspace` - Generate all diagram types
- `traverse.analyzeStorage.workspace` - Analyze storage layout

### Output

All diagrams are generated in:
- **DOT format** for call graphs (GraphViz compatible)
- **Mermaid format** for sequence diagrams
- **Markdown** for storage analysis

## IDE Integration

### VS Code

A fully-featured extension is available in the `vscode-extension/` directory. See [vscode-extension/README.md](../vscode-extension/README.md) for installation instructions.

### Neovim

TODO: Add Neovim LSP configuration example

### Other IDEs

Any IDE with LSP support can use this server. Configure your IDE to:
1. Run `traverse-lsp` as the language server for Solidity files
2. Use stdio for communication
3. Send workspace commands for diagram generation

## Configuration

Environment variables:
- `RUST_LOG=debug` - Enable debug logging
- `TRAVERSE_LSP_TRACE=verbose` - Trace LSP messages

## Testing

```bash
cargo test
```

## License

MIT