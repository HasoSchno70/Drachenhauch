# Drachenhauch 2026.24

*Die Notizen zu dieser Fassung. Einen Tag und sechs Pull Requests nach
2026.23: Python im eigenen Programm, PostgreSQL und MySQL mit denselben
Befehlen wie SQLite, das Handbuch auf Englisch -- und die fremden
Bibliotheken sind fertig ausgebaut, bis zu Structs als Wert, Bitfeldern,
`va_list` und Aufrufen im Maschinencode.*

## Neu in dieser Fassung

### Python einbetten

Python ist selbst eine C-Bibliothek. Die Datei `python/python.dh` aus den
Beispielen holt das Python des Rechners ins Programm -- und mit ihm jedes
Paket, das dort installiert ist, von numpy bis PySide6:

```text
IMPORT "python/python.dh"

pythonStarten()
pythonAusfuehren("import statistics")
PRINT pythonZahl("statistics.median([3, 1, 4, 1, 5])")     ' 3.0
```

* `pythonStarten` sucht ein Python (Argument, `DH_PYTHON`, `py -3`,
  `python3`, `python`) und übernimmt seinen Suchpfad samt venv;
  `pythonZahl`, `pythonText$`, `pythonBytes` holen Ergebnisse, die
  `pythonSetze…`-Befehle legen Werte ab.
* Ein Fehler in Python kommt als Fehler in Drachenhauch an und lässt sich
  mit `CATCH` abfangen.
* Dafür neu: `LIB "$NAME"` nimmt den Namen einer Bibliothek aus einer
  Umgebungsvariablen, gelesen beim ersten Aufruf.
* Neues Beispiel `208_python.dh`: das Spektrum eines Signals, mit numpy oder
  in reinem Python.

### PostgreSQL und MySQL

Das Modul `db` spricht neben SQLite jetzt mit Servern -- mit denselben
Befehlen. `DB_OPEN` entscheidet am Ziel:

```basic
IMPORT "db"
DIM c AS DB_CONN
c = DB_OPEN("postgres://hans:geheim@server:5432/laden")
' oder: c = DB_OPEN("mysql://hans:geheim@server:3306/laden")
DB_EXEC(c, "INSERT INTO kunden (name) VALUES (?)", "Anna")
PRINT DB_KIND$(c), DB_PING(c)
```

* Platzhalter sind überall `?`; genaue Dezimalzahlen (`numeric`,
  `DECIMAL`) kommen unverändert als Text an.
* Verschlüsselt wird, wenn der Server es kann; `?sslmode=verify-full` prüft
  auch das Zertifikat.
* Neu `DB_KIND$` und `DB_PING`; `GUI_FORM_SAVE` funktioniert auch gegen einen
  Server.

### Das Handbuch auf Englisch

Das Handbuch (`docs/`) gibt es jetzt auch auf Englisch: 72 Dokumente in
`docs/en/` -- Sprache, Befehle, alle Module, IDE und Werkzeuge. Mit
englischer Oberfläche liest die IDE es von dort, und der Sprachserver zeigt
mit `DHRT_LANG=en` englische Beschreibungen beim Überfahren. Das englische
Lehrbuch (*Handbook*) ist vollständig übersetzt.

### Fremde Bibliotheken: der Rest

```basic
STRUCT Komplex LAYOUT C
    re AS FLOAT
    im AS FLOAT
END STRUCT
DECLARE FUNCTION csqrt LIB "m" (BYVAL z AS Komplex) AS Komplex
DECLARE FUNCTION vsprintf LIB "msvcrt|c" (ziel AS BUFFER, format AS TEXT, werte AS VALIST) AS LONG

DIM z AS Komplex
z.re = -4
DIM w AS Komplex
w = csqrt(z)
PRINT w.re, w.im                          ' 0.0  2.0

DIM b AS BUFFER
b = BUFFER_NEW(128)
vsprintf(b, "%d Drachen, %s", (3, "feuerrot"))
PRINT TEXT_AUS_ZEIGER$(BUFFER_ZEIGER(b))  ' 3 Drachen, feuerrot
```

* **Structs als Wert:** `BYVAL p AS Punkt` übergibt einen Struct als Wert,
  `AS Punkt` als Rückgabe liefert einen -- auch in einem Rückruf, in beide
  Richtungen. Wie er reist (Register, Stapel, Zeiger auf eine Kopie), legt
  jedes System anders fest; dhrt hält sich an die Regeln von Windows, Linux
  und macOS.
* **Bitfelder:** `fBinary AS ULONG : 1` -- wie in C, die Lage nach dem
  Compiler des Systems (MSVC unter Windows, GCC/Clang sonst).
* **`va_list`:** `VALIST` als Parameter für `vprintf` und Co.; die Werte
  kommen als Tupel.
* **Schneller:** eine Schleife, die eine fremde Funktion mit Zahlen ruft,
  läuft im Maschinencode -- eine Million Aufrufe von `abs` brauchen 54 statt
  201 ms, und auch in der VM nur noch 116.

## Behoben

* MySQL über `localhost` versuchte nur die erste Adresse (`::1`); lauschte
  der Server nur auf IPv4, kam keine Verbindung zustande.
* Im Handbuch zeigte ein Verweis in `builtins-grafik.md` ins Leere,
  `module-chart.md` nannte die Mausbedienung als fehlend, und
  `module-particles.md` beschrieb noch die alte Python-Fassung.
* `dhrt --version` nannte PostgreSQL und MySQL nicht -- einem Bau sah man
  nicht an, ob er mit Servern spricht (die Spiel-Laufzeit für schlanke
  Exporte tut es nicht).

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 2107 auf **2109** Einträge
(`DB_KIND$`, `DB_PING`), es bleiben 48 Module; **224** Beispiele (vorher
223). **4885 Fälle in 373 Prüfsammlungen** (vorher 4832 in 370) und
**530 Rust-Testfunktionen** (vorher 509).

* Der Installer wächst auf **53 MB** (vorher 52), vor allem durch die
  Treiber für PostgreSQL und MySQL; das Lehrbuch hat 537 Seiten (englisch
  530).
* Neu `dhrt pruef uebersetzung`: zu jedem englischen Dokument muss es ein
  deutsches mit denselben Überschriften, Tabellen und Codeblöcken geben --
  wer ein deutsches Dokument ändert, erfährt, welche Übersetzung nachzuziehen
  ist. Dieselbe strenge Prüfung der Verweise samt Sprungmarken läuft jetzt
  auch über die deutschen Dokumente.
* Die Datenbank-Server prüft die CI auf dem Linux-Läufer gegen echte
  PostgreSQL- und MySQL-Server; jeder Ablauf läuft gegen beide mit
  derselben Erwartung.
* Die Lagen der Bitfelder wurden nicht aus dem Gedächtnis gebaut, sondern
  mit Clang für Windows, Linux und Apple-ARM nachgesehen. Ein Aufruf einer
  Bibliothek darf nur in den Maschinencode, solange das Programm keinen
  Rückruf vergeben hat: eine Bibliothek darf sich einen merken und ihn bei
  jedem späteren Aufruf rufen.

## Was offen bleibt

* Ein Struct als Wert hinter `...`, Bitfelder zusammen mit `PACK`, ein
  `va_list` im Rückruf; ein Struct als Feld einer Klasse und Felder von
  Structs; C++ und COM. `VALIST` auf Linux-ARM ist nach den Regeln gebaut,
  aber nicht ausprobiert.
* Die Formular-Fälle gegen die Datenbank-Server brauchen ein Fenster und
  sind nur lokal belegt.
* Ein Tray-Symbol unter macOS und Linux.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt; der Installer ist nicht signiert.
