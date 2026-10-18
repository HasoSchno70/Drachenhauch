# Funkenflug

Ein **Schaltplan-Editor in Drachenhauch** mit GTK 3 als Oberfläche – die
Neufassung der Idee hinter [SparkNode](https://github.com/HasoSchno70/elektronik)
(ursprünglich Python + PySide6). GTK/Cairo-Ansatz wie bei *Glutfaden* (der
GTK-Portierung von PySticky).

![Funkenflug](screenshot.png)

## Starten

```
dhrt run funkenflug/main.dh                       # Demo-Schaltung
dhrt run funkenflug/main.dh -- schaltplan.fkn      # eine Datei öffnen
dhrt run funkenflug/main.dh -- --foto bild.png     # Selbsttest (headless, PNG)
```

GTK 3 muss vorhanden sein: unter Linux fast immer (`libgtk-3-0`), unter macOS
über Homebrew (`brew install gtk+3`), unter Windows aus MSYS2 im PATH – oder aus
einem Programm, das es mitbringt (**Inkscape**, GIMP). Die Bibliotheks-Namen in
`gtk/gtk.dh` nennen diese Fundorte durch `|` getrennt.

## Bedienung

| Werkzeug | |
|---|---|
| **Auswahl** (Taste `A`) | Bauteil, Label oder Draht anklicken, ziehen zum Verschieben; auf freier Fläche einen **Rahmen** aufziehen (wählt alles ganz Umschlossene), `Umschalt` ergänzt bzw. schaltet einzelne Elemente um; **Doppelklick**, `E` oder `Enter` bearbeitet (Bauteil: Referenz/Wert; Label und Draht: Netz umbenennen) |
| **Draht** (Taste `W`) | klicken setzt Eckpunkte, Doppelklick / `Enter` / rechte Maustaste schließt ab, `Esc` bricht ab. Enden fangen an Pins und Drahtenden |
| **Label** (Taste `L`) | klicken, Netznamen eingeben. Der Anker ist der elektrische Punkt; **gleichnamige Labels sind ein Netz** |
| **Bauteil** | in der Leiste aus der Liste wählen, dann auf die Fläche klicken; rechte Maustaste oder `R` dreht die Vorschau |

Weiteres: `Strg+Z` / `Strg+Y` (oder `Strg+Umschalt+Z`) bzw. die Pfeile in der
Kopfleiste machen rückgängig und wiederholen. `R` dreht die Auswahl, `Entf`
löscht sie, Mausrad zoomt um den Zeiger, mittlere Maustaste schiebt,
`+`/`-`/`0` zoomen, `Strg+S` speichert, `Strg+O` öffnet. Eine schon vergebene
Referenz lehnt der Eigenschaften-Dialog ab.

**Gummiband:** Zieht man Bauteile (einzeln oder als Auswahl), wandern die
angeschlossenen Drahtenden mit. Damit die Leitung rechtwinklig bleibt, rückt
die Nachbarecke mit oder es entsteht ein Knick; Drähte, deren beide Enden
mitwandern, verschieben sich starr, ebenso Labels an bewegten Pins. Das
Gummiband **erfindet nie eine Verbindung**: Würde ein Knick einen fremden Pin,
ein Label oder einen anderen Draht berühren (etwa wenn beim Drehen beide
Anschlüsse eines Widerstands zum selben Punkt knicken), nimmt es den Knick
andersherum oder zuletzt eine Schräge.

**Drehen und Spiegeln:** `R` dreht um 90°, `X` spiegelt links/rechts, `Y`
oben/unten – ein Bauteil um seinen Ursprung, eine Auswahl um ihre Mitte (auf
dem Raster, damit alle Pins auf dem Raster bleiben). Angeschlossene Drähte
folgen per Gummiband, ein Rückgängig-Schritt. Beim Platzieren gelten
`R`/`X`/`Y` der Vorschau.

**Ausgabe** (Knopf „Ausgabe ▾" in der Kopfleiste): **PDF**, **SVG** und
**PNG** (150 dpi), **Drucken** über den Druckdialog des Systems, dazu
Stückliste und Netzliste. Alles läuft durch dieselbe Zeichenroutine wie der
Bildschirm, nur mit Druckfarben auf Weiß, ohne Raster, Auswahl und Pin-Punkte,
auf A4 quer eingepasst (kleine Schaltungen höchstens 3:1) und mit Schriftfeld.

![Ausgabe als PDF/PNG](druck.png)

**Eigene Bauteile und ICs** (Knopf „Symbol …" in der Leiste; ist ein eigenes
Bauteil gewählt, bearbeitet er dessen Symbol): Name, Präfix, Wert und eine
**Pinliste**, eine Zeile je Pin –

```
# Nummer  Name   Seite   Typ
1  GND    unten   vin
2  TRIG   links   ein
3  OUT    rechts  aus
8  VCC    oben    vin
```

Seite: `links rechts oben unten` (Vorgabe links), Typ: `passiv ein aus bidir
vin vout nc` (Vorgabe passiv). Fehler stehen zeilengenau unter der Liste,
rechts zeigt eine Vorschau das Symbol, genau wie es im Schaltplan erscheint.
Gehäuse und Pins legt der Generator rasterfest an, Pin-Nummern und -Namen
werden mitgezeichnet. Die Pin-Typen gehen in den ERC (Eingang ohne Treiber,
Ausgänge gegeneinander …).

![Eigenes IC](ic.png)

Eigene Symbole **stecken in der `.fkn`** – eine Datei bleibt eigenständig,
auch auf einem anderen Rechner. Mit dem Haken „auch in meiner Bibliothek
speichern" landen sie zusätzlich in `%APPDATA%\Funkenflug\symbole.fkl`
(Linux: `~/.config/funkenflug/`, abweichend per `FUNKENFLUG_BIBLIOTHEK`) und
stehen in jedem Projekt in der Bauteilliste. Ein Symbol bearbeiten ändert alle
Bauteile, die es benutzen – ein Rückgängig-Schritt. Fehlt ein Symbol wirklich,
erscheint ein roter Platzhalter, und der ERC meldet es.

**Stückliste** (Kopfleiste): gruppiert nach Bauteil und Wert, natürlich
sortiert (R2 vor R10), Bereiche zusammengefasst (`R1-R3, R10`), ohne
Versorgungssymbole; Export als CSV (Semikolon), Markdown oder Text.

**Ablage:** `Strg+C` kopiert, `Strg+X` schneidet aus, `Strg+V` fügt an der
Mausposition ein, `Strg+A` wählt alles. Das läuft über die Zwischenablage des
Systems, also auch zwischen zwei Funkenflug-Fenstern. Eingefügte Bauteile
bekommen freie Referenzen (eine Kopie von R1 wird z. B. R8), Labels und
Versorgungen behalten ihre Namen und hängen damit am selben Netz wie das
Original. Das Eingefügte ist danach gewählt und lässt sich gleich verschieben.

**Drähte:** Ein Klick wählt das Segment unter der Maus. Ziehen verschiebt nur
dieses Segment – waagrechte senkrecht, senkrechte waagrecht, damit die Nachbarn
rechtwinklig bleiben. Hängt an einem Drahtende etwas (Pin, Label, anderer
Draht), bleibt das Ende stehen und ein Verbindungsknick entsteht: Ziehen reißt
keine Verbindung ab. Danach räumt Funkenflug den Draht auf (Ecken auf einer
Geraden fallen weg). `Entf` löscht das Segment, der Draht zerfällt in die
Reststücke; `Umschalt+Entf` löscht den ganzen Draht.

**Umbenennen** (`F2`, oder Doppelklick auf Label/Draht): Ein Netzname wird an
**allen Stellen** ersetzt – gleichnamige Labels und Versorgungssymbole (aus GND
wird so z. B. `AGND`, es bleibt trotzdem SPICE-Masse). Bei einem Label lässt
ein Haken auch nur dieses eine umbenennen. Hat das Netz nur einen automatischen
Namen, setzt Funkenflug ein Label an die angeklickte Stelle. Trägt ein anderes
Netz den neuen Namen schon, fragt es vorher – Umbenennen würde beide verbinden.

**Netze:** Fährt die Maus über einen Draht, einen Pin oder ein Label, zeigt die
Statuszeile den Netznamen samt Pinzahl und die Drähte des Netzes leuchten auf.
Namen: Versorgungssymbol (GND, VCC, auch ein umbenanntes wie `+5V`) vor Label vor
automatisch (`N1`, `N2` …). Gleichnamige Versorgungen und Labels sind ein Netz,
auch ohne Draht.

**ERC** (Kopfleiste): listet Fehler und Warnungen in einem Fenster, das offen
bleiben darf; eine Zeile anklicken holt die Stelle in die Mitte, auf der Fläche
markieren Kreise jede Stelle. Geprüft werden doppelte und fehlende Referenzen,
offene Pins, offene Drahtenden, kurzgeschlossene Bauteile (zwei Pins im selben
Netz), mehrere Labels an einem Netz, Labels auf einer fremden Versorgung,
**Kurzschlüsse zwischen Versorgungen** und die Pin-Typen gegeneinander (zwei
Ausgänge, Ausgang gegen Versorgung, Eingang ohne Treiber …). Versorgungssymbole
gelten als Quelle – ein PWR_FLAG wie in KiCad braucht es nicht.

**Netzliste** (Kopfleiste): das Format folgt der Endung – `.cir`/`.sp` SPICE
(direkt für ngspice: GND wird Knoten `0`, `4k7` → `4.7k`, `1M` → `1Meg`, denn
`M` heißt in SPICE *milli*; Dioden und Transistoren bekommen `.model`-Zeilen),
`.csv` (Semikolon) oder `.net`/`.txt` als lesbarer Text.

## Aufbau

```
kern/       Modell, Geometrie, Bibliothek, Netze, Datei – ohne Oberfläche,
            Namensraum "kern"
  geometrie.dh   Punkt, Lage (Transform), Raster/Snap, Konstanten
  modell.dh      Pin, Primitiv, Symbol, Bauteil, Draht, Knoten, Label, Blatt, Dokument
  bibliothek.dh  10 eingebaute Symbole (R, C, Elko, L, Diode, LED, NPN, Schalter, GND, VCC)
  netze.dh       Netz-Erkennung (eine reine Funktion), benannte Netze, Auto-Knoten
  erc.dh         Elektrische Regelpruefung
  netzliste.dh   Export: SPICE, CSV, Text
  symbole.dh     eigene Symbole: Pinliste, IC-Generator, JSON, Register, Bibliothek
  stueckliste.dh Stückliste: Gruppen, natürliche Sortierung, CSV/Markdown/Text
  datei.dh       Speichern/Laden als .fkn (JSON, Version 1)
gtk/        Deklarationen für GTK 3, GDK, Cairo, Pango, GLib (aus Glutfaden)
ansicht/    Zeichenfläche und Werkzeuge – globaler Namensraum
  leinwand.dh    Welt↔Schirm, Grid, Zeichnen von Bauteilen/Drähten/Knoten/Labels
  werkzeuge.dh   Maus, Tastatur, Werkzeug-Logik
  verlauf.dh     Rückgängig/Wiederholen
  auswahl.dh     Auswahl als Menge, Rahmen, Gruppen-Zug mit Gummiband, Ablage
  symboleditor.dh  Symbol-Editor mit Vorschau
  ausgabe.dh     PDF, SVG, PNG, Drucken, Menü „Ausgabe"
main.dh     Fenster, Kopfleiste, Werkzeugleiste, Statuszeile, Dialoge
```

Die Tests laufen ohne Mensch: `dhrt run funkenflug/pruef_kern.dh` prüft Modell,
Transform, Raster, Netze (auch T-Verbindung und Labels) und Save/Load.
`main.dh -- --foto bild.png` spielt die Werkzeuge durch – **samt der echten
Dialoge**, die ein Timer ausfüllt und bestätigt –, prüft Undo/Redo, gibt je
Prüfung `ok`/`FEHLER` aus und zeichnet das Fenster in eine PNG (119 Prüfungen,
darunter Drehen/Spiegeln einzeln und als Gruppe ohne Kurzschluss, PDF/SVG/PNG
und Drucken über GTKs Export-Weg, die echten Werkzeug-Knöpfe, Symbol-Editor,
Stückliste, Gummiband, Rahmen und Gruppen-Zug über den Klick-Handler,
Kopieren/Einfügen, Umbenennen, ERC und SPICE). `pruef_kern.dh` prüft u. a. die
Gruppen-Lage in allen 8 Orientierungen × 3 Arten (72 Pin-Prüfungen). Der Selbsttest
lässt die Zwischenablage des Systems in Ruhe und schreibt seine Bibliothek in
eine Wegwerf-Datei neben dem Bild. `--nur-foto bild.png` macht nur das Bild.

## Was gegenüber SparkNode bewusst anders ist

SparkNode hatte mehrere Konzeptfehler, die hier von Anfang an vermieden sind:

1. **Eine Rotations-/Transform-Konvention für Modell UND Zeichnen.** In
   SparkNode drehte das Modell gegen, der Maler (`QPainter`) mit dem
   Uhrzeigersinn – gedrehte Pins lagen gespiegelt zu den elektrischen. Hier geht
   **alles** durch `kern.transformPunkt` / `kern.transformRichtung` (Y nach
   unten, Drehung im Uhrzeigersinn), und der Canvas baut seine Cairo-Matrix so
   auf, dass sie dieselbe Rechnung ist. Gezeichnet == elektrisch, per
   Konstruktion.
2. **Pin-Punkt eindeutig.** `pin.pos` IST der elektrische Andockpunkt;
   `pin.laenge` ist nur der gezeichnete Stummel. Kein Nebeneinander von
   „position" und „connection point".
3. **Eine Netz-Erkennung.** `kern.netzeBerechnen` ist die einzige Quelle für
   Konnektivität (später auch ERC/Netzliste). Alle Punkte aufs Raster
   quantisiert; T-Verbindung zählt, eine reine Kreuzung ohne Knoten nicht.
4. **Label-Anker getrennt von Darstellung.** Der Anker ist der elektrische
   Punkt; Text und Unterstrich sind nur Anzeige und verschieben ihn nie.
5. **Undo über vollständige Schnappschüsse** statt einzelner Befehle. SparkNodes
   Undo-Fehler kamen von Befehlen, die nur einen Teil des Zustands kannten
   (welches Blatt, Annotation). Ein Schnappschuss kennt immer alles; Ziehen ist
   ein Schritt, ein bloßer Klick keiner.
6. **Bewusst weggelassen** (in SparkNode überproportional viele Bugquellen):
   Bus/Bus-Entry, Hierarchie-Sheets, Pin-Swap, Multi-Unit. Kommt erst, wenn
   wirklich gebraucht.

## Hinweis für Drachenhauch-Code

`/` liefert in Drachenhauch **immer** eine Kommazahl (seit 2026-09-24), auch
bei zwei ganzen Zahlen; ganzzahlig teilt `\`. `rastern()` hatte das falsch und
verschob jeden Punkt um ein halbes Raster – unbemerkt, weil `RETURN` eines
ganzzahligen Kommawerts aus einer `AS INTEGER`-Funktion still durchgeht.

`MID$` zählt ab **0**, und `INSTR` liefert die 0-basierte Position oder **-1** –
anders als QBasic/VB. `INSTR(...) > 0` übersieht deshalb einen Treffer ganz
vorn; besser `CONTAINS(s, teil)`. Genau daran vergab Funkenflug anfangs jedem
zweiten Kondensator wieder „C1".

## Noch offen (nächste Runden)

Mehrere Blätter in der Oberfläche, Symbole mit freier Grafik (bisher
Rechteck-Gehäuse aus dem Generator), Überstrich für aktiv-low (`~RESET`),
Bauteil-Felder wie Gehäuse/Bestellnummer für die Stückliste, Labels drehen,
Platinen-Layout.
