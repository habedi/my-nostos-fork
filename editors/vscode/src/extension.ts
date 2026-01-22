import * as path from 'path';
import * as fs from 'fs';
import { workspace, ExtensionContext, window, commands, WebviewPanel, ViewColumn, Uri } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from 'vscode-languageclient/node';

// REPL webview panel (singleton)
let replPanel: WebviewPanel | undefined;

function extLog(msg: string) {
    const line = `${new Date().toISOString()} ${msg}\n`;
    fs.appendFileSync('/tmp/nostos_ext.log', line);
    console.log(msg);
}

let client: LanguageClient | undefined;

// Track registered commands to avoid duplicates within this session
const registeredCommands = new Set<string>();

// Helper to safely register a command (ignores if already exists)
function safeRegisterCommand(context: ExtensionContext, commandId: string, callback: (...args: any[]) => any) {
    if (registeredCommands.has(commandId)) {
        console.log(`Command ${commandId} already registered by us, skipping`);
        return;
    }

    // Mark as registered first to prevent race conditions
    registeredCommands.add(commandId);

    try {
        const disposable = commands.registerCommand(commandId, callback);
        context.subscriptions.push(disposable);
        console.log(`Registered command: ${commandId}`);
    } catch (e: any) {
        console.log(`Command ${commandId} registration failed (may already exist): ${e.message}`);
        // Don't throw - just log and continue
    }
}

export function activate(context: ExtensionContext) {
    try { fs.unlinkSync('/tmp/nostos_ext.log'); } catch {}
    extLog('=== activate() called ===');
    console.log('Nostos extension is activating...');

    // Register all commands FIRST (before starting LSP, so they work even if LSP fails)

    // Register restart command
    safeRegisterCommand(context, 'nostos.restartServer', async () => {
        if (client) {
            await client.stop();
            client = undefined;
        }
        startLanguageServer(context);
        window.showInformationMessage('Nostos language server restarted');
    });

    // Register build cache command
    safeRegisterCommand(context, 'nostos.buildCache', async () => {
        if (client) {
            try {
                await client.sendRequest('workspace/executeCommand', {
                    command: 'nostos.buildCache',
                    arguments: []
                });
            } catch (e) {
                window.showErrorMessage(`Failed to build cache: ${e}`);
            }
        } else {
            window.showWarningMessage('Language server not running');
        }
    });

    // Register clear cache command
    safeRegisterCommand(context, 'nostos.clearCache', async () => {
        if (client) {
            try {
                await client.sendRequest('workspace/executeCommand', {
                    command: 'nostos.clearCache',
                    arguments: []
                });
            } catch (e) {
                window.showErrorMessage(`Failed to clear cache: ${e}`);
            }
        } else {
            window.showWarningMessage('Language server not running');
        }
    });

    // Register commit current file command (Ctrl+Alt+C)
    safeRegisterCommand(context, 'nostos.commit', async () => {
        if (client) {
            const editor = window.activeTextEditor;
            if (!editor) {
                window.showWarningMessage('No active editor');
                return;
            }

            // Only commit .nos files
            if (!editor.document.fileName.endsWith('.nos')) {
                window.showWarningMessage('Not a Nostos file');
                return;
            }

            try {
                const uri = editor.document.uri.toString();
                await client.sendRequest('workspace/executeCommand', {
                    command: 'nostos.commit',
                    arguments: [uri]
                });
                window.showInformationMessage('Committed to live system');
            } catch (e) {
                window.showErrorMessage(`Failed to commit: ${e}`);
            }
        } else {
            window.showWarningMessage('Language server not running');
        }
    });

    // Register commit all files command
    safeRegisterCommand(context, 'nostos.commitAll', async () => {
        if (client) {
            try {
                await client.sendRequest('workspace/executeCommand', {
                    command: 'nostos.commitAll',
                    arguments: []
                });
            } catch (e) {
                window.showErrorMessage(`Failed to commit all: ${e}`);
            }
        } else {
            window.showWarningMessage('Language server not running');
        }
    });

    // Register REPL command
    safeRegisterCommand(context, 'nostos.openRepl', () => {
        openReplPanel(context);
    });

    // Start the language server AFTER registering commands
    startLanguageServer(context);
}

function startLanguageServer(context: ExtensionContext) {
    // Stop existing client if any
    if (client) {
        client.stop();
        client = undefined;
    }

    const serverPath = findServerPath(context);

    if (!serverPath) {
        window.showWarningMessage(
            'Nostos language server (nostos-lsp) not found. ' +
            'Please install it or set nostos.serverPath in settings.'
        );
        return;
    }

    // Check if file exists
    if (!fs.existsSync(serverPath)) {
        window.showErrorMessage(`LSP binary not found at: ${serverPath}`);
        return;
    }

    console.log(`Starting Nostos LSP server: ${serverPath}`);
    window.showInformationMessage(`Starting LSP: ${serverPath}`);

    // Server executable - let vscode-languageclient handle transport
    const serverExecutable: Executable = {
        command: serverPath,
        args: [],
        // Don't specify transport - let the client auto-detect stdio
        options: {
            env: { ...process.env },
            shell: false,  // Direct execution, no shell wrapper
        },
    };

    const serverOptions: ServerOptions = {
        run: serverExecutable,
        debug: serverExecutable,
    };

    // Client options
    const traceChannel = window.createOutputChannel('Nostos LSP Trace');
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'nostos' },
            { scheme: 'untitled', language: 'nostos' },
            { scheme: 'file', pattern: '**/*.nos' }  // Also match by file pattern
        ],
        outputChannelName: 'Nostos Language Server',
        traceOutputChannel: traceChannel,
        synchronize: {
            // Watch .nos files in workspace
            fileEvents: workspace.createFileSystemWatcher('**/*.nos')
        }
    };

    // Log to trace channel
    traceChannel.appendLine('Starting LSP client...');

    // Create and start the client
    client = new LanguageClient(
        'nostos',
        'Nostos Language Server',
        serverOptions,
        clientOptions
    );

    // Add state change listener for debugging
    client.onDidChangeState((event) => {
        const stateNames = ['Starting', 'Stopped', 'Running'];
        const oldName = stateNames[event.oldState] || String(event.oldState);
        const newName = stateNames[event.newState] || String(event.newState);
        extLog(`STATE: ${oldName} -> ${newName}`);
        console.log(`LSP state change: ${oldName} -> ${newName}`);
        if (event.newState === 1) { // Stopped
            extLog('!!! SERVER STOPPED !!!');
        }
    });

    // Handle process close
    client.outputChannel.appendLine('Client initialized, starting server...');

    // Handle errors
    client.onTelemetry((data: any) => {
        console.log('LSP telemetry:', data);
    });

    extLog('Calling client.start()...');

    // Start the client (also starts the server)
    client.start().then(() => {
        extLog('client.start() resolved - CONNECTED');
        console.log('Nostos language server started successfully');
    }).catch((error: any) => {
        extLog(`client.start() FAILED: ${error.message || error}`);
        console.error('Failed to start Nostos language server:', error);
        client = undefined;
    });

    extLog('startLanguageServer() returning');
}

function findServerPath(context: ExtensionContext): string | undefined {
    const config = workspace.getConfiguration('nostos');

    // 1. Check user-configured path
    const configuredPath = config.get<string>('serverPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
        return configuredPath;
    }

    // 2. Check bundled binary in extension
    const bundledPath = path.join(context.extensionPath, 'bin', 'nostos-lsp');
    if (fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    // 3. Check common install locations
    const homeDir = process.env.HOME || process.env.USERPROFILE || '';
    const commonPaths = [
        path.join(homeDir, '.cargo', 'bin', 'nostos-lsp'),
        path.join(homeDir, '.local', 'bin', 'nostos-lsp'),
        '/usr/local/bin/nostos-lsp',
        '/usr/bin/nostos-lsp',
    ];

    for (const p of commonPaths) {
        if (fs.existsSync(p)) {
            return p;
        }
    }

    // 4. Try to find in PATH (will fail at runtime if not found)
    return 'nostos-lsp';
}

function openReplPanel(context: ExtensionContext) {
    // If panel exists, reveal it
    if (replPanel) {
        replPanel.reveal(ViewColumn.Beside);
        return;
    }

    // Create new panel
    replPanel = window.createWebviewPanel(
        'nostosRepl',
        'Nostos REPL',
        ViewColumn.Beside,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );

    // Set HTML content
    replPanel.webview.html = getReplHtml();

    // Handle messages from webview
    replPanel.webview.onDidReceiveMessage(
        async (message) => {
            if (message.type === 'eval') {
                const expr = message.expression;

                if (!client) {
                    replPanel?.webview.postMessage({
                        type: 'result',
                        success: false,
                        error: 'Language server not running'
                    });
                    return;
                }

                try {
                    const result: any = await client.sendRequest('workspace/executeCommand', {
                        command: 'nostos.eval',
                        arguments: [expr]
                    });

                    replPanel?.webview.postMessage({
                        type: 'result',
                        success: result?.success ?? false,
                        result: result?.result,
                        error: result?.error
                    });
                } catch (e: any) {
                    replPanel?.webview.postMessage({
                        type: 'result',
                        success: false,
                        error: e.message || String(e)
                    });
                }
            } else if (message.type === 'clear') {
                // Clear is handled in webview, no server action needed
            }
        },
        undefined,
        context.subscriptions
    );

    // Clean up when panel is closed
    replPanel.onDidDispose(
        () => {
            replPanel = undefined;
        },
        undefined,
        context.subscriptions
    );
}

function getReplHtml(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nostos REPL</title>
    <style>
        * {
            box-sizing: border-box;
        }
        body {
            font-family: var(--vscode-editor-font-family, 'Consolas', 'Courier New', monospace);
            font-size: var(--vscode-editor-font-size, 14px);
            padding: 0;
            margin: 0;
            height: 100vh;
            display: flex;
            flex-direction: column;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
        }
        #output {
            flex: 1;
            overflow-y: auto;
            padding: 10px;
            white-space: pre-wrap;
            word-wrap: break-word;
        }
        .entry {
            margin-bottom: 8px;
        }
        .prompt {
            color: var(--vscode-terminal-ansiCyan, #4ec9b0);
        }
        .input-line {
            color: var(--vscode-editor-foreground);
        }
        .result {
            color: var(--vscode-terminal-ansiGreen, #4ec9b0);
            margin-left: 20px;
        }
        .error {
            color: var(--vscode-errorForeground, #f44747);
            margin-left: 20px;
        }
        .info {
            color: var(--vscode-descriptionForeground, #888);
            font-style: italic;
        }
        #input-container {
            display: flex;
            align-items: center;
            padding: 8px 10px;
            border-top: 1px solid var(--vscode-panel-border, #444);
            background: var(--vscode-input-background);
        }
        #prompt-label {
            color: var(--vscode-terminal-ansiCyan, #4ec9b0);
            margin-right: 8px;
            font-weight: bold;
        }
        #input {
            flex: 1;
            background: transparent;
            border: none;
            outline: none;
            color: var(--vscode-input-foreground);
            font-family: inherit;
            font-size: inherit;
        }
        #input::placeholder {
            color: var(--vscode-input-placeholderForeground, #888);
        }
        .toolbar {
            padding: 4px 10px;
            border-bottom: 1px solid var(--vscode-panel-border, #444);
            display: flex;
            gap: 8px;
        }
        .toolbar button {
            background: var(--vscode-button-secondaryBackground);
            color: var(--vscode-button-secondaryForeground);
            border: none;
            padding: 4px 8px;
            cursor: pointer;
            font-size: 12px;
        }
        .toolbar button:hover {
            background: var(--vscode-button-secondaryHoverBackground);
        }
    </style>
</head>
<body>
    <div class="toolbar">
        <button id="clear-btn">Clear</button>
        <span class="info">Press Enter to evaluate, ↑/↓ for history</span>
    </div>
    <div id="output">
        <div class="info">Nostos REPL - Type expressions to evaluate</div>
    </div>
    <div id="input-container">
        <span id="prompt-label">></span>
        <input type="text" id="input" placeholder="Enter expression..." autofocus />
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        const output = document.getElementById('output');
        const input = document.getElementById('input');
        const clearBtn = document.getElementById('clear-btn');

        let history = [];
        let historyIndex = -1;
        let currentInput = '';

        // Restore state if available
        const previousState = vscode.getState();
        if (previousState) {
            history = previousState.history || [];
            output.innerHTML = previousState.output || '<div class="info">Nostos REPL - Type expressions to evaluate</div>';
        }

        function saveState() {
            vscode.setState({
                history: history,
                output: output.innerHTML
            });
        }

        function addOutput(html) {
            output.innerHTML += html;
            output.scrollTop = output.scrollHeight;
            saveState();
        }

        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && input.value.trim()) {
                const expr = input.value.trim();

                // Add to history
                if (history.length === 0 || history[history.length - 1] !== expr) {
                    history.push(expr);
                    if (history.length > 100) history.shift();
                }
                historyIndex = -1;
                currentInput = '';

                // Show input in output
                addOutput('<div class="entry"><span class="prompt">> </span><span class="input-line">' +
                    escapeHtml(expr) + '</span></div>');

                // Send to extension
                vscode.postMessage({ type: 'eval', expression: expr });

                input.value = '';
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                if (history.length > 0) {
                    if (historyIndex === -1) {
                        currentInput = input.value;
                        historyIndex = history.length - 1;
                    } else if (historyIndex > 0) {
                        historyIndex--;
                    }
                    input.value = history[historyIndex];
                }
            } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                if (historyIndex !== -1) {
                    if (historyIndex < history.length - 1) {
                        historyIndex++;
                        input.value = history[historyIndex];
                    } else {
                        historyIndex = -1;
                        input.value = currentInput;
                    }
                }
            }
        });

        clearBtn.addEventListener('click', () => {
            output.innerHTML = '<div class="info">Nostos REPL - Type expressions to evaluate</div>';
            vscode.postMessage({ type: 'clear' });
            saveState();
        });

        // Handle messages from extension
        window.addEventListener('message', (event) => {
            const message = event.data;
            if (message.type === 'result') {
                if (message.success) {
                    addOutput('<div class="result">' + escapeHtml(message.result || '()') + '</div>');
                } else {
                    addOutput('<div class="error">Error: ' + escapeHtml(message.error) + '</div>');
                }
            }
        });

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // Focus input on load
        input.focus();
    </script>
</body>
</html>`;
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
