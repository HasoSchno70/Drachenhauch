// Live-Playground-Glue zwischen der emscripten-erzeugten dhrt.wasm und der Seite.
//
// Ablauf: Der Nutzer tippt Drachenhauch-Quelltext in die Textarea. "Ausführen"
// legt die Quelle in sessionStorage ab und lädt die Seite neu -> beim Neuladen
// schreibt das Harness sie ins virtuelle FS unter /program.dh und ruft main()
// EINMAL auf. dhrt kompiliert die Quelle SELBST im WASM (Lexer..Compiler in
// Rust, alle Stufen) und führt sie aus -- kein Pyodide, kein vorab kompiliertes
// .dhc nötig (siehe main.rs, cfg target_os="emscripten": /program.dh hat
// Vorrang). Der Reload garantiert eine FRISCHE Runtime pro Lauf -- ein erneutes
// main() mit raylib-Init säße sonst auf altem Zustand.

(function () {
  const outEl = document.getElementById("output");
  const statusEl = document.getElementById("status");
  const runBtn = document.getElementById("run");
  const shareBtn = document.getElementById("share");
  const srcEl = document.getElementById("src");

  // --- Teilbare Links: Quelle base64url-kodiert im URL-Hash (#gb=...) ---
  // Reine Client-Logik, kein Backend: ein Link reproduziert das Programm.
  function encodeSrc(str) {
    return btoa(unescape(encodeURIComponent(str)))
      .replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }
  function decodeSrc(b64) {
    b64 = b64.replace(/-/g, "+").replace(/_/g, "/");
    return decodeURIComponent(escape(atob(b64)));
  }
  function hashSrc() {
    const m = location.hash.match(/^#gb=(.+)$/);
    if (!m) return null;
    try { return decodeSrc(m[1]); } catch (e) { return null; }
  }

  const STORAGE_KEY = "gb_src";   // zuletzt editierte Quelle (bleibt erhalten)
  const RUN_FLAG = "gb_run";      // Einmal-Flag: nach Reload genau einen Lauf
  const DEFAULT_SRC = [
    "' Willkommen im Drachenhauch-Web-Playground!",
    "' Tippe Code und klicke Ausfuehren -- dhrt kompiliert im Browser.",
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
  // Quelle-Auswahl: bei einem Run-Reload das, was laeuft; sonst ein Share-Link
  // (#gb=...) falls vorhanden; sonst die zuletzt editierte bzw. die Default-Quelle.
  const shared = hashSrc();
  const fromShare = !doRun && shared !== null;
  srcEl.value = doRun ? (saved !== null ? saved : srcEl.value)
              : (shared !== null ? shared : (saved !== null ? saved : DEFAULT_SRC));
  if (fromShare) sessionStorage.setItem(STORAGE_KEY, srcEl.value);

  function runEmbedded() {
    setStatus("läuft …");
    try { window.Module.FS.writeFile("/program.dh", srcEl.value); }
    catch (e) { log("FS-Fehler: " + e); }
    try {
      window.Module.callMain([]);           // liest /program.dh, kompiliert, läuft
      setStatus("fertig");
    } catch (e) {
      if (e && e.name === "ExitStatus") setStatus("fertig (Code " + e.status + ")");
      else { setStatus("Fehler"); log("" + e); }
    }
  }

  // Beim Entwickeln alle Build-Artefakte am Zwischenspeicher vorbei laden.
  // Der Zeitstempel an `dhrt.js` allein reicht NICHT: die `.wasm` und die
  // `.data` holt emscripten selbst nach, ueber `locateFile`. Ohne das laedt
  // der Browser nach einem Neubau die alte `.wasm` -- und man sucht den Fehler
  // im Quelltext statt im Zwischenspeicher (genau das ist hier passiert).
  window.GB_CACHE_BUSTER =
    /^(localhost|127\.0\.0\.1|\[::1\])$/.test(location.hostname)
      ? "?b=" + Date.now() : "";

  // emscripten-Modulkonfiguration (global, von dhrt.js konsumiert).
  window.Module = {
    locateFile: function (pfad) { return pfad + window.GB_CACHE_BUSTER; },
    canvas: document.getElementById("canvas"),
    noInitialRun: true,   // wir rufen main() selbst auf (zuverlässiger als auto-run)
    print: log,
    printErr: log,
    onRuntimeInitialized: function () {
      runBtn.disabled = false;
      // nach "Ausführen"-Reload genau ein Lauf; ein frisch geoeffneter Share-Link
      // startet direkt (die Runtime ist nach dem Laden ohnehin frisch).
      if (doRun || fromShare) runEmbedded();
      else setStatus("bereit — Code eingeben & Ausführen");
    },
  };

  // "Ausführen": Quelle + Einmal-Run-Flag sichern, Hash aktuell halten, neu laden.
  runBtn.addEventListener("click", function () {
    sessionStorage.setItem(STORAGE_KEY, srcEl.value);
    sessionStorage.setItem(RUN_FLAG, "1");
    location.hash = "gb=" + encodeSrc(srcEl.value);   // URL bleibt teilbar + konsistent
    location.reload();
  });

  // "Link teilen": aktuelle Quelle in den Hash packen + URL in die Zwischenablage.
  shareBtn.addEventListener("click", function () {
    location.hash = "gb=" + encodeSrc(srcEl.value);
    const url = location.href;
    const done = function () { setStatus("Link kopiert ✓"); };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(url).then(done, function () {
        setStatus("Link ist in der Adresszeile");
      });
    } else {
      setStatus("Link ist in der Adresszeile");
    }
  });

  // --- Beispiel-Galerie (beispiele.js) ---------------------------------
  // Ein Klick laedt das Beispiel in den Editor und startet es -- derselbe Weg
  // wie "Ausführen", damit die Runtime auch hier frisch ist. Die Programme
  // laden bewusst KEINE Dateien, sonst liefen sie nur in einem Build mit
  // passendem assets/-Paket.
  (function galerieAufbauen() {
    const halter = document.getElementById("galerie");
    const liste = window.GB_BEISPIELE;
    if (!halter || !Array.isArray(liste)) return;
    liste.forEach(function (b) {
      const knopf = document.createElement("button");
      knopf.textContent = b.name;
      knopf.title = b.titel || "Beispiel laden und starten";
      knopf.addEventListener("click", function () {
        if (!b.datei) { srcEl.value = b.src; runBtn.click(); return; }
        // Aus einer Datei: die Quelle liegt neben der Seite (program.dh wird
        // vom Build dorthin kopiert). Absichtlich NICHT aus dem virtuellen
        // Dateisystem gelesen -- dort wird /program.dh vor jedem Lauf mit dem
        // Editorinhalt ueberschrieben, man bekaeme also den letzten Lauf.
        setStatus("lade " + b.datei + " …");
        fetch(b.datei + (window.GB_CACHE_BUSTER || ""))
          .then(function (r) {
            if (!r.ok) throw new Error(r.status + " " + r.statusText);
            return r.text();
          })
          .then(function (text) { srcEl.value = text; runBtn.click(); })
          .catch(function (e) {
            setStatus(b.datei + " nicht gefunden — erst bauen: "
              + "python rust/build_wasm.py <datei.dh>  (" + e.message + ")");
          });
      });
      halter.appendChild(knopf);
    });
  })();

  // Strg+Enter als Shortcut.
  srcEl.addEventListener("keydown", function (e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      runBtn.click();
    }
  });

  // Wenn dhrt.js fehlt (noch nicht gebaut), bleibt onRuntimeInitialized aus.
  setTimeout(function () {
    if (runBtn.disabled) {
      setStatus("dhrt.wasm nicht gefunden — erst bauen: "
        + "python rust/build_wasm.py <datei.dh>");
    }
  }, 4000);
})();
