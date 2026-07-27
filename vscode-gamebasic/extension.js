// VSCode-Client fuer den GameBasic-Language-Server.
//
// Startet `python -m gamebasic.lsp` (stdio) und verbindet ihn als
// LanguageClient. Syntax-Highlighting kommt aus der TextMate-Grammatik und
// laeuft auch ohne Server; der Server liefert Diagnostics/Completion/Hover/
// Goto-Definition/References/Outline.

const { workspace, window } = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

// Die Settings, die den Server-Prozess selbst betreffen (Pfad/Modul/Ein-Aus).
// Aendert sich eine davon, muss der Client neu gestartet werden -- vorher
// gab es dafuer keine Reaktion: nach einem "Server konnte nicht starten"-
// Fehler half nur ein manueller Fenster-Reload, ohne jeden Hinweis darauf,
// dass genau das noetig waere (Review-Fund).
const _RESTART_KEYS = [
  "gamebasic.pythonPath",
  "gamebasic.serverModule",
  "gamebasic.enableLanguageServer",
];

function _buildClient() {
  const cfg = workspace.getConfiguration("gamebasic");
  const pythonPath = cfg.get("pythonPath", "python");
  const serverModule = cfg.get("serverModule", "gamebasic.lsp");

  // cwd = erster Workspace-Ordner, damit `gamebasic` importierbar ist
  // (am besten den GameBasic-Projektordner oeffnen, oder pythonPath auf
  //  einen Python setzen, der das Paket findet).
  const folder =
    workspace.workspaceFolders && workspace.workspaceFolders.length
      ? workspace.workspaceFolders[0].uri.fsPath
      : undefined;

  const serverOptions = {
    run: {
      command: pythonPath,
      args: ["-m", serverModule],
      transport: TransportKind.stdio,
      options: { cwd: folder },
    },
    debug: {
      command: pythonPath,
      args: ["-m", serverModule],
      transport: TransportKind.stdio,
      options: { cwd: folder },
    },
  };

  const clientOptions = {
    documentSelector: [{ scheme: "file", language: "gamebasic" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.gb"),
    },
  };

  return new LanguageClient(
    "gamebasic",
    "GameBasic Language Server",
    serverOptions,
    clientOptions
  );
}

async function _startClient() {
  const cfg = workspace.getConfiguration("gamebasic");
  if (cfg.get("enableLanguageServer", true) === false) {
    return; // nur Syntax-Highlighting
  }
  client = _buildClient();
  try {
    await client.start();
  } catch (err) {
    window.showErrorMessage(
      "GameBasic-Language-Server konnte nicht starten: " +
        err.message +
        " (gamebasic.pythonPath pruefen)"
    );
  }
}

async function _stopClient() {
  if (!client) {
    return;
  }
  const c = client;
  client = undefined;
  await c.stop();
}

async function _restartClient() {
  await _stopClient();
  await _startClient();
}

function activate(context) {
  _startClient();

  context.subscriptions.push(
    workspace.onDidChangeConfiguration((e) => {
      if (_RESTART_KEYS.some((k) => e.affectsConfiguration(k))) {
        _restartClient();
      }
    })
  );
}

function deactivate() {
  return _stopClient();
}

module.exports = { activate, deactivate };
