# Allzweck-Roadmap (Audit 2026-08-17)

Drachenhauch ist als **Game Basic** gestartet — ein Dialekt zum Schreiben von
Spielen. Diese Roadmap beantwortet eine andere Frage: **was fehlt, damit man
damit alles schreiben kann**, was man schreiben möchte — Werkzeuge, Skripte,
Datenbank-Anwendungen, Dienste, Auswertungen.

**Der Befund vorweg:** die Lücken sitzen *nicht* in der Syntax. Als Sprache ist
Drachenhauch längst ausdrucksstärker als das, wofür es gedacht war. Sie sitzen
an zwei Stellen:

1. **Das Programm kann nicht mit seinem Betriebssystem reden** — keine
   Argumente, keine Umgebung, kein Prozessaufruf, kein Rückgabewert. Damit
   fällt die gesamte Klasse „Werkzeug in einer Kette" weg.
2. **Großer Code lässt sich nicht ordnen** — ein flacher Namensraum, textuelles
   `IMPORT`, keine getrennte Übersetzung.

Alles Übrige in dieser Liste ist Bibliothek, und Bibliothek ist Fleißarbeit.

## Was schon trägt (damit die Lücken einordbar bleiben)

Typen mit Pascal-Strenge, Klassen mit Vererbung, `PROPERTY`, Operator-Overloading,
`STATIC CONST`, Comprehensions (Liste/Dict/Set), Tupel + Destructuring, f-Strings,
Coroutines, `BYREF`, Named Arguments, `FUNCREF`, Variadics, `WITH`, Slicing.
Dazu SQLite, JSON, Regex, HTTP-Client, Diagramme, eine 22-Widget-GUI mit
Tabellen und Formular-Designer, Sprachserver, Debugger, Profiler, Export als
eigenständige `.exe`.

Das Tippspiel (`buch-tippspiel/`) ist der Beleg: echte Datenbank-Anwendung mit
Netz, Fenstern, Reitern und Saison-Verwaltung — vollständig in Drachenhauch.

## Umsetzungs-Checkliste pro Befehl (Stand Stufe B — dhrt ist die einzige Runtime)

- [ ] `rust/drachenhauch_runtime/src/builtins.rs` (pur) bzw. `vm.rs` `try_*`
      (braucht VM-/Fenster-State). Arity + Typen im Wrapper prüfen,
      Fehlermeldung im gewohnten Wortlaut (`"NAME: erwartet …"`).
- [ ] `drachenhauch/editor_qt/builtin_index.json` — Name/kind/**Signatur**/Modul.
      Die Signatur muss stimmen, `dhrt --check` leitet daraus die erlaubte
      Argumentzahl ab; eine zu enge Signatur erzeugt Falsch-Alarme in fremdem Code.
- [ ] Golden-Test in `tests/` (`assert run_gb(src) == erwartet`), plus
      Rust-`#[test]` für reine Rechen-Anteile.
- [ ] Hover-Doku in `editor_qt/builtin_docs.py`.
- [ ] Bei neuem Keyword: `vscode-drachenhauch/build_grammar.py` neu generieren.
- [ ] Bei neuem Modul: Name in `preprocess.rs` `MODULES` **und**
      `drachenhauch/modules/__init__.py` `KNOWN_MODULES` (synchron halten),
      dazu `docs/module-<name>.md` + Zeile in `docs/README.md`.

---

## WP A — Betriebssystem-Anbindung

**Das ist die größte Lücke.** Verifiziert gegen `builtins.rs`/`vm.rs`: die
einzigen `std::env::args()` im Quelltext sind die von `dhrt` selbst (`main.rs`).
Ein Drachenhauch-Programm kann seine eigenen Argumente nicht lesen.

Folge: eine exportierte `.exe` nimmt keine Argumente entgegen, kann sich in
keine Werkzeugkette einreihen, kein anderes Programm aufrufen und ihrem
Aufrufer nicht sagen, ob es geklappt hat. Genau das ist der Kern von
„Werkzeug, Skript, Automatisierung".

- [ ] `ARGC()` / `ARG$(n)` — Kommandozeilenargumente des *Programms*, nicht die
      von `dhrt`. **Stolperstein:** beim Start über `dhrt run datei.dh` müssen
      die eigenen Argumente von denen der Runtime getrennt werden (Konvention
      `--`); im Export-Fall gehören alle Argumente dem Programm.
- [ ] `GETENV$(name$ [, vorgabe$])`, `SETENV(name$, wert$)`
- [ ] `EXIT(code)` — geordnetes Ende mit Rückgabewert. Ohne das kann kein
      Aufrufer und kein Testlauf Erfolg von Fehlschlag unterscheiden.
- [ ] `SHELL(befehl$ [, args])` → Rückgabewert; `SHELL_OUT$(...)` → Ausgabe
      einsammeln. Bewusst zwei Befehle statt eines mit Schalter — der häufige
      Fall soll kurz bleiben.
- [ ] Ausgabe nach stderr (`EPRINT`), damit `PRINT` als Nutzdaten durchgereicht
      werden kann.
- [ ] `CWD$()` / `CHDIR(pfad$)`. **Achtung:** `dhrt` wechselt beim Start selbst
      ins Verzeichnis der `.dh`-Datei (relative Asset-Pfade) — das muss
      dokumentiert bleiben, sonst überrascht `CWD$()`.

## WP B — Bytes und Binärdateien

Es gibt keinen Byte-Typ. Dateien sind Text (`READALL`/`WRITEALL`/`READLINES`),
`STRING` ist UTF-8 und taugt darum nicht als Byte-Behälter, `SEEK` fehlt.
Damit gehen nicht: eigene Dateiformate, Protokolle mitlesen, ZIP/PNG anfassen,
Serial-Geräte jenseits von Text, Bilder aus dem Netz weiterverarbeiten.

WP B ist Voraussetzung für WP C (Rumpf als Bytes), WP D (Prüfsummen über
Dateien) und ZIP in WP J.

- [ ] Externer Typ `BUFFER`: `BUFFER_NEW(groesse)`, `BUFFER_LEN`,
      `BUFFER_GET/SET(i, byte)`, `BUFFER_SLICE`, `BUFFER_CONCAT`, `BUFFER_FILL`
- [ ] Umwandlung: `BUFFER_FROM_STRING$`/`BUFFER_TO_STRING$` (UTF-8, mit
      klarer Ansage bei ungültigen Folgen), `BUFFER_TO_HEX$`, `BASE64_*`
      auf `BUFFER` erweitern
- [ ] Datei: `OPENFILE` um `"rb"`/`"wb"`/`"ab"` erweitern, dazu
      `READ_BYTES(f, n)` → BUFFER, `WRITE_BYTES(f, buf)`, `SEEK(f, pos)`,
      `TELL(f)`. Alternativ pfadbasiert `READALL_BYTES`/`WRITEALL_BYTES`
      analog zu den vorhandenen `READALL`/`WRITEALL`.
- [ ] Zahlen packen: `BUFFER_GET_I16/I32/I64/F32/F64` + Setter, Byte-Reihenfolge
      als Argument (`"le"`/`"be"`) — nicht raten.

## WP C — HTTP für echte Dienste

Heute: nur `HTTP_GET(url$)`, `HTTP_POST(url$, body$)`, `HTTP_DOWNLOAD`. **Keine
eigenen Request-Header**, kein PUT/PATCH/DELETE, Timeout fest bei 10 Sekunden,
keine Sitzungen. Damit ist praktisch jede angemeldete REST-Schnittstelle außer
Reichweite.

`docs/module-html.md` rät im Abschnitt „Grenzen" selbst zu „Token-basierte Auth
via Header" — eine API, die es nicht gibt. Das ist der klarste Beleg, dass hier
etwas fehlt.

- [ ] `HTTP_REQUEST(methode$, url$, body$, header AS MAP OF STRING)` als *eine*
      allgemeine Funktion; `HTTP_GET`/`HTTP_POST` bleiben als Kurzform
- [ ] `HTTP_TIMEOUT(sekunden)`
- [ ] Rumpf als `BUFFER` senden und empfangen (nach WP B) — heute erzwingt
      UTF-8-mit-Ersetzung den Umweg über `HTTP_DOWNLOAD` in eine Datei
- [ ] Hintergrund-Variante (`HTTP_GET_START`-Muster) für alle Methoden, nicht
      nur GET
- [ ] Doku nachziehen: `docs/module-html.md` sagt „alles aus der Python-
      Standardbibliothek" — seit Stufe B ist es Rust; und der Header-Rat in
      „Grenzen" wird erst mit diesem WP wahr

## WP D — Identität und Prüfsummen

Vorhanden sind `CRC32`, `HASH` (FNV-1a) und Base64 — Prüfsummen fürs
Wiedererkennen, nichts für Vertrauen. Es fehlt alles, was eine Anmeldung, eine
Signatur oder eine stabile eindeutige Nummer braucht.

- [ ] `SHA256$`, `SHA1$`, `MD5$` — je über STRING und über BUFFER
- [ ] `HMAC_SHA256$(schluessel, daten)` — Web-APIs, Webhooks
- [ ] `UUID4$()`
- [ ] `RANDOM_BYTES(n)` → BUFFER, aus der Betriebssystem-Quelle (**nicht** aus
      dem Spiel-PRNG — der ist gesät und vorhersagbar, das ist hier ein Fehler
      und kein Detail)

## WP E — Prüfen und Melden

`buch-tippspiel/code/tippspiel_pruefung.dh` ist ein von Hand geschriebener
Ersatz für einen Testrahmen. Dass es ihn gibt, ist der beste Beleg, dass er
fehlt — und er zahlt sich bei jedem weiteren WP hier selbst zurück.

- [ ] `ASSERT(bedingung, meldung$)` und `ASSERT_EQ(ist, soll [, meldung$])` —
      bei Fehlschlag mit Datei:Zeile abbrechen
- [ ] Sammel-Modus: alle Prüfungen laufen lassen, am Ende Bilanz + Exit-Code
      ungleich 0 (braucht `EXIT` aus WP A)
- [ ] `LOG_INFO/WARN/ERROR(text$)` nach stderr mit Zeitstempel, Pegel über
      Umgebungsvariable einstellbar (braucht `GETENV$` aus WP A)

## WP F — Fehler, die man behandeln kann

`THROW` nimmt nur einen STRING, `CATCH` bekommt nur diesen String. Kein
`FINALLY` (verifiziert: im Lexer nicht vorhanden), kein Fehler-Code, keine
Fundstelle. Für Programme, die Aufräumen garantieren müssen — Datei schließen,
Transaktion zurückrollen, Gerät freigeben — ist das mühsam und fehleranfällig.

- [ ] `FINALLY`-Zweig in `TRY`
- [ ] Fundstelle im `CATCH`: `ERROR_LINE()`, `ERROR_FILE$()`, wenn machbar ein
      Aufruf-Pfad `ERROR_TRACE$()`
- [ ] Fehler-Code neben der Meldung: `THROW code$, meldung$` plus
      `ERROR_CODE$()` — damit ein `CATCH` entscheiden kann, statt Meldungstexte
      zu vergleichen

## WP G — Vererbung rund machen

`EXTENDS` gibt es, die Methodensuche läuft die Elternkette hoch
(`compiler.rs`). Aber es gibt **kein `SUPER`** — die überschriebene
Elternmethode ist nicht mehr erreichbar. In `docs/sprache.md` steht das
Ergebnis wörtlich im Beispiel: „Eigene Init, ruft super.Init nicht automatisch
auf" und dann drei Zeilen abgeschriebene Zuweisungen.

- [ ] `SUPER.Methode(...)` innerhalb einer Methode
- [ ] `ABSTRACT`-Methoden (deklariert, ohne Rumpf; `NEW` auf die Klasse ist ein
      Fehler) — der kleine Bruder von Interfaces, ohne neues Typ-Konzept

## WP H — Nebenläufigkeit

Coroutines sind kooperativ und frame-getrieben. Nur HTTP hat einen
Hintergrund-Pfad (`HTTP_GET_START`/`READY`/`RESULT`); `db`, `net` und `serial`
nicht. Ein langer SQL-Lauf friert das Fenster ein.

**Randbedingung, die den Entwurf bestimmt:** raylib will den Hauptthread, und
die VM ist nicht threadsicher. Ein Arbeitsthread darf darum weder zeichnen noch
VM-Zustand anfassen.

- [ ] Kurzfristig, billig: das `HTTP_GET_START`-Muster auf `db` (lange
      Abfragen) und `net` übertragen — dasselbe Polling wie `INPUT_UPDATE`
- [ ] Mittelfristig: `TASK_START(fnref, arg)` → ID, `TASK_READY(id)`,
      `TASK_RESULT(id)`. Aufgabe = **reine** Funktion, Argumente und Ergebnis
      werden **kopiert** übergeben. Keine geteilten Arrays, keine Objekte, kein
      Zeichnen im Task — das ist die Beschränkung, die den Rest sicher macht

## WP I — Namensräume und Module (der große Brocken)

`IMPORT "datei.dh"` ist textuelles Inkludieren in einen flachen globalen
Namensraum, in dem bereits **1316 Builtins** liegen. `IMPORT ... AS alias` ist
kein Namensraum, sondern Präfix-Kopieren, und funktioniert nur, wenn der
Builtin-Präfix zufällig dem Modulnamen entspricht (bei `imgfx`, das `IMAGE_*`
registriert, schon nicht — steht so in CLAUDE.md).

Folgen: keine getrennte Übersetzung, keine fremde Bibliothek ohne
Kollisionsrisiko, kein `PRIVATE`, keine Paketverwaltung. Unterhalb von ein paar
tausend Zeilen fällt das nicht auf; darüber ist es die Hauptbremse.

**Dieser Punkt hat als einziger Rückwirkung auf allen bestehenden Code** — er
gehört nicht nebenbei erledigt, sondern braucht einen eigenen Entwurf. Skizze
als Ausgangspunkt:

- Datei = Modul, Name aus dem Dateinamen; `IMPORT "mathe.dh" AS mathe` macht
  Namen als `mathe.Distanz(...)` erreichbar statt sie einzumischen
- `PRIVATE` vor `SUB`/`FUNCTION`/`DIM` hält Namen im Modul
- Ohne `AS` bleibt das heutige Verhalten (Einmischen) — sonst bricht jedes
  bestehende Programm
- Built-in-Module bleiben flach; sie umzustellen wäre eine zweite, größere
  Entscheidung

## WP J — Kleinkram mit Wirkung

- [ ] CSV lesen/schreiben, mit Anführungszeichen und Trennzeichen korrekt — der
      häufigste Datenaustausch überhaupt, heute Handarbeit mit `SPLIT$`
- [ ] ZIP lesen/schreiben (nach WP B) — Sicherungen, Belegsammlungen, Export
- [ ] Ein `SET`-Typ **oder** MAP mit INTEGER-Schlüssel. Heute sind Map-Schlüssel
      immer STRING; die Set-Comprehension liefert ersatzweise ein
      dedupliziertes TUPLE (steht so in CLAUDE.md) — brauchbar, aber O(n)
- [ ] `docs/editor.md` (Abschnitt „Debugger") behauptet, der Debugger laufe auf
      dem Python-Tree-Walker. Der ist seit Stufe B entfernt; der Debugger läuft
      längst über `dhrt debug` (`editor_qt/debugger.py`). Reiner Doku-Fehler,
      aber einer, der beim Lesen abschreckt

## Plattform

Die CI baut und testet nur unter Windows; für Linux und macOS läuft lediglich
`cargo check` (README.md). `wifi` ist ohnehin Windows-only. Solange
Drachenhauch Spiele exportiert, ist das vertretbar. Sobald es Werkzeuge bauen
soll, die anderswo laufen sollen, wird ein echter Build- und Testlauf auf
mindestens Linux fällig.

---

## Empfohlene Reihenfolge

**A → B → C → D → E → F → G → H → I**

Die Begründung ist durchgehend „wie viele neue Programme wird das möglich
machen, pro Aufwand":

- **A** ist klein und macht aus Drachenhauch schlagartig eine Skriptsprache.
  Ohne A bleibt jedes Programm eine Insel — das ist der eine Punkt, an dem
  „Tausendsassa" heute konkret scheitert.
- **B** ist die Voraussetzung für C, D und ZIP.
- **C** und **D** sind je ein überschaubarer Brocken und öffnen zusammen die
  gesamte Welt der angemeldeten Web-Dienste.
- **E** zahlt sich ab dann bei jedem weiteren Schritt selbst zurück.
- **F** und **G** sind Bequemlichkeit für großen Code, kein Türöffner.
- **H** wird erst dringend, wenn Programme länger laufen als ein Frame.
- **I** zuletzt, weil es als einziges den Bestand anfasst.

## Bewusst nicht auf der Liste

- **Generics.** Passt nicht zu BASIC-Lesbarkeit; `ARRAY OF ANY` plus
  Laufzeit-Typprüfung (`TYPEOF`) deckt die realen Fälle.
- **`async`/`await`.** Zwei Nebenläufigkeits-Modelle nebeneinander (Coroutinen
  *und* async) wären ein eigener Stolperstein; WP H reicht.
- **Eine Paketverwaltung.** Ohne WP I hätte sie nichts, was sie verwalten
  könnte.
- **Ein eigener HTTP-Server.** `net` kann TCP annehmen (`NET_TCP_LISTEN`);
  wer wirklich einen Dienst braucht, stellt einen fertigen Server davor.
