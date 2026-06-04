// Live-Playground-Glue zwischen der emscripten-erzeugten gbrt.wasm und der Seite.
//
// Ablauf: Der Nutzer tippt GameBasic-Quelltext in die Textarea. "Ausführen"
// legt die Quelle in sessionStorage ab und lädt die Seite neu -> beim Neuladen
// schreibt das Harness sie ins virtuelle FS unter /program.gb und ruft main()
// EINMAL auf. gbrt kompiliert die Quelle SELBST im WASM (Lexer..Compiler in
// Rust, alle Stufen) und führt sie aus -- kein Pyodide, kein vorab kompiliertes
// .gbc nötig (siehe main.rs, cfg target_os="emscripten": /program.gb hat
// Vorrang). Der Reload garantiert eine FRISCHE Runtime pro Lauf -- ein erneutes
// main() mit raylib-Init säße sonst auf altem Zustand.

(function () {
  const outEl = document.getElementById("output");
  const statusEl = document.getElementById("status");
  const runBtn = document.getElementById("run");
  const srcEl = document.getElementById("src");

  const STORAGE_KEY = "gb_src";   // zuletzt editierte Quelle (bleibt erhalten)
  const RUN_FLAG = "gb_run";      // Einmal-Flag: nach Reload genau einen Lauf
  const DEFAULT_SRC = [
    "' Willkommen im GameBasic-Web-Playground!",
    "' Tippe Code und klicke Ausfuehren -- gbrt kompiliert im Browser.",
    "' (PRINT-Werte mit Komma trennen, nicht Semikolon.)",
    "PRINT \"Hallo aus dem Browser!\"",
    "DIM i AS INTEGER",
    "DIM s AS INTEGER",
    "s = 0",
    "FOR i = 1 TO 10",
    "    s = s + i",
    "    PRINT \"Summe bis\", i, \"=\", s",
    "NEXT",
    "",
  ].join("\n");

  function log(text) {
    outEl.textContent += text + "\n";
    outEl.scrollTop = outEl.scrollHeight;
  }
  function setStatus(text) { statusEl.textContent = text; }

  // Sicherheitsnetz: emscriptens Default-stdout ist console.log. Falls die
  // Runtime unser Module.print nicht uebernimmt, fangen wir die Ausgabe hier
  // trotzdem ab (genau EINER der Pfade -- Module.print ODER console.log --
  // bedient stdout, also keine Doppelausgabe).
  const _clog = console.log.bind(console);
  const _cerr = console.error.bind(console);
  console.log = function () {
    try { log(Array.prototype.join.call(arguments, " ")); } catch (e) {}
    return _clog.apply(console, arguments);
  };
  console.error = function () {
    try { log(Array.prototype.join.call(arguments, " ")); } catch (e) {}
    return _cerr.apply(console, arguments);
  };

  // Gespeicherte Quelle aus dem letzten "Ausführen" (bleibt fuer den Editor
  // erhalten). Das Run-Flag wird SOFORT konsumiert -> ein hängendes Programm
  // (z.B. Grafik-Render-Loop, siehe Grenzen) laeuft nach einem simplen Reload
  // NICHT erneut an; die Seite ist so wieder erreichbar.
  const saved = sessionStorage.getItem(STORAGE_KEY);
  const doRun = sessionStorage.getItem(RUN_FLAG) === "1";
  sessionStorage.removeItem(RUN_FLAG);
  srcEl.value = saved !== null ? saved : DEFAULT_SRC;

  function runEmbedded() {
    setStatus("läuft …");
    try { window.Module.FS.writeFile("/program.gb", saved || srcEl.value); }
    catch (e) { log("FS-Fehler: " + e); }
    try {
      window.Module.callMain([]);           // liest /program.gb, kompiliert, läuft
      setStatus("fertig");
    } catch (e) {
      if (e && e.name === "ExitStatus") setStatus("fertig (Code " + e.status + ")");
      else { setStatus("Fehler"); log("" + e); }
    }
  }

  // emscripten-Modulkonfiguration (global, von gbrt.js konsumiert).
  window.Module = {
    canvas: document.getElementById("canvas"),
    noInitialRun: true,   // wir rufen main() selbst auf (zuverlässiger als auto-run)
    print: log,
    printErr: log,
    onRuntimeInitialized: function () {
      runBtn.disabled = false;
      if (doRun) runEmbedded();    // nach "Ausführen"-Reload: genau einen Lauf
      else setStatus("bereit — Code eingeben & Ausführen");
    },
  };

  // "Ausführen": Quelle + Einmal-Run-Flag sichern, neu laden -> frische Runtime.
  runBtn.addEventListener("click", function () {
    sessionStorage.setItem(STORAGE_KEY, srcEl.value);
    sessionStorage.setItem(RUN_FLAG, "1");
    location.reload();
  });

  // Strg+Enter als Shortcut.
  srcEl.addEventListener("keydown", function (e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      runBtn.click();
    }
  });

  // Wenn gbrt.js fehlt (noch nicht gebaut), bleibt onRuntimeInitialized aus.
  setTimeout(function () {
    if (runBtn.disabled) {
      setStatus("gbrt.wasm nicht gefunden — erst bauen: "
        + "python rust/build_wasm.py <datei.gb>");
    }
  }, 4000);
})();
