// VSCode-Client fuer den GameBasic-Language-Server.
//
// Startet `python -m gamebasic.lsp` (stdio) und verbindet ihn als
// LanguageClient. Syntax-Highlighting kommt aus der TextMate-Grammatik und
// laeuft auch ohne Server; der Server liefert Diagnostics/Completion/Hover/
// Goto-Definition/References/Outline.

const { workspace, window } = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

function activate(context) {
  const cfg = workspace.getConfiguration("gamebasic");
  if (cfg.get("enableLanguageServer", true) === false) {
    return; // nur Syntax-Highlighting
  }

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

  client = new LanguageClient(
    "gamebasic",
    "GameBasic Language Server",
    serverOptions,
    clientOptions
  );

  client.start().catch((err) => {
    window.showErrorMessage(
      "GameBasic-Language-Server konnte nicht starten: " +
        err.message +
        " (gamebasic.pythonPath pruefen)"
    );
  });
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
