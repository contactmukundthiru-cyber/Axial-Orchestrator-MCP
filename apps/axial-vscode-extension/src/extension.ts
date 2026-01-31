import * as vscode from 'vscode';
import axios from 'axios';
import * as cp from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(cp.exec);
const exec = cp.exec;

function getDaemonUrl(): string {
    return vscode.workspace.getConfiguration('axial').get('daemonUrl') || 'http://127.0.0.1:8080';
}

export function activate(context: vscode.ExtensionContext) {
    console.log('AXIAL Bridge is now active!');

    // Register commands to relay to the daemon
    context.subscriptions.push(
        vscode.commands.registerCommand('axial.submitPlan', async (plan: any) => {
            const daemonUrl = getDaemonUrl();
            try {
                const response = await axios.post(`${daemonUrl}/plan`, plan);
                vscode.window.showInformationMessage(`AXIAL: Plan submitted (Index: ${response.data.ledger_index})`);
                return response.data;
            } catch (err: any) {
                vscode.window.showErrorMessage(`AXIAL Daemon Error: ${err.message}`);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('axial.approveGate', async (gateId: string, approved: boolean) => {
            const daemonUrl = getDaemonUrl();
            try {
                await axios.post(`${daemonUrl}/approve`, { gate_id: gateId, approved });
                vscode.window.showInformationMessage(`AXIAL: Gate ${gateId} ${approved ? 'Approved' : 'Rejected'}`);
            } catch (err: any) {
                vscode.window.showErrorMessage(`AXIAL Daemon Error: ${err.message}`);
            }
        })
    );

    const workforceProvider = new WorkforceViewProvider(context.extensionUri);
    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider('axial-workforce-view', workforceProvider)
    );

    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = "$(shield) AXIAL";
    statusBarItem.tooltip = "AXIAL Local-First Neural Shield";
    statusBarItem.command = 'axial-ghost.showInterface';
    statusBarItem.show();

    let disposable = vscode.commands.registerCommand('axial-ghost.runTask', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor found.');
            return;
        }

        const selection = editor.selection;
        const text = editor.document.getText(selection);

        if (!text) {
            vscode.window.showErrorMessage('Please select some text/instruction.');
            return;
        }

        vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: "AXIAL: Routing Task...",
            cancellable: false
        }, async (progress) => {
            try {
                // In a real v1, we call the local axial binary
                const { stdout, stderr } = await execAsync(`axial route --task "${text.replace(/"/g, '\\"')}"`);
                if (stderr) console.error(stderr);
                vscode.window.showInformationMessage(`AXIAL Result: ${stdout.trim()}`);
            } catch (err: any) {
                vscode.window.showErrorMessage(`AXIAL Error: ${err.message}`);
            }
        });
    });

    let openInterface = vscode.commands.registerCommand('axial-ghost.showInterface', () => {
        vscode.window.showInformationMessage('Launching AXIAL Command Center...');
        exec('axial ui');
    });

    context.subscriptions.push(disposable, openInterface, statusBarItem);

    let forkSession = vscode.commands.registerCommand('axial-ghost.forkSession', async () => {
        const id = await vscode.window.showInputBox({ prompt: "Enter fork ID (e.g. debug-auth-bug)" });
        if (id) {
            await execAsync(`axial git fork --id "${id}"`);
            vscode.window.showInformationMessage(`Session forked: ${id}`);
        }
    });
    context.subscriptions.push(forkSession);
}

class WorkforceViewProvider implements vscode.WebviewViewProvider {
    constructor(private readonly _extensionUri: vscode.Uri) {}

    public resolveWebviewView(
        webviewView: vscode.WebviewView,
        context: vscode.WebviewViewResolveContext,
        _token: vscode.CancellationToken,
    ) {
        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this._extensionUri]
        };

        webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);

        webviewView.webview.onDidReceiveMessage(data => {
            switch (data.type) {
                case 'runTask':
                    vscode.commands.executeCommand('axial-ghost.runTask');
                    break;
                case 'forkSession':
                    vscode.commands.executeCommand('axial-ghost.forkSession');
                    break;
            }
        });
    }

    private _getHtmlForWebview(webview: vscode.Webview) {
        return `<!DOCTYPE html>
            <html lang="en">
            <head>
                <style>
                    body { font-family: sans-serif; padding: 10px; color: var(--vscode-foreground); }
                    .agent { margin-bottom: 10px; padding: 10px; border: 1px solid var(--vscode-widget-border); border-radius: 4px; }
                    .status { font-size: 0.8em; color: var(--vscode-descriptionForeground); }
                    button { width: 100%; margin-top: 5px; cursor: pointer; background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; padding: 5px; }
                    button:hover { background: var(--vscode-button-hoverBackground); }
                </style>
            </head>
            <body>
                <h3>Neural Workforce</h3>
                <div id="agents">
                    <div class="agent">
                        <strong>Aider (Local)</strong>
                        <div class="status">IDLE</div>
                    </div>
                    <div class="agent">
                        <strong>Claude Code</strong>
                        <div class="status">MONITORING</div>
                    </div>
                </div>
                <hr>
                <button onclick="post('runTask')">Run Selection as Task</button>
                <button onclick="post('forkSession')">New Fork</button>
                <script>
                    const vscode = acquireVsCodeApi();
                    function post(type) { vscode.postMessage({ type }); }
                </script>
            </body>
            </html>`;
    }
}

export function deactivate() {}
