# Traverse LSP Server

[![Version](https://img.shields.io/badge/version-0.1.4-blue)](https://github.com/calltrace/traverse-lsp/releases)
[![Traverse](https://img.shields.io/badge/traverse-0.1.4-green)](https://github.com/calltrace/traverse)

Language Server Protocol implementation for [Traverse](https://github.com/calltrace/traverse) analysis engine v0.1.4, providing call graph and sequence diagram generation for Solidity smart contracts.

## Features

- **Automatic Mermaid Chunking**: Large sequence diagrams are automatically split into manageable chunks
- **Multi-file Analysis**: Analyzes entire Solidity workspaces, not just single files
- **Background Processing**: Diagram generation runs in separate thread to keep UI responsive
- **Multiple Output Formats**: DOT, Mermaid, and Markdown formats

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

| Command | Description | Parameters |
|---------|-------------|------------|
| `traverse.generateCallGraph.workspace` | Generate call graph for all contracts | `workspace_folder`: string |
| `traverse.generateSequenceDiagram.workspace` | Create sequence diagrams | `workspace_folder`: string<br>`no_chunk`: boolean (optional, default: false) |
| `traverse.generateAll.workspace` | Generate all diagram types | `workspace_folder`: string |
| `traverse.analyzeStorage.workspace` | Analyze storage layout | `workspace_folder`: string |

#### Example Command Request

```json
{
  "command": "traverse.generateSequenceDiagram.workspace",
  "arguments": [{
    "workspace_folder": "/path/to/project",
    "no_chunk": false
  }]
}
```

### Output

All diagrams are generated in:
- **DOT format** for call graphs (GraphViz compatible)
- **Mermaid format** for sequence diagrams (with automatic chunking for large diagrams)
- **Markdown** for storage analysis

#### Mermaid Chunking

Large sequence diagrams are automatically split into manageable chunks (default: 400 lines per chunk) to prevent rendering issues. This behavior can be controlled:

- **Default**: Chunking is enabled automatically for large diagrams
- **Disable chunking**: Pass `no_chunk: true` in command arguments
- **Output**: Chunks are saved to `./mermaid-chunks/` directory with an index file

## IDE Integration

### VS Code

A fully-featured extension is available at https://github.com/calltrace/traverse-vscode

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