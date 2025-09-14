import * as path from 'path';
import * as vscode from 'vscode';
import * as fs from 'fs';
import * as os from 'os';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

/**
 * Determines the platform-specific binary name
 */
function getServerBinaryName(): string {
    const platform = os.platform();
    const arch = os.arch();
    
    // Binary naming convention: traverse-lsp-{platform}-{arch}
    const platformMap: Record<string, string> = {
        'darwin': 'darwin',
        'linux': 'linux',
        'win32': 'windows'
    };
    
    const archMap: Record<string, string> = {
        'x64': 'x86_64',
        'arm64': 'aarch64',
        'arm': 'arm'
    };
    
    const platformName = platformMap[platform] || platform;
    const archName = archMap[arch] || arch;
    
    const binaryName = `traverse-lsp-${platformName}-${archName}`;
    return platform === 'win32' ? `${binaryName}.exe` : binaryName;
}

/**
 * Gets the path to the bundled server binary
 */
function getServerPath(context: vscode.ExtensionContext): string | undefined {
    // First check if user specified a custom path
    const config = vscode.workspace.getConfiguration('traverse-lsp');
    const customPath = config.get<string>('serverPath');
    if (customPath && fs.existsSync(customPath)) {
        console.log('Using custom server path:', customPath);
        return customPath;
    }
    
    // Look for bundled binary in extension
    const binaryName = getServerBinaryName();
    const bundledPath = path.join(
        context.extensionPath,
        'server',
        'bin',
        binaryName
    );
    
    console.log('Looking for bundled binary at:', bundledPath);
    if (fs.existsSync(bundledPath)) {
        console.log('Found bundled binary!');
        // Ensure it's executable on Unix systems
        if (os.platform() !== 'win32') {
            try {
                fs.chmodSync(bundledPath, 0o755);
            } catch (err) {
                console.error('Failed to make binary executable:', err);
            }
        }
        return bundledPath;
    }
    
    // Fallback: Try to find locally built binary (development mode)
    const devPath = '/Users/gianlucabrigandi/Development/wa/traverse-lsp/traverse-lsp/target/release/traverse-lsp';
    
    if (fs.existsSync(devPath)) {
        console.log('Found local development binary at:', devPath);
        return devPath;
    }
    
    // Also try relative path
    const relativePath = path.join(
        context.extensionPath,
        '..',
        '..',
        'traverse-lsp',
        'target',
        'release',
        'traverse-lsp'
    );
    
    if (fs.existsSync(relativePath)) {
        console.log('Found local development binary at:', relativePath);
        return relativePath;
    }
    
    console.log('No server binary found');
    return undefined;
}

export async function activate(context: vscode.ExtensionContext) {
    const outputChannel = vscode.window.createOutputChannel('Traverse LSP');
    outputChannel.appendLine('Traverse LSP extension activating...');
    outputChannel.appendLine(`Extension path: ${context.extensionPath}`);
    
    // Try to find the server
    const serverPath = getServerPath(context);
    
    if (!serverPath) {
        vscode.window.showErrorMessage(
            'Traverse LSP server not found. Please check the extension installation or build the server manually.'
        );
        outputChannel.appendLine('ERROR: Server binary not found');
        return;
    }
    
    outputChannel.appendLine(`Starting Traverse LSP server from: ${serverPath}`);
    
    // Server options - run the binary
    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio,
            options: {
                env: {
                    ...process.env,
                    RUST_LOG: 'info',
                    TRAVERSE_LSP_TRACE: vscode.workspace.getConfiguration('traverse-lsp').get('trace.server') || 'off'
                }
            }
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio,
            options: {
                env: {
                    ...process.env,
                    RUST_LOG: 'debug',
                    TRAVERSE_LSP_TRACE: 'verbose'
                }
            }
        }
    };
    
    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'solidity' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.sol')
        },
        outputChannel,
        traceOutputChannel: outputChannel
    };
    
    // Create and start the language client
    client = new LanguageClient(
        'traverse-lsp',
        'Traverse Solidity Language Server',
        serverOptions,
        clientOptions
    );
    
    // Register commands
    // These commands are registered on the client side and communicate with the server
    
    const generateSequenceDiagramWorkspace = vscode.commands.registerCommand(
        'traverse.generateSequenceDiagram.workspace',
        async (uri?: vscode.Uri) => {
            if (!client) {
                vscode.window.showErrorMessage('Traverse LSP is not ready. Please wait for the server to start.');
                return;
            }
            
            const workspaceFolder = uri?.fsPath || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder found');
                return;
            }
            
            try {
                const result = await vscode.window.withProgress({
                    location: vscode.ProgressLocation.Notification,
                    title: "Generating workspace sequence diagram...",
                    cancellable: false
                }, async () => {
                    return await client.sendRequest('workspace/executeCommand', {
                        command: 'traverse.generateSequenceDiagram.workspace',
                        arguments: [{ workspace_folder: workspaceFolder }]
                    });
                });
                
                handleDiagramResult(result, 'Workspace Sequence Diagram');
            } catch (error: any) {
                vscode.window.showErrorMessage(`Failed to generate sequence diagram: ${error.message}`);
                outputChannel.appendLine(`Error: ${error.message}`);
            }
        }
    );
    
    const generateAllWorkspace = vscode.commands.registerCommand(
        'traverse.generateAll.workspace',
        async (uri?: vscode.Uri) => {
            if (!client) {
                vscode.window.showErrorMessage('Traverse LSP is not ready. Please wait for the server to start.');
                return;
            }
            
            const workspaceFolder = uri?.fsPath || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder found');
                return;
            }
            
            try {
                const result = await vscode.window.withProgress({
                    location: vscode.ProgressLocation.Notification,
                    title: "Generating all workspace diagrams...",
                    cancellable: false
                }, async () => {
                    return await client.sendRequest('workspace/executeCommand', {
                        command: 'traverse.generateAll.workspace',
                        arguments: [{ workspace_folder: workspaceFolder }]
                    });
                });
                
                handleDiagramResult(result, 'All Workspace Diagrams');
            } catch (error: any) {
                vscode.window.showErrorMessage(`Failed to generate all diagrams: ${error.message}`);
                outputChannel.appendLine(`Error: ${error.message}`);
            }
        }
    );
    
    
    // Workspace-level commands
    const generateCallGraphWorkspace = vscode.commands.registerCommand(
        'traverse.generateCallGraph.workspace',
        async (uri?: vscode.Uri) => {
            if (!client) {
                vscode.window.showErrorMessage('Traverse LSP is not ready. Please wait for the server to start.');
                return;
            }
            
            const workspaceFolder = uri?.fsPath || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder found');
                return;
            }
            
            try {
                const result = await vscode.window.withProgress({
                    location: vscode.ProgressLocation.Notification,
                    title: "Generating workspace call graph...",
                    cancellable: false
                }, async () => {
                    return await client.sendRequest('workspace/executeCommand', {
                        command: 'traverse.generateCallGraph.workspace',
                        arguments: [{ workspace_folder: workspaceFolder }]
                    });
                });
                
                handleDiagramResult(result, 'Workspace Call Graph');
            } catch (error: any) {
                vscode.window.showErrorMessage(`Failed to generate call graph: ${error.message}`);
                outputChannel.appendLine(`Error: ${error.message}`);
            }
        }
    );
    
    const analyzeStorageWorkspace = vscode.commands.registerCommand(
        'traverse.analyzeStorage.workspace',
        async (uri?: vscode.Uri) => {
            if (!client) {
                vscode.window.showErrorMessage('Traverse LSP is not ready. Please wait for the server to start.');
                return;
            }
            
            const workspaceFolder = uri?.fsPath || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder found');
                return;
            }
            
            try {
                const result = await vscode.window.withProgress({
                    location: vscode.ProgressLocation.Notification,
                    title: "Analyzing workspace storage...",
                    cancellable: false
                }, async () => {
                    return await client.sendRequest('workspace/executeCommand', {
                        command: 'traverse.analyzeStorage.workspace',
                        arguments: [{ workspace_folder: workspaceFolder }]
                    });
                });
                
                handleDiagramResult(result, 'Workspace Storage Analysis');
            } catch (error: any) {
                vscode.window.showErrorMessage(`Failed to analyze storage: ${error.message}`);
                outputChannel.appendLine(`Error: ${error.message}`);
            }
        }
    )
    
    const restartCommand = vscode.commands.registerCommand(
        'traverse.restart',
        async () => {
            if (!client) {
                vscode.window.showErrorMessage('Traverse LSP is not running');
                return;
            }
            
            try {
                await client.stop();
                await client.start();
                vscode.window.showInformationMessage('Traverse LSP server restarted');
            } catch (error: any) {
                vscode.window.showErrorMessage(`Failed to restart server: ${error.message}`);
                outputChannel.appendLine(`Error: ${error.message}`);
            }
        }
    );
    
    context.subscriptions.push(
        generateCallGraphWorkspace,
        generateSequenceDiagramWorkspace,
        generateAllWorkspace,
        analyzeStorageWorkspace,
        restartCommand
    );
    
    // Start the client
    client.start();
    
    outputChannel.appendLine('Traverse LSP extension activated');
}

/**
 * Handle diagram generation results by saving to workspace
 */
function handleDiagramResult(result: any, title: string) {
    if (!result || !result.success) {
        vscode.window.showErrorMessage(`Failed to generate ${title}`);
        return;
    }

    // Get workspace folder
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder found');
        return;
    }

    // Create timestamp for filenames
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').split('T')[0];
    
    // Determine output directory based on diagram type
    let outputDir = '';
    const baseDir = path.join(workspaceFolder.uri.fsPath, 'traverse-output');
    
    if (title.toLowerCase().includes('call graph')) {
        outputDir = path.join(baseDir, 'call-graphs');
    } else if (title.toLowerCase().includes('sequence')) {
        outputDir = path.join(baseDir, 'sequence-diagrams');
    } else if (title.toLowerCase().includes('storage')) {
        outputDir = path.join(baseDir, 'storage-reports');
    } else {
        outputDir = path.join(baseDir, 'diagrams');
    }

    // Create output directory if it doesn't exist
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }

    const savedFiles: string[] = [];
    
    // Handle multi-format response (data contains dot and/or mermaid)
    if (result.data) {
        // Save DOT format if available
        if (result.data.dot) {
            const dotFile = path.join(outputDir, `${title.toLowerCase().replace(/\s+/g, '-')}-${timestamp}.dot`);
            fs.writeFileSync(dotFile, result.data.dot);
            savedFiles.push(dotFile);
        }
        
        // Save Mermaid format if available
        if (result.data.mermaid) {
            const mermaidFile = path.join(outputDir, `${title.toLowerCase().replace(/\s+/g, '-')}-${timestamp}.mmd`);
            fs.writeFileSync(mermaidFile, result.data.mermaid);
            savedFiles.push(mermaidFile);
        }
    } 
    // Handle single-format response (backward compatibility)
    else if (result.diagram) {
        let extension = '.md';
        
        // Detect format from content
        if (result.diagram.includes('digraph') || result.diagram.includes('strict graph')) {
            extension = '.dot';
        } else if (result.diagram.includes('sequenceDiagram') || 
                   result.diagram.includes('graph TD') || 
                   result.diagram.includes('graph LR') ||
                   result.diagram.includes('flowchart')) {
            extension = '.mmd';
        }
        
        const filename = path.join(outputDir, `${title.toLowerCase().replace(/\s+/g, '-')}-${timestamp}${extension}`);
        fs.writeFileSync(filename, result.diagram);
        savedFiles.push(filename);
    }

    // Show notification with file locations
    if (savedFiles.length > 0) {
        const fileList = savedFiles.map(f => path.relative(workspaceFolder.uri.fsPath, f)).join('\n');
        
        vscode.window.showInformationMessage(
            `${title} saved to:\n${fileList}`,
            'Open Folder',
            'Open Files'
        ).then(selection => {
            if (selection === 'Open Folder') {
                // Open the output directory in explorer
                vscode.commands.executeCommand('revealInExplorer', vscode.Uri.file(outputDir));
            } else if (selection === 'Open Files') {
                // Open all generated files
                savedFiles.forEach(file => {
                    vscode.workspace.openTextDocument(file).then(doc => {
                        vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside, false);
                    });
                });
            }
        });
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}