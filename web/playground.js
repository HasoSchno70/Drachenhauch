// Glue zwischen der emscripten-erzeugten gbrt.wasm und der Seite.
//
// emscripten ruft beim Laden das globale `Module`-Objekt ab. Wir konfigurieren
// es VOR dem Laden von gbrt.js: stdout -> Output-Div, Canvas binden, und
// (optional) die .gbc per fetch in das virtuelle FS schreiben, falls sie nicht
// schon via --embed-file eingebettet wurde.
//
// Hinweis: Dieses Harness erwartet die vom Build erzeugten Dateien gbrt.js +
// gbrt.wasm (siehe rust/build_wasm.py). Ohne sie zeigt die Seite nur einen
// Hinweis.

(function () {
  const outEl = document.getElementById("output");
  const statusEl = document.getElementById("status");
  const runBtn = document.getElementById("run");

  function log(text) {
    outEl.textContent += text + "\n";
    outEl.scrollTop = outEl.scrollHeight;
  }
  function setStatus(text) { statusEl.textContent = text; }

  // emscripten-Modulkonfiguration (global, von gbrt.js konsumiert).
  window.Module = {
    canvas: document.getElementById("canvas"),
    noInitialRun: true, // wir starten manuell via callMain() auf Knopfdruck
    print: log,
    printErr: log,
    onRuntimeInitialized: function () {
      setStatus("bereit");
      runBtn.disabled = false;
    },
    // Falls die .gbc nicht eingebettet wurde: vor dem Lauf per fetch laden.
    preRun: [
      function () {
        try {
          const FS = window.Module.FS;
          // Schon eingebettet? Dann nichts tun.
          try { FS.stat("/program.gbc"); return; } catch (e) {}
          // Sonst aus dem Web-Ordner nachladen (synchron via XHR im preRun ok).
          const xhr = new XMLHttpRequest();
          xhr.open("GET", "program.gbc", false);
          xhr.overrideMimeType("text/plain; charset=x-user-defined");
          xhr.send(null);
          if (xhr.status === 200) {
            FS.writeFile("/program.gbc", xhr.responseText);
          }
        } catch (e) {
          log("Konnte program.gbc nicht laden: " + e);
        }
      },
    ],
  };

  runBtn.addEventListener("click", function () {
    runBtn.disabled = true;
    outEl.textContent = "";
    setStatus("läuft …");
    try {
      // main() liest /program.gbc (siehe main.rs, cfg target_os=emscripten).
      window.Module.callMain([]);
      setStatus("fertig");
    } catch (e) {
      // emscripten wirft bei exit() eine ExitStatus-"Exception" -- harmlos.
      if (e && e.name === "ExitStatus") {
        setStatus("fertig (Code " + e.status + ")");
      } else {
        setStatus("Fehler");
        log("" + e);
      }
    } finally {
      runBtn.disabled = false;
    }
  });

  // Wenn gbrt.js fehlt (noch nicht gebaut), bleibt onRuntimeInitialized aus.
  setTimeout(function () {
    if (runBtn.disabled) {
      setStatus("gbrt.wasm nicht gefunden — erst bauen: "
        + "rust/build_wasm.py <datei.gb>");
    }
  }, 1500);
})();
