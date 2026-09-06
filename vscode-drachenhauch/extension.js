// VSCode-Client fuer den Drachenhauch-Language-Server.
//
// Startet `dhrt lsp` (stdio) und verbindet ihn als LanguageClient -- die
// Runtime IST der Sprachserver, Python braucht es dafuer seit 2026-09-06
// nicht mehr (Weg A aus docs/entwurf-python-abbau.md). Syntax-Highlighting kommt aus der TextMate-Grammatik und
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
  "drachenhauch.dhrtPath",
  "drachenhauch.enableLanguageServer",
];

function _buildClient() {
  const cfg = workspace.getConfiguration("drachenhauch");
  const dhrtPath = cfg.get("dhrtPath", "dhrt");

  // cwd = erster Workspace-Ordner: relative IMPORT-Pfade einer Datei ohne
  // eigene URI loesen sich dann wenigstens gegen das Projekt auf.
  const folder =
    workspace.workspaceFolders && workspace.workspaceFolders.length
      ? workspace.workspaceFolders[0].uri.fsPath
      : undefined;

  const serverOptions = {
    run: {
      command: dhrtPath,
      args: ["lsp"],
      transport: TransportKind.stdio,
      options: { cwd: folder },
    },
    debug: {
      command: dhrtPath,
      args: ["lsp"],
      transport: TransportKind.stdio,
      options: { cwd: folder },
    },
  };

  const clientOptions = {
    documentSelector: [{ scheme: "file", language: "drachenhauch" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.dh"),
    },
  };

  return new LanguageClient(
    "drachenhauch",
    "Drachenhauch Language Server",
    serverOptions,
    clientOptions
  );
}

async function _startClient() {
  const cfg = workspace.getConfiguration("drachenhauch");
  if (cfg.get("enableLanguageServer", true) === false) {
    return; // nur Syntax-Highlighting
  }
  client = _buildClient();
  try {
    await client.start();
  } catch (err) {
    window.showErrorMessage(
      "Drachenhauch-Language-Server konnte nicht starten: " +
        err.message +
        " (drachenhauch.dhrtPath pruefen -- dhrt gebaut und im PATH?)"
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
